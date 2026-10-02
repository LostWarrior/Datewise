use chrono::Weekday;
use datewise::fields::{find_date, FoundDate};
use datewise::locale::Locale;

fn found(text: &str, locale: Option<Locale>) -> (String, FoundDate) {
    let date = find_date(text, locale).unwrap();
    (text[date.span.clone()].to_string(), date)
}

fn fields(text: &str) -> (u32, u32, Option<i32>) {
    let (_, date) = found(text, None);
    (date.day, date.month, date.year)
}

#[test]
fn day_first_phrases() {
    assert_eq!(fields("on 3rd October"), (3, 10, None));
    assert_eq!(fields("on 3 October 2026."), (3, 10, Some(2026)));
    assert_eq!(fields("due 03 Oct"), (3, 10, None));
    assert_eq!(found("see 3 Oct 2026", None).0, "3 Oct 2026");
}

#[test]
fn month_first_phrases() {
    assert_eq!(fields("by October 3rd"), (3, 10, None));
    assert_eq!(fields("by October 3, 2026"), (3, 10, Some(2026)));
    assert_eq!(fields("by Oct 3 2026"), (3, 10, Some(2026)));
    assert_eq!(found("by Oct 3 at 4pm", None).0, "Oct 3");
}

#[test]
fn iso_and_numeric_phrases() {
    assert_eq!(fields("on 2026-10-03 10:00"), (3, 10, Some(2026)));
    let (_, ambiguous) = found("on 03/10/2026", None);
    assert!(ambiguous.ambiguous);
    assert_eq!((ambiguous.day, ambiguous.month), (3, 10));
    let (_, gb) = found("on 03/10/2026", Some(Locale::EN_GB));
    assert!(!gb.ambiguous);
    let (_, us) = found("on 03/10/2026", Some(Locale::EN_US));
    assert_eq!((us.day, us.month, us.year), (10, 3, Some(2026)));
    let (_, same) = found("on 5/5/26", None);
    assert!(!same.ambiguous);
}

#[test]
fn weekday_and_earliest() {
    let (span, date) = found("Meet Fri 3 October, or Sat 4th October", None);
    assert_eq!(span, "3 October");
    assert_eq!(date.weekday, Some(Weekday::Fri));
    let (span, _) = found("Oct 5 then 3rd June", None);
    assert_eq!(span, "Oct 5");
    let (span, date) = found("\u{1F600} caf\u{e9} Sunday 7 March 2027 \u{e9}", None);
    assert_eq!(
        (span.as_str(), date.weekday),
        ("7 March 2027", Some(Weekday::Sun))
    );
}

#[test]
fn false_positives_are_ignored() {
    for text in [
        "3 people",
        "May I come",
        "we march on",
        "Year 2026",
        "version 1.2.3 and v1.2.2026.1",
        "at 3.30 or 3:30",
        "October 2026",
        "31 April",
        "30 February 2026",
        "x3 October",
        "xOctober 3",
        "ref 124 October",
        "Oct 123",
        "2026-13-01",
        "12026-10-03",
        "2026-10-033",
    ] {
        assert_eq!(find_date(text, None), None, "{text}");
    }
}

#[test]
fn date_at_the_end_of_long_text() {
    let filler = "word ".repeat(2_000);
    let text = format!("{filler}Wed 3 October 2026");
    let date = find_date(&text, None).unwrap();
    assert_eq!((date.day, date.weekday), (3, Some(Weekday::Wed)));
}
