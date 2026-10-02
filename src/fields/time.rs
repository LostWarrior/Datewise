use crate::scan::{boundary, dash, digits, glued_before, skip_spaces};
use chrono::NaiveTime;
use std::fmt;
use std::ops::Range;

/// A clock time with an explicit `am`/`pm`, found in text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimeOfDay {
    /// Byte range of the time including its meridiem.
    pub span: Range<usize>,
    /// The time on the 24-hour clock.
    pub time: NaiveTime,
}

/// A start and end time found in text, with meridiems resolved.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimeRange {
    /// Byte range from the start time to the end of the end time.
    pub span: Range<usize>,
    /// Start of the range.
    pub start: NaiveTime,
    /// End of the range.
    pub end: NaiveTime,
}

/// A time range whose `am`/`pm` could not be resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TimeError {
    /// Neither reading of the missing meridiem gives an ascending range.
    AmbiguousMeridiem,
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ambiguous am/pm in time range")
    }
}

impl std::error::Error for TimeError {}

#[derive(Clone, Copy)]
struct Clock {
    hour: u32,
    minute: u32,
    pm: Option<bool>,
    len: usize,
}

/// Finds the first 12-hour time with an explicit `am`/`pm`, such as `3pm` or `11.45am`.
#[must_use]
pub fn find_time(text: &str) -> Option<TimeOfDay> {
    text.char_indices().find_map(|(start, _)| {
        let clock = clock_at(text.get(start..)?).filter(|c| c.pm.is_some())?;
        (!glued_before(text, start)).then_some(())?;
        Some(TimeOfDay {
            span: start..start + clock.len,
            time: time_of(clock.hour, clock.minute, clock.pm?)?,
        })
    })
}

/// Finds the first `7 - 8pm` style range, taking a missing meridiem from the other side.
///
/// # Errors
///
/// [`TimeError::AmbiguousMeridiem`] if the range cannot ascend.
pub fn find_time_range(text: &str) -> Result<Option<TimeRange>, TimeError> {
    for (start, _) in text.char_indices() {
        if glued_before(text, start) {
            continue;
        }
        if let Some((first, second, end)) = range_at(text, start) {
            let (from, to) = resolve_meridiems(first, second)?;
            return Ok(Some(TimeRange {
                span: start..end,
                start: from,
                end: to,
            }));
        }
    }
    Ok(None)
}

fn range_at(text: &str, start: usize) -> Option<(Clock, Clock, usize)> {
    let rest = text.get(start..)?;
    let first = clock_at(rest)?;
    let mut p = first.len;
    p += skip_spaces(rest.get(p..)?);
    p += dash(rest.get(p..)?)?;
    p += skip_spaces(rest.get(p..)?);
    let second = clock_at(rest.get(p..)?)?;
    let anchored = first.pm.is_some() || second.pm.is_some();
    anchored.then_some((first, second, start + p + second.len))
}

fn resolve_meridiems(a: Clock, b: Clock) -> Result<(NaiveTime, NaiveTime), TimeError> {
    let pair = |pm_a: bool, pm_b: bool| {
        let from = time_of(a.hour, a.minute, pm_a)?;
        let to = time_of(b.hour, b.minute, pm_b)?;
        Some((from, to))
    };
    let ascending = |pm_a: bool, pm_b: bool| pair(pm_a, pm_b).filter(|(from, to)| from < to);
    match (a.pm, b.pm) {
        (Some(pm_a), Some(pm_b)) => pair(pm_a, pm_b),
        (None, Some(pm)) => ascending(pm, pm).or_else(|| ascending(!pm, pm)),
        (Some(pm), None) => ascending(pm, pm).or_else(|| ascending(pm, !pm)),
        (None, None) => None,
    }
    .ok_or(TimeError::AmbiguousMeridiem)
}

fn time_of(hour12: u32, minute: u32, pm: bool) -> Option<NaiveTime> {
    NaiveTime::from_hms_opt(hour12 % 12 + if pm { 12 } else { 0 }, minute, 0)
}

fn clock_at(s: &str) -> Option<Clock> {
    let (hour, minute, mut len) = time_number(s)?;
    let meridiem = meridiem(s.get(len..)?);
    if let Some((_, n)) = meridiem {
        len += n;
    }
    Some(Clock {
        hour,
        minute,
        pm: meridiem.map(|(pm, _)| pm),
        len,
    })
}

fn time_number(s: &str) -> Option<(u32, u32, usize)> {
    let (hour, mut len) = digits(s, 2)?;
    if !(1..=12).contains(&hour) {
        return None;
    }
    let mut minute = 0;
    let sep = |c: char| c == ':' || c == '.';
    if let Some((m, 2)) = s.get(len..)?.strip_prefix(sep).and_then(|t| digits(t, 2)) {
        if m <= 59 {
            minute = m;
            len += 3;
        }
    }
    Some((hour, minute, len))
}

fn meridiem(s: &str) -> Option<(bool, usize)> {
    let rest = s.strip_prefix(' ').unwrap_or(s);
    let head = rest.get(..2)?;
    let pm = if head.eq_ignore_ascii_case("am") {
        false
    } else if head.eq_ignore_ascii_case("pm") {
        true
    } else {
        return None;
    };
    boundary(rest.get(2..)?).then_some((pm, s.len() - rest.len() + 2))
}
