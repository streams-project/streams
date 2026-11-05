#![no_std]
#![forbid(unsafe_code)]
//! Optional layer around the core: keeps commands until the server has them and screens for when
//! the network is gone. The host steps the layer and the layer steps the core, with the same
//! message shape. The core marks each http effect command, cacheable or plain; this layer takes
//! the first two and passes the rest on. Tokens stay in the core: a kept command goes out through
//! `Action::Request`, so nothing here ever holds one.

extern crate alloc;

pub mod cache;
pub mod queue;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use cache::{Cache, INDEX_KEY, Index, Waiter, key_of};
use contract::command::{Changes, CommandAnswer, CommandBatch};
use contract::core_msg::{
    Action, BodyTarget, CoreIn, CoreInTag, CoreOut, CoreOutTag, Effect, EffectBody, Event,
    EventBody, HttpClass, HttpMethod, NoAnswerReason, ScreenBody, SessionState, SessionView,
    StoreOp, View,
};
use contract::error::ErrorBody;
use contract::version::WireDate;
use core_rules::Step;
use core_rules::net::{Next, problem_of_status};
use queue::{KEY_PREFIX, Kept, Queue};
use serde::Deserialize;

/// What the host tells the layer at start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// Names this boot. It changes when monotonic time restarts.
    pub boot_id: String,
    /// The byte cap from the capability card. 0 means nothing is kept across a restart.
    pub storage_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    New,
    Loading,
    Ready,
}

/// What an effect of this layer is waiting for.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Own {
    QueueList,
    QueueItem,
    Index,
    Write,
    /// An http effect of the core, taken for the cache, by key and scope.
    Fetch {
        key: String,
        scope: Option<String>,
    },
    /// A kept body being read.
    Body {
        key: String,
        stale: bool,
    },
    Flush,
    Pull {
        scope: String,
    },
}

#[derive(Deserialize)]
struct Ttl {
    #[serde(default)]
    ttl_s: u32,
}

#[derive(Default)]
struct Merged {
    effects: Vec<Effect>,
    view: View,
    unsupported: Vec<String>,
}

impl Merged {
    fn view(&mut self, view: View) {
        macro_rules! newest {
            ($($part:ident),+) => {
                $(if view.$part.is_some() {
                    self.view.$part = view.$part;
                })+
            };
        }
        newest!(session, route, gate, play, profiles);
    }
}

fn done(id: String, status: u32, body: Option<String>) -> Event {
    Event {
        id: Some(id),
        body: EventBody::HttpDone {
            status,
            headers: BTreeMap::new(),
            body,
        },
    }
}

fn fetch_headers(etag: Option<String>, headers: &mut BTreeMap<String, String>) {
    if let Some(etag) = etag {
        headers.insert(String::from("If-None-Match"), etag);
    }
}

fn encode(text: &str) -> String {
    let mut kept = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            kept.push(char::from(byte));
        } else {
            kept.push_str(&format!("%{byte:02X}"));
        }
    }
    kept
}

/// The layer around anything that takes the core's message shape.
pub struct Layer<S: Step> {
    inner: S,
    config: Config,
    phase: Phase,
    durable: bool,
    next_id: u64,
    own: BTreeMap<String, Own>,
    waiting: Vec<Event>,
    loaded: Vec<Kept>,
    queue: Queue,
    cache: Cache,
    scope: Option<String>,
    pulling: bool,
    inner_wake: Option<u64>,
}

impl<S: Step> Layer<S> {
    pub fn new(inner: S, config: Config) -> Self {
        Self {
            inner,
            durable: config.storage_bytes > 0,
            config,
            phase: Phase::New,
            next_id: 0,
            own: BTreeMap::new(),
            waiting: Vec::new(),
            loaded: Vec::new(),
            queue: Queue::default(),
            cache: Cache::default(),
            scope: None,
            pulling: false,
            inner_wake: None,
        }
    }

    fn id(&mut self) -> String {
        self.next_id += 1;
        format!("l{}", self.next_id)
    }

