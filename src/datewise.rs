use crate::fields::{find_time, iso_date};
use crate::locale::Locale;
use crate::pattern::Pattern;
use crate::relative::grammar::{self, Parsed as Phrase};
use crate::relative::Window;
use crate::resolve::{local_instant, resolve_date, LocalInstant, YearMode};
use crate::scan::within_limit;
use crate::zone::{find_zone, FoundZone};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};
use std::fmt;

/// [`Datewise`] could not be configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConfigError {
    UnknownLocale,
    InvalidDate,
    InvalidWindow,
    UnsupportedPattern,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ConfigError::UnknownLocale => "unknown or unsupported locale tag",
            ConfigError::InvalidDate => "date is not a valid YYYY-MM-DD",
            ConfigError::InvalidWindow => "window start is after its end",
            ConfigError::UnsupportedPattern => "unsupported date pattern",
        })
    }
}

impl std::error::Error for ConfigError {}

/// Why a phrase did not produce a single result.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ParseError {
    Unparsed,
    Ambiguous(Vec<Parsed>),
    OutOfWindow,
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

/// A value could not be formatted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FormatError {
    MissingField,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("value lacks a field the pattern needs")
    }
}

impl std::error::Error for FormatError {}

/// A parsed phrase; `Display` writes ISO 8601.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Parsed {
    Date(NaiveDate),
    DateTime(NaiveDateTime),
    Instant(DateTime<FixedOffset>),
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

/// Parses and formats dates, either as phrases relative to `today` or with a fixed pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Datewise {
    mode: Mode,
    window: Option<Window>,
    year: YearMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Mode {
    Phrases(Phrases),
    Pattern(Pattern),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Phrases {
    locale: Locale,
    today: NaiveDate,
}

impl Datewise {
    /// Phrases for `locale_tag` (such as `en-GB`) relative to `today` (`YYYY-MM-DD`), preferring future dates.
    ///
    /// ```
    /// use datewise::Datewise;
    ///
    /// let dw = Datewise::new("en-GB", "2026-10-02")?;
    /// assert_eq!(dw.parse("next friday")?.to_string(), "2026-10-09");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(locale_tag: &str, today: &str) -> Result<Self, ConfigError> {
        let locale = Locale::from_tag(locale_tag).ok_or(ConfigError::UnknownLocale)?;
        let today = date_arg(today)?;
        Ok(Self::with(Mode::Phrases(Phrases { locale, today })))
    }

    /// Parses and formats exactly `pattern`; other non-letters are literal:
    ///
    /// - `yyyy`, `yy` (`00`-`68` → 2000s, `69`-`99` → 1900s; use `yyyy` for 2069+)
    /// - `M`, `MM`, `MMM`, `MMMM`, `d`, `dd`, `E`/`EEE`, `EEEE`
    /// - `H`, `HH`, `h`, `hh`, `a`, `m`, `mm`, `s`, `ss`, `XXX` (`Z` for zero), `'text'`, `''`
    ///
    /// ```
    /// use datewise::Datewise;
    ///
    /// let p = Datewise::pattern("dd/MM/yyyy")?;
    /// let d = p.parse("03/11/2026")?;
    /// assert_eq!(Datewise::pattern("yyyy-MM-dd")?.format(&d)?, "2026-11-03");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn pattern(pattern: &str) -> Result<Self, ConfigError> {
        let pattern = Pattern::new(pattern).ok_or(ConfigError::UnsupportedPattern)?;
        Ok(Self::with(Mode::Pattern(pattern)))
    }

    fn with(mode: Mode) -> Self {
        Self {
            mode,
            window: None,
            year: YearMode::NextOnOrAfter,
        }
    }

    /// Rejects dates outside `start..=end` (`YYYY-MM-DD`); spans need only overlap it.
    pub fn within(mut self, start: &str, end: &str) -> Result<Self, ConfigError> {
        let window = Window::new(date_arg(start)?, date_arg(end)?);
        self.window = Some(window.ok_or(ConfigError::InvalidWindow)?);
        Ok(self)
    }

    /// Reads bare weekdays, `the 14th` and yearless dates as on or before today; patterns ignore it.
    #[must_use]
    pub fn prefer_past(mut self) -> Self {
        self.year = YearMode::PreviousOnOrBefore;
        self
    }

    /// Parses the whole of `text`.
    pub fn parse(&self, text: &str) -> Result<Parsed, ParseError> {
        if !within_limit(text) {
            return Err(ParseError::Unparsed);
        }
        match &self.mode {
            Mode::Phrases(phrases) => self.phrase(*phrases, text),
            Mode::Pattern(pattern) => {
                let parsed = pattern.parse(text).ok_or(ParseError::Unparsed)?;
                let date = match parsed {
                    Parsed::Instant(instant) => instant.date_naive(),
                    Parsed::DateTime(local) => local.date(),
                    _ => parsed.as_date().ok_or(ParseError::Unparsed)?,
                };
                self.checked(vec![date]).map(|_| parsed)
            }
        }
    }

    /// Writes `value` with the pattern, or as ISO 8601 (like `Display`) in phrase mode.
    pub fn format(&self, value: &Parsed) -> Result<String, FormatError> {
        match &self.mode {
            Mode::Phrases(_) => Ok(value.to_string()),
            Mode::Pattern(pattern) => pattern.format(value).ok_or(FormatError::MissingField),
        }
    }

    fn phrase(&self, phrases: Phrases, text: &str) -> Result<Parsed, ParseError> {
        let Some(found) = find_time(text) else {
            return self.dates(phrases, text).and_then(Self::date_only);
        };
        let head = text.get(..found.span.start).unwrap_or_default();
        let mut tail = text.get(found.span.end..).unwrap_or_default();
        let zone = find_zone(tail, Some(phrases.locale))
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
            (true, true) => self.checked(vec![phrases.today])?,
            (true, false) => self.dates(phrases, tail)?.dated()?,
            (false, true) => self.dates(phrases, head)?.dated()?,
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

    fn dates(&self, phrases: Phrases, text: &str) -> Result<Dates, ParseError> {
        let Phrases { locale, today } = phrases;
        let phrase =
            grammar::parse(text, today, Some(locale), self.year).ok_or(ParseError::Unparsed)?;
        Ok(match phrase {
            Phrase::Date(date) => Dates::Days(self.checked(vec![date])?),
            Phrase::Numeric(dates) => Dates::Days(self.checked(dates)?),
            Phrase::YearlessDay { day, month } => {
                let date =
                    resolve_date(day, month, None, today, self.year).ok_or(ParseError::Unparsed)?;
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

fn date_arg(text: &str) -> Result<NaiveDate, ConfigError> {
    iso_date(text).ok_or(ConfigError::InvalidDate)
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
