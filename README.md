# datewise

Panic-free parsing of English date and time phrases, with time zone and DST
handling. Depends only on `chrono` and `chrono-tz`. MSRV: Rust 1.70.

```rust
use chrono::NaiveDate;
use datewise::relative::{parse_relative, Relative, Window};

let today = NaiveDate::from_ymd_opt(2026, 10, 2).unwrap();
let end = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
let window = Window::new(today, end).unwrap();

let friday = NaiveDate::from_ymd_opt(2026, 10, 9).unwrap();
assert_eq!(parse_relative("next Friday", today, window), Relative::Resolved(friday));
```

Modules: `names`, `fields` (find dates and times in text), `resolve`,
`relative`, `zone`. Input over 512 bytes is rejected.

Licensed under MIT or Apache-2.0, at your option.