    fn store(
        &mut self,
        op: StoreOp,
        key: String,
        value: Option<String>,
        own: Own,
        out: &mut Merged,
    ) {
        let id = self.id();
        self.own.insert(id.clone(), own);
        out.effects.push(Effect {
            id,
            body: EffectBody::Store {
                op,
                key: Some(key),
                value,
            },
        });
    }

    fn write(&mut self, key: String, value: String, out: &mut Merged) {
        if self.durable {
            self.store(StoreOp::Put, key, Some(value), Own::Write, out);
        }
    }

    fn delete(&mut self, key: String, out: &mut Merged) {
        if self.durable {
            self.store(StoreOp::Delete, key, None, Own::Write, out);
        }
    }

    fn write_index(&mut self, out: &mut Merged) {
        if let Ok(text) = serde_json::to_string(&self.cache.index) {
            self.write(String::from(INDEX_KEY), text, out);
        }
    }

    fn load(&mut self, out: &mut Merged) {
        if self.durable {
            self.phase = Phase::Loading;
            let list = Own::QueueList;
            self.store(StoreOp::List, String::from(KEY_PREFIX), None, list, out);
            let index = Own::Index;
            self.store(StoreOp::Get, String::from(INDEX_KEY), None, index, out);
        } else {
            self.phase = Phase::Ready;
        }
    }

    fn load_step(&mut self, event: Event, out: &mut Merged) {
        let own = event.id.as_deref().and_then(|id| self.own.remove(id));
        match (own, event.body) {
            (Some(Own::QueueList), EventBody::StoreDone { keys, .. }) => {
                for key in keys {
                    self.store(StoreOp::Get, key, None, Own::QueueItem, out);
                }
            }
            (Some(Own::QueueItem), EventBody::StoreDone { value, .. }) => {
                let text = value.unwrap_or_default();
                if let Ok(mut kept) = serde_json::from_str::<Kept>(&text) {
                    kept.bytes = u64::try_from(text.len()).unwrap_or(u64::MAX);
                    self.loaded.push(kept);
                }
            }
            (Some(Own::Index), EventBody::StoreDone { value, .. }) => {
                if let Some(index) =
                    value.and_then(|text| serde_json::from_str::<Index>(&text).ok())
                {
                    self.cache.index = index;
                }
            }
            (Some(_), EventBody::Unsupported) => {
                self.durable = false;
                self.own.clear();
            }
            (Some(_), _) => {}
            (None, body) => self.waiting.push(Event { id: event.id, body }),
        }
        if self.phase == Phase::Loading && self.own.is_empty() {
            self.queue.load(core::mem::take(&mut self.loaded));
            self.phase = Phase::Ready;
        }
    }

    /// Step the inner core until it has nothing more to answer.
    fn pump(&mut self, input: (u64, u64), mut events: Vec<Event>, out: &mut Merged) {
        loop {
            let result = self.inner.step(CoreIn {
                contract: WireDate::current(),
                tag: CoreInTag::default(),
                now_ms: input.0,
                mono_ms: input.1,
                events,
            });
            self.inner_wake = result.next_wake_ms;
            out.unsupported.extend(result.unsupported);
            let mut answers = Vec::new();
            for effect in result.effects {
                self.take_effect(input.1, effect, &mut answers, out);
            }
            if let Some(view) = result.view {
                if let Some(session) = &view.session {
                    self.watch_session(session, input.1, &mut answers, out);
                }
                out.view(view);
            }
            if answers.is_empty() {
                return;
            }
            events = answers;
        }
    }

