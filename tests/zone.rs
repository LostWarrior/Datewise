use chrono::FixedOffset;
use chrono_tz::America::{Chicago, Denver, Los_Angeles, New_York};
use chrono_tz::Tz;
use datewise::locale::Locale;
use datewise::zone::{find_zone, FoundZone};

fn zone(text: &str, locale: Option<Locale>) -> Option<FoundZone> {
    find_zone(text, locale).map(|m| m.zone)
}

fn fixed(seconds: i32) -> Option<FoundZone> {
    FixedOffset::east_opt(seconds).map(FoundZone::Fixed)
}

fn iana(tz: Tz) -> Option<FoundZone> {
    Some(FoundZone::Iana(tz))
}

fn ambiguous(text: &str, locale: Option<Locale>) -> bool {
    matches!(zone(text, locale), Some(FoundZone::Ambiguous(c)) if !c.is_empty())
}

#[test]
fn utc_and_offsets() {
    assert_eq!(zone("at 10:00 UTC", None), fixed(0));
    assert_eq!(zone("10:00 GMT", None), fixed(0));
    assert_eq!(zone("2026-10-03T10:00:00Z", None), fixed(0));
    assert_eq!(zone("UTC+1", None), fixed(3600));
    assert_eq!(zone("GMT-5", None), fixed(-5 * 3600));
    assert_eq!(zone("UTC+05:30", None), fixed(19800));
    assert_eq!(zone("UTC+0530", None), fixed(19800));
    assert_eq!(zone("10:00 +05:30", None), fixed(19800));
    assert_eq!(zone("10:00 +0530", None), fixed(19800));
    assert_eq!(zone("2026-10-03T10:00:00-05:00", None), fixed(-5 * 3600));
    assert_eq!(zone("10:00:00.250+01:00", None), fixed(3600));
    let m = find_zone("meet 10:00 UTC+1 ok", None).unwrap();
    assert_eq!(&"meet 10:00 UTC+1 ok"[m.span], "UTC+1");
}

#[test]
fn offset_false_positives() {
    for text in [
        "UTC+15",
        "UTC+123",
        "UTC+",
        "+5",
        "10-0530",
        "call 555-0100",
        "9:00-10:30",
        "x+05:30",
        "+15:00",
        "+05:60",
        "plan Z",
        "Z",
        "10Z",
        "UTCX",
        "GMTs",
        "2026-10-03",
        "UTC+1x",
    ] {
        assert_eq!(zone(text, None), None, "{text}");
    }
}

#[test]
fn north_american_abbreviations() {
    let cases = [
        ("EST", New_York),
        ("EDT", New_York),
        ("CDT", Chicago),
        ("MST", Denver),
        ("MDT", Denver),
        ("PST", Los_Angeles),
        ("PDT", Los_Angeles),
    ];
    for (text, tz) in cases {
        assert_eq!(zone(text, None), iana(tz), "{text}");
        assert_eq!(zone(text, Some(Locale::EN_GB)), iana(tz), "{text}");
    }
}

#[test]
fn shorthands_need_a_north_american_region() {
    let cases = [
        ("ET", New_York),
        ("CT", Chicago),
        ("MT", Denver),
        ("PT", Los_Angeles),
    ];
    for (text, tz) in cases {
        let at = format!("3pm {text}");
        for locale in [Locale::EN_US, Locale::EN_CA] {
            assert_eq!(zone(&at, Some(locale)), iana(tz), "{at}");
        }
        assert!(ambiguous(&at, None), "{at}");
        assert!(ambiguous(&at, Some(Locale::EN_GB)), "{at}");
    }
    assert_eq!(zone("CST", Some(Locale::EN_US)), iana(Chicago));
    assert_eq!(zone("CST", Some(Locale::EN_CA)), iana(Chicago));
    assert!(ambiguous("CST", None));
    assert!(ambiguous("CST", Some(Locale::EN_AU)));
}

#[test]
fn bst_and_ist_depend_on_region() {
    use chrono_tz::{Asia::Kolkata, Europe::Dublin, Europe::London};
    assert_eq!(zone("BST", Some(Locale::EN_GB)), iana(London));
    assert_eq!(zone("BST", Some(Locale::EN_IE)), iana(London));
    assert!(ambiguous("BST", None));
    assert!(ambiguous("BST", Some(Locale::EN_US)));
    assert_eq!(zone("IST", Some(Locale::EN_IN)), iana(Kolkata));
    assert_eq!(zone("IST", Some(Locale::EN_IE)), iana(Dublin));
    assert!(ambiguous("IST", None));
    assert!(ambiguous("IST", Some(Locale::EN_GB)));
}

#[test]
fn other_abbreviations() {
    use chrono_tz::{Africa::Johannesburg, Australia::Sydney, Europe::Paris, Pacific::Auckland};
    let cases = [
        ("AEST", Sydney),
        ("AEDT", Sydney),
        ("CET", Paris),
        ("CEST", Paris),
        ("NZST", Auckland),
        ("NZDT", Auckland),
        ("SAST", Johannesburg),
    ];
    for (text, tz) in cases {
        assert_eq!(zone(text, None), iana(tz), "{text}");
    }
}

#[test]
fn word_boundaries_prevent_false_positives() {
    for text in [
        "ACT", "ACTION", "OCT", "SECT", "5pm", "5 pm", "pt", "est", "ist", "bst", "CTRL", "PTA",
        "ESTONIA", "TEST", "1ST", "EST1", "MTs", "xCET",
    ] {
        let found = find_zone(text, Some(Locale::EN_US));
        assert!(found.is_none(), "{text}: {found:?}");
    }
    let m = find_zone("Call at 3pm PT today", Some(Locale::EN_US)).unwrap();
    assert_eq!(m.span, 12..14);
    let m = find_zone("(CET)", None).unwrap();
    assert_eq!(m.span, 1..4);
    assert_eq!(find_zone("pm ACT then PST", None).unwrap().span, 12..15);
}

#[test]
fn zone_is_found_at_the_end_of_long_text() {
    let text = format!("{}UTC", " ".repeat(10_000));
    assert_eq!(find_zone(&text, None).unwrap().span, 10_000..10_003);
}

#[test]
fn whole_input_parsers_reject_oversized_input() {
    let long = "a".repeat(datewise::MAX_INPUT_BYTES + 1);
    assert!(datewise::zone::parse_zone(&long).is_err());
    assert!(Locale::from_tag(&long).is_none());
}
