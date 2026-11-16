//! Playback health: how play went at a host, by client class and day.
//!
//! Nothing here takes a profile, an account or a session, and no function in this module has a
//! parameter one could be passed through. That is the whole design: a rebuffer is a fact about a
//! host, a network and a build, so the way to keep a person out of it is to give the code no
//! place to put one, rather than to put a person in it and then test that a setting was read
//! first. The table it writes, `reachability`, has no profile column
//! (`server/migrations/0110_play.sql`, `0140_play_health.sql`), and
//! `server-accounts::holdings` declares it `Keyed::NoOne`.
//!
//! The fold is pure and separate from the store, so what a batch of events becomes can be read
//! and tested without a database, and the Postgres upsert and the in-memory model can be checked
//! against the same expectation.

use std::collections::BTreeMap;

use contract::driver::ErrorClass;
use contract::play::{PlayEvent, PlayEventName};
use time::{Date, OffsetDateTime};

/// The server's day. A client never gets to say which day its counts land on.
#[must_use]
pub fn today() -> Date {
    OffsetDateTime::now_utc().date()
}

/// The day this many days before today, for a read that wants the last few.
#[must_use]
pub fn days_back(days: u16) -> Date {
    today().saturating_sub(time::Duration::days(i64::from(days)))
}

/// The longest host this keeps. Longer than any real one, and the same limit the table carries.
pub const HOST_LIMIT: usize = 120;

/// The longest error-class word this keeps.
pub const FAULT_LIMIT: usize = 40;

/// The most of any one count a single report may contribute. A report arrives from a client, so
/// a batch of ten thousand rebuffers must cost the same as a batch of sixty-four.
pub const PER_REPORT_LIMIT: i64 = 64;

/// The most hosts one report may name. A play walks attempts and the core stops recording after
/// sixteen of them, so a report naming more than this is not a play, and each host it names is a
/// row written. Hosts past the limit are dropped, not merged, because merging would put one
/// host's rebuffers on another.
pub const HOSTS_PER_REPORT: usize = 16;

/// One row of `reachability`: one class, one host, one day, one fault word.
///
/// The row with the empty fault carries the session counts. A row with a fault word carries that
/// error class's failures and nothing else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tally {
    pub class: String,
    pub host: String,
    pub day: Date,
    /// The error class of `failures`, or empty on the session row.
    pub fault: String,
    pub tries: i64,
    pub starts: i64,
    /// The sum of the first-frame delays behind `starts`. The mean is this over `starts`.
    pub start_ms_total: i64,
    /// A count of rebuffers. There is no duration here and no column for one: the Android
    /// driver ticks position at 1 Hz and only while playing, so a measured length is quantised
    /// to a second and biased upward, while a count is not.
    pub rebuffers: i64,
    pub failures: i64,
    /// Sessions that ended at this host having never shown a frame.
    pub gave_up: i64,
}

/// The first day `time` can name. Nothing is ever written with it: both of these are built
/// from a day the server read from its own clock, and this only answers `Default`.
fn no_day() -> Date {
    Date::from_ordinal_date(1, 1).unwrap_or(Date::MIN)
}

impl Default for Tally {
    fn default() -> Self {
        Self {
            class: String::new(),
            host: String::new(),
            day: no_day(),
            fault: String::new(),
            tries: 0,
            starts: 0,
            start_ms_total: 0,
            rebuffers: 0,
            failures: 0,
            gave_up: 0,
        }
    }
}

impl Default for Health {
    fn default() -> Self {
        Self {
            class: String::new(),
            host: String::new(),
            day: no_day(),
            tries: 0,
            starts: 0,
            start_ms_total: 0,
            rebuffers: 0,
            gave_up: 0,
            failures: Vec::new(),
        }
    }
}

impl Tally {
    fn session(class: &str, host: &str, day: Date) -> Self {
        Self {
            class: class.to_owned(),
            host: host.to_owned(),
            day,
            ..Self::default()
        }
    }

    /// True when every count is zero, so there is nothing to write.
    #[must_use]
    pub const fn empty(&self) -> bool {
        self.tries == 0
            && self.starts == 0
            && self.start_ms_total == 0
            && self.rebuffers == 0
            && self.failures == 0
            && self.gave_up == 0
    }