    fn take_effect(
        &mut self,
        mono_ms: u64,
        effect: Effect,
        answers: &mut Vec<Event>,
        out: &mut Merged,
    ) {
        let Effect { id, body } = effect;
        match body {
            EffectBody::Http {
                class: HttpClass::Command,
                body: Some(text),
                method,
                path,
                headers,
                deadline_ms,
                to,
                scope,
            } => match serde_json::from_str::<CommandBatch>(&text) {
                Ok(batch) => {
                    for op in batch.ops {
                        self.keep(op, scope.clone(), mono_ms, out);
                    }
                    answers.push(done(id, 202, None));
                }
                Err(_) => out.effects.push(Effect {
                    id,
                    body: EffectBody::Http {
                        method,
                        path,
                        headers,
                        body: Some(text),
                        deadline_ms,
                        class: HttpClass::Plain,
                        to,
                        scope,
                    },
                }),
            },
            EffectBody::Http {
                class: HttpClass::Cacheable,
                method,
                path,
                mut headers,
                body,
                deadline_ms,
                to,
                scope,
            } => {
                let key = key_of(scope.as_deref(), &path);
                let waiter = Waiter { id: id.clone(), to };
                if let Some(waiters) = self.cache.waiting.get_mut(&key) {
                    waiters.push(waiter);
                    return;
                }
                self.cache.waiting.insert(key.clone(), vec![waiter]);
                let entry = self.cache.index.entries.get(&key).cloned();
                if let Some(entry) = entry.filter(|_| self.durable) {
                    if entry.fresh(&self.config.boot_id, mono_ms) {
                        let wait = Own::Body { key, stale: false };
                        self.store(StoreOp::Get, entry.store_key, None, wait, out);
                        return;
                    }
                    fetch_headers(entry.etag, &mut headers);
                }
                let scope_kept = scope.clone();
                self.own.insert(
                    id.clone(),
                    Own::Fetch {
                        key,
                        scope: scope_kept,
                    },
                );
                out.effects.push(Effect {
                    id,
                    body: EffectBody::Http {
                        method,
                        path,
                        headers,
                        body,
                        deadline_ms,
                        class: HttpClass::Cacheable,
                        to: BodyTarget::Core,
                        scope,
                    },
                });
            }
            other => out.effects.push(Effect { id, body: other }),
        }
    }

    fn keep(
        &mut self,
        op: contract::command::CommandOp,
        scope: Option<String>,
        mono_ms: u64,
        out: &mut Merged,
    ) {
        let (kept, replaced) = self.queue.add(op, scope, &self.config.boot_id, mono_ms);
        if let Some(key) = replaced {
            self.delete(key, out);
        }
        let Ok(text) = serde_json::to_string(&kept) else {
            return;
        };
        let bytes = u64::try_from(text.len()).unwrap_or(u64::MAX);
        if self.make_room(bytes, out) {
            self.queue.set_bytes(kept.seq, bytes);
            self.write(kept.key(), text, out);
        }
    }

    /// Clear room under the byte cap by dropping kept pages. False when it cannot fit.
    fn make_room(&mut self, need: u64, out: &mut Merged) -> bool {
        let cap = self.config.storage_bytes;
        let queued = self.queue.bytes();
        while self.cache.bytes() + queued + need > cap {
            match self.cache.evict_one() {
                Some(store_key) => self.delete(store_key, out),
                None => return queued + need <= cap,
            }
        }
        true
    }

    fn watch_session(
        &mut self,
        session: &SessionView,
        mono_ms: u64,
        answers: &mut Vec<Event>,
        out: &mut Merged,
    ) {
        if session.state == SessionState::SignedOut {
            self.wipe(out);
        }
        if session.scope == self.scope {
            return;
        }
        self.scope.clone_from(&session.scope);
        self.pulling = false;
        if let Some(scope) = session.scope.clone() {
            if self.queue.has_ops(&scope) {
                self.queue.flush_now(mono_ms);
            }
            self.pull(&scope, answers);
        }
    }

    /// Sign out cleanup. Hello and other answers with no scope stay.
    fn wipe(&mut self, out: &mut Merged) {
        let queued = self.queue.clear();
        let bodies = self.cache.clear_scoped();
        // An answer on its way for this session must not land in the store.
        self.own.retain(|_, own| {
            !matches!(
                own,
                Own::Flush | Own::Pull { .. } | Own::Fetch { scope: Some(_), .. }
            )
        });
        self.cache.forget_waiting();
        let changed = !queued.is_empty() || !bodies.is_empty();
        for key in queued.into_iter().chain(bodies) {
            self.delete(key, out);
        }
        if changed {
            self.write_index(out);
        }
    }

