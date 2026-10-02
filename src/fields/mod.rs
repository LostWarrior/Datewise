//! Finders that locate dates and times inside free text.
//!
//! Spans are byte ranges into the input and always lie on character
//! boundaries.

mod date;
mod time;

pub use date::{find_ordinal_date, month_prefix, weekday_before, OrdinalDate};
pub use time::{find_time, find_time_range, TimeError, TimeOfDay, TimeRange};
