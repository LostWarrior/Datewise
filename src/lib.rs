#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]

mod datewise;
pub mod fields;
pub mod locale;
pub mod names;
mod pattern;
pub mod relative;
pub mod resolve;
mod scan;
pub mod zone;

pub use datewise::{ConfigError, Datewise, FormatError, ParseError, Parsed};

/// Longest input, in bytes, accepted by whole-input parsers.
pub const MAX_INPUT_BYTES: usize = 512;
