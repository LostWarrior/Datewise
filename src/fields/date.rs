use crate::names::{ordinal_suffix_len, weekday_from_name, MONTHS};
use crate::scan::{boundary, clear_after, digits, glued_before, year, YEAR_DIGITS};
use chrono::{NaiveDate, Weekday};
use std::ops::Range;

/// A `<day><suffix> <Month>[ <year>]` phrase found in text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrdinalDate {
    pub span: Range<usize>,
    pub day: u32,
    pub month: u32,
    pub year: Option<i32>,
    pub weekday: Option<Weekday>,
}

/// Finds the first `[<Weekday> ]<day><st|nd|rd|th> <Month>[ <year>]`; the suffix is required.
#[must_use]
pub fn find_ordinal_date(text: &str) -> Option<OrdinalDate> {
    text.char_indices().find_map(|(start, _)| {
        if glued_before(text, start) {
            return None;
        }
        let parts = day_first(text.get(start..)?, true)?;
        Some(OrdinalDate {
            span: start..start + parts.len,
            day: parts.day,
            month: parts.month,
            year: parts.year,
            weekday: weekday_before(text.get(..start)?),
        })
    })
}

pub(super) struct Parts {
    pub(super) len: usize,
    pub(super) day: u32,
    pub(super) month: u32,
    pub(super) year: Option<i32>,
}

fn day_number(s: &str) -> Option<(u32, usize)> {
    digits(s, 2).filter(|(day, _)| (1..=31).contains(day))
}

fn year_after(tail: &str, comma: bool) -> Option<(i32, usize)> {
    let skipped = usize::from(comma && tail.starts_with(','));
    let value = tail.get(skipped..)?.strip_prefix(' ').and_then(year)?;
    Some((value, skipped + 1 + YEAR_DIGITS))
}

pub(super) fn day_first(s: &str, need_suffix: bool) -> Option<Parts> {
    let (day, day_len) = day_number(s)?;
    let after_day = s.get(day_len..)?;
    let suffix_len = match ordinal_suffix_len(after_day) {
        Some(len) => len,
        None if need_suffix => return None,
        None => 0,
    };
    let after_suffix = after_day.get(suffix_len..)?.strip_prefix(' ')?;
    let (month, month_len) = month_prefix(after_suffix)?;
    let mut len = day_len + suffix_len + 1 + month_len;
    let year = year_after(s.get(len..)?, false).map(|(y, n)| {
        len += n;
        y
    });
    Some(Parts {
        len,
        day,
        month,
        year,
    })
}

pub(super) fn month_first(s: &str) -> Option<Parts> {
    let (month, month_len) = month_prefix(s)?;
    let after_month = s.get(month_len..)?.strip_prefix(' ')?;
    let (day, day_len) = day_number(after_month)?;
    let tail = after_month.get(day_len..)?;
    let suffix_len = ordinal_suffix_len(tail).unwrap_or(0);
    let tail = tail.get(suffix_len..)?;
    if suffix_len == 0 {
        clear_after(tail)?;
    }
    let mut len = month_len + 1 + day_len + suffix_len;
    let year = year_after(tail, true).map(|(y, n)| {
        len += n;
        y
    });
    Some(Parts {
        len,
        day,
        month,
        year,
    })
}

pub(super) fn iso(s: &str) -> Option<Parts> {
    let (year, rest) = field(s, YEAR_DIGITS)?;
    let (month, rest) = field(rest.strip_prefix('-')?, 2)?;
    let (day, rest) = field(rest.strip_prefix('-')?, 2)?;
    clear_after(rest)?;
    Some(Parts {
        len: s.len() - rest.len(),
        day,
        month,
        year: Some(i32::try_from(year).ok()?),
    })
}

pub(crate) fn iso_date(token: &str) -> Option<NaiveDate> {
    let parts = iso(token).filter(|parts| parts.len == token.len())?;
    NaiveDate::from_ymd_opt(parts.year?, parts.month, parts.day)
}

fn field(s: &str, width: usize) -> Option<(u32, &str)> {
    let (value, count) = digits(s, width).filter(|(_, count)| *count == width)?;
    Some((value, s.get(count..)?))
}

#[must_use]
pub(crate) fn weekday_before(prefix: &str) -> Option<Weekday> {
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
