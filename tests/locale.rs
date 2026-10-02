use chrono::{NaiveDate, Weekday};
use datewise::locale::{DateOrder, Locale};
use datewise::relative::{parse_relative, Relative, Window};

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

// Friday 2 October 2026.
fn parse(text: &str, locale: Option<Locale>) -> Relative {
    let window = Window::new(d(2026, 1, 1), d(2027, 12, 31)).unwrap();
    parse_relative(text, d(2026, 10, 2), window, locale)
}

#[test]
fn tags_are_flexible_but_need_a_known_region() {
    for tag in ["en-GB", "en_GB", "EN-gb", "en_gb"] {
        assert_eq!(Locale::from_tag(tag), Some(Locale::EN_GB), "{tag}");
    }
    for tag in [
        "en",
        "en-",
        "en-XX",
        "fr-FR",
        "en-GB-x",
        "",
        "GB",
        "en-Latn-GB",
        "\u{e9}",
    ] {
        assert_eq!(Locale::from_tag(tag), None, "{tag}");
    }
}

#[test]
fn region_table() {
    let table = [
        ("GB", DateOrder::DayFirst, Weekday::Mon),
        ("IE", DateOrder::DayFirst, Weekday::Mon),
        ("US", DateOrder::MonthFirst, Weekday::Sun),
        ("CA", DateOrder::YearFirst, Weekday::Sun),
        ("AU", DateOrder::DayFirst, Weekday::Mon),
        ("NZ", DateOrder::DayFirst, Weekday::Mon),
        ("IN", DateOrder::DayFirst, Weekday::Sun),
        ("ZA", DateOrder::YearFirst, Weekday::Sun),
        ("SG", DateOrder::DayFirst, Weekday::Sun),
        ("PH", DateOrder::MonthFirst, Weekday::Sun),
    ];
    for (region, order, start) in table {
        let locale = Locale::from_tag(&format!("en-{region}")).unwrap();
        assert_eq!(locale.region(), region);
        assert_eq!(locale.date_order(), order, "{region}");
        assert_eq!(locale.week_start(), start, "{region}");
    }
}

#[test]
fn numeric_date_follows_the_locale() {
    assert_eq!(
        parse("03/10/2026", Some(Locale::EN_GB)),
        Relative::Resolved(d(2026, 10, 3))
    );
    assert_eq!(
        parse("03/10/2026", Some(Locale::EN_US)),
        Relative::Resolved(d(2026, 3, 10))
    );
    assert_eq!(
        parse("03/10/2026", None),
        Relative::Ambiguous(vec![d(2026, 10, 3), d(2026, 3, 10)])
    );
    assert_eq!(
        parse("3/10/26", Some(Locale::EN_CA)),
        Relative::Ambiguous(vec![d(2026, 10, 3), d(2026, 3, 10)])
    );
}

#[test]
fn impossible_readings_leave_one() {
    assert_eq!(
        parse("13/10/2026", None),
        Relative::Resolved(d(2026, 10, 13))
    );
    assert_eq!(
        parse("10/13/2026", None),
        Relative::Resolved(d(2026, 10, 13))
    );
    assert_eq!(
        parse("13/10/2026", Some(Locale::EN_GB)),
        Relative::Resolved(d(2026, 10, 13))
    );
    assert_eq!(parse("13/10/2026", Some(Locale::EN_US)), Relative::Unparsed);
    assert_eq!(parse("31/04/2026", None), Relative::Unparsed);
    assert_eq!(parse("03/03/2026", None), Relative::Resolved(d(2026, 3, 3)));
}

#[test]
fn year_first_and_other_separators() {
    for locale in [
        None,
        Some(Locale::EN_GB),
        Some(Locale::EN_US),
        Some(Locale::EN_CA),
    ] {
        assert_eq!(
            parse("2026/10/03", locale),
            Relative::Resolved(d(2026, 10, 3))
        );
        assert_eq!(
            parse("2026.10.03", locale),
            Relative::Resolved(d(2026, 10, 3))
        );
        assert_eq!(
            parse("2026-10-03", locale),
            Relative::Resolved(d(2026, 10, 3))
        );
    }
    let gb = Some(Locale::EN_GB);
    assert_eq!(parse("3.10.2026", gb), Relative::Resolved(d(2026, 10, 3)));
    assert_eq!(parse("3-10-2026", gb), Relative::Resolved(d(2026, 10, 3)));
    assert_eq!(parse("on 3/10/26.", gb), Relative::Resolved(d(2026, 10, 3)));
    assert_eq!(parse("3/10/99", gb), Relative::OutOfWindow(d(2099, 10, 3)));
    for bad in [
        "3-10-26",
        "26/10/03/1",
        "2026/10",
        "2026-10-3",
        "3/10/202",
        "1/2/3/4",
    ] {
        assert_eq!(parse(bad, None), Relative::Unparsed, "{bad}");
    }
}

#[test]
fn week_span_uses_the_locale_week_start() {
    let span = |a, b| Relative::Span(a, b);
    assert_eq!(
        parse("this week", Some(Locale::EN_GB)),
        span(d(2026, 9, 28), d(2026, 10, 4))
    );
    assert_eq!(
        parse("this week", Some(Locale::EN_US)),
        span(d(2026, 9, 27), d(2026, 10, 3))
    );
    assert_eq!(
        parse("next week", Some(Locale::EN_US)),
        span(d(2026, 10, 4), d(2026, 10, 10))
    );
    assert_eq!(
        parse("next week", Some(Locale::EN_GB)),
        span(d(2026, 10, 5), d(2026, 10, 11))
    );
}

#[test]
fn next_weekday_uses_the_locale_week_start() {
    let ok = |y, m, day| Relative::Resolved(d(y, m, day));
    assert_eq!(parse("next Sunday", Some(Locale::EN_GB)), ok(2026, 10, 11));
    assert_eq!(parse("next Sunday", Some(Locale::EN_US)), ok(2026, 10, 4));
    assert_eq!(
        parse("next Saturday", Some(Locale::EN_US)),
        ok(2026, 10, 10)
    );
    assert_eq!(parse("next Monday", Some(Locale::EN_US)), ok(2026, 10, 5));
}
