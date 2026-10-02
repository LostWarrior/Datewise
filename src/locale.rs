//! English regional conventions: numeric date order and first day of the week.

use chrono::Weekday;

/// Order of the day, month and year fields in a numeric date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DateOrder {
    /// Day, month, year: `03/10/2026` is 3 October.
    DayFirst,
    /// Month, day, year: `03/10/2026` is 10 March.
    MonthFirst,
    /// Year first (`2026/10/03`); other numeric orders are not decided by the locale.
    YearFirst,
}

/// An English-language region with its date conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Locale {
    date_order: DateOrder,
    week_start: Weekday,
    region: &'static str,
}

const fn en(region: &'static str, date_order: DateOrder, week_start: Weekday) -> Locale {
    Locale {
        date_order,
        week_start,
        region,
    }
}

impl Locale {
    /// United Kingdom.
    pub const EN_GB: Locale = en("GB", DateOrder::DayFirst, Weekday::Mon);
    /// Ireland.
    pub const EN_IE: Locale = en("IE", DateOrder::DayFirst, Weekday::Mon);
    /// United States.
    pub const EN_US: Locale = en("US", DateOrder::MonthFirst, Weekday::Sun);
    /// Canada.
    pub const EN_CA: Locale = en("CA", DateOrder::YearFirst, Weekday::Sun);
    /// Australia.
    pub const EN_AU: Locale = en("AU", DateOrder::DayFirst, Weekday::Mon);
    /// New Zealand.
    pub const EN_NZ: Locale = en("NZ", DateOrder::DayFirst, Weekday::Mon);
    /// India.
    pub const EN_IN: Locale = en("IN", DateOrder::DayFirst, Weekday::Sun);
    /// South Africa.
    pub const EN_ZA: Locale = en("ZA", DateOrder::YearFirst, Weekday::Sun);
    /// Singapore.
    pub const EN_SG: Locale = en("SG", DateOrder::DayFirst, Weekday::Sun);
    /// Philippines.
    pub const EN_PH: Locale = en("PH", DateOrder::MonthFirst, Weekday::Sun);

    const ALL: [Locale; 10] = [
        Self::EN_GB,
        Self::EN_IE,
        Self::EN_US,
        Self::EN_CA,
        Self::EN_AU,
        Self::EN_NZ,
        Self::EN_IN,
        Self::EN_ZA,
        Self::EN_SG,
        Self::EN_PH,
    ];

    /// Builds a locale from `en-GB` or `en_GB` (any case); the region is required.
    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Locale> {
        let mut parts = tag.split(['-', '_']);
        let (lang, region) = (parts.next()?, parts.next()?);
        if parts.next().is_some() || !lang.eq_ignore_ascii_case("en") {
            return None;
        }
        Self::ALL
            .into_iter()
            .find(|locale| locale.region.eq_ignore_ascii_case(region))
    }

    /// Order of numeric date fields.
    #[must_use]
    pub fn date_order(&self) -> DateOrder {
        self.date_order
    }

    /// First day of the week.
    #[must_use]
    pub fn week_start(&self) -> Weekday {
        self.week_start
    }

    /// Upper-case two-letter region code.
    #[must_use]
    pub fn region(&self) -> &'static str {
        self.region
    }
}
