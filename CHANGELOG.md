# Changelog

## Unreleased

- Add `Parser`, a whole-input front door returning `Parsed` (`Date`, `DateTime`, `Instant`, `DateSpan`) with ISO 8601 `Display`.
- `Parser::within` checks a window (spans need only overlap); `Parser::prefer_past` reads bare weekdays, `the 14th` and yearless dates backwards.
- Errors: `ParseError` (`Unparsed`, `Ambiguous`, `OutOfWindow`, `NonexistentLocalTime`) and `ConfigError`.
- No changes to existing APIs.

## 0.1.0 (2026-10-02)

- Initial release.
