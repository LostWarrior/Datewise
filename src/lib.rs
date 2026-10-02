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
#![allow(clippy::module_name_repetitions)]

pub mod fields;
pub mod locale;
pub mod names;
pub mod relative;
pub mod resolve;
mod scan;
pub mod zone;

/// Longest input, in bytes, accepted by whole-input parsers; finders scan any length.
pub const MAX_INPUT_BYTES: usize = 512;
