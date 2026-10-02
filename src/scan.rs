use crate::MAX_INPUT_BYTES;

pub(crate) const YEAR_DIGITS: usize = 4;

pub(crate) fn within_limit(text: &str) -> bool {
    text.len() <= MAX_INPUT_BYTES
}

pub(crate) fn digits(s: &str, max: usize) -> Option<(u32, usize)> {
    let count = s.bytes().take(max).take_while(u8::is_ascii_digit).count();
    let value = s
        .get(..count)?
        .bytes()
        .fold(0u32, |acc, b| acc * 10 + u32::from(b - b'0'));
    (count > 0).then_some((value, count))
}

pub(crate) fn year(s: &str) -> Option<i32> {
    let (value, len) = digits(s, YEAR_DIGITS)?;
    let next_is_digit = s.as_bytes().get(len).is_some_and(u8::is_ascii_digit);
    if len == YEAR_DIGITS && !next_is_digit {
        i32::try_from(value).ok()
    } else {
        None
    }
}

pub(crate) fn century(two_digits: u32) -> Option<i32> {
    let base = if two_digits < 69 { 2000 } else { 1900 };
    i32::try_from(two_digits).ok().map(|yy| base + yy)
}

pub(crate) fn boundary(s: &str) -> bool {
    !s.chars().next().is_some_and(char::is_alphabetic)
}

pub(crate) fn glued_before(text: &str, pos: usize) -> bool {
    text.get(..pos)
        .and_then(|head| head.chars().next_back())
        .is_some_and(char::is_alphanumeric)
}

pub(crate) fn skip_spaces(s: &str) -> usize {
    s.len() - s.trim_start_matches(' ').len()
}

pub(crate) fn dash(s: &str) -> Option<usize> {
    s.chars()
        .next()
        .filter(|c| matches!(c, '-' | '\u{2013}'))
        .map(char::len_utf8)
}

pub(crate) fn is_sep(c: char) -> bool {
    matches!(c, '/' | '.' | '-')
}

pub(crate) fn clear_before(text: &str, pos: usize) -> Option<()> {
    let mut back = text.get(..pos)?.chars().rev();
    match (back.next(), back.next()) {
        (Some(c), _) if c.is_alphanumeric() => None,
        (Some(s), Some(d)) if is_sep(s) && d.is_ascii_digit() => None,
        _ => Some(()),
    }
}

pub(crate) fn clear_after(tail: &str) -> Option<()> {
    let mut chars = tail.chars();
    match (chars.next(), chars.next()) {
        (Some(c), _) if c.is_alphanumeric() => None,
        (Some(s), Some(d)) if is_sep(s) && d.is_ascii_digit() => None,
        _ => Some(()),
    }
}
