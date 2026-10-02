//! Turning parsed fields into concrete dates and instants.

use chrono::{DateTime, Datelike, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;

// Search distance in either direction; spans the longest gap between leap days (2096 to 2104).
const MAX_LOOKAHEAD_YEARS: i32 = 8;

/// How a missing year is inferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum YearMode {
    /// First occurrence on or after the anchor; a leap day waits for the next leap year.
    NextOnOrAfter,
    /// Latest occurrence on or before the anchor; a leap day reaches back to the last leap year.
    PreviousOnOrBefore,
}

/// Builds a date from `year`, or infers the year from `anchor`; `None` if the date does not exist.
#[must_use]
pub fn resolve_date(
    day: u32,
    month: u32,
    year: Option<i32>,
    anchor: NaiveDate,
    mode: YearMode,
) -> Option<NaiveDate> {
    if let Some(year) = year {
        return NaiveDate::from_ymd_opt(year, month, day);
    }
    match mode {
        YearMode::NextOnOrAfter => {
            let first = anchor.year();
            (first..=first.saturating_add(MAX_LOOKAHEAD_YEARS))
                .filter_map(|y| NaiveDate::from_ymd_opt(y, month, day))
                .find(|date| *date >= anchor)
        }
        YearMode::PreviousOnOrBefore => {
            let last = anchor.year();
            (last.saturating_sub(MAX_LOOKAHEAD_YEARS)..=last)
                .rev()
                .filter_map(|y| NaiveDate::from_ymd_opt(y, month, day))
                .find(|date| *date <= anchor)
        }
    }
}

/// A local wall-clock time placed in a time zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalInstant {
    /// Exactly one matching instant.
    Single(DateTime<Utc>),
    /// The time does not exist (clocks skipped forward).
    Gap,
    /// The time occurs twice (clocks moved back): earlier, then later.
    Ambiguous(DateTime<Utc>, DateTime<Utc>),
}

impl LocalInstant {
    /// The instant when it is unambiguous, otherwise `None`.
    #[must_use]
    pub fn single(self) -> Option<DateTime<Utc>> {
        match self {
            LocalInstant::Single(instant) => Some(instant),
            _ => None,
        }
    }
}

/// Resolves a local date and time in `tz` to UTC, reporting DST gaps and folds.
///
/// ```
/// use chrono::NaiveDate;
/// use chrono_tz::Europe::London;
/// use datewise::resolve::{local_instant, LocalInstant};
///
/// let spring = NaiveDate::from_ymd_opt(2026, 3, 29).unwrap();
/// let missing = spring.and_hms_opt(1, 30, 0).unwrap();
/// assert_eq!(local_instant(missing, London), LocalInstant::Gap);
/// ```
#[must_use]
pub fn local_instant(local: NaiveDateTime, tz: Tz) -> LocalInstant {
    match tz.from_local_datetime(&local) {
        LocalResult::Single(dt) => LocalInstant::Single(dt.with_timezone(&Utc)),
        LocalResult::Ambiguous(a, b) => {
            LocalInstant::Ambiguous(a.with_timezone(&Utc), b.with_timezone(&Utc))
        }
        LocalResult::None => LocalInstant::Gap,
    }
}
