use crate::names::{ordinal_suffix_len, weekday_from_name, MONTHS};
use crate::scan::{boundary, digits, glued_before, within_limit, year, YEAR_DIGITS};
use chrono::Weekday;
use std::ops::Range;

/// A `<day><suffix> <Month>[ <year>]` phrase found in text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrdinalDate {
    /// Byte range from the day number to the end of the month or year.
    pub span: Range<usize>,
    /// Day of month, 1-31.
    pub day: u32,
    /// Month, 1-12.
    pub month: u32,
    /// Year, when four digits directly follow the month.
    pub year: Option<i32>,
    /// Weekday name written immediately before the day, if any.
    pub weekday: Option<Weekday>,
}

/// Finds the first `[<Weekday> ]<day><st|nd|rd|th> <Month>[ <year>]`; the suffix is required.
#[must_use]
pub fn find_ordinal_date(text: &str) -> Option<OrdinalDate> {
    if !within_limit(text) {
        return None;
    }
    text.char_indices()
        .find_map(|(start, _)| ordinal_date_at(text, start))
}

fn ordinal_date_at(text: &str, start: usize) -> Option<OrdinalDate> {
    if glued_before(text, start) {
        return None;
    }
    let rest = text.get(start..)?;
    let (day, day_len) = digits(rest, 2)?;
    if !(1..=31).contains(&day) {
        return None;
    }
    let after_day = rest.get(day_len..)?;
    let suffix_len = ordinal_suffix_len(after_day)?;
    let after_suffix = after_day.get(suffix_len..)?.strip_prefix(' ')?;
    let (month, month_len) = month_prefix(after_suffix)?;
    let mut end = day_len + suffix_len + 1 + month_len;
    let year = rest
        .get(end..)
        .and_then(|tail| tail.strip_prefix(' '))
        .and_then(year);
    if year.is_some() {
        end += 1 + YEAR_DIGITS;
    }
    Some(OrdinalDate {
        span: start..start + end,
        day,
        month,
        year,
        weekday: weekday_before(text.get(..start)?),
    })
}

/// The weekday named just before the end of `prefix`, followed by exactly one space.
#[must_use]
pub fn weekday_before(prefix: &str) -> Option<Weekday> {
    let trimmed = prefix.strip_suffix(' ')?;
    let word = trimmed.rsplit(|c: char| !c.is_alphabetic()).next()?;
    weekday_from_name(word)
}

/// A month name (full or abbreviated) at the start of `s`, not followed by a letter: `(month, bytes)`.
#[must_use]
pub fn month_prefix(s: &str) -> Option<(u32, usize)> {
    MONTHS.iter().find_map(|&(name, month)| {
        let head = s.get(..name.len())?;
        let matched = head.eq_ignore_ascii_case(name) && boundary(s.get(name.len()..)?);
        matched.then_some((month, name.len()))
    })
}
