//! The hand written half of the validator, and all of it.
//!
//! Everything else the contract refuses is generated from the field options. What
//! is left here is one predicate, one error type and the vocabulary of rule names.
//! `no_std`, no `alloc`, no allocation, no panic: the shape the core needs.
//!
//! What is deliberately NOT here:
//!   * presence. The generated type is `Option<T>` for every field whose absence
//!     the contract refuses, and the generated validator does the `is_none` test,
//!     so a field cannot be read without it.
//!   * an unknown number on an open enum. Keeping it IS the contract's forward
//!     compatibility rule (platform/contract/src/macros.rs:21-33) and prost keeps
//!     the raw `i32`.
//!   * the id range and the closed kind. Both are bounds and a declared value set,
//!     so both are generated.
//!   * the WIRE_MIN floor. `VersionRange::fit` compares dates; it is not a refusal
//!     the boundary makes, and putting it here would invent one.

/// Why a message was refused. One name per rule, so a refusal names its rule and
/// carries none of the sender's bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rule {
    /// A field the contract requires did not arrive.
    Required,
    /// A number below its declared floor.
    Min,
    /// A number above its declared ceiling.
    Max,
    /// An enum field this contract closes, carrying the zero value or a number the
    /// schema does not declare.
    Closed,
    /// A packed day that is not a day of the calendar.
    CalendarDay,
}

/// A refusal: which rule, and the field that broke it, named as the schema names it
/// (`contract.v1.Attempt.group`). A `&'static str` because the core has an
/// allocator but a refusal is not a reason to use it, and because the path is known
/// when the validator is generated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Refused {
    rule: Rule,
    path: &'static str,
}

impl Refused {
    /// Build a refusal. Called only by generated code.
    #[must_use]
    pub const fn new(rule: Rule, path: &'static str) -> Self {
        Self { rule, path }
    }

    /// Which rule was broken.
    #[must_use]
    pub const fn rule(self) -> Rule {
        self.rule
    }

    /// The field, as the schema names it.
    #[must_use]
    pub const fn path(self) -> &'static str {
        self.path
    }
}

/// `VALUE_RULE_CALENDAR_DAY`: a packed decimal `YYYYMMDD` naming a day of the
/// calendar.
///
/// The bounds are the project's own (platform/contract/src/screen.rs:106-112):
/// four year digits, month 1 to 12, day 1 to 31, and no counting of the days of a
/// month. So this refuses exactly what `CalendarDay::parse` refuses about the
/// value, and nothing more. The two it stops refusing are about the text -
/// `refused/CalendarDay.no_dashes.json` is 20261120, which packed is a real day -
/// and that change is recorded beside the message in common.proto.
#[must_use]
pub const fn is_calendar_day(ymd: u64) -> bool {
    let year = ymd / 10_000;
    let month = (ymd / 100) % 100;
    let day = ymd % 100;
    year <= 9_999 && month >= 1 && month <= 12 && day >= 1 && day <= 31
}

#[cfg(test)]
// A packed day is written `2026_11_32`, grouped year_month_day, because that is
// what the number means and because these assertions are about which days the
// project accepts. Clippy wants `20_261_132`, grouped in threes from the right,
// which hides the month and the day - and `mistyped_literal_suffixes` reads the
// `_32` of `2026_11_32` as a mistyped `i32`. Both are right about the shape of the
// literal and wrong about this file.
#[expect(
    clippy::inconsistent_digit_grouping,
    clippy::mistyped_literal_suffixes,
    reason = "a packed YYYYMMDD is grouped year_month_day, which is what it means"
)]
mod tests {
    use super::is_calendar_day;

    #[test]
    fn the_fixture_days_are_answered_as_the_project_answers_them() {
        // valid/, and the two refused CalendarDay fixtures.
        assert!(is_calendar_day(2026_11_05));
        assert!(is_calendar_day(2026_11_20));
        assert!(!is_calendar_day(2026_11_32));
        assert!(!is_calendar_day(2026_13_01));
    }

    #[test]
    fn a_value_too_big_for_a_date_is_refused_rather_than_wrapped() {
        // The reason ymd is uint64: as uint32 a sender could write 4294967296 +
        // 20261105 and protobuf would truncate it to 20261105, a real day. Here the
        // big value arrives whole and the year check refuses it.
        assert!(!is_calendar_day(4_294_967_296 + 2026_11_05));
        assert!(!is_calendar_day(10_000_000_000));
    }

    #[test]
    fn the_edges_are_the_projects_edges_and_not_wider() {
        assert!(!is_calendar_day(2026_00_01));
        assert!(!is_calendar_day(2026_01_00));
        assert!(is_calendar_day(2026_01_31));
        assert!(
            is_calendar_day(2026_02_31),
            "the project does not count the days of a month"
        );
        assert!(is_calendar_day(9999_12_31));
        assert!(!is_calendar_day(10000_12_31));
    }
}
