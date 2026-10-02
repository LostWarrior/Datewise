pub(crate) mod grammar;

use crate::locale::Locale;
use chrono::{Datelike, NaiveDate};

/// An inclusive range of acceptable dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Window {
    start: NaiveDate,
    end: NaiveDate,
}

impl Window {
    /// Creates a window, or `None` when `start` is after `end`.
    #[must_use]
    pub fn new(start: NaiveDate, end: NaiveDate) -> Option<Self> {
        (start <= end).then_some(Self { start, end })
    }

    /// First date in the window.
    #[must_use]
    pub fn start(&self) -> NaiveDate {
        self.start
    }

    /// Last date in the window.
    #[must_use]
    pub fn end(&self) -> NaiveDate {
        self.end
    }

    /// Whether `date` lies within the window, edges included.
    #[must_use]
    pub fn contains(&self, date: NaiveDate) -> bool {
        self.start <= date && date <= self.end
    }
}

/// The result of [`parse_relative`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Relative {
    Resolved(NaiveDate),
    Span(NaiveDate, NaiveDate),
    OutOfWindow(NaiveDate),
    Ambiguous(Vec<NaiveDate>),
    Unparsed,
}

/// Parses `text` relative to `today` and checks it against `window`; `locale` sets order and week start.
#[must_use]
pub fn parse_relative(
    text: &str,
    today: NaiveDate,
    window: Window,
    locale: Option<Locale>,
) -> Relative {
    match grammar::parse(text, today, locale, crate::resolve::YearMode::NextOnOrAfter) {
        Some(grammar::Parsed::Date(date)) => place(date, window),
        Some(grammar::Parsed::Span(first, last)) => {
            if window.start() <= last && first <= window.end() {
                Relative::Span(first, last)
            } else {
                Relative::OutOfWindow(first)
            }
        }
        Some(grammar::Parsed::Numeric(dates)) => among(&dates, window),
        Some(grammar::Parsed::YearlessDay { day, month }) => yearless(day, month, today, window),
        None => Relative::Unparsed,
    }
}

fn place(date: NaiveDate, window: Window) -> Relative {
    if window.contains(date) {
        Relative::Resolved(date)
    } else {
        Relative::OutOfWindow(date)
    }
}

fn yearless(day: u32, month: u32, today: NaiveDate, window: Window) -> Relative {
    use crate::resolve::{resolve_date, YearMode::NextOnOrAfter};
    let Some(first) = resolve_date(day, month, None, today, NextOnOrAfter) else {
        return Relative::Unparsed;
    };
    let from = today.max(window.start());
    let inside: Vec<NaiveDate> = (from.year()..=window.end().year())
        .filter_map(|year| NaiveDate::from_ymd_opt(year, month, day))
        .filter(|date| *date >= from && window.contains(*date))
        .take(2)
        .collect();
    if inside.is_empty() {
        Relative::OutOfWindow(first)
    } else {
        among(&inside, window)
    }
}

fn among(candidates: &[NaiveDate], window: Window) -> Relative {
    let inside: Vec<NaiveDate> = candidates
        .iter()
        .copied()
        .filter(|date| window.contains(*date))
        .collect();
    match (inside.as_slice(), candidates.first()) {
        ([only], _) => Relative::Resolved(*only),
        ([], Some(first)) => Relative::OutOfWindow(*first),
        ([], None) => Relative::Unparsed,
        _ => Relative::Ambiguous(inside),
    }
}
