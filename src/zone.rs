//! Time zone parsing and zone-aware civil dates.

mod abbrev;

use crate::locale::Locale;
use crate::scan::{digits, glued_before, within_limit};
use abbrev::{Meaning, ABBREVIATIONS};
use chrono::{DateTime, FixedOffset, NaiveDate, Offset, TimeDelta, TimeZone};
use chrono_tz::Tz;
use std::fmt;
use std::ops::Range;

const MAX_OFFSET_HOURS: u32 = 14;

/// The text is not a known IANA time zone name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZoneError;

impl fmt::Display for ZoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown IANA time zone")
    }
}

impl std::error::Error for ZoneError {}

/// Parses an IANA zone name.
///
/// # Errors
///
/// [`ZoneError`] if the name is unknown.
pub fn parse_zone(name: &str) -> Result<Tz, ZoneError> {
    if !within_limit(name) {
        return Err(ZoneError);
    }
    name.parse().map_err(|_| ZoneError)
}

/// The civil date at `epoch_ms` (Unix milliseconds) in `tz`; `None` if out of range.
#[must_use]
pub fn local_date(epoch_ms: i64, tz: Tz) -> Option<NaiveDate> {
    let utc = DateTime::from_timestamp_millis(epoch_ms)?.naive_utc();
    let offset = tz.offset_from_utc_datetime(&utc).fix();
    let seconds = i64::from(offset.local_minus_utc());
    Some(
        utc.checked_add_signed(TimeDelta::try_seconds(seconds)?)?
            .date(),
    )
}

/// A zone named in text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FoundZone {
    /// A region-aware zone, used for seasonless shorthands such as `ET`.
    Iana(Tz),
    /// A fixed offset from UTC; `EST` and `EDT` keep their literal offsets.
    Fixed(FixedOffset),
    /// The text has several meanings here: candidate UTC offsets in seconds east, most likely first.
    Ambiguous(&'static [i32]),
}

/// A zone found in text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZoneMatch {
    /// Byte range of the zone text.
    pub span: Range<usize>,
    /// What it means.
    pub zone: FoundZone,
}

/// Finds the first zone: abbreviation, `UTC`/`GMT` offset, `Z` after a time, or numeric offset.
#[must_use]
pub fn find_zone(text: &str, locale: Option<Locale>) -> Option<ZoneMatch> {
    let region = locale.map(|l| l.region());
    text.char_indices().find_map(|(start, _)| {
        let rest = text.get(start..)?;
        let (len, zone) =
            word_at(text, start, rest, region).or_else(|| marker_at(text, start, rest))?;
        Some(ZoneMatch {
            span: start..start + len,
            zone,
        })
    })
}

fn word_end(tail: &str) -> bool {
    !tail.chars().next().is_some_and(char::is_alphanumeric)
}

fn word_at(
    text: &str,
    start: usize,
    rest: &str,
    region: Option<&str>,
) -> Option<(usize, FoundZone)> {
    if glued_before(text, start) {
        return None;
    }
    for name in ["UTC", "GMT"] {
        if let Some(tail) = rest.strip_prefix(name) {
            return utc_tail(tail)
                .map(|(len, offset)| (name.len() + len, FoundZone::Fixed(offset)));
        }
    }
    let abbrev = ABBREVIATIONS
        .iter()
        .find(|a| rest.strip_prefix(a.name).is_some_and(word_end))?;
    let zone = match abbrev.meaning(region) {
        Some(Meaning::Region(tz)) => FoundZone::Iana(tz),
        Some(Meaning::Offset(seconds)) => FixedOffset::east_opt(seconds).map(FoundZone::Fixed)?,
        None => FoundZone::Ambiguous(abbrev.candidates),
    };
    Some((abbrev.name.len(), zone))
}

fn utc_tail(tail: &str) -> Option<(usize, FixedOffset)> {
    if tail.starts_with(['+', '-']) {
        let (len, offset) = signed(tail, false)?;
        return word_end(tail.get(len..)?).then_some((len, offset));
    }
    let zero = FixedOffset::east_opt(0)?;
    word_end(tail).then_some((0, zero))
}

fn marker_at(text: &str, start: usize, rest: &str) -> Option<(usize, FoundZone)> {
    let before = text.get(..start)?.chars().next_back();
    if rest.starts_with('Z') {
        let after_time =
            before.is_some_and(|c| c.is_ascii_digit()) && colons_before(text, start) >= 1;
        let zero = FixedOffset::east_opt(0)?;
        return (after_time && word_end(rest.get(1..)?)).then_some((1, FoundZone::Fixed(zero)));
    }
    let (len, offset) = signed(rest, true)?;
    let min_colons = if rest.starts_with('-') { 2 } else { 1 };
    let anchored = match before {
        Some(c) if c.is_alphabetic() => false,
        Some(c) if c.is_ascii_digit() => colons_before(text, start) >= min_colons,
        _ => true,
    };
    (anchored && word_end(rest.get(len..)?)).then_some((len, FoundZone::Fixed(offset)))
}

fn colons_before(text: &str, pos: usize) -> usize {
    text.get(..pos).map_or(0, |head| {
        head.chars()
            .rev()
            .take_while(|c| c.is_ascii_digit() || matches!(c, ':' | '.'))
            .filter(|c| *c == ':')
            .count()
    })
}

// `bare` rejects `+h`, which needs a UTC/GMT prefix.
fn signed(s: &str, bare: bool) -> Option<(usize, FixedOffset)> {
    let sign = match s.chars().next()? {
        '+' => 1,
        '-' => -1,
        _ => return None,
    };
    let body = s.get(1..)?;
    let (hours, minutes, len) = clock(body, bare)?;
    if hours > MAX_OFFSET_HOURS || minutes >= 60 {
        return None;
    }
    let seconds = i32::try_from(hours * 3600 + minutes * 60).ok()? * sign;
    Some((1 + len, FixedOffset::east_opt(seconds)?))
}

fn clock(body: &str, bare: bool) -> Option<(u32, u32, usize)> {
    if let Some((value, 4)) = digits(body, 4) {
        return Some((value / 100, value % 100, 4));
    }
    let (hours, n) = digits(body, 2)?;
    match body.get(n..)?.strip_prefix(':') {
        Some(more) => {
            let (minutes, m) = digits(more, 2).filter(|(_, m)| *m == 2)?;
            (n == 2 || !bare).then_some((hours, minutes, n + 1 + m))
        }
        None => (!bare).then_some((hours, 0, n)),
    }
}
