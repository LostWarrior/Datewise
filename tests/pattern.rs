use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use datewise::{ConfigError, Datewise, FormatError, ParseError, Parsed};

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

fn dt(y: i32, m: u32, day: u32, h: u32, min: u32, s: u32) -> NaiveDateTime {
    d(y, m, day).and_hms_opt(h, min, s).unwrap()
}

fn instant(rfc3339: &str) -> Parsed {
    Parsed::Instant(DateTime::<FixedOffset>::parse_from_rfc3339(rfc3339).unwrap())
}

fn p(pattern: &str) -> Datewise {
    Datewise::pattern(pattern).unwrap()
}

fn parse(pattern: &str, text: &str) -> Result<Parsed, ParseError> {
    p(pattern).parse(text)
}

fn format(pattern: &str, value: Parsed) -> Result<String, FormatError> {
    p(pattern).format(&value)
}

#[test]
fn acceptance_snippets() -> Result<(), Box<dyn std::error::Error>> {
    let dw = Datewise::new("en-GB", "2026-10-02")?;
    assert_eq!(dw.parse("next friday")?.to_string(), "2026-10-09");
    let p = Datewise::pattern("dd/MM/yyyy")?;
    let d = p.parse("03/11/2026")?;
    assert_eq!(Datewise::pattern("yyyy-MM-dd")?.format(&d)?, "2026-11-03");
    Ok(())
}

#[test]
fn every_letter_parses_formats_and_round_trips() {
    let pm = Parsed::DateTime(dt(2026, 11, 3, 15, 4, 5));
    let am = Parsed::DateTime(dt(2026, 1, 9, 9, 7, 8));
    let cases = [
        ("yyyy-MM-dd HH:mm:ss", "2026-11-03 15:04:05", pm),
        ("dd/MM/yy H:m:s", "03/11/26 15:4:5", pm),
        (
            "d/M/yyyy h:mm a",
            "9/1/2026 9:07 AM",
            Parsed::DateTime(dt(2026, 1, 9, 9, 7, 0)),
        ),
        (
            "EEE d MMM yyyy hh:mm:ss a",
            "Fri 9 Jan 2026 09:07:08 AM",
            am,
        ),
        (
            "EEEE, MMMM d, yyyy HH:mm:ss",
            "Tuesday, November 3, 2026 15:04:05",
            pm,
        ),
        (
            "E dd MMM yyyy",
            "Tue 03 Nov 2026",
            Parsed::Date(d(2026, 11, 3)),
        ),
    ];
    for (pattern, text, value) in cases {
        assert_eq!(parse(pattern, text), Ok(value), "{pattern}");
        assert_eq!(format(pattern, value).as_deref(), Ok(text), "{pattern}");
    }
}

#[test]
fn two_digit_years_pivot_at_69() {
    for (yy, year) in [
        ("70", 1970),
        ("69", 1969),
        ("68", 2068),
        ("46", 2046),
        ("05", 2005),
    ] {
        assert_eq!(
            parse("dd/MM/yy", &format!("03/10/{yy}")),
            Ok(Parsed::Date(d(year, 10, 3)))
        );
    }
    assert_eq!(
        parse("dd/MM/yy", "29/02/00"),
        Ok(Parsed::Date(d(2000, 2, 29)))
    );
    let gb = Datewise::new("en-GB", "2026-10-02").unwrap();
    assert_eq!(gb.parse("03/10/70"), Ok(Parsed::Date(d(1970, 10, 3))));
    assert_eq!(gb.parse("03/10/69"), Ok(Parsed::Date(d(1969, 10, 3))));
    assert_eq!(gb.parse("03/10/68"), Ok(Parsed::Date(d(2068, 10, 3))));
}

#[test]
fn widths_follow_go() {
    assert_eq!(parse("dd/MM/yyyy", "3/11/2026"), Err(ParseError::Unparsed));
    assert_eq!(parse("dd/MM/yyyy", "03/1/2026"), Err(ParseError::Unparsed));
    assert_eq!(
        parse("d/M/yyyy", "3/11/2026"),
        Ok(Parsed::Date(d(2026, 11, 3)))
    );
    assert_eq!(
        parse("d/M/yyyy", "03/11/2026"),
        Ok(Parsed::Date(d(2026, 11, 3)))
    );
    let nine = Ok(Parsed::DateTime(dt(2026, 11, 3, 9, 5, 0)));
    assert_eq!(parse("dd/MM/yyyy HH:mm", "03/11/2026 9:05"), nine);
    assert_eq!(parse("dd/MM/yyyy HH:mm", "03/11/2026 09:05"), nine);
    assert_eq!(
        parse("dd/MM/yyyy HH:mm", "03/11/2026 09:5"),
        Err(ParseError::Unparsed)
    );
    assert_eq!(parse("dd/MM/yyyy H:m", "03/11/2026 9:5"), nine);
    assert_eq!(parse("yyyy", "26"), Err(ParseError::Unparsed));
    assert_eq!(parse("dd/MM/yy", "03/11/2026"), Err(ParseError::Unparsed));
}

