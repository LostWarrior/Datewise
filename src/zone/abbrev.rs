use chrono_tz::America::{Chicago, Denver, Los_Angeles, New_York};
use chrono_tz::Tz;

pub(super) struct Abbrev {
    pub(super) name: &'static str,
    // Zone used in every region, when the abbreviation is not shared.
    always: Option<Tz>,
    // Zone used only when the locale's region is listed.
    by_region: &'static [(&'static str, Tz)],
    // Known meanings reported when neither of the above applies.
    pub(super) candidates: &'static [Tz],
}

impl Abbrev {
    pub(super) fn zone(&self, region: Option<&str>) -> Option<Tz> {
        let regional = region.and_then(|r| {
            self.by_region
                .iter()
                .find(|(code, _)| code.eq_ignore_ascii_case(r))
                .map(|&(_, tz)| tz)
        });
        regional.or(self.always)
    }
}

const fn fixed(name: &'static str, tz: Tz) -> Abbrev {
    Abbrev {
        name,
        always: Some(tz),
        by_region: &[],
        candidates: &[],
    }
}

const fn shared(
    name: &'static str,
    by_region: &'static [(&'static str, Tz)],
    candidates: &'static [Tz],
) -> Abbrev {
    Abbrev {
        name,
        always: None,
        by_region,
        candidates,
    }
}

const fn na(tz: Tz) -> [(&'static str, Tz); 2] {
    [("US", tz), ("CA", tz)]
}

const EASTERN: &[(&str, Tz)] = &na(New_York);
const CENTRAL: &[(&str, Tz)] = &na(Chicago);
const MOUNTAIN: &[(&str, Tz)] = &na(Denver);
const PACIFIC: &[(&str, Tz)] = &na(Los_Angeles);

pub(super) const ABBREVIATIONS: &[Abbrev] = &[
    shared(
        "BST",
        &[
            ("GB", chrono_tz::Europe::London),
            ("IE", chrono_tz::Europe::London),
        ],
        &[chrono_tz::Europe::London, chrono_tz::Asia::Dhaka],
    ),
    fixed("EST", New_York),
    fixed("EDT", New_York),
    shared("ET", EASTERN, &[New_York]),
    fixed("CDT", Chicago),
    shared("CST", CENTRAL, &[Chicago, chrono_tz::Asia::Shanghai]),
    shared("CT", CENTRAL, &[Chicago]),
    fixed("MST", Denver),
    fixed("MDT", Denver),
    shared("MT", MOUNTAIN, &[Denver]),
    fixed("PST", Los_Angeles),
    fixed("PDT", Los_Angeles),
    shared("PT", PACIFIC, &[Los_Angeles]),
    fixed("AEST", chrono_tz::Australia::Sydney),
    fixed("AEDT", chrono_tz::Australia::Sydney),
    shared(
        "IST",
        &[
            ("IN", chrono_tz::Asia::Kolkata),
            ("IE", chrono_tz::Europe::Dublin),
        ],
        &[
            chrono_tz::Asia::Kolkata,
            chrono_tz::Europe::Dublin,
            chrono_tz::Asia::Jerusalem,
        ],
    ),
    fixed("CET", chrono_tz::Europe::Paris),
    fixed("CEST", chrono_tz::Europe::Paris),
    fixed("NZST", chrono_tz::Pacific::Auckland),
    fixed("NZDT", chrono_tz::Pacific::Auckland),
    fixed("SAST", chrono_tz::Africa::Johannesburg),
];