    /// Add another row for the same key. This is what the Postgres upsert does, so the in-memory
    /// store is a model of it and not a different thing that happens to pass.
    pub fn add(&mut self, other: &Self) {
        self.tries += other.tries;
        self.starts += other.starts;
        self.start_ms_total += other.start_ms_total;
        self.rebuffers += other.rebuffers;
        self.failures += other.failures;
        self.gave_up += other.gave_up;
    }
}

/// One day of playback health at one host for one class: the read path's answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Health {
    pub class: String,
    pub host: String,
    pub day: Date,
    pub tries: i64,
    pub starts: i64,
    pub start_ms_total: i64,
    pub rebuffers: i64,
    pub gave_up: i64,
    /// Failures by error class, by the class's own word, in that order.
    pub failures: Vec<(String, i64)>,
}

impl Health {
    /// Sum of first-frame delays in milliseconds across `starts`.
    #[must_use]
    pub const fn first_frame_ms_sum(&self) -> i64 {
        self.start_ms_total
    }

    /// Sessions that ended at this host without ever showing a picture.
    #[must_use]
    pub const fn ended_without_picture(&self) -> i64 {
        self.gave_up
    }

    /// Every failure of the day, whatever the class.
    #[must_use]
    pub fn failures_total(&self) -> i64 {
        self.failures.iter().map(|(_, count)| count).sum()
    }

    /// The mean first-frame delay in milliseconds, or `None` when nothing started. A first frame
    /// that carried no delay counts in `starts` and adds nothing to the sum, so this is exact
    /// only while every report carries one, which the core's does.
    #[must_use]
    pub const fn mean_start_ms(&self) -> Option<i64> {
        if self.starts == 0 {
            None
        } else {
            Some(self.start_ms_total / self.starts)
        }
    }
}

/// Playback health for one `(class, host)` rolled up across a window of days.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HostSummary {
    pub class: String,
    pub host: String,
    pub days: usize,
    pub tries: i64,
    pub starts: i64,
    pub first_frame_ms_sum: i64,
    pub rebuffers: i64,
    pub ended_without_picture: i64,
    pub failures: Vec<(String, i64)>,
}

impl HostSummary {
    /// Total failures across all error classes in the window.
    #[must_use]
    pub fn failures_total(&self) -> i64 {
        self.failures.iter().map(|(_, count)| count).sum()
    }

    /// Mean first-frame delay in milliseconds across `starts`, or `None` when nothing started.
    #[must_use]
    pub const fn mean_start_ms(&self) -> Option<i64> {
        if self.starts == 0 {
            None
        } else {
            Some(self.first_frame_ms_sum / self.starts)
        }
    }
}

/// The host of an event, when it is one this will keep.
fn host_of(event: &PlayEvent) -> Option<&str> {
    let host = event.host.as_deref()?.trim();
    (!host.is_empty() && host.len() <= HOST_LIMIT && !host.contains(char::is_whitespace))
        .then_some(host)
}

/// The word of an error class, as the wire spells it.
fn fault_word(class: ErrorClass) -> Option<String> {
    let word = serde_json::to_value(class).ok()?;
    let word = word.as_str()?;
    (word.len() <= FAULT_LIMIT).then(|| word.to_owned())
}

/// What one report of one play becomes: at most one session row per host, plus one row per error
/// class seen at that host.
///
/// `class` is the client class the request arrived with, which the server reads from a header and
/// the client never gets to key a row by. `day` is the server's day, not the client's.
#[must_use]
pub fn fold(class: &str, day: Date, events: &[PlayEvent]) -> Vec<Tally> {
    let mut sessions: BTreeMap<String, Tally> = BTreeMap::new();
    let mut faults: BTreeMap<(String, String), i64> = BTreeMap::new();
    let mut ended: BTreeMap<String, bool> = BTreeMap::new();
    for event in events {
        let Some(host) = host_of(event) else { continue };
        if sessions.len() >= HOSTS_PER_REPORT && !sessions.contains_key(host) {
            continue;
        }
        let row = sessions
            .entry(host.to_owned())
            .or_insert_with(|| Tally::session(class, host, day));
        row.tries = 1;
        match event.name {
            PlayEventName::FirstFrame => {
                row.starts = 1;
                row.start_ms_total += i64::try_from(event.first_frame_ms.unwrap_or(0)).unwrap_or(0);
            }
            PlayEventName::Rebuffer => row.rebuffers = (row.rebuffers + 1).min(PER_REPORT_LIMIT),
            PlayEventName::Failure => {
                if let Some(word) = event.failure.and_then(fault_word) {
                    let count = faults.entry((host.to_owned(), word)).or_default();
                    *count = (*count + 1).min(PER_REPORT_LIMIT);
                }
            }
            PlayEventName::Ended => {
                ended.insert(host.to_owned(), true);
            }
            _ => {}
        }
    }
    // A session that reached the end of a host without ever showing a frame gave up there. The
    // event names carry no word for it, and none is needed: the absence of a first frame in a
    // report that has an end is the fact itself.
    for (host, row) in &mut sessions {
        if ended.get(host).copied().unwrap_or(false) && row.starts == 0 {
            row.gave_up = 1;
        }
    }
    let mut out: Vec<Tally> = sessions.into_values().filter(|row| !row.empty()).collect();
    for ((host, fault), count) in faults {
        let mut row = Tally::session(class, &host, day);
        row.fault = fault;
        row.failures = count;
        out.push(row);
    }
    out.sort_by(|a, b| (&a.host, &a.fault).cmp(&(&b.host, &b.fault)));
    out
}

