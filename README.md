# datewise

A Rust-based date time parser with locale-aware dates, week start, time zones and DST.

Dependencies:

- [`chrono`](https://crates.io/crates/chrono)
- [`chrono-tz`](https://crates.io/crates/chrono-tz)

```rust
use chrono::NaiveDate;
use datewise::locale::Locale;
use datewise::relative::{parse_relative, Relative, Window};

let today = NaiveDate::from_ymd_opt(2026, 10, 2).unwrap();
let window = Window::new(today, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()).unwrap();

let uk = parse_relative("03/11/2026", today, window, Some(Locale::EN_GB));
assert_eq!(uk, Relative::Resolved(NaiveDate::from_ymd_opt(2026, 11, 3).unwrap()));
```

Licensed under MIT or Apache-2.0, at your option.
