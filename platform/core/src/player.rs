//! Owns the play session: walks attempts, keeps the position across any source change, closes
//! before it opens, drops late events by generation and saves on a heartbeat. Drivers stay small
//! and only carry out what comes from here.

use alloc::string::String;
use alloc::vec::Vec;
use contract::command::{Command, FeedbackEvent};
use contract::core_msg::{PlayState, PlayView};
use contract::driver::{DriverCommand, DriverEvent};
use contract::play::{
    AgeCard, Attempt, AttemptKind, MarksRequest, MoreAttempts, PlayAnswer, TimeWindow, WindowKind,
};
use contract::{EpisodeRef, TitleKey};

/// While playing, the position is sent this often.
pub const HEARTBEAT_MS: u64 = 30_000;

/// A position at or under this is not worth sending.
const WORTH_SAVING_MS: u64 = 5_000;

/// No picture rule: time in, then how long by the clock and by the video.
const NO_PICTURE_AFTER_MS: u64 = 2_000;
const NO_PICTURE_WALL_MS: u64 = 3_000;
const NO_PICTURE_MEDIA_MS: u64 = 2_500;

/// How long the age card stays up.
const AGE_CARD_MS: u64 = 5_000;

/// The host of an address: the text after `://` up to the first `/`, lower cased.
#[must_use]
pub fn host_of(url: &str) -> String {
    url.split_once("://")
        .and_then(|(_, rest)| rest.split('/').next())
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// The attempt to try after `failed`: the next on another host, skipping other streams of the
/// same group when a stream failed. Embeds of that group stay eligible. No wrap around.
#[must_use]
pub fn next_attempt(attempts: &[Attempt], failed: usize) -> Option<usize> {
    let bad = attempts.get(failed)?;
    let bad_host = host_of(&bad.url);
    let bad_stream = bad.kind != AttemptKind::Embed;
    (failed + 1..attempts.len()).find(|&index| {
        let next = &attempts[index];
        let same_group_stream =
            bad_stream && next.kind != AttemptKind::Embed && next.group == bad.group;
        host_of(&next.url) != bad_host && !same_group_stream
    })
}

/// What a player call asks the rest of the core to do.
#[derive(Default)]
pub(crate) struct Tick {
    pub driver: Vec<DriverCommand>,
    pub commands: Vec<Command>,
    /// Ask for late attempts of this play id.
    pub more: Option<String>,
    /// Ask for skip marks of this play id.
    pub marks: Option<(String, MarksRequest)>,
    pub view: bool,
    pub failed: bool,
}

struct Play {
    serial: u32,
    title: TitleKey,
    episode: Option<EpisodeRef>,
    play_id: Option<String>,
    attempts: Vec<Attempt>,
    active: Option<usize>,
    more: bool,
    age_card: Option<AgeCard>,
    state: PlayState,
    /// Kept across every source change.
    position_ms: u64,
    duration_ms: u64,
    paused: bool,
    /// True once this generation showed a frame.
    frame_seen: bool,
    /// True once any generation of this play showed a frame.
    started: bool,
    marks_asked: bool,
    deadline_mono: Option<u64>,
    heartbeat_mono: Option<u64>,
    no_picture_since: Option<(u64, u64)>,
    windows: Vec<TimeWindow>,
}

/// One play session at a time.
pub struct Player {
    play: Option<Play>,
    /// Never goes back, so a late event from any earlier open is dropped.
    generation: u32,
    serial: u32,
    /// True while a driver holds an open source, so close comes before open.
    open: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            play: None,
            generation: 0,
            serial: 0,
            open: false,
        }
    }

    /// The serial of the session in use.
    #[must_use]
    pub const fn serial(&self) -> u32 {
        self.serial
    }

    /// True while a frame is on screen and the viewer has not paused.
    #[must_use]
    pub fn watching(&self) -> bool {
        self.play
            .as_ref()
            .is_some_and(|play| play.state == PlayState::Playing)
    }

    #[must_use]
    pub fn view(&self) -> Option<PlayView> {
        let play = self.play.as_ref()?;
        Some(PlayView {
            state: play.state,
            title: play.title,
            episode: play.episode.clone(),
            attempt: play
                .active
                .and_then(|index| play.attempts.get(index))
                .map(|attempt| attempt.id.clone()),
            windows: play.windows.clone(),
        })
    }

    /// Start a new session. A new resolve means the same address attaches again.
    pub(crate) fn start(
        &mut self,
        title: TitleKey,
        episode: Option<EpisodeRef>,
        tick: &mut Tick,
    ) -> u32 {
        self.leave(tick);
        self.serial += 1;
        self.play = Some(Play {
            serial: self.serial,
            title,
            episode,
            play_id: None,
            attempts: Vec::new(),
            active: None,
            more: false,
            age_card: None,
            state: PlayState::Resolving,
            position_ms: 0,
            duration_ms: 0,
            paused: false,
            frame_seen: false,
            started: false,
            marks_asked: false,
            deadline_mono: None,
            heartbeat_mono: None,
            no_picture_since: None,
            windows: Vec::new(),
        });
        tick.view = true;
        self.serial
    }

    /// Save, close and forget the session in use.
    fn leave(&mut self, tick: &mut Tick) {
        if let Some(play) = self
            .play
            .as_ref()
            .filter(|play| play.state != PlayState::Ended)
        {
            save(play, tick);
        }
        self.close(tick);
        self.play = None;
    }

    fn close(&mut self, tick: &mut Tick) {
        if self.open {
            tick.driver.push(DriverCommand::Close);
            self.open = false;
        }
    }

    fn current(&mut self, serial: u32) -> Option<&mut Play> {
        self.play
            .as_mut()
            .filter(|play| play.serial == serial && play.state != PlayState::Ended)
    }

    /// The server's answer to a play request.
    pub(crate) fn answer(
        &mut self,
        serial: u32,
        answer: PlayAnswer,
        mono_ms: u64,
        deadline_ms: u32,
        tick: &mut Tick,
    ) {
        let Some(play) = self.current(serial) else {
            return;
        };
        play.play_id = Some(answer.play_id);
        play.attempts = answer.attempts;
        play.position_ms = answer.start_at_ms;
        play.more = answer.more;
        play.age_card = answer.age_card;
        if play.attempts.is_empty() {
            self.out_of_attempts(tick);
        } else {
            self.open_attempt(0, PlayState::Starting, mono_ms, deadline_ms, tick);
        }
    }

    /// Late attempts. Walking goes on from the attempt that failed.
    pub(crate) fn more(
        &mut self,
        serial: u32,
        answer: MoreAttempts,
        mono_ms: u64,
        deadline_ms: u32,
        tick: &mut Tick,
    ) {
        let Some(play) = self.current(serial) else {
            return;
        };
        let before = play.attempts.len();
        play.attempts.extend(answer.attempts);
        play.more = answer.more;
        let next = match play.active {
            Some(active) => next_attempt(&play.attempts, active),
            None => (before < play.attempts.len()).then_some(0),
        };
        match next {
            Some(index) => {
                self.open_attempt(index, PlayState::Switching, mono_ms, deadline_ms, tick)
            }
            None => self.out_of_attempts(tick),
        }
    }

    /// The play request or a request for more failed.
    pub(crate) fn request_failed(&mut self, serial: u32, tick: &mut Tick) {
        if let Some(play) = self.current(serial) {
            play.more = false;
            self.give_up(tick);
        }
    }

    /// The server's skip marks arrived. Nothing ticks across the boundary: the host compares
    /// player time to these windows.
    pub(crate) fn marks(&mut self, serial: u32, windows: Vec<TimeWindow>, tick: &mut Tick) {
        if let Some(play) = self.current(serial) {
            play.windows.extend(windows);
            tick.view = true;
        }
    }

    /// Close first, then open with a new generation, at the kept position.
    fn open_attempt(
        &mut self,
        index: usize,
        state: PlayState,
        mono_ms: u64,
        deadline_ms: u32,
        tick: &mut Tick,
    ) {
        self.close(tick);
        self.generation += 1;
        let generation = self.generation;
        let Some(play) = self.play.as_mut() else {
            return;
        };
        let Some(attempt) = play.attempts.get(index) else {
            return;
        };
        tick.driver.push(DriverCommand::Open {
            url: attempt.url.clone(),
            headers: attempt.headers.clone(),
            start_at_ms: play.position_ms,
            generation,
            max_height: None,
            max_bitrate: None,
        });
        let paused = play.paused;
        play.active = Some(index);
        play.state = state;
        play.frame_seen = false;
        play.heartbeat_mono = None;
        play.no_picture_since = None;
        play.deadline_mono = Some(mono_ms + u64::from(deadline_ms));
        self.open = true;
        if paused {
            tick.driver.push(DriverCommand::Pause);
        }
        tick.view = true;
    }

    /// The source in use failed or ran out of time.
    fn failover(&mut self, mono_ms: u64, deadline_ms: u32, tick: &mut Tick) {
        let Some(play) = self.play.as_ref() else {
            return;
        };
        match play
            .active
            .and_then(|active| next_attempt(&play.attempts, active))
        {
            Some(index) => {
                self.open_attempt(index, PlayState::Switching, mono_ms, deadline_ms, tick)
            }
            None => self.out_of_attempts(tick),
        }
    }

    fn out_of_attempts(&mut self, tick: &mut Tick) {
        let Some(play) = self.play.as_mut() else {
            return;
        };
        if play.more {
            play.more = false;
            play.state = PlayState::Switching;
            play.deadline_mono = None;
            tick.more.clone_from(&play.play_id);
            tick.view = true;
            self.close(tick);
        } else {
            self.give_up(tick);
        }
    }

    fn give_up(&mut self, tick: &mut Tick) {
        self.close(tick);
        if let Some(play) = self.play.as_mut() {
            play.state = PlayState::Failed;
            play.deadline_mono = None;
            play.heartbeat_mono = None;
            tick.view = true;
            tick.failed = true;
        }
    }

    /// A driver event. One from any generation but the newest is dropped.
    pub(crate) fn driver(
        &mut self,
        generation: u32,
        event: DriverEvent,
        mono_ms: u64,
        deadline_ms: u32,
        tick: &mut Tick,
    ) {
        if generation != self.generation || !self.open {
            return;
        }
        let Some(play) = self.play.as_mut() else {
            return;
        };
        match event {
            DriverEvent::FirstFrame => play.first_frame(mono_ms, tick),
            DriverEvent::Position {
                time_ms,
                duration_ms,
                video_width,
                has_picture,
            } => {
                let lost = match (video_width, has_picture) {
                    (Some(width), _) => Some(width == 0),
                    (None, Some(picture)) => Some(!picture),
                    (None, None) => None,
                };
                play.position(time_ms, duration_ms, mono_ms, tick);
                if play.no_picture_run_out(time_ms, lost == Some(true), mono_ms) {
                    let next = play
                        .active
                        .and_then(|active| next_attempt(&play.attempts, active));
                    // With nowhere to go, the sound keeps playing.
                    if let Some(index) = next {
                        self.open_attempt(index, PlayState::Switching, mono_ms, deadline_ms, tick);
                    }
                }
            }
            DriverEvent::Stalled => {
                if play.frame_seen {
                    play.state = PlayState::Stalled;
                    play.heartbeat_mono = None;
                    play.deadline_mono = Some(mono_ms + u64::from(deadline_ms));
                    tick.view = true;
                }
            }
            DriverEvent::Ended => {
                play.position_ms = play.position_ms.max(play.duration_ms);
                play.state = PlayState::Ended;
                play.deadline_mono = None;
                play.heartbeat_mono = None;
                save(play, tick);
                tick.view = true;
                self.close(tick);
            }
            DriverEvent::Error { .. } => self.failover(mono_ms, deadline_ms, tick),
            _ => {}
        }
    }

    /// Deadlines and the heartbeat.
    pub(crate) fn tick(&mut self, mono_ms: u64, deadline_ms: u32, tick: &mut Tick) {
        let Some(play) = self.play.as_mut() else {
            return;
        };
        if play.heartbeat_mono.is_some_and(|at| mono_ms >= at) {
            play.heartbeat_mono = Some(mono_ms + HEARTBEAT_MS);
            save(play, tick);
        }
        if play.deadline_mono.is_some_and(|at| mono_ms >= at) {
            play.deadline_mono = None;
            self.failover(mono_ms, deadline_ms, tick);
        }
    }

    #[must_use]
    pub fn next_wake(&self) -> Option<u64> {
        let play = self.play.as_ref()?;
        match (play.deadline_mono, play.heartbeat_mono) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }

    pub(crate) fn pause(&mut self, tick: &mut Tick) {
        let open = self.open;
        let Some(play) = self.play.as_mut() else {
            return;
        };
        if play.paused || play.state == PlayState::Ended {
            return;
        }
        play.paused = true;
        if open {
            tick.driver.push(DriverCommand::Pause);
        }
        if matches!(play.state, PlayState::Playing | PlayState::Stalled) {
            play.state = PlayState::Paused;
            play.heartbeat_mono = None;
            play.deadline_mono = None;
            tick.view = true;
        }
        save(play, tick);
    }

    pub(crate) fn unpause(&mut self, mono_ms: u64, tick: &mut Tick) {
        let open = self.open;
        let Some(play) = self.play.as_mut() else {
            return;
        };
        if !play.paused {
            return;
        }
        play.paused = false;
        if open {
            tick.driver.push(DriverCommand::Play);
        }
        if play.state == PlayState::Paused {
            play.state = PlayState::Playing;
            play.heartbeat_mono = Some(mono_ms + HEARTBEAT_MS);
            tick.view = true;
        }
    }

    /// Seek, held to the video's length. A seek starts the picture check again.
    pub(crate) fn seek(&mut self, to_ms: u64, tick: &mut Tick) {
        let open = self.open;
        let Some(play) = self
            .play
            .as_mut()
            .filter(|play| play.state != PlayState::Ended)
        else {
            return;
        };
        let to_ms = if play.duration_ms > 0 {
            to_ms.min(play.duration_ms)
        } else {
            to_ms
        };
        play.position_ms = to_ms;
        play.no_picture_since = None;
        if open {
            tick.driver.push(DriverCommand::Seek { to_ms });
        }
        save(play, tick);
    }

    pub(crate) fn select_audio(&self, track: String, tick: &mut Tick) {
        if self.open {
            tick.driver.push(DriverCommand::SelectAudio { track });
        }
    }

    pub(crate) fn select_text(&self, track: Option<String>, tick: &mut Tick) {
        if self.open {
            tick.driver.push(DriverCommand::SelectText { track });
        }
    }

    /// Exit: save, close and show the end. The session stays for its last view.
    pub(crate) fn stop(&mut self, tick: &mut Tick) {
        if let Some(play) = self
            .play
            .as_ref()
            .filter(|play| play.state != PlayState::Ended)
        {
            save(play, tick);
        }
        self.close(tick);
        if let Some(play) = self.play.as_mut() {
            play.state = PlayState::Ended;
            play.deadline_mono = None;
            play.heartbeat_mono = None;
            tick.view = true;
        }
    }

    /// Going to the background or suspending saves where the viewer is.
    pub(crate) fn save_now(&self, tick: &mut Tick) {
        if let Some(play) = &self.play {
            save(play, tick);
        }
    }

    /// The session ended. Nothing is saved, because no token is left to send it with.
    pub(crate) fn drop_session(&mut self, tick: &mut Tick) {
        self.close(tick);
        if let Some(play) = self.play.as_mut() {
            play.state = PlayState::Ended;
            play.deadline_mono = None;
            play.heartbeat_mono = None;
            tick.view = true;
        }
    }
}

