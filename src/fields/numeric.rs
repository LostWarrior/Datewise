use crate::locale::DateOrder;
use crate::scan::{century, clear_after, clear_before, digits, is_sep};
use chrono::NaiveDate;
use std::ops::Range;

/// A numeric date such as `03/10/2026` found in text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NumericDate {
    pub span: Range<usize>,
    pub dates: Vec<NaiveDate>,
}

/// Finds the first numeric date; two-digit years `00`-`68` are 2000-2068, `69`-`99` are 1969-1999.
#[must_use]
pub fn find_numeric_date(text: &str, order: Option<DateOrder>) -> Option<NumericDate> {
    text.char_indices()
        .find_map(|(start, _)| numeric_at(text, start, order))
}

pub(super) fn numeric_at(
    text: &str,
    start: usize,
    order: Option<DateOrder>,
) -> Option<NumericDate> {
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
        (1 | 2, 2) if !dashed => readings(century(c)?, a, b, order),
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
