# datewise

Panic-free parsing of English date and time phrases, with locale-aware date
order, week start, time zones and DST. Depends only on `chrono` and `chrono-tz`.
MSRV: Rust 1.70.

```rust
use chrono::NaiveDate;
use datewise::locale::Locale;
use datewise::relative::{parse_relative, Relative, Window};

let today = NaiveDate::from_ymd_opt(2026, 10, 2).unwrap();
let window = Window::new(today, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()).unwrap();

let uk = parse_relative("03/11/2026", today, window, Some(Locale::EN_GB));
assert_eq!(uk, Relative::Resolved(NaiveDate::from_ymd_opt(2026, 11, 3).unwrap()));
```

Modules: `names`, `fields`, `resolve`, `relative`, `locale`, `zone`.
Input over 512 bytes is rejected. Ambiguous input is reported, never guessed.

Licensed under MIT or Apache-2.0, at your option.
