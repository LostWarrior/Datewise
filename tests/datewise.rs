use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use datewise::{ConfigError, Datewise, ParseError, Parsed};

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

fn dt(y: i32, m: u32, day: u32, h: u32, min: u32) -> NaiveDateTime {
    d(y, m, day).and_hms_opt(h, min, 0).unwrap()
}

fn instant(rfc3339: &str) -> Parsed {
    Parsed::Instant(DateTime::<FixedOffset>::parse_from_rfc3339(rfc3339).unwrap())
}

fn gb() -> Datewise {
    Datewise::new("en-GB", "2026-10-02").unwrap()
}

fn us() -> Datewise {
    Datewise::new("en-US", "2026-10-02").unwrap()
}

#[test]
fn readme_example() {
    let parser = Datewise::new("en-GB", "2026-10-02").unwrap();
    let parsed = parser.parse("03/11/2026").unwrap();
    assert_eq!(parsed, Parsed::Date(d(2026, 11, 3)));
    assert_eq!(parsed.to_string(), "2026-11-03");
    assert_eq!(
        parser.parse("tomorrow at 3pm").unwrap().to_string(),
        "2026-10-03T15:00:00"
    );
}

#[test]
fn unknown_locale_is_a_config_error() {
    assert_eq!(
        Datewise::new("fr-FR", "2026-10-02").unwrap_err(),
        ConfigError::UnknownLocale
    );
    assert_eq!(
        Datewise::new("en", "2026-10-02").unwrap_err(),
        ConfigError::UnknownLocale
    );
}

#[test]
fn today_and_window_must_be_valid_dates() {
    for bad in [
        "2026-02-30",
        "2026-1-05",
        "02/10/2026",
        "",
        " 2026-10-02",
        "2026-10-02x",
    ] {
        assert_eq!(
            Datewise::new("en-GB", bad).unwrap_err(),
            ConfigError::InvalidDate
        );
        assert_eq!(
            gb().within(bad, "2026-12-31").unwrap_err(),
            ConfigError::InvalidDate
        );
        assert_eq!(
            gb().within("2026-01-01", bad).unwrap_err(),
            ConfigError::InvalidDate
        );
    }
    assert_eq!(
        gb().within("2026-12-31", "2026-10-01").unwrap_err(),
        ConfigError::InvalidWindow
    );
    assert!(gb().within("2026-10-01", "2026-10-01").is_ok());
}

#[test]
fn dates_and_spans() {
    assert_eq!(gb().parse("today"), Ok(Parsed::Date(d(2026, 10, 2))));
    assert_eq!(gb().parse("on the 14th"), Ok(Parsed::Date(d(2026, 10, 14))));
    assert_eq!(gb().parse("Monday"), Ok(Parsed::Date(d(2026, 10, 5))));
    assert_eq!(
        gb().parse("next week"),
        Ok(Parsed::DateSpan(d(2026, 10, 5), d(2026, 10, 11)))
    );
    assert_eq!(
        us().parse("this week"),
        Ok(Parsed::DateSpan(d(2026, 9, 27), d(2026, 10, 3)))
    );
}

#[test]
fn yearless_dates_are_unlimited_without_a_window() {
    assert_eq!(gb().parse("3 March"), Ok(Parsed::Date(d(2027, 3, 3))));
    assert_eq!(gb().parse("2 October"), Ok(Parsed::Date(d(2026, 10, 2))));
    assert_eq!(gb().parse("29 Feb"), Ok(Parsed::Date(d(2028, 2, 29))));
    assert_eq!(gb().parse("30 Feb"), Err(ParseError::Unparsed));
}

#[test]
fn yearless_dates_take_the_nearest_year_then_check_the_window() {
    let p = gb().within("2026-10-01", "2026-12-31").unwrap();
    assert_eq!(p.parse("3 March"), Err(ParseError::OutOfWindow));
    // No skipping ahead to a later year that would fit.
    let later = gb().within("2027-06-01", "2028-12-31").unwrap();
    assert_eq!(later.parse("3 March"), Err(ParseError::OutOfWindow));
    let fits = gb().within("2027-01-01", "2028-12-31").unwrap();
    assert_eq!(fits.parse("3 March"), Ok(Parsed::Date(d(2027, 3, 3))));
}

#[test]
fn prefer_past_flips_weekdays_days_of_month_and_yearless_dates() {
    let p = gb().prefer_past();
    assert_eq!(p.parse("Friday"), Ok(Parsed::Date(d(2026, 10, 2))));
    assert_eq!(p.parse("Monday"), Ok(Parsed::Date(d(2026, 9, 28))));
    assert_eq!(p.parse("Saturday"), Ok(Parsed::Date(d(2026, 9, 26))));
    assert_eq!(p.parse("the 14th"), Ok(Parsed::Date(d(2026, 9, 14))));
    assert_eq!(p.parse("the 2nd"), Ok(Parsed::Date(d(2026, 10, 2))));
    assert_eq!(p.parse("the 31st"), Ok(Parsed::Date(d(2026, 8, 31))));
    assert_eq!(p.parse("3 March"), Ok(Parsed::Date(d(2026, 3, 3))));
    assert_eq!(p.parse("29 Feb"), Ok(Parsed::Date(d(2024, 2, 29))));
}