    fn pull(&mut self, scope: &str, events: &mut Vec<Event>) {
        if self.pulling {
            return;
        }
        let path = match self.cache.index.cursors.get(scope) {
            Some(cursor) => format!("/v1/changes?cursor={}", encode(cursor)),
            None => String::from("/v1/changes"),
        };
        self.pulling = true;
        let id = self.id();
        self.own.insert(
            id.clone(),
            Own::Pull {
                scope: String::from(scope),
            },
        );
        events.push(Event {
            id: Some(id),
            body: EventBody::Action {
                action: Action::Request {
                    method: HttpMethod::Get,
                    path,
                    body: None,
                },
            },
        });
    }

    /// The event that sends the next batch through the core, if one can go.
    fn flush_event(&mut self, mono_ms: u64) -> Option<Event> {
        let scope = self.scope.clone()?;
        let batch = self.queue.batch(&scope, &self.config.boot_id, mono_ms)?;
        let body = serde_json::to_string(&batch).ok()?;
        let id = self.id();
        self.own.insert(id.clone(), Own::Flush);
        Some(Event {
            id: Some(id),
            body: EventBody::Action {
                action: Action::Request {
                    method: HttpMethod::Post,
                    path: String::from("/v1/commands"),
                    body: Some(body),
                },
            },
        })
    }

    fn answer_waiters(
        &mut self,
        key: &str,
        status: u32,
        text: Option<&str>,
        stale: bool,
        events: &mut Vec<Event>,
        out: &mut Merged,
    ) {
        for waiter in self.cache.waiting.remove(key).unwrap_or_default() {
            let Some(text) = text else {
                events.push(done(waiter.id, status, None));
                continue;
            };
            if waiter.to == BodyTarget::Screen {
                let id = self.id();
                out.effects.push(Effect {
                    id,
                    body: EffectBody::View {
                        notice: None,
                        body: Some(ScreenBody {
                            request: waiter.id.clone(),
                            body: String::from(text),
                            stale,
                        }),
                    },
                });
                events.push(done(waiter.id, status, None));
            } else {
                events.push(done(waiter.id, status, Some(String::from(text))));
            }
        }
    }

    /// Hand the same no answer to everyone waiting.
    fn fail_waiters(&mut self, key: &str, reason: NoAnswerReason, events: &mut Vec<Event>) {
        for waiter in self.cache.waiting.remove(key).unwrap_or_default() {
            events.push(Event {
                id: Some(waiter.id),
                body: EventBody::HttpFailed { reason },
            });
        }
    }

    fn read_kept(&mut self, key: String, stale: bool, out: &mut Merged) -> bool {
        let Some(store_key) = self
            .cache
            .index
            .entries
            .get(&key)
            .filter(|_| self.durable)
            .map(|entry| entry.store_key.clone())
        else {
            return false;
        };
        self.store(StoreOp::Get, store_key, None, Own::Body { key, stale }, out);
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn fetch_done(
        &mut self,
        mono_ms: u64,
        key: String,
        scope: Option<String>,
        status: u32,
        headers: BTreeMap<String, String>,
        body: Option<String>,
        events: &mut Vec<Event>,
        out: &mut Merged,
    ) {
        if status == 304 {
            self.cache.renewed(&key, mono_ms, &self.config.boot_id);
            self.write_index(out);
            if self.read_kept(key.clone(), false, out) {
                return;
            }
            self.fail_waiters(&key, NoAnswerReason::Offline, events);
            return;
        }
        if !(200..300).contains(&status) {
            // failures are forgotten and go on as they came
            for waiter in self.cache.waiting.remove(&key).unwrap_or_default() {
                events.push(Event {
                    id: Some(waiter.id),
                    body: EventBody::HttpDone {
                        status,
                        headers: headers.clone(),
                        body: body.clone(),
                    },
                });
            }
            return;
        }
        let Some(text) = body else {
            self.answer_waiters(&key, status, None, false, events, out);
            return;
        };
        let ttl_s = serde_json::from_str::<Ttl>(&text).map_or(0, |ttl| ttl.ttl_s);
        let bytes = u64::try_from(text.len()).unwrap_or(u64::MAX);
        if self.durable && self.make_room(bytes, out) {
            let etag = headers
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("etag"))
                .map(|(_, value)| value.clone());
            let entry = cache::Entry::new(
                u64::from(ttl_s) * 1_000,
                mono_ms,
                &self.config.boot_id,
                etag,
                bytes,
            );
            let store_key = self.cache.stored(key.clone(), scope, entry);
            self.write(store_key, text.clone(), out);
            self.write_index(out);
        }
        self.answer_waiters(&key, status, Some(&text), false, events, out);
    }

