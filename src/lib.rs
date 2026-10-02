#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
// Item names repeat their module on purpose (`fields::find_time`).
#![allow(clippy::module_name_repetitions)]

pub mod fields;
pub mod locale;
pub mod names;
pub mod relative;
pub mod resolve;
mod scan;
pub mod zone;

/// Longest input, in bytes, that any parser will examine.
pub const MAX_INPUT_BYTES: usize = 512;
