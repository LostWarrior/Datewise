use crate::names::{full_name_prefix, month_from_name, weekday_from_name, MONTHS, WEEKDAYS};
use crate::scan::{century, digits, YEAR_DIGITS};
use crate::Parsed;
use chrono::{
    DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Weekday,
};
use std::fmt::Write;

const NAME_ABBREV: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Field {
    Year,
    ShortYear,
    Month(bool),
    MonthAbbrev,
    MonthName,
    Day(bool),
    WeekdayAbbrev,
    WeekdayName,
    Hour(bool),
    Hour12(bool),
    Minute(bool),
    Second(bool),
    AmPm,
    Offset,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Item {
    Field(Field),
    Literal(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Pattern(Vec<Item>);

#[derive(Default)]
struct Fields {
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    weekday: Option<Weekday>,
    hour: Option<u32>,
    hour12: Option<u32>,
    minute: Option<u32>,
    second: Option<u32>,
    pm: Option<bool>,
    offset: Option<i32>,
}

impl Pattern {
    pub(crate) fn new(pattern: &str) -> Option<Self> {
        let mut items = Vec::new();
        let mut chars = pattern.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\'' {
                let mut literal = String::new();
                if chars.peek() == Some(&'\'') {
                    chars.next();
                    literal.push('\'');
                } else {
                    loop {
                        match chars.next()? {
                            '\'' if chars.peek() == Some(&'\'') => {
                                chars.next();
                                literal.push('\'');
                            }
                            '\'' => break,
                            other => literal.push(other),
                        }
                    }
                }
                push_literal(&mut items, &literal);
            } else if c.is_ascii_alphabetic() {
                let mut count = 1;
                while chars.next_if_eq(&c).is_some() {
                    count += 1;
                }
                items.push(Item::Field(field(c, count)?));
            } else {
                push_literal(&mut items, c.encode_utf8(&mut [0; 4]));
            }
        }
        Some(Self(items))
    }

    pub(crate) fn parse(&self, text: &str) -> Option<Parsed> {
        let mut fields = Fields::default();
        let mut rest = text;
        for item in &self.0 {
            rest = match item {
                Item::Literal(literal) => rest.strip_prefix(literal.as_str())?,
                Item::Field(field) => scan(*field, rest, &mut fields)?,
            };
        }
        if rest.is_empty() {
            fields.build()
        } else {
            None
        }
    }

    pub(crate) fn format(&self, value: &Parsed) -> Option<String> {
        match value {
            Parsed::Date(date) => {
                self.render(None, |out, spec| write!(out, "{}", date.format(spec)))
            }
            Parsed::DateTime(local) => {
                self.render(None, |out, spec| write!(out, "{}", local.format(spec)))
            }
            Parsed::Instant(instant) => self.render(Some(instant), |out, spec| {
                write!(out, "{}", instant.format(spec))
            }),
            Parsed::DateSpan(first, last) => Some(format!(
                "{}/{}",
                self.format(&Parsed::Date(*first))?,
                self.format(&Parsed::Date(*last))?
            )),
        }
    }

    fn render(
        &self,
        instant: Option<&DateTime<FixedOffset>>,
        write: impl Fn(&mut String, &str) -> std::fmt::Result,
    ) -> Option<String> {
        let mut out = String::new();
        for item in &self.0 {
            match item {
                Item::Literal(literal) => out.push_str(literal),
                Item::Field(Field::Offset) => match instant.map(|i| i.offset().local_minus_utc()) {
                    Some(0) => out.push('Z'),
                    Some(_) => write(&mut out, "%:z").ok()?,
                    None => return None,
                },
                Item::Field(field) => write(&mut out, spec(*field)).ok()?,
            }
        }
        Some(out)
    }
}

fn push_literal(items: &mut Vec<Item>, text: &str) {
    if let Some(Item::Literal(last)) = items.last_mut() {
        last.push_str(text);
    } else {
        items.push(Item::Literal(text.to_owned()));
    }
}

fn field(letter: char, count: usize) -> Option<Field> {
    Some(match (letter, count) {
        ('y', 4) => Field::Year,
        ('y', 2) => Field::ShortYear,
        ('M', 1 | 2) => Field::Month(count == 2),
        ('M', 3) => Field::MonthAbbrev,
        ('M', 4) => Field::MonthName,
        ('d', 1 | 2) => Field::Day(count == 2),
        ('E', 1..=3) => Field::WeekdayAbbrev,
        ('E', 4) => Field::WeekdayName,
        ('H', 1 | 2) => Field::Hour(count == 2),
        ('h', 1 | 2) => Field::Hour12(count == 2),
        ('m', 1 | 2) => Field::Minute(count == 2),
        ('s', 1 | 2) => Field::Second(count == 2),
        ('a', 1) => Field::AmPm,
        ('X', 3) => Field::Offset,
        _ => return None,
    })
}

fn spec(field: Field) -> &'static str {
    match field {
        Field::Year => "%Y",
        Field::ShortYear => "%y",
        Field::Month(padded) => pick(padded, "%m", "%-m"),
        Field::MonthAbbrev => "%b",
        Field::MonthName => "%B",
        Field::Day(padded) => pick(padded, "%d", "%-d"),
        Field::WeekdayAbbrev => "%a",
        Field::WeekdayName => "%A",
        Field::Hour(padded) => pick(padded, "%H", "%-H"),
        Field::Hour12(padded) => pick(padded, "%I", "%-I"),
        Field::Minute(padded) => pick(padded, "%M", "%-M"),
        Field::Second(padded) => pick(padded, "%S", "%-S"),
        Field::AmPm => "%p",
        Field::Offset => "%:z",
    }
}

fn pick(padded: bool, yes: &'static str, no: &'static str) -> &'static str {
    if padded {
        yes
    } else {
        no
    }
}