#[test]
fn prefer_past_leaves_other_phrases_alone() {
    let p = gb().prefer_past();
    assert_eq!(p.parse("tomorrow"), Ok(Parsed::Date(d(2026, 10, 3))));
    assert_eq!(p.parse("next Friday"), Ok(Parsed::Date(d(2026, 10, 9))));
    assert_eq!(p.parse("3 March 2027"), Ok(Parsed::Date(d(2027, 3, 3))));
    assert_eq!(
        p.parse("next week"),
        Ok(Parsed::DateSpan(d(2026, 10, 5), d(2026, 10, 11)))
    );
}

#[test]
fn window_is_inclusive_and_rejects_outside_dates() {
    let p = gb().within("2026-10-02", "2026-10-03").unwrap();
    assert_eq!(p.parse("today"), Ok(Parsed::Date(d(2026, 10, 2))));
    assert_eq!(p.parse("tomorrow"), Ok(Parsed::Date(d(2026, 10, 3))));
    assert_eq!(p.parse("yesterday"), Err(ParseError::OutOfWindow));
    assert_eq!(p.parse("Sunday"), Err(ParseError::OutOfWindow));
    assert_eq!(
        p.parse("tomorrow at 3pm"),
        Ok(Parsed::DateTime(dt(2026, 10, 3, 15, 0)))
    );
    assert_eq!(p.parse("Sunday 3pm"), Err(ParseError::OutOfWindow));
}

#[test]
fn spans_need_only_partial_overlap() {
    let p = gb().within("2026-10-02", "2026-10-05").unwrap();
    assert_eq!(
        p.parse("this week"),
        Ok(Parsed::DateSpan(d(2026, 9, 28), d(2026, 10, 4)))
    );
    assert_eq!(
        p.parse("next week"),
        Ok(Parsed::DateSpan(d(2026, 10, 5), d(2026, 10, 11)))
    );
    let short = gb().within("2026-10-02", "2026-10-04").unwrap();
    assert_eq!(short.parse("next week"), Err(ParseError::OutOfWindow));
}

#[test]
fn whole_input_must_match() {
    for text in [
        "",
        "   ",
        "whenever",
        "tomorrow banana",
        "see you tomorrow",
        "tomorrow at 3pm please",
        "3pm ET sharp",
        "3pm tomorrow 4pm",
        "Friday 3pm 3 March",
        "3 March 2026 ET",
    ] {
        assert_eq!(gb().parse(text), Err(ParseError::Unparsed), "{text:?}");
    }
}

#[test]
fn time_ranges_and_timed_spans_are_unparsed_for_now() {
    assert_eq!(gb().parse("tomorrow 3-4pm"), Err(ParseError::Unparsed));
    assert_eq!(gb().parse("3pm - 5pm"), Err(ParseError::Unparsed));
    assert_eq!(gb().parse("next week at 3pm"), Err(ParseError::Unparsed));
}

#[test]
fn input_longer_than_the_limit_is_unparsed() {
    let long = format!("tomorrow{}", " ".repeat(datewise::MAX_INPUT_BYTES));
    assert_eq!(gb().parse(&long), Err(ParseError::Unparsed));
}

#[test]
fn date_times_without_a_zone() {
    assert_eq!(
        gb().parse("3pm"),
        Ok(Parsed::DateTime(dt(2026, 10, 2, 15, 0)))
    );
    assert_eq!(
        gb().parse("at 9.30am"),
        Ok(Parsed::DateTime(dt(2026, 10, 2, 9, 30)))
    );
    assert_eq!(
        gb().parse("Friday, 11:45 am"),
        Ok(Parsed::DateTime(dt(2026, 10, 2, 11, 45)))
    );
    assert_eq!(
        gb().parse("3pm on the 14th"),
        Ok(Parsed::DateTime(dt(2026, 10, 14, 15, 0)))
    );
    assert_eq!(
        gb().parse("3rd October 2026 at 7pm"),
        Ok(Parsed::DateTime(dt(2026, 10, 3, 19, 0)))
    );
}

#[test]
fn fixed_zones_give_instants() {
    assert_eq!(
        gb().parse("tomorrow 3pm UTC"),
        Ok(instant("2026-10-03T15:00:00+00:00"))
    );
    assert_eq!(
        gb().parse("3 Oct 2026 9:30am UTC+05:30"),
        Ok(instant("2026-10-03T09:30:00+05:30"))
    );
    assert_eq!(
        gb().parse("3pm BST"),
        Ok(instant("2026-10-02T15:00:00+01:00"))
    );
    assert_eq!(
        us().parse("3pm EST"),
        Ok(instant("2026-10-02T15:00:00-05:00"))
    );
    assert_eq!(
        gb().parse("3pm -0400"),
        Ok(instant("2026-10-02T15:00:00-04:00"))
    );
}

