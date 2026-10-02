use chrono::NaiveDate;
use datewise::relative::{parse_relative, Relative, Window};

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

// Friday 2 October 2026, window through the end of the year.
fn parse(text: &str) -> Relative {
    parse_at(text, d(2026, 10, 2), d(2026, 10, 1), d(2026, 12, 31))
}

fn parse_at(text: &str, today: NaiveDate, start: NaiveDate, end: NaiveDate) -> Relative {
    parse_relative(text, today, Window::new(start, end).unwrap(), None)
}

fn ok(y: i32, m: u32, day: u32) -> Relative {
    Relative::Resolved(d(y, m, day))
}

#[test]
fn window_rejects_inverted_range_and_includes_edges() {
    assert!(Window::new(d(2026, 2, 1), d(2026, 1, 1)).is_none());
    let w = Window::new(d(2026, 1, 1), d(2026, 1, 1)).unwrap();
    assert!(w.contains(d(2026, 1, 1)));
    assert!(!w.contains(d(2026, 1, 2)));
    assert_eq!((w.start(), w.end()), (d(2026, 1, 1), d(2026, 1, 1)));
}

#[test]
fn simple_days() {
    assert_eq!(parse("today"), ok(2026, 10, 2));
    assert_eq!(parse("tomorrow"), ok(2026, 10, 3));
    assert_eq!(parse("yesterday"), ok(2026, 10, 1));
}

#[test]
fn bare_weekday_is_next_on_or_after_today() {
    assert_eq!(parse("Friday"), ok(2026, 10, 2));
    assert_eq!(parse("saturday"), ok(2026, 10, 3));
    assert_eq!(parse("thu"), ok(2026, 10, 8));
    assert_eq!(parse("Monday"), ok(2026, 10, 5));
}

#[test]
fn next_weekday_is_in_the_following_week() {
    assert_eq!(parse("next Monday"), ok(2026, 10, 5));
    assert_eq!(parse("next Tuesday"), ok(2026, 10, 6));
    assert_eq!(parse("next Friday"), ok(2026, 10, 9));
    assert_eq!(parse("next Sunday"), ok(2026, 10, 11));
    let sunday = d(2026, 10, 4);
    let (s, e) = (d(2026, 10, 1), d(2026, 12, 31));
    assert_eq!(parse_at("next Monday", sunday, s, e), ok(2026, 10, 5));
    assert_eq!(parse_at("next Sunday", sunday, s, e), ok(2026, 10, 11));
}

#[test]
fn week_phrases_return_a_monday_span_by_default() {
    let span = |a, b| Relative::Span(a, b);
    assert_eq!(parse("this week"), span(d(2026, 9, 28), d(2026, 10, 4)));
    assert_eq!(parse("next week"), span(d(2026, 10, 5), d(2026, 10, 11)));
    let (s, e) = (d(2026, 12, 1), d(2027, 1, 31));
    assert_eq!(
        parse_at("next week", d(2026, 12, 30), s, e),
        span(d(2027, 1, 4), d(2027, 1, 10))
    );
    let before = parse_at("this week", d(2026, 10, 2), d(2026, 10, 5), d(2026, 10, 9));
    assert_eq!(before, Relative::OutOfWindow(d(2026, 9, 28)));
}

#[test]
fn day_of_month_skips_short_months() {
    assert_eq!(parse("the 14th"), ok(2026, 10, 14));
    assert_eq!(parse("the 2nd"), ok(2026, 10, 2));
    assert_eq!(parse("the 1st"), ok(2026, 11, 1));
    assert_eq!(parse("on the 31st"), ok(2026, 10, 31));
    let (s, e) = (d(2026, 1, 1), d(2026, 12, 31));
    assert_eq!(parse_at("the 30th", d(2026, 2, 1), s, e), ok(2026, 3, 30));
    assert_eq!(parse_at("the 31st", d(2026, 4, 15), s, e), ok(2026, 5, 31));
    assert_eq!(parse_at("the 31st", d(2026, 2, 28), s, e), ok(2026, 3, 31));
    assert_eq!(
        parse_at("the 2nd", d(2026, 12, 30), s, d(2027, 1, 31)),
        ok(2027, 1, 2)
    );
    assert_eq!(parse_at("the 29th", d(2026, 2, 1), s, e), ok(2026, 3, 29));
}

#[test]
fn day_month_forms() {
    for text in [
        "3rd October",
        "October 3rd",
        "3 Oct",
        "on 3 oct.",
        "Oct 3",
        "THE 3RD OCT",
    ] {
        assert_eq!(parse(text), ok(2026, 10, 3), "{text}");
    }
    assert_eq!(parse("3rd October 2026"), ok(2026, 10, 3));
    assert_eq!(parse("October 3rd, 2026"), ok(2026, 10, 3));
    assert_eq!(parse("25 dec"), ok(2026, 12, 25));
}