fn scan<'a>(field: Field, s: &'a str, fields: &mut Fields) -> Option<&'a str> {
    let number = |padded: bool| {
        let (value, len) = digits(s, 2)?;
        (!padded || len == 2).then(|| (value, s.get(len..)))
    };
    let (rest, slot, value) = match field {
        Field::Year => {
            let (value, len) = digits(s, YEAR_DIGITS).filter(|&(_, len)| len == YEAR_DIGITS)?;
            fields.year = Some(i32::try_from(value).ok()?);
            return s.get(len..);
        }
        Field::ShortYear => {
            let (value, rest) = number(true)?;
            fields.year = Some(century(value)?);
            return rest;
        }
        Field::MonthAbbrev => {
            fields.month = Some(month_from_name(s.get(..NAME_ABBREV)?)?);
            return s.get(NAME_ABBREV..);
        }
        Field::MonthName => {
            let (month, len) = full_name_prefix(MONTHS, s)?;
            fields.month = Some(month);
            return s.get(len..);
        }
        Field::WeekdayAbbrev => {
            fields.weekday = Some(weekday_from_name(s.get(..NAME_ABBREV)?)?);
            return s.get(NAME_ABBREV..);
        }
        Field::WeekdayName => {
            let (weekday, len) = full_name_prefix(WEEKDAYS, s)?;
            fields.weekday = Some(weekday);
            return s.get(len..);
        }
        Field::AmPm => {
            let head = s.get(..2)?;
            let pm = head.eq_ignore_ascii_case("pm");
            (pm || head.eq_ignore_ascii_case("am")).then_some(())?;
            fields.pm = Some(pm);
            return s.get(2..);
        }
        Field::Offset => return offset(s, fields),
        Field::Month(padded) => {
            let (value, rest) = number(padded)?;
            (rest, &mut fields.month, value)
        }
        Field::Day(padded) => {
            let (value, rest) = number(padded)?;
            (rest, &mut fields.day, value)
        }
        Field::Hour(_) => {
            let (value, rest) = number(false)?;
            (rest, &mut fields.hour, value)
        }
        Field::Hour12(padded) => {
            let (value, rest) = number(padded)?;
            (rest, &mut fields.hour12, value)
        }
        Field::Minute(padded) => {
            let (value, rest) = number(padded)?;
            (rest, &mut fields.minute, value)
        }
        Field::Second(padded) => {
            let (value, rest) = number(padded)?;
            (rest, &mut fields.second, value)
        }
    };
    *slot = Some(value);
    rest
}

fn offset<'a>(s: &'a str, fields: &mut Fields) -> Option<&'a str> {
    if let Some(rest) = s.strip_prefix('Z') {
        fields.offset = Some(0);
        return Some(rest);
    }
    let sign = match s.chars().next()? {
        '+' => 1,
        '-' => -1,
        _ => return None,
    };
    let two = |t: &str| digits(t, 2).filter(|&(_, len)| len == 2).map(|(v, _)| v);
    let hours = two(s.get(1..)?)?;
    let minutes = two(s.get(3..)?.strip_prefix(':')?)?;
    (minutes < 60).then_some(())?;
    fields.offset = Some(sign * i32::try_from(hours * 3600 + minutes * 60).ok()?);
    s.get(6..)
}

impl Fields {
    fn build(&self) -> Option<Parsed> {
        let date = NaiveDate::from_ymd_opt(self.year?, self.month?, self.day?)?;
        if self.weekday.is_some_and(|w| w != date.weekday()) {
            return None;
        }
        let hour = match (self.hour, self.hour12, self.pm) {
            (None, None, None) => {
                return (self.minute.is_none() && self.second.is_none() && self.offset.is_none())
                    .then_some(Parsed::Date(date));
            }
            (Some(hour), None, None) => hour,
            (None, Some(hour @ 1..=12), Some(pm)) => hour % 12 + if pm { 12 } else { 0 },
            _ => return None,
        };
        if self.second.is_some() && self.minute.is_none() {
            return None;
        }
        let time =
            NaiveTime::from_hms_opt(hour, self.minute.unwrap_or(0), self.second.unwrap_or(0))?;
        let local = NaiveDateTime::new(date, time);
        match self.offset {
            None => Some(Parsed::DateTime(local)),
            Some(seconds) => FixedOffset::east_opt(seconds)?
                .from_local_datetime(&local)
                .single()
                .map(Parsed::Instant),
        }
    }
}
