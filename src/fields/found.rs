use super::date::{day_first, iso, month_first, weekday_before, Parts};
use super::numeric::numeric_at;
use crate::locale::{DateOrder, Locale};
use crate::scan::{clear_before, glued_before};
use chrono::{Datelike, NaiveDate, Weekday};
use std::ops::Range;

const LEAP_YEAR: i32 = 2000;

/// A date phrase found in text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FoundDate {
    /// Byte range of the date, excluding any weekday before it.
    pub span: Range<usize>,
    /// Day of month, 1-31.
    pub day: u32,
    /// Month, 1-12.
    pub month: u32,
    /// Year, when written; numeric dates always have one.
    pub year: Option<i32>,
    /// Weekday name written immediately before the date, if any.
    pub weekday: Option<Weekday>,
    /// A numeric date whose order is unknown: swapping `day` and `month` is also valid.
    pub ambiguous: bool,
}

/// Finds the first date phrase (ordinal, month-name, numeric or ISO); `locale` fixes numeric order.
///
/// ```
/// use datewise::fields::find_date;
///
/// let found = find_date("Fri 3 October 2026, 3 people", None).unwrap();
/// assert_eq!((found.day, found.month, found.year), (3, 10, Some(2026)));
/// assert!(find_date("May I march on 3 people", None).is_none());
/// ```
#[must_use]
pub fn find_date(text: &str, locale: Option<Locale>) -> Option<FoundDate> {
    let order = locale.map(|l| l.date_order());
    text.char_indices()
        .find_map(|(start, _)| date_at(text, start, order))
}

fn date_at(text: &str, start: usize, order: Option<DateOrder>) -> Option<FoundDate> {
    let (parts, ambiguous) = match numeric_at(text, start, order) {
        Some(found) => (
            numeric_parts(&found.dates, found.span.len())?,
            found.dates.len() > 1,
        ),
        None => (phrase_at(text, start)?, false),
    };
    NaiveDate::from_ymd_opt(parts.year.unwrap_or(LEAP_YEAR), parts.month, parts.day)?;
    Some(FoundDate {
        span: start..start + parts.len,
        day: parts.day,
        month: parts.month,
        year: parts.year,
        weekday: weekday_before(text.get(..start)?),
        ambiguous,
    })
}

fn numeric_parts(dates: &[NaiveDate], len: usize) -> Option<Parts> {
    let first = dates.first()?;
    Some(Parts {
        len,
        day: first.day(),
        month: first.month(),
        year: Some(first.year()),
    })
}

fn phrase_at(text: &str, start: usize) -> Option<Parts> {
    let rest = text.get(start..)?;
    if clear_before(text, start).is_some() {
        if let Some(parts) = iso(rest) {
            return Some(parts);
        }
    }
    if glued_before(text, start) {
        return None;
    }
    day_first(rest, false).or_else(|| month_first(rest))
}