#[test]
fn am_pm_ignores_case_without_dots() {
    let at = |text: &str| parse("dd/MM/yyyy hh:mm a", text);
    let pm = Ok(Parsed::DateTime(dt(2026, 11, 3, 15, 30, 0)));
    assert_eq!(at("03/11/2026 03:30 PM"), pm);
    assert_eq!(at("03/11/2026 03:30 pm"), pm);
    assert_eq!(
        at("03/11/2026 12:00 AM"),
        Ok(Parsed::DateTime(dt(2026, 11, 3, 0, 0, 0)))
    );
    assert_eq!(
        at("03/11/2026 12:00 pm"),
        Ok(Parsed::DateTime(dt(2026, 11, 3, 12, 0, 0)))
    );
    assert_eq!(at("03/11/2026 03:30 p.m."), Err(ParseError::Unparsed));
    assert_eq!(at("03/11/2026 13:00 PM"), Err(ParseError::Unparsed));
    assert_eq!(at("03/11/2026 00:00 AM"), Err(ParseError::Unparsed));
    assert_eq!(
        parse("dd/MM/yyyy hh:mm", "03/11/2026 03:30"),
        Err(ParseError::Unparsed)
    );
}

#[test]
fn out_of_range_values_are_rejected() {
    assert_eq!(parse("dd/MM/yyyy", "31/02/2026"), Err(ParseError::Unparsed));
    assert_eq!(parse("dd/MM/yyyy", "03/13/2026"), Err(ParseError::Unparsed));
    assert_eq!(
        parse("dd/MM/yyyy HH:mm", "03/11/2026 24:00"),
        Err(ParseError::Unparsed)
    );
    assert_eq!(
        parse("dd/MM/yyyy HH:mm", "03/11/2026 23:60"),
        Err(ParseError::Unparsed)
    );
    assert_eq!(
        parse("dd/MM/yyyy HH:mm XXX", "03/11/2026 10:00 +01:60"),
        Err(ParseError::Unparsed)
    );
}

#[test]
fn names_ignore_case_and_weekday_must_match() {
    for month in ["jan", "JAN", "Jan"] {
        assert_eq!(
            parse("dd MMM yyyy", &format!("05 {month} 2026")),
            Ok(Parsed::Date(d(2026, 1, 5)))
        );
    }
    assert_eq!(
        parse("MMMM d yyyy", "JANUARY 5 2026"),
        Ok(Parsed::Date(d(2026, 1, 5)))
    );
    assert_eq!(
        parse("MMMM d yyyy", "Jan 5 2026"),
        Err(ParseError::Unparsed)
    );
    assert_eq!(
        parse("EEE dd/MM/yyyy", "tue 06/10/2026"),
        Ok(Parsed::Date(d(2026, 10, 6)))
    );
    assert_eq!(
        parse("EEE dd/MM/yyyy", "Mon 06/10/2026"),
        Err(ParseError::Unparsed)
    );
    assert_eq!(
        parse("EEEE dd/MM/yyyy", "Tue 06/10/2026"),
        Err(ParseError::Unparsed)
    );
}

#[test]
fn offsets_use_z_for_zero() {
    let pattern = "yyyy-MM-dd'T'HH:mm:ssXXX";
    let utc = instant("2026-11-03T15:04:05+00:00");
    assert_eq!(parse(pattern, "2026-11-03T15:04:05Z"), Ok(utc));
    assert_eq!(format(pattern, utc).as_deref(), Ok("2026-11-03T15:04:05Z"));
    let bst = instant("2026-11-03T15:04:05+01:00");
    assert_eq!(parse(pattern, "2026-11-03T15:04:05+01:00"), Ok(bst));
    assert_eq!(
        format(pattern, bst).as_deref(),
        Ok("2026-11-03T15:04:05+01:00")
    );
    let ist = instant("2026-11-03T15:04:05-05:30");
    assert_eq!(parse(pattern, "2026-11-03T15:04:05-05:30"), Ok(ist));
    assert_eq!(
        parse(pattern, "2026-11-03T15:04:05+0100"),
        Err(ParseError::Unparsed)
    );
    assert_eq!(
        parse(pattern, "2026-11-03T15:04:05z"),
        Err(ParseError::Unparsed)
    );
    assert_eq!(
        parse("yyyy-MM-ddXXX", "2026-11-03Z"),
        Err(ParseError::Unparsed)
    );
}

#[test]
fn omitted_minutes_and_seconds_are_zero() {
    assert_eq!(
        parse("dd/MM/yyyy HH:mm", "03/11/2026 15:30"),
        Ok(Parsed::DateTime(dt(2026, 11, 3, 15, 30, 0)))
    );
    assert_eq!(
        parse("dd/MM/yyyy HH", "03/11/2026 15"),
        Ok(Parsed::DateTime(dt(2026, 11, 3, 15, 0, 0)))
    );
    assert_eq!(
        parse("dd/MM/yyyy h a", "03/11/2026 3 pm"),
        Ok(Parsed::DateTime(dt(2026, 11, 3, 15, 0, 0)))
    );
    assert_eq!(
        parse("dd/MM/yyyy ss", "03/11/2026 05"),
        Err(ParseError::Unparsed)
    );
}

