use crate::fields::find_numeric_date;
use crate::locale::Locale;
use crate::names::{month_from_name, ordinal_suffix_len, weekday_from_name};
use crate::resolve::{resolve_date, YearMode};
use crate::scan::{digits, within_limit, year, YEAR_DIGITS};
use chrono::{Datelike, Days, NaiveDate, Weekday};

const MAX_TOKENS: usize = 6;

pub(super) enum Parsed {
    Date(NaiveDate),
    Span(NaiveDate, NaiveDate),
    Numeric(Vec<NaiveDate>),
    YearlessDay { day: u32, month: u32 },
}

pub(super) fn parse(text: &str, today: NaiveDate, locale: Option<Locale>) -> Option<Parsed> {
    if !within_limit(text) {
        return None;
    }
    let mut buf = [""; MAX_TOKENS];
    let mut count = 0;
    for token in text
        .split(|c: char| c.is_whitespace() || c == ',')
        .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|t| !t.is_empty())
    {
        *buf.get_mut(count)? = token;
        count += 1;
    }
    let mut tokens = buf.get(..count)?;
    for filler in ["on", "the"] {
        if let Some((first, rest)) = tokens.split_first() {
            if first.eq_ignore_ascii_case(filler) && !rest.is_empty() {
                tokens = rest;
            }
        }
    }
    match tokens {
        [one] => single(one, today, locale),
        [word, target] if is(word, "next") => next_target(target, today, week_start(locale)),
        [word, target] if is(word, "this") && is(target, "week") => {
            week_span(today, week_start(locale), 0)
        }
        _ => month_day(tokens),
    }
}

fn is(token: &str, word: &str) -> bool {
    token.eq_ignore_ascii_case(word)
}

fn date(date: Option<NaiveDate>) -> Option<Parsed> {
    date.map(Parsed::Date)
}

fn single(token: &str, today: NaiveDate, locale: Option<Locale>) -> Option<Parsed> {
    if is(token, "today") {
        return date(Some(today));
    }
    if is(token, "tomorrow") {
        return date(today.checked_add_days(Days::new(1)));
    }
    if is(token, "yesterday") {
        return date(today.checked_sub_days(Days::new(1)));
    }
    if let Some(weekday) = weekday_from_name(token) {
        let ahead = days_between(today.weekday(), weekday);
        return date(today.checked_add_days(Days::new(ahead)));
    }
    if let Some(iso) = iso_date(token) {
        return date(Some(iso));
    }
    let order = locale.map(|l| l.date_order());
    if let Some(found) = find_numeric_date(token, order).filter(|f| f.span.len() == token.len()) {
        return Some(Parsed::Numeric(found.dates));
    }
    day_of_month(parse_day(token, true)?, today)
}

fn next_target(target: &str, today: NaiveDate, start: Weekday) -> Option<Parsed> {
    if is(target, "week") {
        return week_span(today, start, 1);
    }
    let weekday = weekday_from_name(target)?;
    let offset = 7 + days_between(start, weekday);
    date(start_of_week(today, start)?.checked_add_days(Days::new(offset)))
}

fn week_start(locale: Option<Locale>) -> Weekday {
    locale.map_or(Weekday::Mon, |l| l.week_start())
}

fn week_span(today: NaiveDate, start: Weekday, weeks_ahead: u64) -> Option<Parsed> {
    let first = start_of_week(today, start)?.checked_add_days(Days::new(7 * weeks_ahead))?;
    Some(Parsed::Span(first, first.checked_add_days(Days::new(6))?))
}

fn start_of_week(day: NaiveDate, start: Weekday) -> Option<NaiveDate> {
    day.checked_sub_days(Days::new(days_between(start, day.weekday())))
}

fn days_between(from: Weekday, to: Weekday) -> u64 {
    u64::from((7 + to.num_days_from_monday() - from.num_days_from_monday()) % 7)
}

fn day_of_month(day: u32, today: NaiveDate) -> Option<Parsed> {
    let base = today.year() * 12 + i32::try_from(today.month0()).ok()?;
    (0..=12)
        .filter_map(|k| {
            let months = base + k;
            let month = u32::try_from(months.rem_euclid(12)).ok()? + 1;
            NaiveDate::from_ymd_opt(months.div_euclid(12), month, day)
        })
        .find(|candidate| *candidate >= today)
        .map(Parsed::Date)
}

fn month_day(tokens: &[&str]) -> Option<Parsed> {
    let (a, b, year) = match *tokens {
        [a, b] => (a, b, None),
        [a, b, y] => (a, b, Some(whole_year(y)?)),
        _ => return None,
    };
    let (day, month) = match (parse_day(a, false), month_from_name(b)) {
        (Some(day), Some(month)) => (day, month),
        _ => (parse_day(b, false)?, month_from_name(a)?),
    };
    match year {
        Some(y) => date(NaiveDate::from_ymd_opt(y, month, day)),
        None => Some(Parsed::YearlessDay { day, month }),
    }
}

/// Day number 1-31 with an optional (or, if `need_suffix`, required) ordinal suffix.
fn parse_day(token: &str, need_suffix: bool) -> Option<u32> {
    let (day, len) = digits(token, 2)?;
    let suffix = token.get(len..)?;
    let ok = match suffix {
        "" => !need_suffix,
        _ => ordinal_suffix_len(suffix) == Some(suffix.len()),
    };
    (ok && (1..=31).contains(&day)).then_some(day)
}

fn whole_year(token: &str) -> Option<i32> {
    (token.len() == YEAR_DIGITS).then_some(())?;
    year(token)
}

fn iso_date(token: &str) -> Option<NaiveDate> {
    let mut parts = token.split('-');
    let (y, m, d) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || m.len() != 2 || d.len() != 2 {
        return None;
    }
    let field = |s: &str| digits(s, YEAR_DIGITS).filter(|(_, n)| *n == s.len());
    let (y, m, d) = (whole_year(y)?, field(m)?.0, field(d)?.0);
    resolve_date(d, m, Some(y), NaiveDate::MIN, YearMode::NextOnOrAfter)
}
