//! English month and weekday names, and ordinal suffixes.

use crate::scan::boundary;
use chrono::Weekday;

// Longest spelling first within each entry group.
pub(crate) const MONTHS: &[(&str, u32)] = &[
    ("january", 1),
    ("jan", 1),
    ("february", 2),
    ("feb", 2),
    ("march", 3),
    ("mar", 3),
    ("april", 4),
    ("apr", 4),
    ("may", 5),
    ("june", 6),
    ("jun", 6),
    ("july", 7),
    ("jul", 7),
    ("august", 8),
    ("aug", 8),
    ("september", 9),
    ("sept", 9),
    ("sep", 9),
    ("october", 10),
    ("oct", 10),
    ("november", 11),
    ("nov", 11),
    ("december", 12),
    ("dec", 12),
];

const WEEKDAYS: &[(&str, Weekday)] = &[
    ("monday", Weekday::Mon),
    ("mon", Weekday::Mon),
    ("tuesday", Weekday::Tue),
    ("tues", Weekday::Tue),
    ("tue", Weekday::Tue),
    ("wednesday", Weekday::Wed),
    ("wed", Weekday::Wed),
    ("thursday", Weekday::Thu),
    ("thurs", Weekday::Thu),
    ("thur", Weekday::Thu),
    ("thu", Weekday::Thu),
    ("friday", Weekday::Fri),
    ("fri", Weekday::Fri),
    ("saturday", Weekday::Sat),
    ("sat", Weekday::Sat),
    ("sunday", Weekday::Sun),
    ("sun", Weekday::Sun),
];

/// Month number (1-12) for a full or abbreviated name, ignoring case.
#[must_use]
pub fn month_from_name(name: &str) -> Option<u32> {
    MONTHS
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|&(_, month)| month)
}

/// Weekday for a full or abbreviated name, ignoring case.
#[must_use]
pub fn weekday_from_name(name: &str) -> Option<Weekday> {
    WEEKDAYS
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|&(_, day)| day)
}

/// Length (2) of a leading `st`/`nd`/`rd`/`th` not followed by a letter.
#[must_use]
pub fn ordinal_suffix_len(s: &str) -> Option<usize> {
    let head = s.get(..2)?;
    let known = ["st", "nd", "rd", "th"]
        .iter()
        .any(|suffix| head.eq_ignore_ascii_case(suffix));
    (known && boundary(s.get(2..)?)).then_some(2)
}