#[test]
fn time_only_regional_zone_uses_today() {
    assert_eq!(
        us().parse("3pm ET"),
        Ok(instant("2026-10-02T15:00:00-04:00"))
    );
    let winter = Datewise::new("en-US", "2026-12-01").unwrap();
    assert_eq!(
        winter.parse("3pm ET"),
        Ok(instant("2026-12-01T15:00:00-05:00"))
    );
    assert_eq!(
        us().parse("tomorrow 9am PT"),
        Ok(instant("2026-10-03T09:00:00-07:00"))
    );
}

#[test]
fn dst_gap_is_nonexistent_and_fold_is_ambiguous() {
    assert_eq!(
        us().parse("03/08/2026 2:30am ET"),
        Err(ParseError::NonexistentLocalTime)
    );
    assert_eq!(
        us().parse("11/01/2026 1:30am ET"),
        Err(ParseError::Ambiguous(vec![
            instant("2026-11-01T01:30:00-04:00"),
            instant("2026-11-01T01:30:00-05:00"),
        ]))
    );
}

#[test]
fn ambiguous_zones_and_dates_list_every_reading() {
    assert_eq!(
        gb().parse("3pm IST"),
        Err(ParseError::Ambiguous(vec![
            instant("2026-10-02T15:00:00+05:30"),
            instant("2026-10-02T15:00:00+01:00"),
            instant("2026-10-02T15:00:00+02:00"),
        ]))
    );
    let india = Datewise::new("en-IN", "2026-10-02").unwrap();
    assert_eq!(
        india.parse("3pm IST"),
        Ok(instant("2026-10-02T15:00:00+05:30"))
    );
    let canada = Datewise::new("en-CA", "2026-10-02").unwrap();
    assert_eq!(
        canada.parse("03/04/2026"),
        Err(ParseError::Ambiguous(vec![
            Parsed::Date(d(2026, 4, 3)),
            Parsed::Date(d(2026, 3, 4)),
        ]))
    );
    assert_eq!(
        canada.parse("03/04/2026 3pm"),
        Err(ParseError::Ambiguous(vec![
            Parsed::DateTime(dt(2026, 4, 3, 15, 0)),
            Parsed::DateTime(dt(2026, 3, 4, 15, 0)),
        ]))
    );
    let april = canada.within("2026-04-01", "2026-04-30").unwrap();
    assert_eq!(april.parse("03/04/2026"), Ok(Parsed::Date(d(2026, 4, 3))));
    let summer = canada.within("2026-06-01", "2026-08-31").unwrap();
    assert_eq!(summer.parse("03/04/2026"), Err(ParseError::OutOfWindow));
}

#[test]
fn display_is_fixed_iso_8601() {
    assert_eq!(Parsed::Date(d(2026, 1, 5)).to_string(), "2026-01-05");
    assert_eq!(
        Parsed::DateTime(dt(2026, 1, 5, 9, 7)).to_string(),
        "2026-01-05T09:07:00"
    );
    assert_eq!(
        instant("2026-01-05T09:07:00+05:30").to_string(),
        "2026-01-05T09:07:00+05:30"
    );
    assert_eq!(
        instant("2026-01-05T09:07:00Z").to_string(),
        "2026-01-05T09:07:00+00:00"
    );
    assert_eq!(
        Parsed::DateSpan(d(2026, 1, 5), d(2026, 1, 11)).to_string(),
        "2026-01-05/2026-01-11"
    );
}

#[test]
fn conversions_only_succeed_for_their_own_kind() {
    let date = Parsed::Date(d(2026, 1, 5));
    assert_eq!(date.as_date(), Some(d(2026, 1, 5)));
    assert_eq!(date.as_date_time(), None);
    assert_eq!(date.as_instant(), None);
    assert_eq!(date.as_date_span(), None);
    let local = Parsed::DateTime(dt(2026, 1, 5, 9, 0));
    assert_eq!(local.as_date_time(), Some(dt(2026, 1, 5, 9, 0)));
    assert_eq!(local.as_date(), None);
    let at = instant("2026-01-05T09:00:00+01:00");
    assert_eq!(
        at.as_instant().map(|i| i.to_rfc3339()),
        Some("2026-01-05T09:00:00+01:00".into())
    );
    assert_eq!(at.as_date_time(), None);
    let span = Parsed::DateSpan(d(2026, 1, 5), d(2026, 1, 11));
    assert_eq!(span.as_date_span(), Some((d(2026, 1, 5), d(2026, 1, 11))));
    assert_eq!(span.as_date(), None);
}

#[test]
fn errors_display() {
    assert_eq!(
        ConfigError::UnknownLocale.to_string(),
        "unknown or unsupported locale tag"
    );
    assert_eq!(
        ConfigError::InvalidDate.to_string(),
        "date is not a valid YYYY-MM-DD"
    );
    assert_eq!(
        ConfigError::InvalidWindow.to_string(),
        "window start is after its end"
    );
    assert_eq!(
        ParseError::Unparsed.to_string(),
        "not a supported date or time"
    );
    assert_eq!(
        ParseError::OutOfWindow.to_string(),
        "date is outside the allowed window"
    );
}
