//! Low-level token scanners shared by the parsers. All offsets are byte
//! offsets and every consumed token is ASCII, so slicing stays on boundaries.

use crate::MAX_INPUT_BYTES;

pub(crate) const YEAR_DIGITS: usize = 4;

pub(crate) fn within_limit(text: &str) -> bool {
    text.len() <= MAX_INPUT_BYTES
}

/// Up to `max` leading ASCII digits: `(value, digits consumed)`.
pub(crate) fn digits(s: &str, max: usize) -> Option<(u32, usize)> {
    let count = s.bytes().take(max).take_while(u8::is_ascii_digit).count();
    let value = s
        .get(..count)?
        .bytes()
        .fold(0u32, |acc, b| acc * 10 + u32::from(b - b'0'));
    (count > 0).then_some((value, count))
}

/// Exactly four leading digits, not followed by another digit.
pub(crate) fn year(s: &str) -> Option<i32> {
    let (value, len) = digits(s, YEAR_DIGITS)?;
    let next_is_digit = s.as_bytes().get(len).is_some_and(u8::is_ascii_digit);
    if len == YEAR_DIGITS && !next_is_digit {
        i32::try_from(value).ok()
    } else {
        None
    }
}

/// True when `s` does not continue with a letter.
pub(crate) fn boundary(s: &str) -> bool {
    !s.chars().next().is_some_and(char::is_alphabetic)
}

/// True when the character just before byte offset `pos` is a letter or digit.
pub(crate) fn glued_before(text: &str, pos: usize) -> bool {
    text.get(..pos)
        .and_then(|head| head.chars().next_back())
        .is_some_and(char::is_alphanumeric)
}

pub(crate) fn skip_spaces(s: &str) -> usize {
    s.len() - s.trim_start_matches(' ').len()
}

/// `-` or en dash; returns bytes consumed.
pub(crate) fn dash(s: &str) -> Option<usize> {
    s.chars()
        .next()
        .filter(|c| matches!(c, '-' | '\u{2013}'))
        .map(char::len_utf8)
}
