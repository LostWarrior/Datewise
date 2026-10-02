//! Time zone parsing and zone-aware civil dates.

use chrono::{DateTime, NaiveDate};
use chrono_tz::Tz;
use std::fmt;

/// The text is not a known IANA time zone name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZoneError;

impl fmt::Display for ZoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown IANA time zone")
    }
}

impl std::error::Error for ZoneError {}

/// Parses an IANA zone name.
///
/// # Errors
///
/// [`ZoneError`] when `name` is not a known zone.
pub fn parse_zone(name: &str) -> Result<Tz, ZoneError> {
    name.parse().map_err(|_| ZoneError)
}

/// The civil date at `epoch_ms` (Unix milliseconds) in `tz`; `None` if out of range.
#[must_use]
pub fn local_date(epoch_ms: i64, tz: Tz) -> Option<NaiveDate> {
    let instant = DateTime::from_timestamp_millis(epoch_ms)?;
    Some(instant.with_timezone(&tz).date_naive())
}
