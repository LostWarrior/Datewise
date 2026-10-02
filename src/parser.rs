use crate::fields::find_time;
use crate::locale::Locale;
use crate::relative::grammar::{self, Parsed as Phrase};
use crate::relative::Window;
use crate::resolve::{local_instant, resolve_date, LocalInstant, YearMode};
use crate::scan::within_limit;
use crate::zone::{find_zone, FoundZone};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};
use std::fmt;

/// The parser could not be configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConfigError {
    /// The locale tag is not a supported English region such as `en-GB`.
    UnknownLocale,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown or unsupported locale tag")
    }
}

impl std::error::Error for ConfigError {}

/// Why a phrase did not produce a single result.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ParseError {
    /// The whole text is not a supported phrase.
    Unparsed,
    /// Several readings are valid, in order of preference.
    Ambiguous(Vec<Parsed>),
    /// The phrase is clear but falls outside the window.
    OutOfWindow,
    /// The local time is skipped by a daylight saving change.
    NonexistentLocalTime,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ParseError::Unparsed => "not a supported date or time",
            ParseError::Ambiguous(_) => "several readings are possible",
            ParseError::OutOfWindow => "date is outside the allowed window",
            ParseError::NonexistentLocalTime => "local time does not exist in that time zone",
        })
    }
}

impl std::error::Error for ParseError {}

/// A parsed phrase; `Display` writes ISO 8601.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Parsed {
    /// A calendar date: `2026-10-03`.
    Date(NaiveDate),
    /// A local date and time with no zone: `2026-10-03T15:00:00`.
    DateTime(NaiveDateTime),
    /// A moment with its UTC offset: `2026-10-03T15:00:00+01:00`.
    Instant(DateTime<FixedOffset>),
    /// First and last day, inclusive: `2026-10-05/2026-10-11`.
    DateSpan(NaiveDate, NaiveDate),
}

impl Parsed {
    /// The date, if this is a [`Parsed::Date`].
    #[must_use]
    pub fn as_date(&self) -> Option<NaiveDate> {
        match self {
            Parsed::Date(date) => Some(*date),
            _ => None,
        }
    }

    /// The local date and time, if this is a [`Parsed::DateTime`].
    #[must_use]
    pub fn as_date_time(&self) -> Option<NaiveDateTime> {
        match self {
            Parsed::DateTime(local) => Some(*local),
            _ => None,
        }
    }

    /// The instant, if this is a [`Parsed::Instant`].
    #[must_use]
    pub fn as_instant(&self) -> Option<DateTime<FixedOffset>> {
        match self {
            Parsed::Instant(instant) => Some(*instant),
            _ => None,
        }
    }

    /// First and last day, if this is a [`Parsed::DateSpan`].
    #[must_use]
    pub fn as_date_span(&self) -> Option<(NaiveDate, NaiveDate)> {
        match self {
            Parsed::DateSpan(first, last) => Some((*first, *last)),
            _ => None,
        }
    }
}

impl fmt::Display for Parsed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Parsed::Date(date) => write!(f, "{}", date.format("%Y-%m-%d")),
            Parsed::DateTime(local) => write!(f, "{}", local.format("%Y-%m-%dT%H:%M:%S")),
            Parsed::Instant(instant) => write!(f, "{}", instant.format("%Y-%m-%dT%H:%M:%S%:z")),
            Parsed::DateSpan(first, last) => {
                write!(
                    f,
                    "{}/{}",
                    first.format("%Y-%m-%d"),
                    last.format("%Y-%m-%d")
                )
            }
        }
    }
}

/// Parses whole phrases such as `tomorrow at 3pm ET` relative to a fixed `today`.
///
/// ```
/// use chrono::NaiveDate;
/// use datewise::{Parsed, Parser};
///
/// let today = NaiveDate::from_ymd_opt(2026, 10, 2).unwrap();
/// let parser = Parser::new("en-GB", today).unwrap();
/// assert_eq!(parser.parse("03/11/2026").unwrap().to_string(), "2026-11-03");
/// assert_eq!(parser.parse("tomorrow at 3pm").unwrap().to_string(), "2026-10-03T15:00:00");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Parser {
    locale: Locale,
    today: NaiveDate,
    window: Option<Window>,
    mode: YearMode,
}

impl Parser {
    /// A parser for `locale_tag` (such as `en-GB`) with no window, preferring future dates.
    ///
    /// # Errors
    ///
    /// [`ConfigError::UnknownLocale`] if the tag is not a supported region.
    pub fn new(locale_tag: &str, today: NaiveDate) -> Result<Self, ConfigError> {
        let locale = Locale::from_tag(locale_tag).ok_or(ConfigError::UnknownLocale)?;
        Ok(Self {
            locale,
            today,
            window: None,
            mode: YearMode::NextOnOrAfter,
        })
    }

    /// Rejects dates outside `window`; spans need only overlap it.
    #[must_use]
    pub fn within(mut self, window: Window) -> Self {
        self.window = Some(window);
        self
    }

    /// Reads bare weekdays, `the 14th` and yearless dates as on or before today.
    #[must_use]
    pub fn prefer_past(mut self) -> Self {
        self.mode = YearMode::PreviousOnOrBefore;
        self
    }

