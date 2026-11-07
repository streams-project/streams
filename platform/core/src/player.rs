//! Owns the play session: walks attempts, keeps the position across any source change, closes
//! before it opens, drops late events by generation and saves on a heartbeat. Drivers stay small
//! and only carry out what comes from here.

use alloc::string::String;
use alloc::vec::Vec;
use contract::command::{Command, FeedbackEvent, SettingKey};
use contract::core_msg::{PlayState, PlayView};
use contract::driver::{DriverCommand, DriverEvent, Track};
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
/// Episodes that may start by themselves before the core asks whether anyone is watching.
const STILL_AFTER: u32 = 3;

/// A skip button hides this long before the end it moves to.
const SKIP_HIDES_EARLY_MS: u64 = 1_000;

/// The credits offer hides this long before the end of the video.
const CREDITS_HIDES_EARLY_MS: u64 = 2_000;

/// The host of an address: the text after `://` up to the first `/`, lower cased.
#[must_use]
pub fn host_of(url: &str) -> String {
    url.split_once("://")
        .and_then(|(_, rest)| rest.split('/').next())
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// The main part of a language tag, so pt-BR meets pt.
fn main_language(tag: &str) -> &str {
    tag.split(['-', '_']).next().unwrap_or(tag)
}

/// The track of a language: the exact tag first, then the same language in any region.
fn track_for<'a>(tracks: &'a [Track], wanted: &str) -> Option<&'a Track> {
    let tagged = |same: &dyn Fn(&str) -> bool| {
        tracks
            .iter()
            .find(|track| track.lang.as_deref().is_some_and(same))
    };
    tagged(&|tag| tag.eq_ignore_ascii_case(wanted))
        .or_else(|| tagged(&|tag| main_language(tag).eq_ignore_ascii_case(main_language(wanted))))
}

fn language_of_track(tracks: &[Track], id: &str) -> Option<String> {
    tracks
        .iter()
        .find(|track| track.id == id)
        .and_then(|track| track.lang.clone())
}