#[test]
fn year_rolls_forward_and_may_leave_the_window() {
    assert_eq!(parse("2nd Oct"), ok(2026, 10, 2));
    assert_eq!(parse("1st October"), Relative::OutOfWindow(d(2027, 10, 1)));
    assert_eq!(
        parse("3rd October 2025"),
        Relative::OutOfWindow(d(2025, 10, 3))
    );
    let (s, e) = (d(2026, 12, 1), d(2027, 1, 31));
    assert_eq!(
        parse_at("2nd January", d(2026, 12, 30), s, e),
        ok(2027, 1, 2)
    );
}

#[test]
fn window_edges_are_inclusive() {
    let (today, edge) = (d(2026, 10, 2), d(2026, 10, 9));
    assert_eq!(parse_at("next Friday", today, today, edge), ok(2026, 10, 9));
    let before = d(2026, 10, 8);
    assert_eq!(
        parse_at("next Friday", today, today, before),
        Relative::OutOfWindow(edge)
    );
    assert_eq!(parse_at("today", today, today, today), ok(2026, 10, 2));
    assert_eq!(
        parse_at("tomorrow", today, today, today),
        Relative::OutOfWindow(d(2026, 10, 3))
    );
}

#[test]
fn leap_years() {
    let (s, e) = (d(2026, 1, 1), d(2030, 12, 31));
    assert_eq!(parse_at("29 Feb", d(2026, 3, 1), s, e), ok(2028, 2, 29));
    assert_eq!(
        parse_at("29 February 2028", d(2026, 3, 1), s, e),
        ok(2028, 2, 29)
    );
    assert_eq!(
        parse_at("29 February 2027", d(2026, 3, 1), s, e),
        Relative::Unparsed
    );
    assert_eq!(parse_at("tomorrow", d(2028, 2, 28), s, e), ok(2028, 2, 29));
    assert_eq!(parse_at("tomorrow", d(2027, 2, 28), s, e), ok(2027, 3, 1));
}

#[test]
fn month_and_year_end() {
    let (s, e) = (d(2026, 1, 1), d(2027, 12, 31));
    assert_eq!(parse_at("tomorrow", d(2026, 1, 31), s, e), ok(2026, 2, 1));
    assert_eq!(parse_at("tomorrow", d(2026, 12, 31), s, e), ok(2027, 1, 1));
    assert_eq!(parse_at("yesterday", d(2027, 1, 1), s, e), ok(2026, 12, 31));
}

#[test]
fn iso_dates() {
    assert_eq!(parse("2026-10-03"), ok(2026, 10, 3));
    assert_eq!(parse("on 2026-12-31."), ok(2026, 12, 31));
    assert_eq!(parse("2027-01-01"), Relative::OutOfWindow(d(2027, 1, 1)));
    for bad in [
        "2026-13-01",
        "2026-02-30",
        "2026-2-3",
        "2026-10-03-1",
        "26-10-03",
        "2026--10",
    ] {
        assert_eq!(parse(bad), Relative::Unparsed, "{bad}");
    }
}

#[test]
fn yearless_date_with_several_occurrences_is_ambiguous() {
    let (today, s, e) = (d(2026, 10, 2), d(2026, 10, 1), d(2028, 12, 31));
    let expected = Relative::Ambiguous(vec![d(2026, 10, 3), d(2027, 10, 3)]);
    assert_eq!(parse_at("3 Oct", today, s, e), expected);
    assert_eq!(parse_at("3 Oct 2027", today, s, e), ok(2027, 10, 3));
}

#[test]
fn tolerant_of_case_space_and_punctuation() {
    for text in [
        "  ON   Tomorrow!! ",
        "TOMORROW.",
        "\ttomorrow\n",
        "(tomorrow)",
        "on, tomorrow",
    ] {
        assert_eq!(parse(text), ok(2026, 10, 3), "{text:?}");
    }
    assert_eq!(parse("Next   FRIDAY,"), ok(2026, 10, 9));
}

#[test]
fn unsupported_text_is_unparsed() {
    let cases = [
        "",
        "   ",
        "soon",
        "next",
        "the",
        "on",
        "next week please",
        "32nd October",
        "0th October",
        "31 April",
        "13 Oct 26",
        "Oct",
        "this Friday",
        "1 2 3 4 5 6 7",
        "3rd Octobe",
        "caf\u{e9}",
        "\u{1F600}",
        "14",
        "the 14",
    ];
    for text in cases {
        assert_eq!(parse(text), Relative::Unparsed, "{text:?}");
    }
    assert_eq!(parse(&"a ".repeat(400)), Relative::Unparsed);
}

#[test]
fn extreme_dates_do_not_panic() {
    let w = Window::new(NaiveDate::MIN, NaiveDate::MAX).unwrap();
    for today in [NaiveDate::MIN, NaiveDate::MAX] {
        for text in [
            "tomorrow",
            "yesterday",
            "next Friday",
            "this week",
            "the 14th",
            "3 Oct",
            "Friday",
        ] {
            let _ = parse_relative(text, today, w, None);
        }
    }
}