    fn fetch_failed(
        &mut self,
        key: String,
        reason: NoAnswerReason,
        events: &mut Vec<Event>,
        out: &mut Merged,
    ) {
        if !self.read_kept(key.clone(), true, out) {
            self.fail_waiters(&key, reason, events);
        }
    }

    fn body_read(
        &mut self,
        key: String,
        stale: bool,
        value: Option<String>,
        events: &mut Vec<Event>,
        out: &mut Merged,
    ) {
        match value {
            Some(text) => self.answer_waiters(&key, 200, Some(&text), stale, events, out),
            None => {
                self.cache.forget(&key);
                self.write_index(out);
                self.fail_waiters(&key, NoAnswerReason::Offline, events);
            }
        }
    }

    fn flush_answer(&mut self, mono_ms: u64, body: &EventBody, out: &mut Merged) {
        match body {
            EventBody::HttpDone { status, body, .. } => {
                let text = body.as_deref().unwrap_or_default();
                let error = serde_json::from_str::<ErrorBody>(text).ok();
                let retry = error.and_then(|error| error.retry_after_s);
                match problem_of_status(*status, retry).map(|problem| problem.next()) {
                    None => match serde_json::from_str::<CommandAnswer>(text) {
                        Ok(answer) => {
                            for key in self.queue.acknowledge(&answer.answers, mono_ms) {
                                self.delete(key, out);
                            }
                        }
                        Err(_) => self.queue.failed(mono_ms),
                    },
                    Some(Next::Retry) => self.queue.failed(mono_ms),
                    Some(Next::Wait { after_ms }) => self.queue.wait(Some(after_ms), mono_ms),
                    // the core refreshes and sends it again under this id
                    Some(Next::SignIn) => {}
                    Some(_) => self.queue.wait(None, mono_ms),
                }
            }
            EventBody::HttpFailed { .. } => self.queue.failed(mono_ms),
            EventBody::Unsupported | EventBody::Cancelled => self.queue.failed(mono_ms),
            _ => {}
        }
    }

    fn pull_answer(
        &mut self,
        scope: &str,
        body: &EventBody,
        events: &mut Vec<Event>,
        out: &mut Merged,
    ) {
        self.pulling = false;
        let EventBody::HttpDone {
            status,
            body: Some(text),
            ..
        } = body
        else {
            return;
        };
        if !(200..300).contains(status) {
            return;
        }
        let Ok(changes) = serde_json::from_str::<Changes>(text) else {
            return;
        };
        // Changes never enter the queue, so a server row cannot echo back.
        if !changes.changes.is_empty() {
            self.cache.mark_stale(scope);
        }
        let moved = self.cache.index.cursors.get(scope) != Some(&changes.cursor.0);
        self.cache
            .index
            .cursors
            .insert(String::from(scope), changes.cursor.0);
        self.write_index(out);
        if changes.has_more && moved && self.scope.as_deref() == Some(scope) {
            self.pull(scope, events);
        }
    }

    fn forget(&mut self, id: Option<&str>) {
        if let Some(id) = id {
            self.own.remove(id);
        }
    }

