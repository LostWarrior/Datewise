use chrono::{NaiveTime, Weekday};
use datewise::fields::{find_ordinal_date, find_time, find_time_range, TimeError};

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
    assert!(find_time("at 9:75am").is_none());
    let text = "meet 3pm";
    assert_eq!(&text[find_time(text).unwrap().span], "3pm");
}

#[test]
fn finders_see_the_end_of_long_text() {
    use chrono::NaiveDate;
    use datewise::fields::find_numeric_date;
    use datewise::locale::DateOrder;
    use datewise::zone::{find_zone, FoundZone};
    let filler = "lorem ipsum dolor sit amet, ".repeat(400);
    assert!(filler.len() > 10_000);
    let text = format!("{filler}Sat 24th June, 3pm - 4:30pm UTC, 03/10/2026");
    assert_eq!(find_ordinal_date(&text).unwrap().day, 24);
    assert_eq!(&text[find_time(&text).unwrap().span], "3pm");
    assert!(find_time_range(&text).unwrap().is_some());
    assert!(matches!(
        find_zone(&text, None).unwrap().zone,
        FoundZone::Fixed(_)
    ));
    let numeric = find_numeric_date(&text, Some(DateOrder::DayFirst)).unwrap();
    assert_eq!(
        numeric.dates,
        vec![NaiveDate::from_ymd_opt(2026, 10, 3).unwrap()]
    );
}

#[test]
fn numeric_date_in_text_reports_span_and_readings() {
    use chrono::NaiveDate;
    use datewise::fields::find_numeric_date;
    use datewise::locale::DateOrder;
    let text = "Due 03/10/2026, then 13.10.26.";
    let first = find_numeric_date(text, None).unwrap();
    assert_eq!(&text[first.span.clone()], "03/10/2026");
    assert_eq!(first.dates.len(), 2);
    let gb = find_numeric_date(text, Some(DateOrder::DayFirst)).unwrap();
    assert_eq!(
        gb.dates,
        vec![NaiveDate::from_ymd_opt(2026, 10, 3).unwrap()]
    );
    let rest = &text[first.span.end..];
    let second = find_numeric_date(rest, None).unwrap();
    assert_eq!(&rest[second.span], "13.10.26");
    assert_eq!(
        second.dates,
        vec![NaiveDate::from_ymd_opt(2026, 10, 13).unwrap()]
    );
    for bad in [
        "v1.2.2026.1",
        "a03/10/2026",
        "03/10/2026x",
        "1.3.10.2026",
        "12345/10/2026",
    ] {
        assert_eq!(find_numeric_date(bad, None), None, "{bad}");
    }
}

#[test]
fn malformed_time_continuations_are_rejected() {
    for text in [
        "7pm-8:99",
        "7pm-8foo",
        "7pm-8:9",
        "7pm-8:123",
        "7pm-8:30x",
        "7pm-123",
    ] {
        assert_eq!(find_time_range(text), Ok(None), "{text}");
    }
    assert!(find_time("meet 8:99pm").is_none());
    assert!(find_time("meet 7pm5").is_none());
    assert!(find_time("meet 8:5pm").is_none());
}

#[test]
fn a_match_never_starts_inside_another_clock() {
    for text in [
        "13:05am",
        "00:30pm",
        "0:30pm",
        "0pm",
        "9:75pm",
        "5.45.30pm",
        "9:30:45pm",
    ] {
        assert!(find_time(text).is_none(), "{text}");
    }
    assert_eq!(find_time("13:05pm").unwrap().time, t(13, 5));
    assert_eq!(find_time("at 13pm").unwrap().time, t(13, 0));
    assert_eq!(find_time("23:59pm").unwrap().time, t(23, 59));
    assert!(find_time("24:00pm").is_none());
}

#[test]
fn valid_ranges_still_parse() {
    assert_eq!(range("7pm-8pm"), (t(19, 0), t(20, 0)));
    assert_eq!(range("7-8pm"), (t(19, 0), t(20, 0)));
    assert_eq!(range("9.30am-10.15am"), (t(9, 30), t(10, 15)));
    assert_eq!(find_time_range("19:00-20:30"), Ok(None));
    assert_eq!(find_time("9.30am").unwrap().time, t(9, 30));
}

#[test]
fn dotted_meridiems_parse() {
    assert_eq!(find_time("at 9a.m.").unwrap().time, t(9, 0));
    assert_eq!(find_time("at 9 p.m. sharp").unwrap().time, t(21, 0));
    assert_eq!(find_time("at 9.30 A.M.").unwrap().time, t(9, 30));
    assert_eq!(range("7 p.m.-8 p.m."), (t(19, 0), t(20, 0)));
    assert!(find_time("9 a.mx").is_none());
}