/// One line per (class, host, day), for an operator to read.
///
/// This is the whole point of the read half. `reachability` sat in the schema from
/// `0110_play.sql` with no insert, update or select anywhere, and `play_events` had a writer no
/// client ever called: a table nobody reads is the shape both of those took. A number that has
/// to be queried by hand is a number nobody looks at, so the server prints these on the same
/// timer as the prune.
#[must_use]
pub fn lines(days: &[Health]) -> Vec<String> {
    days.iter()
        .map(|day| {
            let mean = day
                .mean_start_ms()
                .map_or_else(|| String::from("-"), |ms| format!("{ms}ms"));
            let faults: Vec<String> = day
                .failures
                .iter()
                .map(|(word, count)| format!("{word} {count}"))
                .collect();
            format!(
                "{} {} {}: tried {}, started {} (first frame {}), rebuffers {}, gave up {}, \
                 failures {}{}",
                day.day,
                day.class,
                day.host,
                day.tries,
                day.starts,
                mean,
                day.rebuffers,
                day.gave_up,
                day.failures_total(),
                if faults.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", faults.join(", "))
                }
            )
        })
        .collect()
}

/// Group stored rows into one answer per (class, host, day). This is the read path's half that
/// does not need a database, so the in-memory store and Postgres return the same shape.
#[must_use]
pub fn group(rows: Vec<Tally>) -> Vec<Health> {
    let mut found: BTreeMap<(Date, String, String), Health> = BTreeMap::new();
    for row in rows {
        let key = (row.day, row.class.clone(), row.host.clone());
        let day = found.entry(key).or_insert_with(|| Health {
            class: row.class.clone(),
            host: row.host.clone(),
            day: row.day,
            ..Health::default()
        });
        if row.fault.is_empty() {
            day.tries += row.tries;
            day.starts += row.starts;
            day.start_ms_total += row.start_ms_total;
            day.rebuffers += row.rebuffers;
            day.gave_up += row.gave_up;
        } else if row.failures > 0 {
            day.failures.push((row.fault, row.failures));
        }
    }
    let mut out: Vec<Health> = found.into_values().collect();
    for day in &mut out {
        day.failures.sort();
    }
    // Newest day first, then class and host, which is the index order of the table.
    out.sort_by(|a, b| {
        b.day
            .cmp(&a.day)
            .then_with(|| (&a.class, &a.host).cmp(&(&b.class, &b.host)))
    });
    out
}