/// Sets a new source to the languages this profile keeps. With no subtitle language kept,
/// subtitles stay off whatever the source marks as its default.
fn first_tracks(play: &Play, audio: &[Track], text: &[Track], tick: &mut Tick) {
    let sound = play
        .audio_default
        .as_deref()
        .and_then(|wanted| track_for(audio, wanted));
    if let Some(track) = sound.filter(|track| !track.selected) {
        tick.driver.push(DriverCommand::SelectAudio {
            track: track.id.clone(),
        });
    }
    let words = play
        .subtitle_default
        .as_deref()
        .and_then(|wanted| track_for(text, wanted));
    let showing = text.iter().find(|track| track.selected);
    if words.map(|track| &track.id) != showing.map(|track| &track.id) {
        tick.driver.push(DriverCommand::SelectText {
            track: words.map(|track| track.id.clone()),
        });
    }
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
    /// The next episode starts by itself.
    pub next: Option<(TitleKey, EpisodeRef)>,
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
    next_episode: Option<EpisodeRef>,
    still_asking: bool,
    /// The languages this profile keeps, and the tracks the driver last reported.
    audio_default: Option<String>,
    subtitle_default: Option<String>,
    audio: Vec<Track>,
    text: Vec<Track>,
    /// False until the tracks of this attempt have been set to what the profile keeps.
    tracks_set: bool,
    autoplay_next: bool,
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
    /// Episodes that started by themselves since the viewer last did anything.
    run: u32,
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
            run: 0,
        }
    }

    /// The serial of the session in use.
    #[must_use]
    pub const fn serial(&self) -> u32 {
        self.serial
    }

    /// True while this request still belongs to the session in use.
    #[must_use]
    pub fn in_use(&self, serial: u32) -> bool {
        self.play
            .as_ref()
            .is_some_and(|play| play.serial == serial && play.state != PlayState::Ended)
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
            next_episode: play.next_episode.clone(),
            still_asking: play.still_asking,
        })
    }

    /// The viewer did something, so the run of episodes that started by themselves is over.
    pub(crate) fn viewer_here(&mut self) {
        self.run = 0;
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
            next_episode: None,
            still_asking: false,
            audio_default: None,
            subtitle_default: None,
            audio: Vec::new(),
            text: Vec::new(),
            tracks_set: false,
            autoplay_next: false,
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
        play.next_episode = answer.next_episode;
        play.autoplay_next = answer.autoplay_next;
        play.audio_default = answer.audio_default;
        play.subtitle_default = answer.subtitle_default;
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
            let (duration_ms, last_episode) = (play.duration_ms, play.next_episode.is_none());
            play.windows.extend(
                windows
                    .into_iter()
                    .filter_map(|window| shown(window, duration_ms, last_episode)),
            );
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
        play.tracks_set = false;
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
                // Paused time does not move, so a stall reported then could only run out.
                if play.frame_seen && !play.paused {
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
                if play.autoplay_next && play.next_episode.is_some() {
                    if self.run + 1 < STILL_AFTER {
                        self.run += 1;
                        tick.next = play.next_episode.clone().map(|next| (play.title, next));
                    } else {
                        play.still_asking = true;
                    }
                }
                self.close(tick);
            }
            DriverEvent::Tracks { audio, text } => {
                if !play.tracks_set && !(audio.is_empty() && text.is_empty()) {
                    play.tracks_set = true;
                    first_tracks(play, &audio, &text, tick);
                }
                play.audio = audio;
                play.text = text;
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

    /// Seek, held to the video's length. A seek starts the picture check again. A seek stalls by
    /// nature, so while the viewer keeps seeking the wait starts again from the latest one, and
    /// a dead source still runs out once they stop.
    pub(crate) fn seek(&mut self, to_ms: u64, mono_ms: u64, deadline_ms: u32, tick: &mut Tick) {
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
        if play.state == PlayState::Stalled {
            play.deadline_mono = Some(mono_ms + u64::from(deadline_ms));
        }
        if open {
            tick.driver.push(DriverCommand::Seek { to_ms });
        }
        save(play, tick);
    }

    /// The viewer's choice is kept for the profile, so the next episode starts the same way.
    pub(crate) fn select_audio(&self, track: String, tick: &mut Tick) {
        if !self.open {
            return;
        }
        let tracks = self.play.as_ref().map_or(&[][..], |play| &play.audio);
        if let Some(language) = language_of_track(tracks, &track) {
            tick.commands.push(Command::Setting {
                key: SettingKey::AudioLanguage,
                value: language,
            });
        }
        tick.driver.push(DriverCommand::SelectAudio { track });
    }

    /// Subtitles turned off are kept as no language.
    pub(crate) fn select_text(&self, track: Option<String>, tick: &mut Tick) {
        if !self.open {
            return;
        }
        let tracks = self.play.as_ref().map_or(&[][..], |play| &play.text);
        let language = match &track {
            Some(id) => language_of_track(tracks, id),
            None => Some(String::new()),
        };
        if let Some(language) = language {
            tick.commands.push(Command::Setting {
                key: SettingKey::SubtitleLanguage,
                value: language,
            });
        }
        tick.driver.push(DriverCommand::SelectText { track });
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

    /// Going to the background or suspending saves where the viewer is. A session that ended
    /// has saved already, and sending that old position again could undo a newer one.
    pub(crate) fn save_now(&self, tick: &mut Tick) {
        if let Some(play) = self
            .play
            .as_ref()
            .filter(|play| play.state != PlayState::Ended)
        {
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

/// The span a screen compares to player time. `None` when nothing is left to show. The
/// credits offer leads to the next episode, so the last episode has none.
fn shown(mut window: TimeWindow, duration_ms: u64, last_episode: bool) -> Option<TimeWindow> {
    match window.kind {
        WindowKind::Recap | WindowKind::Intro => {
            window.seek_to_ms = window.seek_to_ms.or(Some(window.end_ms));
            window.end_ms = window.end_ms.saturating_sub(SKIP_HIDES_EARLY_MS);
        }
        WindowKind::Credits if last_episode => return None,
        WindowKind::Credits => {
            window.end_ms = window
                .end_ms
                .min(duration_ms.saturating_sub(CREDITS_HIDES_EARLY_MS));
        }
        _ => {}
    }
    (window.start_ms < window.end_ms).then_some(window)
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
