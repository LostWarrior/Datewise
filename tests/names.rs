use chrono::Weekday;
use datewise::fields::{month_prefix, weekday_before};
use datewise::names::{month_from_name, ordinal_suffix_len, weekday_from_name};

#[test]
fn every_month_spelling_resolves() {
    let spellings = [
        ("jan", 1),
        ("January", 1),
        ("FEB", 2),
        ("february", 2),
        ("Mar", 3),
        ("march", 3),
        ("apr", 4),
        ("April", 4),
        ("may", 5),
        ("Jun", 6),
        ("june", 6),
        ("jul", 7),
        ("July", 7),
        ("aug", 8),
        ("August", 8),
        ("sep", 9),
        ("Sept", 9),
        ("September", 9),
        ("oct", 10),
        ("October", 10),
        ("nov", 11),
        ("November", 11),
        ("dec", 12),
        ("December", 12),
    ];
    for (name, month) in spellings {
        assert_eq!(month_from_name(name), Some(month), "{name}");
    }
    for bad in ["", "Marching", "Mayor", " may", "septem", "été"] {
        assert_eq!(month_from_name(bad), None, "{bad}");
    }
}

#[test]
fn weekday_spellings_resolve() {
    assert_eq!(weekday_from_name("Thursday"), Some(Weekday::Thu));
    assert_eq!(weekday_from_name("thurs"), Some(Weekday::Thu));
    assert_eq!(weekday_from_name("THU"), Some(Weekday::Thu));
    assert_eq!(weekday_from_name("Sunday-ish"), None);
    assert_eq!(weekday_from_name(""), None);
}

#[test]
fn ordinal_suffix_needs_a_boundary() {
    assert_eq!(ordinal_suffix_len("th"), Some(2));
    assert_eq!(ordinal_suffix_len("ND June"), Some(2));
    assert_eq!(ordinal_suffix_len("rd,"), Some(2));
    assert_eq!(ordinal_suffix_len("thx"), None);
    assert_eq!(ordinal_suffix_len("é"), None);
    assert_eq!(ordinal_suffix_len("t"), None);
}

#[test]
fn month_prefix_prefers_longest_and_respects_boundary() {
    assert_eq!(month_prefix("September 2026"), Some((9, 9)));
    assert_eq!(month_prefix("Sept 2026"), Some((9, 4)));
    assert_eq!(month_prefix("sep."), Some((9, 3)));
    assert_eq!(month_prefix("Marching"), None);
    assert_eq!(month_prefix("Mayor"), None);
    assert_eq!(month_prefix("may"), Some((5, 3)));
}

#[test]
fn weekday_before_needs_single_trailing_space() {
    assert_eq!(
        weekday_before("Sports Day on Wednesday "),
        Some(Weekday::Wed)
    );
    assert_eq!(weekday_before("xWednesday "), None);
    assert_eq!(weekday_before("Wednesday"), None);
    assert_eq!(weekday_before("é "), None);
}
