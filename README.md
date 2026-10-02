# datewise

A Rust-based date time parser with locale-aware dates, week start, time zones and DST.

Dependencies:

- [`chrono`](https://crates.io/crates/chrono)
- [`chrono-tz`](https://crates.io/crates/chrono-tz)

```rust
use datewise::Datewise;

let dw = Datewise::new("en-GB", "2026-10-02")?;

assert_eq!(dw.parse("next friday")?.to_string(), "2026-10-09");
assert_eq!(dw.parse("tomorrow at 3pm")?.to_string(), "2026-10-03T15:00:00");
assert_eq!(dw.parse("Friday 9am BST")?.to_string(), "2026-10-02T09:00:00+01:00");
assert_eq!(dw.parse("next week")?.to_string(), "2026-10-05/2026-10-11");

let p = Datewise::pattern("dd/MM/yyyy")?;
let d = p.parse("03/11/2026")?;

assert_eq!(Datewise::pattern("yyyy-MM-dd")?.format(&d)?, "2026-11-03");

Ok::<(), Box<dyn std::error::Error>>(())
```

Licensed under Apache-2.0.
