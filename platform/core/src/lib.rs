#![no_std]
#![forbid(unsafe_code)]
//! The core: decides, never draws. No network, disk, screen, clock, key or address of its own.

extern crate alloc;

pub mod config;
pub mod gate;
pub mod net;
mod pairing;
pub mod player;
pub mod routes;
mod session;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use contract::TitleKey;
use contract::account::{
    CodeSend, CodeSignIn, DeviceCardAnswer, DeviceCardRequest, DeviceCode, DeviceCodeRequest,
    DeviceTokenAnswer, DeviceTokenRequest, PasswordSignIn, ProfileEnter, ProfileEntered,
    ProfileList, RefreshRequest, SignOut, SignedIn,
};
use contract::command::{Command, CommandBatch, CommandOp};
use contract::core_msg::{
    Action, CoreConfig, CoreIn, CoreOut, CoreOutTag, Effect, EffectBody, EndReason, Event,
    EventBody, GateView, HttpMethod, LogLevel, NoAnswerReason, RouteTarget, SecretOp, SessionState,
    Target, View,
};
use contract::error::{ErrorBody, ErrorCode, Notice};
use contract::hello::Hello;
use contract::play::{Marks, MoreAttempts, PlayAnswer, PlayRequest};
use contract::screen::ScreenName;
use contract::version::WireDate;

use config::{Config, Standing};
use gate::Gate;
use net::{Backoff, Held, Net, Pending, Problem, Purpose, Request};
use pairing::{Due, Pairing};
use player::{Player, Tick};
use routes::{Who, decide, safe_next, target};
use session::{DEVICE_SECRET, Kept, SESSION_SECRET, Session};

/// The two times every input carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Now {
    pub wall_ms: u64,
    pub mono_ms: u64,
}

/// One step of the message shape. The core takes it, and anything that wraps the core takes the
/// same shape, so a host needs no second path.
pub trait Step {
    fn step(&mut self, input: CoreIn) -> CoreOut;
}

/// A message that could not be read or written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadMessage;

/// Step with JSON in and JSON out.
///
/// # Errors
/// An in message that does not read. State is unchanged.
pub fn step_json<S: Step>(stepper: &mut S, input: &str) -> Result<String, BadMessage> {
    let input: CoreIn = serde_json::from_str(input).map_err(|_| BadMessage)?;
    serde_json::to_string(&stepper.step(input)).map_err(|_| BadMessage)
}

#[derive(Default)]
struct Out {
    effects: Vec<Effect>,
    view: View,
    unsupported: Vec<String>,
}

impl Out {
    fn push(&mut self, id: String, body: EffectBody) {
        self.effects.push(Effect { id, body });
    }
}

#[derive(Clone, Copy)]
enum SecretWait {
    Session,
    Device,
    Write,
}

/// All state of one running core, with one owner.
pub struct Core {
    config: Config,
    net: Net,
    session: Session,
    pairing: Pairing,
    player: Player,
    gate: Gate,
    started: bool,
    secrets: BTreeMap<String, SecretWait>,
    secret_store: bool,
    hello_backoff: Backoff,
    /// Monotonic time of the next read of hello.
    hello_next_mono: Option<u64>,
    /// True while that read is a retry, which a network return or a play press cuts short.
    hello_failing: bool,
    upgrade: bool,
    /// The address asked for, kept across sign in and the profile picker.
    asked: Option<String>,
    route: Option<RouteTarget>,
    nav_loaded: bool,
    visible: bool,
    run: String,
    next_op: u64,
}