    /// Parses the whole of `text`.
    ///
    /// # Errors
    ///
    /// A [`ParseError`] when the text is unsupported, ambiguous, outside the window or skipped by DST.
    pub fn parse(&self, text: &str) -> Result<Parsed, ParseError> {
        if !within_limit(text) {
            return Err(ParseError::Unparsed);
        }
        let Some(found) = find_time(text) else {
            return self.dates(text).and_then(Self::date_only);
        };
        let head = text.get(..found.span.start).unwrap_or_default();
        let mut tail = text.get(found.span.end..).unwrap_or_default();
        let zone = find_zone(tail, Some(self.locale))
            .filter(|z| {
                tail.get(..z.span.start)
                    .is_some_and(|gap| gap.trim().is_empty())
            })
            .map(|z| {
                tail = tail.get(z.span.end..).unwrap_or_default();
                z.zone
            });
        let head = strip_at(head);
        let dates = match (head.trim().is_empty(), tail.trim().is_empty()) {
            (true, true) => self.checked(vec![self.today])?,
            (true, false) => self.dates(tail)?.dated()?,
            (false, true) => self.dates(head)?.dated()?,
            (false, false) => return Err(ParseError::Unparsed),
        };
        let mut readings = Vec::new();
        let mut gap = false;
        for date in dates {
            let local = date.and_time(found.time);
            match zone {
                None => readings.push(Parsed::DateTime(local)),
                Some(zone) => gap |= !in_zone(local, zone, &mut readings),
            }
        }
        one_of(readings, gap)
    }

    fn dates(&self, text: &str) -> Result<Dates, ParseError> {
        let phrase = grammar::parse(text, self.today, Some(self.locale), self.mode)
            .ok_or(ParseError::Unparsed)?;
        Ok(match phrase {
            Phrase::Date(date) => Dates::Days(self.checked(vec![date])?),
            Phrase::Numeric(dates) => Dates::Days(self.checked(dates)?),
            Phrase::YearlessDay { day, month } => {
                let date = resolve_date(day, month, None, self.today, self.mode)
                    .ok_or(ParseError::Unparsed)?;
                Dates::Days(self.checked(vec![date])?)
            }
            Phrase::Span(first, last) => {
                let overlaps = self
                    .window
                    .map_or(true, |w| w.start() <= last && first <= w.end());
                if !overlaps {
                    return Err(ParseError::OutOfWindow);
                }
                Dates::Span(first, last)
            }
        })
    }

    fn checked(&self, mut dates: Vec<NaiveDate>) -> Result<Vec<NaiveDate>, ParseError> {
        if let Some(window) = self.window {
            dates.retain(|date| window.contains(*date));
        }
        if dates.is_empty() {
            Err(ParseError::OutOfWindow)
        } else {
            Ok(dates)
        }
    }

    fn date_only(dates: Dates) -> Result<Parsed, ParseError> {
        match dates {
            Dates::Span(first, last) => Ok(Parsed::DateSpan(first, last)),
            Dates::Days(days) => one_of(days.into_iter().map(Parsed::Date).collect(), false),
        }
    }
}

enum Dates {
    Days(Vec<NaiveDate>),
    Span(NaiveDate, NaiveDate),
}

impl Dates {
    fn dated(self) -> Result<Vec<NaiveDate>, ParseError> {
        match self {
            Dates::Days(days) => Ok(days),
            Dates::Span(..) => Err(ParseError::Unparsed),
        }
    }
}

fn strip_at(head: &str) -> &str {
    let trimmed = head.trim_end();
    match trimmed.rsplit(|c: char| !c.is_alphanumeric()).next() {
        Some(word) if word.eq_ignore_ascii_case("at") => trimmed.strip_suffix(word).unwrap_or(head),
        _ => head,
    }
}

fn in_zone(local: NaiveDateTime, zone: FoundZone, readings: &mut Vec<Parsed>) -> bool {
    let fixed = |seconds: i32| {
        FixedOffset::east_opt(seconds)
            .and_then(|offset| offset.from_local_datetime(&local).single())
            .map(Parsed::Instant)
    };
    match zone {
        FoundZone::Fixed(offset) => readings.extend(fixed(offset.local_minus_utc())),
        FoundZone::Ambiguous(candidates) => {
            readings.extend(candidates.iter().filter_map(|s| fixed(*s)));
        }
        FoundZone::Iana(tz) => {
            let to_local =
                |utc: DateTime<Utc>| Parsed::Instant(utc.with_timezone(&tz).fixed_offset());
            match local_instant(local, tz) {
                LocalInstant::Single(utc) => readings.push(to_local(utc)),
                LocalInstant::Ambiguous(earlier, later) => {
                    readings.extend([to_local(earlier), to_local(later)]);
                }
                LocalInstant::Gap => return false,
            }
        }
    }
    true
}

fn one_of(mut readings: Vec<Parsed>, gap: bool) -> Result<Parsed, ParseError> {
    readings.dedup();
    match readings.as_slice() {
        [only] => Ok(*only),
        [] if gap => Err(ParseError::NonexistentLocalTime),
        [] => Err(ParseError::Unparsed),
        _ => Err(ParseError::Ambiguous(readings)),
    }
}