/// Roll up daily [`Health`] rows into one [`HostSummary`] per `(class, host)`.
#[must_use]
pub fn summarize(days: &[Health]) -> Vec<HostSummary> {
    use std::collections::BTreeSet;

    struct Acc {
        days: BTreeSet<Date>,
        tries: i64,
        starts: i64,
        first_frame_ms_sum: i64,
        rebuffers: i64,
        ended_without_picture: i64,
        failures: BTreeMap<String, i64>,
    }

    let mut by_key: BTreeMap<(String, String), Acc> = BTreeMap::new();
    for row in days {
        let entry = by_key
            .entry((row.class.clone(), row.host.clone()))
            .or_insert_with(|| Acc {
                days: BTreeSet::new(),
                tries: 0,
                starts: 0,
                first_frame_ms_sum: 0,
                rebuffers: 0,
                ended_without_picture: 0,
                failures: BTreeMap::new(),
            });
        entry.days.insert(row.day);
        entry.tries += row.tries;
        entry.starts += row.starts;
        entry.first_frame_ms_sum += row.start_ms_total;
        entry.rebuffers += row.rebuffers;
        entry.ended_without_picture += row.gave_up;
        for (fault, count) in &row.failures {
            *entry.failures.entry(fault.clone()).or_default() += count;
        }
    }

    by_key
        .into_iter()
        .map(|((class, host), acc)| HostSummary {
            class,
            host,
            days: acc.days.len(),
            tries: acc.tries,
            starts: acc.starts,
            first_frame_ms_sum: acc.first_frame_ms_sum,
            rebuffers: acc.rebuffers,
            ended_without_picture: acc.ended_without_picture,
            failures: acc.failures.into_iter().collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use contract::driver::ErrorClass;

    use super::*;

    fn day(n: u16) -> Date {
        Date::from_ordinal_date(2026, 300 + n).expect("a day")
    }

    fn event(name: PlayEventName, host: &str) -> PlayEvent {
        PlayEvent {
            name,
            at_ms: 0,
            attempt: Some("a1".to_owned()),
            first_frame_ms: None,
            failure: None,
            host: Some(host.to_owned()),
            rung: None,
        }
    }

    /// One play that walked from a host that broke to one that worked, as the core reports it.
    fn a_walk() -> Vec<PlayEvent> {
        let mut failed = event(PlayEventName::Failure, "bad.example");
        failed.failure = Some(ErrorClass::Segment);
        let mut started = event(PlayEventName::FirstFrame, "good.example");
        started.first_frame_ms = Some(1_400);
        vec![
            failed,
            started,
            event(PlayEventName::Rebuffer, "good.example"),
            event(PlayEventName::Rebuffer, "good.example"),
            event(PlayEventName::Ended, "good.example"),
        ]
    }

    #[test]
    fn a_report_becomes_counts_per_host_and_the_failure_keeps_its_class() {
        let rows = fold("tv", day(0), &a_walk());
        let bad = rows
            .iter()
            .find(|row| row.host == "bad.example" && row.fault == "segment")
            .expect("the failed host keeps its error class");
        assert_eq!(bad.failures, 1);
        let good = rows
            .iter()
            .find(|row| row.host == "good.example" && row.fault.is_empty())
            .expect("the host that played");
        assert_eq!((good.tries, good.starts), (1, 1));
        assert_eq!(good.start_ms_total, 1_400);
        assert_eq!(good.rebuffers, 2, "a count of rebuffers, never a length");
        assert_eq!(good.gave_up, 0);
        let session = rows
            .iter()
            .find(|row| row.host == "bad.example" && row.fault.is_empty())
            .expect("the failed host was still tried");
        assert_eq!((session.tries, session.starts), (1, 0));
    }

    #[test]
    fn an_end_with_no_first_frame_is_an_exit_before_the_picture_came() {
        let rows = fold(
            "web",
            day(0),
            &[event(PlayEventName::Ended, "slow.example")],
        );
        let row = rows.iter().find(|row| row.fault.is_empty()).expect("a row");
        assert_eq!((row.tries, row.starts, row.gave_up), (1, 0, 1));
    }

    #[test]
    fn a_first_frame_means_the_session_did_not_give_up() {
        let mut started = event(PlayEventName::FirstFrame, "ok.example");
        started.first_frame_ms = Some(800);
        let rows = fold(
            "web",
            day(0),
            &[started, event(PlayEventName::Ended, "ok.example")],
        );
        let row = rows.iter().find(|row| row.fault.is_empty()).expect("a row");
        assert_eq!((row.starts, row.gave_up), (1, 0));
    }

    #[test]
    fn a_host_a_client_made_up_is_dropped_rather_than_stored() {
        let long = "x".repeat(HOST_LIMIT + 1);
        for host in [String::new(), " ".to_owned(), "a host".to_owned(), long] {
            let rows = fold("web", day(0), &[event(PlayEventName::Ended, &host)]);
            assert!(rows.is_empty(), "kept {host:?}");
        }
    }

    #[test]
    fn one_report_cannot_contribute_more_than_the_limit() {
        let flood: Vec<PlayEvent> = (0..PER_REPORT_LIMIT * 4)
            .map(|_| event(PlayEventName::Rebuffer, "noisy.example"))
            .collect();
        let rows = fold("web", day(0), &flood);
        assert_eq!(rows[0].rebuffers, PER_REPORT_LIMIT);
    }

    #[test]
    fn a_day_reads_as_one_line_with_its_numbers_in_it() {
        let found = group(fold("tv", day(0), &a_walk()));
        let text = lines(&found);
        let good = text
            .iter()
            .find(|line| line.contains("good.example"))
            .expect("the host that played");
        assert!(
            good.contains("tried 1, started 1 (first frame 1400ms)"),
            "{good}"
        );
        assert!(good.contains("rebuffers 2"), "{good}");
        assert!(good.contains("gave up 0"), "{good}");
        assert!(good.contains("failures 0"), "{good}");
        let bad = text
            .iter()
            .find(|line| line.contains("bad.example"))
            .expect("the host that broke");
        assert!(bad.contains("failures 1 [segment 1]"), "{bad}");
        assert!(bad.contains("first frame -"), "{bad}");
    }

    #[test]
    fn one_report_cannot_name_more_hosts_than_a_play_can_walk() {
        let flood: Vec<PlayEvent> = (0..HOSTS_PER_REPORT * 4)
            .map(|n| event(PlayEventName::Rebuffer, &format!("h{n}.example")))
            .collect();
        let rows = fold("web", day(0), &flood);
        assert_eq!(rows.len(), HOSTS_PER_REPORT);
        assert!(
            rows.iter().all(|row| row.rebuffers == 1),
            "a host past the limit is dropped and never merged into another: {rows:?}"
        );
    }

    #[test]
    fn the_read_path_groups_a_day_and_keeps_the_failures_apart() {
        let mut rows = fold("tv", day(0), &a_walk());
        rows.extend(fold("tv", day(1), &a_walk()));
        let found = group(rows);
        assert_eq!(found.len(), 4, "two hosts on each of two days: {found:?}");
        assert_eq!(found[0].day, day(1), "the newest day comes first");
        let good = found
            .iter()
            .find(|row| row.host == "good.example" && row.day == day(0))
            .expect("the host that played");
        assert_eq!(good.rebuffers, 2);
        assert_eq!(good.mean_start_ms(), Some(1_400));
        assert!(good.failures.is_empty());
        let bad = found
            .iter()
            .find(|row| row.host == "bad.example" && row.day == day(0))
            .expect("the host that broke");
        assert_eq!(bad.failures, vec![("segment".to_owned(), 1)]);
        assert_eq!(bad.failures_total(), 1);
        assert_eq!(bad.mean_start_ms(), None);
    }

    #[test]
    fn summarize_rolls_up_days_by_class_and_host() {
        let mut rows = fold("tv", day(0), &a_walk());
        rows.extend(fold("tv", day(1), &a_walk()));
        rows.extend(fold(
            "tv",
            day(1),
            &[event(PlayEventName::Ended, "bad.example")],
        ));
        let daily = group(rows);
        let rolled = summarize(&daily);
        assert_eq!(rolled.len(), 2);

        let bad = rolled
            .iter()
            .find(|row| row.host == "bad.example")
            .expect("bad.example summary");
        assert_eq!(bad.class, "tv");
        assert_eq!(bad.days, 2);
        assert_eq!(bad.tries, 3);
        assert_eq!(bad.starts, 0);
        assert_eq!(bad.first_frame_ms_sum, 0);
        assert_eq!(bad.ended_without_picture, 1);
        assert_eq!(bad.failures, vec![("segment".to_owned(), 2)]);
        assert_eq!(bad.failures_total(), 2);
        assert_eq!(bad.mean_start_ms(), None);

        let good = rolled
            .iter()
            .find(|row| row.host == "good.example")
            .expect("good.example summary");
        assert_eq!(good.days, 2);
        assert_eq!(good.tries, 2);
        assert_eq!(good.starts, 2);
        assert_eq!(good.first_frame_ms_sum, 2_800);
        assert_eq!(good.mean_start_ms(), Some(1_400));
        assert_eq!(good.rebuffers, 4);
        assert_eq!(good.ended_without_picture, 0);
        assert!(good.failures.is_empty());
    }
}
