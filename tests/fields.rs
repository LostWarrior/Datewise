use chrono::{NaiveTime, Weekday};
use datewise::fields::{find_ordinal_date, find_time, find_time_range, TimeError};
use datewise::MAX_INPUT_BYTES;

fn t(h: u32, m: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(h, m, 0).unwrap()
}

fn range(text: &str) -> (NaiveTime, NaiveTime) {
    let found = find_time_range(text).unwrap().unwrap();
    (found.start, found.end)
}

#[test]
fn ordinal_date_requires_suffix() {
    assert!(find_ordinal_date("Applications close: 30 September 2026").is_none());
    let text = "Sports Day 24th June \u{2013} Weather Reminder";
    let m = find_ordinal_date(text).unwrap();
    assert_eq!((m.day, m.month, m.year), (24, 6, None));
    assert!(m.weekday.is_none());
    assert_eq!(&text[m.span], "24th June");
}

#[test]
fn ordinal_date_captures_weekday_and_year() {
    let text = "Sports Day on Wednesday 24th June 2026 will be fun";
    let m = find_ordinal_date(text).unwrap();
    assert_eq!(m.weekday, Some(Weekday::Wed));
    assert_eq!((m.day, m.month, m.year), (24, 6, Some(2026)));
    assert_eq!(&text[m.span], "24th June 2026");
}

#[test]
fn ordinal_date_accepts_abbreviations_and_ignores_embedded_numbers() {
    let m = find_ordinal_date("Sat 1st Sept").unwrap();
    assert_eq!((m.weekday, m.day, m.month), (Some(Weekday::Sat), 1, 9));
    assert!(find_ordinal_date("ref 124th June").is_none());
    assert!(find_ordinal_date("32nd June").is_none());
    assert!(find_ordinal_date("5th Marching").is_none());
    let m = find_ordinal_date("3rd May 20261").unwrap();
    assert_eq!(m.year, None);
}

#[test]
fn ordinal_date_span_is_valid_with_multibyte_text() {
    let text = "\u{1F600} caf\u{e9} \u{2013} 7th March 2027 \u{e9}";
    let m = find_ordinal_date(text).unwrap();
    assert_eq!(&text[m.span], "7th March 2027");
}

#[test]
fn time_range_infers_shared_meridiem() {
    assert_eq!(range("Reception 9-10am"), (t(9, 0), t(10, 0)));
    assert_eq!(range("Club 10:20-11:45am"), (t(10, 20), t(11, 45)));
    assert_eq!(range("library 7 - 8pm plus"), (t(19, 0), t(20, 0)));
    assert_eq!(range("Show (6pm \u{2013} 7pm)"), (t(18, 0), t(19, 0)));
    assert_eq!(range("Drop-in (2:50pm - 3:20pm)"), (t(14, 50), t(15, 20)));
    assert_eq!(range("11-1pm"), (t(11, 0), t(13, 0)));
    assert_eq!(range("11am-1"), (t(11, 0), t(13, 0)));
    assert_eq!(range("12-1pm"), (t(12, 0), t(13, 0)));
}

#[test]
fn time_range_span_covers_the_range() {
    let text = "Reception 9-10am sharp";
    let found = find_time_range(text).unwrap().unwrap();
    assert_eq!(&text[found.span], "9-10am");
}

#[test]
fn time_range_errors_when_unresolvable() {
    assert_eq!(
        find_time_range("11-10am"),
        Err(TimeError::AmbiguousMeridiem)
    );
}

#[test]
fn bare_numbers_are_not_a_range() {
    assert_eq!(find_time_range("rooms 9-10"), Ok(None));
    assert_eq!(find_time_range("Y2 - 3pm Choir"), Ok(None));
}

#[test]
fn doors_open_sentence_has_single_time_only() {
    let text = "Performance at 2pm \u{2013} doors open at 1:50pm";
    assert_eq!(find_time_range(text), Ok(None));
    assert_eq!(find_time(text).unwrap().time, t(14, 0));
}

#[test]
fn single_time_forms() {
    assert_eq!(find_time("Term Ends at 11.45am").unwrap().time, t(11, 45));
    assert_eq!(find_time("Y2 Choir Performance 8am").unwrap().time, t(8, 0));
    assert_eq!(find_time("at 12am").unwrap().time, t(0, 0));
    assert_eq!(find_time("at 12 PM.").unwrap().time, t(12, 0));
    assert!(find_time("at 9").is_none());
    assert!(find_time("at 13pm").is_none());
    assert!(find_time("at 9:75am").is_none());
    let text = "meet 3pm";
    assert_eq!(&text[find_time(text).unwrap().span], "3pm");
}

#[test]
fn over_long_input_is_rejected() {
    let long = format!("{} 3pm 24th June", "x".repeat(MAX_INPUT_BYTES));
    assert!(find_time(&long).is_none());
    assert!(find_ordinal_date(&long).is_none());
    assert_eq!(find_time_range(&long), Ok(None));
}
