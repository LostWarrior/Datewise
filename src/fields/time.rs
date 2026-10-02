use crate::scan::{dash, digits, glued_before, skip_spaces};
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

/// Finds the first 12-hour time with an explicit `am`/`pm`, such as `3pm` or `11.45am`; an hour above 12 needs `pm`.
#[must_use]
pub fn find_time(text: &str) -> Option<TimeOfDay> {
    text.char_indices().find_map(|(start, _)| {
        let clock = clock_at(text.get(start..)?).filter(|c| c.pm.is_some())?;
        clock_start(text, start).then_some(())?;
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
        if !clock_start(text, start) {
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

fn clock_start(text: &str, pos: usize) -> bool {
    let mut back = text.get(..pos).unwrap_or_default().chars().rev();
    let inside = match (back.next(), back.next()) {
        (Some(':'), _) => true,
        (Some('.'), Some(c)) => c.is_ascii_digit(),
        _ => false,
    };
    !inside && !glued_before(text, pos)
}

fn clock_at(s: &str) -> Option<Clock> {
    let (hour, minute, mut len) = time_number(s)?;
    let meridiem = meridiem(s.get(len..)?);
    match meridiem {
        Some((_, n)) => len += n,
        None if s.get(len..)?.starts_with(char::is_alphanumeric) => return None,
        None => {}
    }
    let pm = meridiem.map(|(pm, _)| pm);
    (hour <= 12 || pm == Some(true)).then_some(Clock {
        hour,
        minute,
        pm,
        len,
    })
}

fn time_number(s: &str) -> Option<(u32, u32, usize)> {
    let (hour, mut len) = digits(s, 2)?;
    if !(1..=23).contains(&hour) {
        return None;
    }
    let mut minute = 0;
    let tail = s.get(len..)?.strip_prefix([':', '.']);
    if let Some(more) = tail.filter(|t| t.starts_with(|c: char| c.is_ascii_digit())) {
        let (m, n) = digits(more, 3)?;
        if n != 2 || m > 59 {
            return None;
        }
        minute = m;
        len += 3;
    }
    Some((hour, minute, len))
}

fn meridiem(s: &str) -> Option<(bool, usize)> {
    let rest = s.strip_prefix(' ').unwrap_or(s);
    let (pm, len) = [("am", false), ("pm", true), ("a.m.", false), ("p.m.", true)]
        .into_iter()
        .find(|(word, _)| {
            rest.get(..word.len())
                .is_some_and(|h| h.eq_ignore_ascii_case(word))
        })
        .map(|(word, pm)| (pm, word.len()))?;
    let after = rest.get(len..)?;
    (!after.starts_with(char::is_alphanumeric)).then_some((pm, s.len() - rest.len() + len))
}
