use chrono::{NaiveDate, NaiveDateTime, TimeDelta};
use datewise::fields::{
    find_date, find_numeric_date, find_ordinal_date, find_time, find_time_range, month_prefix,
};
use datewise::locale::{DateOrder, Locale};
use datewise::names::{month_from_name, ordinal_suffix_len, weekday_from_name};
use datewise::relative::{parse_relative, Window};
use datewise::resolve::{local_instant, resolve_date, YearMode};
use datewise::zone::{find_zone, local_date, parse_zone};
use datewise::Parser;

const FRAGMENTS: &[&str] = &[
    "jan",
    "Sept",
    "MAY",
    "dec",
    "24th",
    "1st",
    "3",
    "12",
    "2026",
    "12:30",
    "11.45",
    "am",
    "pm",
    " ",
    "  ",
    "-",
    "\u{2013}",
    "\u{e9}",
    "\u{1F600}",
    "\u{0}",
    ".",
    ",",
    ":",
    "next",
    "the",
    "on",
    "this",
    "week",
    "Friday",
    "thurs",
    "today",
    "2026-10-03",
    "of",
    "/",
    "03/10/2026",
    "3.10.26",
    "2026/10/03",
    "UTC+5",
    "+05:30",
    "-0530",
    "GMT-",
    "10:00Z",
    "PST",
    "IST",
    "CT",
    "13:05pm",
    "8:99",
    "pm5",
    "a.m.",
    "p.m.",
    "EST",
    "EDT",
    "ET",
    "UTC-",
    "7pm-8",
];

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next()).unwrap() % n
    }

    fn text(&mut self) -> String {
        let mut out = String::new();
        for _ in 0..self.below(14) {
            if self.below(4) == 0 {
                let code = u32::try_from(self.next() % 0x11_0000).unwrap();
                out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
            } else {
                out.push_str(FRAGMENTS[self.below(FRAGMENTS.len())]);
            }
        }
        out
    }
}

fn exercise(text: &str, today: NaiveDate, window: Window) {
    let _ = find_ordinal_date(text);
    let _ = find_time(text);
    let _ = find_time_range(text);
    let _ = find_date(text, None);
    let _ = month_prefix(text);
    let _ = month_from_name(text);
    let _ = weekday_from_name(text);
    let _ = ordinal_suffix_len(text);
    let _ = parse_zone(text);
    for locale in [
        None,
        Some(Locale::EN_GB),
        Some(Locale::EN_US),
        Some(Locale::EN_CA),
    ] {
        let _ = parse_relative(text, today, window, locale);
        let _ = find_zone(text, locale);
        let _ = find_date(text, locale);
        let _ = find_numeric_date(text, locale.map(|l| l.date_order()));
    }
    for order in [
        DateOrder::DayFirst,
        DateOrder::MonthFirst,
        DateOrder::YearFirst,
    ] {
        let _ = find_numeric_date(text, Some(order));
    }
    let _ = Locale::from_tag(text);
    let _ = Parser::new(text, today);
    for tag in ["en-GB", "en-US", "en-CA", "en-IN"] {
        if let Ok(parser) = Parser::new(tag, today) {
            let _ = parser.parse(text);
            let _ = parser.within(window).prefer_past().parse(text);
        }
    }
}

#[test]
fn random_text_never_panics() {
    let todays = [
        NaiveDate::from_ymd_opt(2026, 10, 2).unwrap(),
        NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(),
        NaiveDate::MIN,
        NaiveDate::MAX,
    ];
    let window = Window::new(NaiveDate::MIN, NaiveDate::MAX).unwrap();
    let mut rng = Lcg(0x5eed);
    for i in 0..20_000 {
        let text = rng.text();
        let today = todays[rng.below(todays.len())];
        exercise(&text, today, window);
        if i % 100 == 0 {
            exercise(&text.repeat(40), today, window);
        }
        if i % 1000 == 0 {
            exercise(&text.repeat(20_000 / text.len().max(1) + 1), today, window);
        }
    }
}

#[test]
fn long_pathological_text_stays_fast() {
    let today = NaiveDate::from_ymd_opt(2026, 10, 2).unwrap();
    let window = Window::new(NaiveDate::MIN, NaiveDate::MAX).unwrap();
    for fragment in ["1:", "1111111111", "    ", "1 - ", "1+", "May ", "10:00Z "] {
        exercise(&fragment.repeat(20_000 / fragment.len()), today, window);
    }
}

#[test]
fn random_numbers_never_panic() {
    let mut rng = Lcg(42);
    for _ in 0..20_000 {
        let day = u32::try_from(rng.next() % 64).unwrap();
        let month = u32::try_from(rng.next() % 16).unwrap();
        let year = i32::try_from(rng.next() % 600_000).unwrap() - 300_000;
        let anchor = NaiveDate::from_num_days_from_ce_opt(i32::try_from(rng.next()).unwrap())
            .unwrap_or(NaiveDate::MAX);
        let _ = resolve_date(day, month, Some(year), anchor, YearMode::NextOnOrAfter);
        let _ = resolve_date(day, month, None, anchor, YearMode::NextOnOrAfter);
        let _ = resolve_date(day, month, Some(year), anchor, YearMode::PreviousOnOrBefore);
        let _ = resolve_date(day, month, None, anchor, YearMode::PreviousOnOrBefore);
        let epoch = i64::try_from(rng.next()).unwrap() * i64::try_from(rng.next()).unwrap();
        let _ = local_date(epoch, chrono_tz::Europe::London);
    }
}

#[test]
fn extreme_instants_never_panic() {
    let zones = [
        chrono_tz::Europe::London,
        chrono_tz::America::New_York,
        chrono_tz::Asia::Tokyo,
        chrono_tz::Pacific::Kiritimati,
        chrono_tz::Pacific::Pago_Pago,
    ];
    let edges = [
        i64::MIN,
        i64::MAX,
        -8_334_601_228_800_000,
        8_210_266_876_799_999,
    ];
    let mut rng = Lcg(7);
    for tz in zones {
        for edge in edges {
            for delta in (-100_000_000..=100_000_000).step_by(1_000_000) {
                let _ = local_date(edge.saturating_add(delta), tz);
            }
            let _ = local_date(edge, tz);
        }
        for _ in 0..2_000 {
            let _ = local_date(i64::try_from(rng.next()).unwrap() << 20, tz);
        }
        for edge in [NaiveDateTime::MIN, NaiveDateTime::MAX] {
            for hours in -48..=48 {
                let shifted = edge.checked_add_signed(TimeDelta::hours(hours));
                let _ = local_instant(shifted.unwrap_or(edge), tz);
            }
        }
    }
}
