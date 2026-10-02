# datewise

A Rust-based date time parser with locale-aware dates, week start, time zones and DST.

Dependencies:

- [`chrono`](https://crates.io/crates/chrono)
- [`chrono-tz`](https://crates.io/crates/chrono-tz)

```rust
use datewise::{Datewise, ParseError};

let parser = Datewise::new("en-GB", "2026-10-02").unwrap();

assert_eq!(parser.parse("03/11/2026").unwrap().to_string(), "2026-11-03");
assert_eq!(parser.parse("tomorrow at 3pm").unwrap().to_string(), "2026-10-03T15:00:00");
assert_eq!(parser.parse("Friday 9am BST").unwrap().to_string(), "2026-10-02T09:00:00+01:00");
assert_eq!(parser.parse("next week").unwrap().to_string(), "2026-10-05/2026-10-11");
assert_eq!(parser.parse("tomorrow-ish"), Err(ParseError::Unparsed));
assert_eq!(parser.prefer_past().parse("3 March").unwrap().to_string(), "2026-03-03");
```

Licensed under Apache-2.0.