    /// Handle one event from the host. Events for the core go on `events`.
    fn take_event(
        &mut self,
        input: (u64, u64),
        event: Event,
        events: &mut Vec<Event>,
        flush_after: &mut bool,
        out: &mut Merged,
    ) {
        let mono_ms = input.1;
        let own = event.id.as_deref().and_then(|id| self.own.get(id)).cloned();
        match own {
            Some(Own::Write) => {
                self.forget(event.id.as_deref());
                if matches!(event.body, EventBody::Unsupported) {
                    self.durable = false;
                }
            }
            Some(Own::Body { key, stale }) => {
                self.forget(event.id.as_deref());
                let value = match event.body {
                    EventBody::StoreDone { value, .. } => value,
                    _ => None,
                };
                self.body_read(key, stale, value, events, out);
            }
            Some(Own::Fetch { key, scope }) => {
                self.forget(event.id.as_deref());
                match event.body {
                    EventBody::HttpDone {
                        status,
                        headers,
                        body,
                    } => self.fetch_done(mono_ms, key, scope, status, headers, body, events, out),
                    EventBody::HttpFailed { reason } => {
                        self.fetch_failed(key, reason, events, out);
                    }
                    _ => self.fail_waiters(&key, NoAnswerReason::Cancelled, events),
                }
            }
            Some(Own::Flush) => {
                // After a 401 the core sends it again under this same id.
                if !refused_token(&event.body) {
                    self.forget(event.id.as_deref());
                }
                self.flush_answer(mono_ms, &event.body, out);
                events.push(without_body(event));
            }
            Some(Own::Pull { scope }) => {
                if !refused_token(&event.body) {
                    self.forget(event.id.as_deref());
                    self.pull_answer(&scope, &event.body, events, out);
                }
                events.push(without_body(event));
            }
            Some(_) => {}
            None => match &event.body {
                EventBody::Action {
                    action: Action::SignOut { .. } | Action::EnterProfile { .. },
                } => {
                    // The core saves the position before the session or profile moves, so that
                    // save joins the queue and goes in the last batch of this profile.
                    events.push(Event {
                        id: None,
                        body: EventBody::Action {
                            action: Action::Stop,
                        },
                    });
                    self.pump(input, core::mem::take(events), out);
                    if let Some(flush) = self.flush_event(mono_ms) {
                        events.push(flush);
                    }
                    events.push(event);
                }
                EventBody::Suspend => {
                    *flush_after = true;
                    events.push(event);
                }
                // a network return or a play press cuts any wait
                EventBody::Network { online: true }
                | EventBody::Action {
                    action: Action::Play { .. },
                } => {
                    *flush_after = true;
                    events.push(event);
                }
                EventBody::Visibility { visible: true } => {
                    if let Some(scope) = self.scope.clone() {
                        self.pull(&scope, events);
                    }
                    events.push(event);
                }
                _ => events.push(event),
            },
        }
    }

    fn run(&mut self, input: (u64, u64), incoming: Vec<Event>, out: &mut Merged) {
        let mut events = Vec::new();
        let mut flush_after = false;
        for event in incoming {
            self.take_event(input, event, &mut events, &mut flush_after, out);
        }
        self.pump(input, events, out);
        let mono_ms = input.1;
        if flush_after || self.queue.flush_due(mono_ms) {
            match self.flush_event(mono_ms) {
                Some(flush) => self.pump(input, vec![flush], out),
                None => {
                    if !self.queue.sending() {
                        self.queue.stop_waiting();
                    }
                }
            }
        }
    }
}

const fn refused_token(body: &EventBody) -> bool {
    matches!(body, EventBody::HttpDone { status: 401, .. })
}

/// A good answer to a prepared request is the layer's. The core reads only an error body.
fn without_body(event: Event) -> Event {
    let Event { id, body } = event;
    let body = match body {
        EventBody::HttpDone {
            status, headers, ..
        } if (200..300).contains(&status) => EventBody::HttpDone {
            status,
            headers,
            body: None,
        },
        other => other,
    };
    Event { id, body }
}

impl<S: Step> Step for Layer<S> {
    fn step(&mut self, input: CoreIn) -> CoreOut {
        let times = (input.now_ms, input.mono_ms);
        let mut out = Merged::default();
        if self.phase == Phase::New {
            self.load(&mut out);
        }
        let mut events = input.events;
        if self.phase == Phase::Loading {
            for event in events {
                self.load_step(event, &mut out);
            }
            events = Vec::new();
            if self.phase == Phase::Ready {
                events = core::mem::take(&mut self.waiting);
            }
        }
        if self.phase == Phase::Ready {
            self.run(times, events, &mut out);
        }
        let wake = match (self.inner_wake, self.queue.next_wake()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        CoreOut {
            contract: WireDate::current(),
            tag: CoreOutTag::default(),
            effects: out.effects,
            view: (out.view != View::default()).then_some(out.view),
            next_wake_ms: wake,
            unsupported: out.unsupported,
        }
    }
}