fn save(play: &Play, tick: &mut Tick) {
    if play.started && play.position_ms > WORTH_SAVING_MS {
        tick.commands.push(Command::Position {
            title: play.title,
            episode: play.episode.clone(),
            position_s: seconds(play.position_ms),
            duration_s: seconds(play.duration_ms),
        });
    }
}

fn seconds(ms: u64) -> u32 {
    u32::try_from(ms / 1_000).unwrap_or(u32::MAX)
}

impl Play {
    fn first_frame(&mut self, mono_ms: u64, tick: &mut Tick) {
        let first_of_play = !self.started;
        self.frame_seen = true;
        self.started = true;
        self.deadline_mono = None;
        if self.paused {
            self.state = PlayState::Paused;
        } else {
            self.state = PlayState::Playing;
            self.heartbeat_mono = Some(mono_ms + HEARTBEAT_MS);
        }
        tick.view = true;
        if first_of_play {
            tick.commands.push(Command::Feedback {
                title: self.title,
                event: FeedbackEvent::Start,
            });
            if let Some(card) = self.age_card.take() {
                self.windows.push(TimeWindow {
                    kind: WindowKind::AgeCard,
                    start_ms: self.position_ms,
                    end_ms: self.position_ms + AGE_CARD_MS,
                    seek_to_ms: None,
                    text: Some(card.label),
                });
            }
        }
    }

