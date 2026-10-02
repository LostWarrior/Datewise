# datewise

A Rust-based date time parser with locale-aware dates, week start, time zones and DST.

Dependencies:

- [`chrono`](https://crates.io/crates/chrono)
- [`chrono-tz`](https://crates.io/crates/chrono-tz)

```rust
use chrono::NaiveDate;
use datewise::{ParseError, Parser};

let today = NaiveDate::from_ymd_opt(2026, 10, 2).unwrap();
let parser = Parser::new("en-GB", today).unwrap();

assert_eq!(parser.parse("03/11/2026").unwrap().to_string(), "2026-11-03");
assert_eq!(parser.parse("tomorrow at 3pm").unwrap().to_string(), "2026-10-03T15:00:00");
assert_eq!(parser.parse("Friday 9am BST").unwrap().to_string(), "2026-10-02T09:00:00+01:00");
assert_eq!(parser.parse("next week").unwrap().to_string(), "2026-10-05/2026-10-11");
assert_eq!(parser.prefer_past().parse("3 March").unwrap().to_string(), "2026-03-03");
assert_eq!(parser.parse("tomorrow-ish"), Err(ParseError::Unparsed));
```

Licensed under Apache-2.0.
