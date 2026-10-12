//! What typing leaves, what it takes back, and the two empties it must not
//! confuse.

use super::{THE_MOST_THAT_CAN_BE_TYPED, WhatTheyTyped, which_are_left};

/// What *More apps* would be showing, in the order it shows them: pinned in
/// the person's order, then open ones in the order they opened.
const WHAT_IS_SHOWN: [&str; 5] = [
    "org.kde.dolphin",
    "Settings",
    "org.mozilla.firefox",
    "org.gnome.TextEditor",
    "org.blender.Blender",
];

fn typing(what: &str) -> WhatTheyTyped {
    let mut typed = WhatTheyTyped::nothing();
    for letter in what.chars() {
        typed.typed(letter);
    }
    typed
}

fn left(what: &str) -> Vec<&'static str> {
    which_are_left(WHAT_IS_SHOWN, &typing(what))
        .into_iter()
        .filter_map(|which| WHAT_IS_SHOWN.get(which).copied())
        .collect()
}

/// **An empty field shows everything**, which is what makes the filtered list
/// and the unfiltered one one code path rather than two.
#[test]
fn an_empty_search_leaves_every_entry_in_its_order() {
    assert_eq!(left(""), WHAT_IS_SHOWN.to_vec());
    assert_eq!(left("   "), WHAT_IS_SHOWN.to_vec());
}

/// **The order is the list's, never how well a name matched.**
///
/// Asserted as *the indices come back increasing*, which is the property
/// itself rather than a consequence of one fixture: any ranking filter breaks
/// it for some search, and this asks several. An earlier version of this test
/// claimed a letter was in all five names, was wrong about that, and would
/// have gone on testing something weaker if the claim had happened to hold.
#[test]
fn what_is_left_keeps_the_order_it_came_in() {
    for search in ["", "o", "e", "l", "org", "SETTINGS", "zzz"] {
        let which = which_are_left(WHAT_IS_SHOWN, &typing(search));
        assert!(
            which.windows(2).all(|pair| match pair {
                [earlier, later] => earlier < later,
                // `windows(2)` yields pairs; a slice of any other length is
                // unreachable and answers *in order* rather than panicking,
                // because this workspace denies indexing a slice.
                _ => true,
            }),
            "{search:?} came back out of order: {which:?}"
        );
    }

    // And the names themselves, for the two searches worth naming. **Computed
    // rather than recalled** — two earlier versions of this test asserted which
    // names held a letter from memory and were wrong both times, which is a
    // fixture teaching the reader something false even while it passes.
    assert_eq!(
        left("l"),
        vec![
            "org.kde.dolphin",
            "org.mozilla.firefox",
            "org.blender.Blender",
        ],
        "`l` is in mozilla as well as dolphin and blender"
    );
    assert_eq!(left("e"), WHAT_IS_SHOWN.to_vec(), "`e` is in all five");
    assert_eq!(left(""), WHAT_IS_SHOWN.to_vec());
}

/// **A name is matched anywhere in it, not only at its start.**
///
/// A person knows their browser as *Firefox*; this machine calls it
/// `org.mozilla.firefox`. A search anchored at the start would find nothing
/// for the only name they have.
#[test]
fn a_search_matches_inside_a_name() {
    assert_eq!(left("firefox"), vec!["org.mozilla.firefox"]);
    assert_eq!(left("blender"), vec!["org.blender.Blender"]);
}

/// **Case is folded both ways.**
#[test]
fn case_does_not_decide_what_is_found() {
    assert_eq!(left("FIREFOX"), left("firefox"));
    assert_eq!(left("SeTtInGs"), vec!["Settings"]);
}

/// **The Settings entry is searchable by the name a person reads.**
///
/// It has no identifier — the owner's ruling forbids inventing one — so the
/// only string it could ever be found by is its translated label, which is
/// what the caller passes. This test is why `which_are_left` takes names
/// rather than entries.
#[test]
fn the_settings_entry_is_found_by_its_own_label() {
    assert_eq!(left("sett"), vec!["Settings"]);
}

/// **A search that matches nothing leaves nothing — and is not an empty
/// field.**
///
/// The two produce a list of zero rows and are opposite states: one shows
/// everything, the other owes the person a sentence saying nothing matched.
/// `is_nothing` is what tells them apart, and a drawing that asked only the
/// list length could not.
#[test]
fn a_search_that_finds_nothing_is_not_an_empty_field() {
    assert!(left("zzzz").is_empty());
    assert!(
        !typing("zzzz").is_nothing(),
        "a search with text in it reported itself empty, so the person would be shown the whole \
         list instead of being told nothing matched"
    );
    assert!(typing("").is_nothing());
    assert!(
        typing("   ").is_nothing(),
        "spaces alone are not a search, so the field shows everything rather than nothing"
    );
}

/// **Erasing takes back one letter a person would recognise**, not one `char`.
///
/// `é` as `e` plus a combining accent is one grapheme and two `char`s. Erasing
/// a `char` leaves a bare `e` the person never typed and must erase again.
#[test]
fn erasing_takes_back_one_grapheme() {
    let mut typed = WhatTheyTyped::nothing();
    for letter in "ab".chars() {
        typed.typed(letter);
    }
    for letter in ['e', '\u{301}'] {
        typed.typed(letter);
    }
    assert_eq!(typed.how_many_letters(), 3, "the premise: a, b and e-acute");

    assert!(typed.erased());

    assert_eq!(
        typed.as_text(),
        "ab",
        "erasing left half of a letter the person typed as one"
    );
    assert_eq!(typed.how_many_letters(), 2);
}

/// **Erasing an empty field takes nothing and says so**, so a caller does not
/// redraw a frame for a key that changed nothing.
#[test]
fn erasing_nothing_changes_nothing() {
    let mut typed = WhatTheyTyped::nothing();
    assert!(!typed.erased());
    assert!(!typed.cleared());
    assert_eq!(typed.as_text(), "");
}

/// **Clearing takes everything**, which is what Escape does to a field that has
/// something in it.
#[test]
fn clearing_takes_everything_back() {
    let mut typed = typing("firefox");
    assert!(typed.cleared());
    assert_eq!(typed.as_text(), "");
    assert!(typed.is_nothing());
}

/// **A held key cannot grow the field without end.**
///
/// The field takes keys from a keyboard and a keyboard repeats. Without a bound
/// a person leaning on a key would grow a `String` until the compositor is what
/// is wrong with their machine.
#[test]
fn a_field_stops_taking_letters_at_its_bound() {
    let mut typed = WhatTheyTyped::nothing();
    for _ in 0..THE_MOST_THAT_CAN_BE_TYPED {
        assert!(
            typed.typed('a'),
            "the field refused a letter before its bound"
        );
    }

    assert!(
        !typed.typed('a'),
        "the field took a letter past its bound, so a held key grows it without end"
    );
    assert_eq!(typed.how_many_letters(), THE_MOST_THAT_CAN_BE_TYPED);
}

/// **A control character is not a letter.**
///
/// The keys road refuses these too, and this is the second of the two places —
/// deliberately, because a model that trusted its caller would put a newline in
/// a one-line field the first time somebody wired a new seat to it.
#[test]
fn a_control_character_is_not_typed() {
    let mut typed = WhatTheyTyped::nothing();
    for control in ['\n', '\t', '\u{7}', '\u{0}'] {
        assert!(
            !typed.typed(control),
            "{control:?} was typed into the field"
        );
    }
    assert_eq!(typed.as_text(), "");
}