    fn position(&mut self, time_ms: u64, duration_ms: Option<u64>, mono_ms: u64, tick: &mut Tick) {
        // Before a frame shows, a driver may still report 0, and that must not replace the
        // kept position.
        if !self.frame_seen {
            return;
        }
        if self.state == PlayState::Stalled && time_ms > self.position_ms {
            self.state = PlayState::Playing;
            self.deadline_mono = None;
            self.heartbeat_mono = Some(mono_ms + HEARTBEAT_MS);
            tick.view = true;
        }
        self.position_ms = time_ms;
        if let Some(duration) = duration_ms.filter(|duration| *duration > 0) {
            self.duration_ms = duration;
            self.ask_marks(tick);
        }
    }

    /// Skip marks wait for the first frame and a known length, series only.
    fn ask_marks(&mut self, tick: &mut Tick) {
        if self.marks_asked || self.episode.is_none() {
            return;
        }
        let (Some(play_id), Some(attempt)) = (
            self.play_id.clone(),
            self.active.and_then(|index| self.attempts.get(index)),
        ) else {
            return;
        };
        self.marks_asked = true;
        tick.marks = Some((
            play_id,
            MarksRequest {
                attempt: attempt.id.clone(),
                file: None,
                duration_ms: self.duration_ms,
                delay_ms: 0,
            },
        ));
    }

    /// True when sound has played with no picture for long enough.
    fn no_picture_run_out(&mut self, time_ms: u64, lost: bool, mono_ms: u64) -> bool {
        let watching = lost
            && self.frame_seen
            && !self.paused
            && self.state == PlayState::Playing
            && time_ms > NO_PICTURE_AFTER_MS;
        if !watching {
            self.no_picture_since = None;
            return false;
        }
        let (at, from) = *self.no_picture_since.get_or_insert((mono_ms, time_ms));
        let run_out = mono_ms.saturating_sub(at) > NO_PICTURE_WALL_MS
            && time_ms.saturating_sub(from) > NO_PICTURE_MEDIA_MS;
        if run_out {
            self.no_picture_since = None;
        }
        run_out
    }
}
