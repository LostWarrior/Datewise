use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::{America::New_York, Europe::London, Tz};
use datewise::resolve::{local_instant, resolve_date, LocalInstant, YearMode::NextOnOrAfter};
use datewise::zone::{local_date, parse_zone};

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

fn local(date: NaiveDate, h: u32, m: u32) -> NaiveDateTime {
    date.and_hms_opt(h, m, 0).unwrap()
}

fn utc(y: i32, mo: u32, day: u32, h: u32, m: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(y, mo, day, h, m, 0).unwrap()
}

fn resolve(day: u32, month: u32, year: Option<i32>, anchor: NaiveDate) -> Option<NaiveDate> {
    resolve_date(day, month, year, anchor, NextOnOrAfter)
}

#[test]
fn year_inference_picks_next_occurrence_on_or_after_anchor() {
    let anchor = d(2026, 6, 19);
    assert_eq!(resolve(24, 6, None, anchor), Some(d(2026, 6, 24)));
    assert_eq!(resolve(9, 9, None, anchor), Some(d(2026, 9, 9)));
    assert_eq!(resolve(19, 6, None, anchor), Some(d(2026, 6, 19)));
    assert_eq!(resolve(18, 6, None, anchor), Some(d(2027, 6, 18)));
}

#[test]
fn year_inference_rolls_over_december() {
    assert_eq!(resolve(2, 1, None, d(2026, 12, 30)), Some(d(2027, 1, 2)));
}

#[test]
fn explicit_year_is_used_verbatim() {
    assert_eq!(
        resolve(1, 1, Some(2020), d(2026, 6, 19)),
        Some(d(2020, 1, 1))
    );
    assert_eq!(resolve(29, 2, Some(2027), d(2026, 1, 1)), None);
}

#[test]
fn leap_day_skips_to_next_leap_year() {
    assert_eq!(resolve(29, 2, None, d(2026, 3, 1)), Some(d(2028, 2, 29)));
    assert_eq!(resolve(29, 2, None, d(2096, 3, 1)), Some(d(2104, 2, 29)));
}

#[test]
fn impossible_dates_are_none() {
    assert_eq!(resolve(31, 4, None, d(2026, 1, 1)), None);
    assert_eq!(resolve(0, 4, None, d(2026, 1, 1)), None);
    assert_eq!(resolve(1, 13, None, d(2026, 1, 1)), None);
    assert_eq!(resolve(1, 1, None, NaiveDate::MAX), None);
}

#[test]
fn bst_and_gmt_instants() {
    let summer = local_instant(local(d(2026, 6, 24), 9, 0), London);
    assert_eq!(summer, LocalInstant::Single(utc(2026, 6, 24, 8, 0)));
    let winter = local_instant(local(d(2026, 1, 10), 9, 0), London);
    assert_eq!(winter.single(), Some(utc(2026, 1, 10, 9, 0)));
}

#[test]
fn london_gap_and_fold() {
    let gap = local_instant(local(d(2026, 3, 29), 1, 30), London);
    assert_eq!(gap, LocalInstant::Gap);
    assert_eq!(gap.single(), None);
    let fold = local_instant(local(d(2026, 10, 25), 1, 30), London);
    let expected = LocalInstant::Ambiguous(utc(2026, 10, 25, 0, 30), utc(2026, 10, 25, 1, 30));
    assert_eq!(fold, expected);
    assert_eq!(fold.single(), None);
}

#[test]
fn new_york_gap_and_fold() {
    let gap = local_instant(local(d(2026, 3, 8), 2, 30), New_York);
    assert_eq!(gap, LocalInstant::Gap);
    let fold = local_instant(local(d(2026, 11, 1), 1, 30), New_York);
    let expected = LocalInstant::Ambiguous(utc(2026, 11, 1, 5, 30), utc(2026, 11, 1, 6, 30));
    assert_eq!(fold, expected);
    let after = local_instant(local(d(2026, 3, 8), 3, 30), New_York);
    assert_eq!(after, LocalInstant::Single(utc(2026, 3, 8, 7, 30)));
}

#[test]
fn zones_parse_against_the_database() {
    assert_eq!(parse_zone("Europe/London"), Ok(London));
    assert_eq!(parse_zone("UTC").map(|z: Tz| z.name()), Ok("UTC"));
    assert!(parse_zone("Mars/Olympus").is_err());
    assert!(parse_zone("").is_err());
    assert_eq!(
        parse_zone("nope").unwrap_err().to_string(),
        "unknown IANA time zone"
    );
}

#[test]
fn local_date_follows_the_zone() {
    let epoch_ms = utc(2026, 1, 1, 23, 30).timestamp_millis();
    assert_eq!(local_date(epoch_ms, chrono_tz::UTC), Some(d(2026, 1, 1)));
    assert_eq!(
        local_date(epoch_ms, chrono_tz::Australia::Sydney),
        Some(d(2026, 1, 2))
    );
    assert_eq!(
        local_date(epoch_ms, chrono_tz::Pacific::Honolulu),
        Some(d(2026, 1, 1))
    );
    assert_eq!(local_date(i64::MAX, London), None);
    assert_eq!(local_date(i64::MIN, London), None);
}

#[test]
fn previous_on_or_before_looks_back() {
    use datewise::resolve::YearMode;
    let (today, mode) = (d(2026, 10, 2), YearMode::PreviousOnOrBefore);
    assert_eq!(resolve_date(2, 10, None, today, mode), Some(d(2026, 10, 2)));
    assert_eq!(resolve_date(3, 10, None, today, mode), Some(d(2025, 10, 3)));
    assert_eq!(resolve_date(1, 1, None, today, mode), Some(d(2026, 1, 1)));
    assert_eq!(resolve_date(29, 2, None, today, mode), Some(d(2024, 2, 29)));
    assert_eq!(resolve_date(31, 4, None, today, mode), None);
    assert_eq!(
        resolve_date(3, 10, Some(2030), today, mode),
        Some(d(2030, 10, 3))
    );
}

#[test]
fn local_date_handles_chrono_limits() {
    use chrono_tz::{America::New_York as Ny, Asia::Tokyo, Pacific::Kiritimati};
    let (min, max) = (NaiveDate::MIN, NaiveDate::MAX);
    assert_eq!(local_date(i64::MAX, Ny), None);
    assert_eq!(local_date(i64::MIN, Ny), None);
    assert_eq!(local_date(8_210_266_876_799_999, Tokyo), None);
    assert_eq!(local_date(8_210_266_876_799_999, Ny), Some(max));
    assert_eq!(local_date(8_210_266_876_799_999, Kiritimati), None);
    assert_eq!(local_date(-8_334_601_228_800_000, Ny), None);
    assert!(local_date(-8_334_601_228_800_000, Tokyo).is_some_and(|date| date >= min));
    assert_eq!(local_date(0, Tokyo), Some(d(1970, 1, 1)));
}

#[test]
fn local_instant_handles_chrono_limits() {
    for tz in [New_York, London, chrono_tz::Asia::Tokyo] {
        for edge in [NaiveDateTime::MIN, NaiveDateTime::MAX] {
            let _ = local_instant(edge, tz);
        }
    }
}