#[test]
fn incomplete_patterns_format_but_do_not_parse() {
    assert_eq!(parse("HH:mm", "15:30"), Err(ParseError::Unparsed));
    assert_eq!(parse("MM/yyyy", "11/2026"), Err(ParseError::Unparsed));
    assert_eq!(parse("dd/MM", "03/11"), Err(ParseError::Unparsed));
    let value = Parsed::DateTime(dt(2026, 11, 3, 15, 30, 0));
    assert_eq!(format("HH:mm", value).as_deref(), Ok("15:30"));
    assert_eq!(format("MM/yyyy", value).as_deref(), Ok("11/2026"));
}

#[test]
fn literals_and_quotes() {
    let value = Parsed::DateTime(dt(2026, 11, 3, 15, 30, 0));
    assert_eq!(parse("yyyy-MM-dd'T'HH:mm", "2026-11-03T15:30"), Ok(value));
    assert_eq!(
        parse("yyyy-MM-dd'T'HH:mm", "2026-11-03 15:30"),
        Err(ParseError::Unparsed)
    );
    assert_eq!(
        format("HH 'o''clock' dd''MM''yyyy", value).as_deref(),
        Ok("15 o'clock 03'11'2026")
    );
    assert_eq!(
        parse("HH 'o''clock' dd''MM''yyyy", "15 o'clock 03'11'2026"),
        Ok(Parsed::DateTime(dt(2026, 11, 3, 15, 0, 0)))
    );
    assert_eq!(format("'at' HH%mm", value).as_deref(), Ok("at 15%30"));
    assert_eq!(
        parse("dd.MM.yyyy", "03.11.2026"),
        Ok(Parsed::Date(d(2026, 11, 3)))
    );
}

#[test]
fn unsupported_patterns_are_config_errors() {
    for bad in [
        "Q", "w", "yyy", "yyyyy", "MMMMM", "ddd", "HHH", "aa", "X", "XX", "SSS", "'open", "dd 'x",
    ] {
        assert_eq!(
            Datewise::pattern(bad).unwrap_err(),
            ConfigError::UnsupportedPattern,
            "{bad}"
        );
    }
    assert_eq!(
        ConfigError::UnsupportedPattern.to_string(),
        "unsupported date pattern"
    );
}

#[test]
fn whole_input_must_match() {
    for text in [
        "03/11/2026x",
        " 03/11/2026",
        "03/11/2026 ",
        "03/11/20261",
        "",
        "03-11-2026",
    ] {
        assert_eq!(
            parse("dd/MM/yyyy", text),
            Err(ParseError::Unparsed),
            "{text:?}"
        );
    }
    assert_eq!(
        p("dd/MM/yyyy").parse(&"1".repeat(600)),
        Err(ParseError::Unparsed)
    );
}

#[test]
fn window_applies_and_prefer_past_is_ignored() {
    let w = p("dd/MM/yyyy").within("2026-10-01", "2026-12-31").unwrap();
    assert_eq!(w.parse("03/11/2026"), Ok(Parsed::Date(d(2026, 11, 3))));
    assert_eq!(w.parse("03/11/2027"), Err(ParseError::OutOfWindow));
    let timed = p("dd/MM/yyyy HH:mmXXX")
        .within("2026-11-03", "2026-11-03")
        .unwrap();
    assert_eq!(
        timed.parse("03/11/2026 23:30-05:00"),
        Ok(instant("2026-11-03T23:30:00-05:00"))
    );
    assert_eq!(
        timed.parse("04/11/2026 00:30+01:00"),
        Err(ParseError::OutOfWindow)
    );
    assert_eq!(
        p("dd/MM/yyyy").prefer_past().parse("03/11/2026"),
        Ok(Parsed::Date(d(2026, 11, 3)))
    );
}

#[test]
fn format_errors_and_spans() {
    let date = Parsed::Date(d(2026, 11, 3));
    assert_eq!(
        format("dd/MM/yyyy HH:mm", date),
        Err(FormatError::MissingField)
    );
    assert_eq!(format("hh a", date), Err(FormatError::MissingField));
    let local = Parsed::DateTime(dt(2026, 11, 3, 15, 0, 0));
    assert_eq!(format("HH:mmXXX", local), Err(FormatError::MissingField));
    assert_eq!(
        FormatError::MissingField.to_string(),
        "value lacks a field the pattern needs"
    );
    let span = Parsed::DateSpan(d(2026, 10, 5), d(2026, 10, 11));
    assert_eq!(format("dd/MM", span).as_deref(), Ok("05/10/11/10"));
    assert_eq!(format("HH", span), Err(FormatError::MissingField));
}

#[test]
fn phrase_mode_formats_iso() {
    let gb = Datewise::new("en-GB", "2026-10-02").unwrap();
    let span = Parsed::DateSpan(d(2026, 10, 5), d(2026, 10, 11));
    assert_eq!(gb.format(&span).as_deref(), Ok("2026-10-05/2026-10-11"));
    let at = instant("2026-11-03T15:04:05+01:00");
    assert_eq!(gb.format(&at), Ok(at.to_string()));
}
