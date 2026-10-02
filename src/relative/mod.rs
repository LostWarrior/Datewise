//! Relative and calendar phrases resolved against a reference date.
//!
//! Case, spacing, punctuation and a leading `on` are ignored. Supported:
//!
//! - `today`, `tomorrow`, `yesterday`
//! - `Friday`: the first such day on or after today
//! - `next Tuesday`: that weekday in the following Monday-to-Sunday week
//! - `this week`, `next week`: the Monday of that ISO week (may precede today)
//! - `the 14th`: the next 14th on or after today, skipping shorter months
//! - `3rd October`, `October 3rd`, `3 Oct`, with an optional year
//! - ISO dates such as `2026-10-03`

mod grammar;

use chrono::NaiveDate;

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
    /// A single date inside the window.
    Resolved(NaiveDate),
    /// The phrase is clear but its date falls outside the window.
    OutOfWindow(NaiveDate),
    /// A phrase without a year that matches several dates in the window.
    /// Holds the first two occurrences.
    Ambiguous(Vec<NaiveDate>),
    /// The text is not a supported phrase.
    Unparsed,
}

/// Parses `text` relative to `today` and checks the result against `window`.
///
/// ```
/// use chrono::NaiveDate;
/// use datewise::relative::{parse_relative, Relative, Window};
///
/// let today = NaiveDate::from_ymd_opt(2026, 10, 2).unwrap();
/// let window = Window::new(today, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()).unwrap();
/// let oct_3 = NaiveDate::from_ymd_opt(2026, 10, 3).unwrap();
/// assert_eq!(parse_relative("On 3rd October", today, window), Relative::Resolved(oct_3));
/// assert_eq!(parse_relative("whenever", today, window), Relative::Unparsed);
/// ```
#[must_use]
pub fn parse_relative(text: &str, today: NaiveDate, window: Window) -> Relative {
    match grammar::parse(text, today) {
        Some(grammar::Parsed::Date(date)) => place(date, window),
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
    let second = first
        .succ_opt()
        .and_then(|next| resolve_date(day, month, None, next, NextOnOrAfter));
    let inside: Vec<NaiveDate> = [Some(first), second]
        .into_iter()
        .flatten()
        .filter(|date| window.contains(*date))
        .collect();
    match inside.as_slice() {
        [] => Relative::OutOfWindow(first),
        [only] => Relative::Resolved(*only),
        _ => Relative::Ambiguous(inside),
    }
}
