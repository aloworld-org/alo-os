//! **What `icu`'s unstable region names answer, held to known answers** —
//! [ADR 0135](../../../docs/decisions/0135-the-region-names-come-from-icus-unstable-half.md).
//!
//! `alo_formats::region_names` is the only file that uses
//! `icu::experimental::displaynames`, which CLDR's own implementers have not
//! promised to keep. An upgrade can move it in two ways: **loudly**, by not
//! compiling, which needs no test; and **quietly**, by compiling and answering
//! something else — no names at all, or names in the wrong language. The quiet
//! one is what this file is for, because `region_names` falls back to English
//! and then to the code, so a names API that stopped answering would still give
//! every person a full, sorted list: in English, whatever they read.
//!
//! So each check below would fail on that fallback, not only on a crash. Run
//! on every build, by `cargo test -p alo-formats`; nothing here is a hand
//! measurement.

use alo_formats::region_names::the_name_of;
use alo_formats::regions::EVERY_REGION;
use alo_formats::the_regions_named_in;

/// **Known names, in two languages and the source**, exactly as CLDR has them.
#[test]
fn known_regions_are_called_what_cldr_calls_them() {
    let known = [
        ("DE", "en", "Germany"),
        ("DE", "de", "Deutschland"),
        ("DE", "fr", "Allemagne"),
        ("BE", "en", "Belgium"),
        ("BE", "de", "Belgien"),
        ("BE", "fr", "Belgique"),
        ("XK", "en", "Kosovo"),
    ];
    for (code, language, name) in known {
        assert_eq!(
            the_name_of(code, language).as_deref(),
            Some(name),
            "{code} in {language}: icu's region names answered something else"
        );
    }
}

/// **Every one of the 24 has a name of its own for Germany** — not the English
/// one, which is what the fallback would give if the names API stopped
/// answering in that language.
#[test]
fn every_language_names_germany_in_itself_not_in_english() {
    for official in alo_strings::union::OFFICIAL {
        if official.tag == "en" {
            continue;
        }
        let name = the_name_of("DE", official.tag);
        assert!(
            name.as_deref()
                .is_some_and(|it| it != "Germany" && it != "DE"),
            "{}: Germany is called {name:?}, which is the fallback rather than the language",
            official.tag
        );
    }
}

/// **Kosovo is on the list, named, in every language** — the owner confirmed
/// on 2026-10-28 that it stays, and a names API that dropped `XK` would leave
/// its code where its name should be.
#[test]
fn kosovo_is_offered_and_named_in_every_language() {
    assert!(EVERY_REGION.contains(&"XK"));
    for official in alo_strings::union::OFFICIAL {
        let named = the_regions_named_in(official.tag);
        let kosovo = named.iter().find(|region| region.code == "XK");
        assert!(
            kosovo.is_some_and(|it| it.name != "XK"),
            "{}: Kosovo is {kosovo:?}",
            official.tag
        );
    }
}
