use crate::locale::DateOrder;
use crate::scan::{digits, within_limit};
use chrono::NaiveDate;
use std::ops::Range;

// Two-digit years map to 2000-2099.
const TWO_DIGIT_YEAR_BASE: i32 = 2000;

/// A numeric date such as `03/10/2026` found in text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NumericDate {
    /// Byte range of the whole date.
    pub span: Range<usize>,
    /// The valid readings: one, or two (day-first then month-first) when the order is unknown.
    pub dates: Vec<NaiveDate>,
}

/// Finds the first `d/m/y`, `m/d/y` or `y/m/d` date with `/`, `.` or `-` separators.
///
/// A four-digit first field is year-first and ignores `order`. Otherwise `None` as
/// `order` (or `YearFirst`) returns every valid reading; a fixed order returns one.
/// Two-digit years need `/` or `.`; year-first needs `/` or `.` (use ISO for `-`).
#[must_use]
pub fn find_numeric_date(text: &str, order: Option<DateOrder>) -> Option<NumericDate> {
    if !within_limit(text) {
        return None;
    }
    text.char_indices()
        .find_map(|(start, _)| numeric_at(text, start, order))
}

fn is_sep(c: char) -> bool {
    matches!(c, '/' | '.' | '-')
}

fn clear_before(text: &str, pos: usize) -> Option<()> {
    let mut back = text.get(..pos)?.chars().rev();
    match (back.next(), back.next()) {
        (Some(c), _) if c.is_alphanumeric() => None,
        (Some(s), Some(d)) if is_sep(s) && d.is_ascii_digit() => None,
        _ => Some(()),
    }
}

fn clear_after(tail: &str) -> Option<()> {
    let mut chars = tail.chars();
    match (chars.next(), chars.next()) {
        (Some(c), _) if c.is_alphanumeric() => None,
        (Some(s), Some(d)) if is_sep(s) && d.is_ascii_digit() => None,
        _ => Some(()),
    }
}

fn numeric_at(text: &str, start: usize, order: Option<DateOrder>) -> Option<NumericDate> {
    clear_before(text, start)?;
    let rest = text.get(start..)?;
    let (a, a_len) = digits(rest, 4)?;
    let sep = rest.get(a_len..)?.chars().next().filter(|c| is_sep(*c))?;
    let after_a = rest.get(a_len + 1..)?;
    let (b, b_len) = digits(after_a, 2)?;
    let after_b = after_a.get(b_len..)?.strip_prefix(sep)?;
    let (c, c_len) = digits(after_b, 4)?;
    let end = a_len + 1 + b_len + 1 + c_len;
    clear_after(rest.get(end..)?)?;
    let dashed = sep == '-';
    let dates = match (a_len, c_len) {
        (4, 1 | 2) if !dashed => vec![NaiveDate::from_ymd_opt(i32::try_from(a).ok()?, b, c)?],
        (1 | 2, 4) => readings(i32::try_from(c).ok()?, a, b, order),
        (1 | 2, 2) if !dashed => {
            readings(TWO_DIGIT_YEAR_BASE + i32::try_from(c).ok()?, a, b, order)
        }
        _ => return None,
    };
    (!dates.is_empty()).then(|| NumericDate {
        span: start..start + end,
        dates,
    })
}

fn readings(year: i32, first: u32, second: u32, order: Option<DateOrder>) -> Vec<NaiveDate> {
    let day_first = NaiveDate::from_ymd_opt(year, second, first);
    let month_first = NaiveDate::from_ymd_opt(year, first, second);
    let mut dates: Vec<NaiveDate> = match order {
        Some(DateOrder::DayFirst) => vec![day_first],
        Some(DateOrder::MonthFirst) => vec![month_first],
        _ => vec![day_first, month_first],
    }
    .into_iter()
    .flatten()
    .collect();
    dates.dedup();
    dates
}
