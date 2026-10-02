use chrono_tz::America::{Chicago, Denver, Los_Angeles, New_York};
use chrono_tz::Tz;

const HOUR: i32 = 3600;

#[derive(Clone, Copy)]
pub(super) enum Meaning {
    Offset(i32),
    Region(Tz),
}

pub(super) struct Abbrev {
    pub(super) name: &'static str,
    always: Option<Meaning>,
    by_region: &'static [(&'static str, Meaning)],
    pub(super) candidates: &'static [i32],
}

impl Abbrev {
    pub(super) fn meaning(&self, region: Option<&str>) -> Option<Meaning> {
        let regional = region.and_then(|r| {
            self.by_region
                .iter()
                .find(|(code, _)| code.eq_ignore_ascii_case(r))
                .map(|&(_, meaning)| meaning)
        });
        regional.or(self.always)
    }
}

const fn fixed(name: &'static str, offset: i32) -> Abbrev {
    Abbrev {
        name,
        always: Some(Meaning::Offset(offset)),
        by_region: &[],
        candidates: &[],
    }
}

const fn shared(
    name: &'static str,
    by_region: &'static [(&'static str, Meaning)],
    candidates: &'static [i32],
) -> Abbrev {
    Abbrev {
        name,
        always: None,
        by_region,
        candidates,
    }
}

const fn na(tz: Tz) -> [(&'static str, Meaning); 2] {
    [("US", Meaning::Region(tz)), ("CA", Meaning::Region(tz))]
}

const EASTERN: &[(&str, Meaning)] = &na(New_York);
const CENTRAL: &[(&str, Meaning)] = &na(Chicago);
const MOUNTAIN: &[(&str, Meaning)] = &na(Denver);
const PACIFIC: &[(&str, Meaning)] = &na(Los_Angeles);
const CST_US: &[(&str, Meaning)] = &[
    ("US", Meaning::Offset(-6 * HOUR)),
    ("CA", Meaning::Offset(-6 * HOUR)),
];
const BST_UK: &[(&str, Meaning)] = &[("GB", Meaning::Offset(HOUR)), ("IE", Meaning::Offset(HOUR))];
const IST_REGIONS: &[(&str, Meaning)] = &[
    ("IN", Meaning::Offset(5 * HOUR + 1800)),
    ("IE", Meaning::Offset(HOUR)),
];

pub(super) const ABBREVIATIONS: &[Abbrev] = &[
    shared("BST", BST_UK, &[HOUR, 6 * HOUR]),
    fixed("EST", -5 * HOUR),
    fixed("EDT", -4 * HOUR),
    shared("ET", EASTERN, &[-5 * HOUR, -4 * HOUR]),
    fixed("CDT", -5 * HOUR),
    shared("CST", CST_US, &[-6 * HOUR, 8 * HOUR]),
    shared("CT", CENTRAL, &[-6 * HOUR, -5 * HOUR]),
    fixed("MST", -7 * HOUR),
    fixed("MDT", -6 * HOUR),
    shared("MT", MOUNTAIN, &[-7 * HOUR, -6 * HOUR]),
    fixed("PST", -8 * HOUR),
    fixed("PDT", -7 * HOUR),
    shared("PT", PACIFIC, &[-8 * HOUR, -7 * HOUR]),
    fixed("AEST", 10 * HOUR),
    fixed("AEDT", 11 * HOUR),
    shared("IST", IST_REGIONS, &[5 * HOUR + 1800, HOUR, 2 * HOUR]),
    fixed("CET", HOUR),
    fixed("CEST", 2 * HOUR),
    fixed("NZST", 12 * HOUR),
    fixed("NZDT", 13 * HOUR),
    fixed("SAST", 2 * HOUR),
];