impl Core {
    /// Start from the config. The seed is the only random input this core ever gets.
    ///
    /// # Errors
    /// A seed that is not at least 32 hex digits.
    pub fn open(start: CoreConfig) -> Result<Self, BadMessage> {
        let seed = start.seed.as_str();
        if seed.len() < 32 || !seed.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(BadMessage);
        }
        let run = session::run_name(seed);
        let session = Session::new(seed);
        let config = Config::new(start);
        Ok(Self {
            net: Net::new(&config),
            gate: Gate::new(config.form()),
            config,
            session,
            pairing: Pairing::new(),
            player: Player::new(),
            started: false,
            secrets: BTreeMap::new(),
            secret_store: true,
            hello_backoff: Backoff::default(),
            hello_next_mono: None,
            hello_failing: false,
            upgrade: false,
            asked: None,
            route: None,
            nav_loaded: false,
            visible: true,
            run,
            next_op: 0,
        })
    }

    /// Start from a JSON config.
    ///
    /// # Errors
    /// A config that does not read, or a bad seed.
    pub fn open_json(start: &str) -> Result<Self, BadMessage> {
        Self::open(serde_json::from_str(start).map_err(|_| BadMessage)?)
    }

    fn secret(
        &mut self,
        op: SecretOp,
        name: &str,
        value: Option<String>,
        wait: SecretWait,
        out: &mut Out,
    ) {
        if !self.secret_store {
            return;
        }
        let id = self.net.new_id();
        self.secrets.insert(id.clone(), wait);
        out.push(
            id,
            EffectBody::Secret {
                op,
                name: name.to_string(),
                value,
            },
        );
    }

    fn keep_session(&mut self, out: &mut Out) {
        if let Some(text) = self.session.kept_text(self.gate.last_seen()) {
            self.secret(
                SecretOp::Put,
                SESSION_SECRET,
                Some(text),
                SecretWait::Write,
                out,
            );
        }
    }

    fn start(&mut self, now: Now, out: &mut Out) {
        self.gate.seen(now.wall_ms);
        // The device key comes first: a kept session uses it as soon as it is back.
        self.secret(SecretOp::Get, DEVICE_SECRET, None, SecretWait::Device, out);
        self.secret(
            SecretOp::Get,
            SESSION_SECRET,
            None,
            SecretWait::Session,
            out,
        );
        self.send_hello(now, out);
        self.go(now, target(Target::Wait), false, out);
    }

    fn send_hello(&mut self, now: Now, out: &mut Out) {
        self.hello_next_mono = None;
        let request = Request::new(HttpMethod::Get, "/v1/hello", Purpose::Hello);
        self.send(now, request, out);
    }

    /// Send a request, holding it when its token is about to run out.
    fn send(&mut self, now: Now, request: Request, out: &mut Out) {
        let id = self.net.new_id();
        self.send_as(now, id, request, out);
    }

    fn send_as(&mut self, now: Now, id: String, request: Request, out: &mut Out) {
        if request.takes_token() && self.session.must_wait(now) {
            self.net.hold(Held {
                id,
                request,
                retried: false,
            });
            self.refresh(out);
        } else {
            self.emit(id, request, false, out);
        }
    }

    fn emit(&mut self, id: String, request: Request, retried: bool, out: &mut Out) {
        let token = request.takes_token().then(|| self.session.access_token());
        let scope = self.session.scope();
        let body = self.net.effect(&request, token, scope.as_deref());
        out.push(id.clone(), body);
        let serial = self.session.serial();
        self.net.track(
            id,
            Pending {
                request,
                serial,
                retried,
            },
        );
    }

    /// Start the one refresher.
    fn refresh(&mut self, out: &mut Out) {
        if !self.session.has_refresh() || self.session.refreshing() {
            return;
        }
        self.session.start_refresh();
        let body = RefreshRequest {
            refresh_token: self.session.refresh_token().to_string(),
        };
        let Ok(body) = serde_json::to_string(&body) else {
            return;
        };
        let request =
            Request::new(HttpMethod::Post, "/v1/auth/refresh", Purpose::Refresh).with_body(body);
        let id = self.net.new_id();
        self.emit(id, request, false, out);
    }

    fn release_held(&mut self, out: &mut Out) {
        for held in self.net.take_held() {
            self.emit(held.id, held.request, held.retried, out);
        }
    }

    /// The refresh got no usable answer. A request waiting near expiry goes on the token in
    /// hand. One the server already refused waits for a new token, because sending the same
    /// one again could only earn the second 401 that ends a good session.
    fn refresh_missed(&mut self, now: Now, out: &mut Out) {
        self.session.refresh_failed(now);
        for held in self.net.take_held() {
            if held.retried {
                self.net.hold(held);
            } else {
                self.emit(held.id, held.request, false, out);
            }
        }
    }

    /// Where this profile starts. The profile answer named it.
    const fn home(&self) -> Target {
        if self.session.kids() {
            Target::KidsHome
        } else {
            Target::Home
        }
    }

    fn who(&self) -> Who<'_> {
        Who {
            member: self.session.signed_in(),
            accounts_required: self.config.accounts_required(),
            moved: self.config.moved(),
            home: self.home(),
        }
    }

    fn profile_needed(&self, route: &RouteTarget) -> bool {
        let takes_profile = matches!(
            route.target,
            Target::Home | Target::KidsHome | Target::Screen | Target::Watch
        );
        takes_profile && self.session.signed_in() && self.session.profile().is_none()
    }

    /// Open an address. Asked again by a host, it loads again.
    fn open_address(&mut self, now: Now, address: String, force: bool, out: &mut Out) {
        if self.upgrade {
            self.go(now, target(Target::Upgrade), false, out);
            return;
        }
        // Who is asking is not known yet, so nothing is drawn.
        if self.session.state() == SessionState::Starting {
            self.asked = Some(address);
            self.go(now, target(Target::Wait), false, out);
            return;
        }
        let route = decide(&address, &self.who());
        if route.target == Target::SignIn {
            self.asked = route.next.clone().or(self.asked.take());
        } else {
            self.asked = Some(address);
        }
        if self.profile_needed(&route) {
            self.go(now, target(Target::Profiles), force, out);
        } else {
            self.go(now, route, force, out);
        }
    }

    /// Facts changed, so the address asked for is decided again.
    fn settle(&mut self, now: Now, out: &mut Out) {
        if self.session.state() == SessionState::Starting {
            return;
        }
        // Facts that arrive later do not move a visitor off the sign in screen.
        let signing_in = self
            .route
            .as_ref()
            .is_some_and(|r| r.target == Target::SignIn);
        if signing_in && !self.session.signed_in() {
            return;
        }
        let address = self.asked.clone().unwrap_or_else(|| "/".to_string());
        self.open_address(now, address, false, out);
    }

    fn go(&mut self, now: Now, route: RouteTarget, force: bool, out: &mut Out) {
        // the upgrade screen shows no code, so nothing is left to poll for
        if route.target == Target::Upgrade {
            self.pairing.stop();
        }
        let same = self.route.as_ref() == Some(&route);
        if same && !force {
            return;
        }
        if !same {
            self.route = Some(route.clone());
            out.view.route = Some(route.clone());
            let id = self.net.new_id();
            out.push(
                id,
                EffectBody::Open {
                    target: route.clone(),
                },
            );
        }
        match route.target {
            Target::Home => self.load_screen(now, ScreenName::Home, None, out),
            Target::KidsHome => self.load_screen(now, ScreenName::Kids, None, out),
            Target::Screen => {
                if let Some(name) = route.screen {
                    self.load_screen(now, name, route.genre.as_deref(), out);
                }
            }
            Target::Title => {
                if let Some(title) = route.title {
                    self.load_title(now, title, out);
                }
            }
            Target::Watch => {
                if let Some(title) = route.title {
                    self.play(now, title, route.episode, out);
                }
            }
            _ => {}
        }
    }

    fn load_screen(&mut self, now: Now, name: ScreenName, genre: Option<&str>, out: &mut Out) {
        // With accounts off a visitor has screens too, and the server answers for its tier.
        if !self.session.signed_in() && self.config.accounts_required() != Some(false) {
            return;
        }
        if !self.nav_loaded {
            self.nav_loaded = true;
            let request = Request::new(HttpMethod::Get, "/v1/nav", Purpose::Page).for_screen();
            self.send(now, request, out);
        }
        // These have no page of rows. Search reads once the viewer has typed, and the others
        // read their own routes.
        if matches!(
            name,
            ScreenName::Search | ScreenName::Help | ScreenName::Settings | ScreenName::Games
        ) {
            return;
        }
        let Some(word) = screen_word(name) else {
            return;
        };
        let path = genre.map_or_else(
            || format!("/v1/screens/{word}"),
            |genre| format!("/v1/screens/{word}?genre={genre}"),
        );
        let request = Request::new(HttpMethod::Get, path, Purpose::Page).for_screen();
        self.send(now, request, out);
    }

    fn load_title(&mut self, now: Now, title: TitleKey, out: &mut Out) {
        let (kind, id) = (title.kind().word(), title.id());
        let path = if self.session.profile().is_some() {
            format!("/v1/titles/{kind}/{id}")
        } else {
            format!("/v1/public/titles/{kind}/{id}")
        };
        let request = Request::new(HttpMethod::Get, path, Purpose::Page).for_screen();
        self.send(now, request, out);
    }

    fn notice(&mut self, notice: Notice, out: &mut Out) {
        let id = self.net.new_id();
        out.push(
            id,
            EffectBody::View {
                notice: Some(notice),
                body: None,
            },
        );
    }

    fn label_notice(&mut self, label: &str, out: &mut Out) {
        self.notice(
            Notice {
                label: label.to_string(),
                text: String::new(),
                action_label: None,
                action_address: None,
            },
            out,
        );
    }

    fn log(&mut self, level: LogLevel, code: &str, out: &mut Out) {
        let id = self.net.new_id();
        out.push(
            id,
            EffectBody::Log {
                level,
                code: code.to_string(),
                detail: None,
            },
        );
    }

    fn command(&mut self, now: Now, command: Command, out: &mut Out) {
        if !self.session.signed_in() || self.session.profile().is_none() {
            return;
        }
        self.next_op += 1;
        let op = CommandOp {
            op_id: format!("{}-{}", self.run, self.next_op),
            age_ms: Some(0),
            command,
        };
        let Ok(body) = serde_json::to_string(&CommandBatch { ops: vec![op] }) else {
            return;
        };
        let request =
            Request::new(HttpMethod::Post, "/v1/commands", Purpose::Command).with_body(body);
        self.send(now, request, out);
    }

    fn run_tick(&mut self, now: Now, tick: Tick, out: &mut Out) {
        for command in tick.driver {
            let id = self.net.new_id();
            out.push(id, EffectBody::Player { command });
        }
        for command in tick.commands {
            self.command(now, command, out);
        }
        if let Some(play_id) = tick.more {
            let path = format!("/v1/play/{play_id}/more");
            let serial = self.player.serial();
            self.send(
                now,
                Request::new(HttpMethod::Post, path, Purpose::More(serial)),
                out,
            );
        }
        if let Some((play_id, Ok(body))) = tick
            .marks
            .map(|(play_id, marks)| (play_id, serde_json::to_string(&marks)))
        {
            let path = format!("/v1/play/{play_id}/marks");
            let purpose = Purpose::Marks(self.player.serial());
            let request = Request::new(HttpMethod::Post, path, purpose).with_body(body);
            self.send(now, request, out);
        }
        if tick.failed {
            self.label_notice("play-failed", out);
        }
        if tick.view {
            out.view.play = self.player.view();
        }
        if let Some((title, next)) = tick.next {
            let (kind, id) = (title.kind().word(), title.id());
            let address = format!("/watch/{kind}/{id}/{}/{}", next.season, next.episode);
            self.open_address(now, address, false, out);
        }
    }

    fn play(
        &mut self,
        now: Now,
        title: TitleKey,
        episode: Option<contract::EpisodeRef>,
        out: &mut Out,
    ) {
        let mut tick = Tick::default();
        let serial = self.player.start(title, episode.clone(), &mut tick);
        if !self.config.playback() {
            self.player.request_failed(serial, &mut tick);
            self.run_tick(now, tick, out);
            return;
        }
        self.run_tick(now, tick, out);
        let Ok(body) = serde_json::to_string(&PlayRequest { title, episode }) else {
            return;
        };
        let request =
            Request::new(HttpMethod::Post, "/v1/play", Purpose::Play(serial)).with_body(body);
        self.send(now, request, out);
    }

    fn end(&mut self, now: Now, reason: EndReason, out: &mut Out) {
        let mut tick = Tick::default();
        self.player.drop_session(&mut tick);
        self.run_tick(now, tick, out);
        let here = self.asked.take();
        self.session.end(reason);
        self.net.forget_session();
        self.gate.clear();
        self.nav_loaded = false;
        self.secret(
            SecretOp::Delete,
            SESSION_SECRET,
            None,
            SecretWait::Write,
            out,
        );
        out.view.session = Some(self.session.view());
        out.view.gate = Some(GateView { ask: false });
        out.view.profiles = Some(ProfileList {
            profiles: Vec::new(),
        });
        let mut route = match reason {
            EndReason::Everywhere | EndReason::Refused => target(Target::SignIn),
            _ => target(Target::Start),
        };
        if reason == EndReason::Refused {
            route.next = here.as_deref().and_then(safe_next);
            self.asked.clone_from(&route.next);
        }
        self.go(now, route, true, out);
    }

    fn sign_out(&mut self, now: Now, everywhere: bool, out: &mut Out) {
        if !self.session.signed_in() {
            return;
        }
        let mut tick = Tick::default();
        self.player.stop(&mut tick);
        self.run_tick(now, tick, out);
        if let Ok(body) = serde_json::to_string(&SignOut { everywhere }) {
            let request = Request::new(HttpMethod::Post, "/v1/auth/signout", Purpose::SignOut)
                .with_body(body);
            let id = self.net.new_id();
            self.emit(id, request, false, out);
        }
        let reason = if everywhere {
            EndReason::Everywhere
        } else {
            EndReason::SignedOut
        };
        self.end(now, reason, out);
    }

    fn after_tokens(&mut self, now: Now, out: &mut Out) {
        out.view.session = Some(self.session.view());
        self.keep_session(out);
        self.release_held(out);
        self.send(
            now,
            Request::new(HttpMethod::Get, "/v1/profiles", Purpose::Profiles),
            out,
        );
        self.send_card(now, false, out);
    }

    fn send_card(&mut self, now: Now, whole: bool, out: &mut Out) {
        let Some(card) = self.config.card() else {
            return;
        };
        let Ok(text) = serde_json::to_string(card) else {
            return;
        };
        let body = DeviceCardRequest {
            device_key: self.session.device_key().to_string(),
            card_hash: format!("{:016x}", fnv(text.as_bytes())),
            card: whole.then(|| card.clone()),
        };
        if let Ok(body) = serde_json::to_string(&body) {
            let request =
                Request::new(HttpMethod::Post, "/v1/devices/me", Purpose::Card).with_body(body);
            self.send(now, request, out);
        }
    }

    fn signed_in(&mut self, now: Now, answer: SignedIn, out: &mut Out) {
        self.pairing.stop();
        self.session.accept(answer, now);
        self.after_tokens(now, out);
        self.settle(now, out);
    }

    fn action(&mut self, now: Now, id: Option<String>, action: Action, out: &mut Out) {
        match action {
            Action::SignIn { email, password } => {
                self.post_sign_in(
                    now,
                    "/v1/auth/password",
                    &PasswordSignIn { email, password },
                    out,
                );
            }
            Action::SendCode { email } => {
                if let Ok(body) = serde_json::to_string(&CodeSend { email }) {
                    let request =
                        Request::new(HttpMethod::Post, "/v1/auth/code", Purpose::CodeSent)
                            .with_body(body);
                    self.send(now, request, out);
                }
            }
            Action::SignInCode { email, code } => {
                self.post_sign_in(
                    now,
                    "/v1/auth/code/verify",
                    &CodeSignIn { email, code },
                    out,
                );
            }
            Action::DeviceSignIn { device_name } => {
                if !self.session.signed_in() && !self.upgrade {
                    self.pairing.start(device_name, now.mono_ms);
                }
            }
            Action::DeviceSignInStop => self.pairing.stop(),
            Action::SignOut { everywhere } => self.sign_out(now, everywhere, out),
            Action::EnterProfile { profile, pin } => {
                // nothing plays on into another profile
                let mut tick = Tick::default();
                self.player.stop(&mut tick);
                self.run_tick(now, tick, out);
                if let Ok(body) = serde_json::to_string(&ProfileEnter { pin }) {
                    let path = format!("/v1/profiles/{profile}/enter");
                    let request =
                        Request::new(HttpMethod::Post, path, Purpose::Enter).with_body(body);
                    self.send(now, request, out);
                }
            }
            Action::Play { title, episode } => {
                self.cut_waits(now);
                self.play(now, title, episode, out);
            }
            Action::Command { command } => self.command(now, command, out),
            Action::Request { method, path, body } => {
                if let Some(id) = id.filter(|_| path.starts_with("/v1/")) {
                    let mut request = Request::new(method, path, Purpose::Passed);
                    request.body = body;
                    self.send_as(now, id, request, out);
                }
            }
            other => {
                let mut tick = Tick::default();
                self.player.viewer_here();
                match other {
                    Action::Pause => self.player.pause(&mut tick),
                    Action::Unpause => self.player.unpause(now.mono_ms, &mut tick),
                    Action::Seek { to_ms } => {
                        let deadline = self.config.start_deadline_ms();
                        self.player.seek(to_ms, now.mono_ms, deadline, &mut tick);
                    }
                    Action::SelectAudio { track } => self.player.select_audio(track, &mut tick),
                    Action::SelectText { track } => self.player.select_text(track, &mut tick),
                    Action::Stop => self.player.stop(&mut tick),
                    _ => out.unsupported.extend(id),
                }
                self.run_tick(now, tick, out);
            }
        }
    }

    fn post_sign_in<T: serde::Serialize>(&mut self, now: Now, path: &str, body: &T, out: &mut Out) {
        if let Ok(body) = serde_json::to_string(body) {
            let request = Request::new(HttpMethod::Post, path, Purpose::SignIn).with_body(body);
            self.send(now, request, out);
        }
    }

    fn event(&mut self, now: Now, event: Event, out: &mut Out) {
        let Event { id, body } = event;
        match body {
            EventBody::HttpDone { status, body, .. } => {
                if let Some(pending) = id.as_deref().and_then(|id| self.net.take(id)) {
                    self.http_done(now, id.unwrap_or_default(), pending, status, body, out);
                }
            }
            EventBody::HttpFailed { reason } => {
                if let Some(pending) = id.as_deref().and_then(|id| self.net.take(id)) {
                    self.http_failed(now, pending, reason, out);
                }
            }
            EventBody::Unsupported => self.unsupported(now, id.as_deref(), out),
            EventBody::Cancelled => self.cancelled(now, id.as_deref(), out),
            EventBody::SecretDone { value } => {
                if let Some(wait) = id.as_deref().and_then(|id| self.secrets.remove(id)) {
                    self.secret_done(now, wait, value, out);
                }
            }
            EventBody::Suspend => self.left(now, out),
            EventBody::Resume => {
                self.visible = true;
                self.used(now, out);
            }
            EventBody::Visibility { visible } => {
                self.visible = visible;
                if visible {
                    self.used(now, out);
                } else {
                    self.left(now, out);
                }
            }
            EventBody::Activity => self.used(now, out),
            EventBody::Network { online } => {
                if online {
                    self.cut_waits(now);
                }
            }
            EventBody::EndedElsewhere => {
                if self.session.signed_in() {
                    self.end(now, EndReason::Elsewhere, out);
                }
            }
            EventBody::Address { address } => {
                self.player.viewer_here();
                self.open_address(now, address, true, out);
            }
            EventBody::Action { action } => self.action(now, id, action, out),
            EventBody::Driver { generation, event } => {
                let deadline = self.config.start_deadline_ms();
                let mut tick = Tick::default();
                self.player
                    .driver(generation, event, now.mono_ms, deadline, &mut tick);
                self.run_tick(now, tick, out);
                if self.visible && self.player.watching() {
                    self.used(now, out);
                }
            }
            EventBody::TimerFired | EventBody::StoreDone { .. } => {}
            _ => {
                if let Some(id) = id {
                    out.unsupported.push(id);
                }
            }
        }
    }

    /// The host dropped an effect. A screen's request goes quietly, and what the core itself
    /// is waiting on counts as no answer, so nothing waits forever.
    fn cancelled(&mut self, now: Now, id: Option<&str>, out: &mut Out) {
        let Some(id) = id else {
            return;
        };
        if let Some(wait) = self.secrets.remove(id) {
            self.secret_done(now, wait, None, out);
        } else if let Some(pending) = self.net.take(id) {
            let waited_on = matches!(
                pending.request.purpose,
                Purpose::Hello
                    | Purpose::Refresh
                    | Purpose::Play(_)
                    | Purpose::More(_)
                    | Purpose::DeviceCode(_)
                    | Purpose::DevicePoll(_)
            );
            if waited_on {
                self.http_failed(now, pending, NoAnswerReason::Cancelled, out);
            }
        }
    }

    /// The host cannot do this. With no secret store, tokens stay in memory for this run.
    fn unsupported(&mut self, now: Now, id: Option<&str>, out: &mut Out) {
        let Some(id) = id else {
            return;
        };
        if let Some(wait) = self.secrets.remove(id) {
            self.secret_store = false;
            self.secret_done(now, wait, None, out);
        } else if let Some(pending) = self.net.take(id) {
            self.http_failed(now, pending, NoAnswerReason::Offline, out);
        }
    }

    fn secret_done(&mut self, now: Now, wait: SecretWait, value: Option<String>, out: &mut Out) {
        match wait {
            SecretWait::Session => {
                match value.and_then(|text| serde_json::from_str::<Kept>(&text).ok()) {
                    Some(kept) => {
                        if let Some(seen_ms) = kept.seen_ms {
                            self.gate.restore(seen_ms);
                        }
                        self.session.restore(kept);
                        self.pairing.stop();
                        self.gate.set_active(self.session.profile().is_some());
                        out.view.session = Some(self.session.view());
                        self.refresh(out);
                        self.send(
                            now,
                            Request::new(HttpMethod::Get, "/v1/profiles", Purpose::Profiles),
                            out,
                        );
                        self.send_card(now, false, out);
                    }
                    None => {
                        self.session.none_kept();
                        out.view.session = Some(self.session.view());
                    }
                }
                self.settle(now, out);
            }
            SecretWait::Device => match value.filter(|key| !key.is_empty()) {
                Some(key) => self.session.set_device_key(key),
                None => {
                    let key = self.session.device_key().to_string();
                    self.secret(
                        SecretOp::Put,
                        DEVICE_SECRET,
                        Some(key),
                        SecretWait::Write,
                        out,
                    );
                }
            },
            SecretWait::Write => {}
        }
    }

    fn used(&mut self, now: Now, out: &mut Out) {
        if self.gate.activity(now.wall_ms) {
            out.view.gate = Some(GateView { ask: true });
            // nothing plays behind the picker
            let mut tick = Tick::default();
            self.player.pause(&mut tick);
            self.run_tick(now, tick, out);
        }
        self.keep_seen(out);
    }

    fn left(&mut self, now: Now, out: &mut Out) {
        self.gate.left(now.wall_ms);
        let mut tick = Tick::default();
        self.player.save_now(&mut tick);
        self.run_tick(now, tick, out);
        self.keep_seen(out);
    }

    /// The last activity is written with the session, at most every 30 s.
    fn keep_seen(&mut self, out: &mut Out) {
        if self.gate.take_moved() {
            self.keep_session(out);
        }
    }

    /// A network return or a play press cuts any wait.
    fn cut_waits(&mut self, now: Now) {
        if self.hello_failing {
            self.hello_next_mono = Some(now.mono_ms);
        }
        self.session.cut_wait();
        self.pairing.cut_wait(now.mono_ms);
    }

    fn http_done(
        &mut self,
        now: Now,
        id: String,
        pending: Pending,
        status: u32,
        body: Option<String>,
        out: &mut Out,
    ) {
        let error: Option<ErrorBody> = if (200..300).contains(&status) {
            None
        } else {
            body.as_deref()
                .and_then(|text| serde_json::from_str(text).ok())
        };
        let Pending {
            request,
            serial,
            retried,
        } = pending;
        if status == 401 && request.takes_token() {
            self.unauthorized(now, id, request, serial, retried, out);
            return;
        }
        if error
            .as_ref()
            .is_some_and(|error| error.code == ErrorCode::UpgradeRequired)
        {
            self.upgrade = true;
            self.go(now, target(Target::Upgrade), false, out);
            return;
        }
        let retry_after = error.as_ref().and_then(|error| error.retry_after_s);
        match net::problem_of_status(status, retry_after) {
            None => self.answer(now, request.purpose, body, out),
            Some(problem) => self.refused(now, request.purpose, status, problem, error, out),
        }
    }

    /// A 401 refreshes once and retries once. A second 401 ends the session.
    fn unauthorized(
        &mut self,
        now: Now,
        id: String,
        request: Request,
        serial: u32,
        retried: bool,
        out: &mut Out,
    ) {
        if retried || !self.session.has_refresh() {
            self.end(now, EndReason::Refused, out);
        } else if serial == self.session.serial() {
            self.session.token_refused(now);
            self.net.hold(Held {
                id,
                request,
                retried: true,
            });
            self.refresh(out);
        } else {
            self.emit(id, request, true, out);
        }
    }

    fn answer(&mut self, now: Now, purpose: Purpose, body: Option<String>, out: &mut Out) {
        let text = body.unwrap_or_default();
        match purpose {
            Purpose::Hello => match serde_json::from_str::<Hello>(&text) {
                Ok(hello) => self.hello_read(now, hello, out),
                Err(_) => self.hello_failed(now, Problem::Failed, out),
            },
            Purpose::Refresh => match serde_json::from_str::<SignedIn>(&text) {
                Ok(answer) => {
                    self.session.accept(answer, now);
                    out.view.session = Some(self.session.view());
                    self.keep_session(out);
                    self.release_held(out);
                }
                Err(_) => self.refresh_missed(now, out),
            },
            Purpose::SignIn => match serde_json::from_str::<SignedIn>(&text) {
                Ok(answer) => self.signed_in(now, answer, out),
                Err(_) => self.label_notice(Problem::Failed.label(), out),
            },
            Purpose::DeviceCode(serial) => match serde_json::from_str::<DeviceCode>(&text) {
                Ok(code) => self.pairing.code_read(serial, code, now.mono_ms),
                Err(_) => self.pairing.no_answer(serial, None, now.mono_ms),
            },
            Purpose::DevicePoll(serial) => self.device_polled(now, serial, &text, out),
            Purpose::Profiles => {
                if let Ok(list) = serde_json::from_str::<ProfileList>(&text) {
                    self.gate.set_profiles(list.profiles.len());
                    out.view.profiles = Some(list);
                    // the question needs this count, and after a restart it may be due now
                    self.used(now, out);
                }
            }
            Purpose::Enter => {
                if let Ok(entered) = serde_json::from_str::<ProfileEntered>(&text) {
                    self.entered(now, entered, out);
                }
            }
            Purpose::Card => {
                if serde_json::from_str::<DeviceCardAnswer>(&text).is_ok_and(|a| a.send_card) {
                    self.send_card(now, true, out);
                }
            }
            Purpose::Play(serial) => {
                let deadline = self.config.start_deadline_ms();
                let mut tick = Tick::default();
                match serde_json::from_str::<PlayAnswer>(&text) {
                    Ok(answer) => {
                        self.player
                            .answer(serial, answer, now.mono_ms, deadline, &mut tick);
                    }
                    Err(_) => self.player.request_failed(serial, &mut tick),
                }
                self.run_tick(now, tick, out);
            }
            Purpose::More(serial) => {
                let deadline = self.config.start_deadline_ms();
                let mut tick = Tick::default();
                match serde_json::from_str::<MoreAttempts>(&text) {
                    Ok(answer) => {
                        self.player
                            .more(serial, answer, now.mono_ms, deadline, &mut tick);
                    }
                    Err(_) => self.player.request_failed(serial, &mut tick),
                }
                self.run_tick(now, tick, out);
            }
            Purpose::Marks(serial) => {
                if let Ok(marks) = serde_json::from_str::<Marks>(&text) {
                    let mut tick = Tick::default();
                    self.player.marks(serial, marks.windows, &mut tick);
                    self.run_tick(now, tick, out);
                }
            }
            Purpose::CodeSent
            | Purpose::SignOut
            | Purpose::Page
            | Purpose::Command
            | Purpose::Passed => {}
        }
    }

    /// One answer to a poll. Tokens that arrive after a stop are dropped, because the viewer left
    /// this screen, and the session they opened ends with the server's idle limit.
    fn device_polled(&mut self, now: Now, serial: u32, text: &str, out: &mut Out) {
        match serde_json::from_str::<DeviceTokenAnswer>(text) {
            Ok(DeviceTokenAnswer {
                signed_in: Some(answer),
                ..
            }) => {
                if self.pairing.is_current(serial) {
                    self.signed_in(now, answer, out);
                }
            }
            Ok(DeviceTokenAnswer {
                poll: Some(word), ..
            }) => self.pairing.polled(serial, word, now.mono_ms),
            _ => self.pairing.no_answer(serial, None, now.mono_ms),
        }
    }

    fn entered(&mut self, now: Now, entered: ProfileEntered, out: &mut Out) {
        let from_gate = self.session.profile().is_none() || self.gate.asking();
        let kids = entered.home == ScreenName::Kids;
        self.session.set_profile(entered.profile.id, kids);
        self.nav_loaded = false;
        self.gate.answered(now.wall_ms);
        out.view.gate = Some(GateView { ask: false });
        out.view.session = Some(self.session.view());
        self.keep_session(out);
        let asked = self.asked.clone().filter(|_| from_gate);
        let route = asked.map(|address| decide(&address, &self.who()));
        let mut home = target(self.home());
        match route {
            Some(route)
                if !matches!(
                    route.target,
                    Target::Home
                        | Target::KidsHome
                        | Target::Start
                        | Target::SignIn
                        | Target::Profiles
                ) =>
            {
                self.go(now, route, true, out);
            }
            Some(route) => {
                home.title = route.title;
                self.go(now, home, true, out);
            }
            None => self.go(now, home, true, out),
        }
    }

    /// The server answered no. Pages map its code to a target.
    fn refused(
        &mut self,
        now: Now,
        purpose: Purpose,
        status: u32,
        problem: Problem,
        error: Option<ErrorBody>,
        out: &mut Out,
    ) {
        let code = error
            .as_ref()
            .map_or(ErrorCode::Unknown, |error| error.code);
        match purpose {
            Purpose::Page => match code {
                ErrorCode::ProfileRequired => {
                    self.go(now, target(Target::Profiles), false, out);
                }
                ErrorCode::Refused
                    if self.route.as_ref().map(|r| r.target) != Some(self.home()) =>
                {
                    self.go(now, target(self.home()), false, out);
                }
                ErrorCode::NotFound | ErrorCode::NoLongerAvailable => {
                    self.go(now, target(Target::NotFound), false, out);
                }
                _ if status == 404 => self.go(now, target(Target::NotFound), false, out),
                _ => self.problem(problem, error, out),
            },
            Purpose::Refresh => {
                if (400..=403).contains(&status) {
                    self.end(now, EndReason::Refused, out);
                } else {
                    self.refresh_missed(now, out);
                }
            }
            Purpose::Hello => self.hello_failed(now, problem, out),
            Purpose::Play(serial) | Purpose::More(serial) => {
                let in_use = self.player.in_use(serial);
                let mut tick = Tick::default();
                self.player.request_failed(serial, &mut tick);
                tick.failed = false;
                self.run_tick(now, tick, out);
                if in_use {
                    self.problem(problem, error, out);
                }
            }
            Purpose::Command => self.log(LogLevel::Warn, "command-dropped", out),
            Purpose::DeviceCode(serial) | Purpose::DevicePoll(serial) => match problem {
                _ if code == ErrorCode::Unsupported => {
                    self.pairing.off(serial);
                }
                Problem::Budget { wait_ms } => {
                    self.pairing.no_answer(serial, Some(wait_ms), now.mono_ms);
                }
                Problem::Failed | Problem::SignedOut => {
                    if self.pairing.off(serial) {
                        self.problem(problem, error, out);
                    }
                }
                _ => self.pairing.no_answer(serial, None, now.mono_ms),
            },
            Purpose::SignIn | Purpose::CodeSent | Purpose::Enter | Purpose::Profiles => {
                self.problem(problem, error, out);
            }
            Purpose::SignOut | Purpose::Card | Purpose::Marks(_) | Purpose::Passed => {}
        }
    }

    fn problem(&mut self, problem: Problem, error: Option<ErrorBody>, out: &mut Out) {
        match error {
            Some(ErrorBody {
                notice: Some(notice),
                ..
            }) => self.notice(notice, out),
            Some(error) => self.label_notice(&error.label, out),
            None => self.label_notice(problem.label(), out),
        }
    }

    /// Hold a hello. The screen moves only when an address could decide differently, because
    /// this runs again at every ttl.
    fn hello_read(&mut self, now: Now, hello: Hello, out: &mut Out) {
        let facts = |config: &Config| (config.accounts_required(), config.moved().to_vec());
        let before = facts(&self.config);
        let ttl_ms = u64::from(hello.ttl_s) * 1_000;
        let upgrade = self.config.take_hello(hello) == Standing::Upgrade;
        self.hello_backoff.reset();
        self.hello_failing = false;
        self.hello_next_mono = (ttl_ms > 0).then_some(now.mono_ms + ttl_ms);
        out.view.hello = self.config.hello().cloned();
        let changed = upgrade != self.upgrade || before != facts(&self.config);
        self.upgrade = upgrade;
        if upgrade {
            self.go(now, target(Target::Upgrade), false, out);
        } else if changed {
            self.settle(now, out);
        }
    }

    fn hello_failed(&mut self, now: Now, problem: Problem, out: &mut Out) {
        let before = self.config.accounts_required();
        self.config.unreachable();
        let wait = match problem {
            Problem::Budget { wait_ms } => wait_ms,
            _ => self.hello_backoff.next_wait(),
        };
        self.hello_failing = true;
        self.hello_next_mono = Some(now.mono_ms + wait);
        if before != self.config.accounts_required() {
            self.settle(now, out);
        }
    }

    fn http_failed(&mut self, now: Now, pending: Pending, reason: NoAnswerReason, out: &mut Out) {
        let problem = net::problem_of_failure(reason);
        match pending.request.purpose {
            // a failed command is dropped: the next heartbeat carries the newer position
            Purpose::Command => self.log(LogLevel::Warn, "command-dropped", out),
            purpose => self.refused(now, purpose, 0, problem, None, out),
        }
    }

    fn tick(&mut self, now: Now, out: &mut Out) {
        if self.hello_next_mono.is_some_and(|at| now.mono_ms >= at) {
            self.send_hello(now, out);
        }
        if self.session.renewal_due(now) {
            self.refresh(out);
        }
        self.device_tick(now, out);
        let deadline = self.config.start_deadline_ms();
        let mut tick = Tick::default();
        self.player.tick(now.mono_ms, deadline, &mut tick);
        self.run_tick(now, tick, out);
    }

    fn device_tick(&mut self, now: Now, out: &mut Out) {
        let serial = self.pairing.serial();
        let (path, purpose, body) = match self.pairing.due(now.mono_ms) {
            Some(Due::Code(device_name)) => (
                "/v1/auth/device/code",
                Purpose::DeviceCode(serial),
                serde_json::to_string(&DeviceCodeRequest { device_name }),
            ),
            Some(Due::Token(device_code)) => (
                "/v1/auth/device/token",
                Purpose::DevicePoll(serial),
                serde_json::to_string(&DeviceTokenRequest { device_code }),
            ),
            None => return,
        };
        if let Ok(body) = body {
            let request = Request::new(HttpMethod::Post, path, purpose).with_body(body);
            self.send(now, request, out);
        }
    }

    fn next_wake(&self) -> Option<u64> {
        [
            self.hello_next_mono,
            self.session.next_wake(),
            self.pairing.next_wake(),
            self.player.next_wake(),
        ]
        .into_iter()
        .flatten()
        .min()
    }
}

impl Step for Core {
    fn step(&mut self, input: CoreIn) -> CoreOut {
        let now = Now {
            wall_ms: input.now_ms,
            mono_ms: input.mono_ms,
        };
        let mut out = Out::default();
        if !self.started {
            self.started = true;
            self.start(now, &mut out);
        }
        for event in input.events {
            self.event(now, event, &mut out);
        }
        self.tick(now, &mut out);
        out.view.device_sign_in = self.pairing.take_view();
        CoreOut {
            contract: WireDate::current(),
            tag: CoreOutTag::default(),
            effects: out.effects,
            view: (out.view != View::default()).then_some(out.view),
            next_wake_ms: self.next_wake(),
            unsupported: out.unsupported,
        }
    }
}

fn screen_word(name: ScreenName) -> Option<String> {
    serde_json::to_value(name)
        .ok()
        .and_then(|value| value.as_str().map(ToString::to_string))
}

/// A small stable hash of the card, so a change can be noticed. It proves nothing.
fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
