//! A place as a window shows it is meant by the names typing takes: its own
//! name, or — with the keyboard, or unnamed — what its window's title holds.

use super::{APlaceForText, is_meant_by};

const FILE: &str = "hello.txt";
const WINDOW: &str = "hello.txt - VSCodium";

#[test]
fn a_place_as_shown_has_the_keyboard_it_was_shown_with() {
    assert!(APlaceForText::as_shown(FILE, WINDOW, true).has_the_keyboard());
    assert!(!APlaceForText::as_shown(FILE, WINDOW, false).has_the_keyboard());
}

/// **The window's title is a name typing takes**, for the place with the
/// keyboard: VSCodium's entry is named after the file, and a model that names
/// it by the window it read is typing into the same place.
#[test]
fn the_title_names_the_place_with_the_keyboard() {
    let place = APlaceForText::as_shown(FILE, WINDOW, true);
    for said in [FILE, WINDOW] {
        assert!(is_meant_by(&place, said), "{said}");
    }
    assert!(!is_meant_by(&place, "Document"));
}

/// **And only that one**: a named place without the keyboard is meant by its
/// own name alone.
#[test]
fn the_title_does_not_name_a_named_place_without_the_keyboard() {
    let place = APlaceForText::as_shown(FILE, WINDOW, false);
    assert!(is_meant_by(&place, FILE));
    assert!(!is_meant_by(&place, WINDOW));
}

/// **An unnamed place is meant by its window's title**, keyboard or not.
#[test]
fn the_title_names_an_unnamed_place() {
    assert!(is_meant_by(
        &APlaceForText::as_shown("", WINDOW, false),
        FILE
    ));
}

/// The tree's two words of states, holding these.
fn holding(states: &[u32]) -> Vec<u32> {
    let mut words = vec![0_u32, 0];
    for state in states {
        if let Some(word) = words.get_mut(usize::try_from(state / 32).unwrap_or(0)) {
            *word |= 1 << (state % 32);
        }
    }
    words
}

/// `ATSPI_ROLE_SPIN_BUTTON`, `_TEXT`, `_ENTRY`, `_DOCUMENT_FRAME`,
/// `_DOCUMENT_TEXT`, `_PASSWORD_TEXT`.
const SPIN_BUTTON: u32 = 52;
const TEXT: u32 = 61;
const ENTRY: u32 = 79;
const DOCUMENT_FRAME: u32 = 82;
const DOCUMENT_TEXT: u32 = 94;
const PASSWORD_TEXT: u32 = 40;

/// **A search box, a number box and a `TEXT` that says it is one line are
/// one-line fields** (the owner, 2026-11-20): typing that saves is not aimed
/// at them.
#[test]
fn an_entry_and_a_single_line_text_are_one_line() {
    let editable = [super::EDITABLE, super::SHOWING];
    assert!(super::is_one_line(ENTRY, &holding(&editable)));
    assert!(super::is_one_line(
        ENTRY,
        &holding(&[super::EDITABLE, super::SHOWING, super::SINGLE_LINE])
    ));
    assert!(super::is_one_line(SPIN_BUTTON, &holding(&editable)));
    assert!(super::is_one_line(
        TEXT,
        &holding(&[super::EDITABLE, super::SINGLE_LINE])
    ));
}

/// **A multi-line text and a document are not** — VSCodium's editor is an
/// `ENTRY` that says `MULTI_LINE` (measured 2026-11-13), and GNOME Text
/// Editor's document a `TEXT` — and a `TEXT` that says neither is not said to
/// be one line. A password field is its own refusal, not this one.
#[test]
fn a_multi_line_text_and_a_document_are_not_one_line() {
    let multi = holding(&[super::EDITABLE, super::MULTI_LINE]);
    assert!(!super::is_one_line(TEXT, &multi));
    assert!(!super::is_one_line(ENTRY, &multi));
    assert!(!super::is_one_line(
        ENTRY,
        &holding(&[super::SINGLE_LINE, super::MULTI_LINE])
    ));
    assert!(!super::is_one_line(TEXT, &holding(&[super::EDITABLE])));
    for document in [DOCUMENT_FRAME, DOCUMENT_TEXT] {
        assert!(!super::is_one_line(
            document,
            &holding(&[super::EDITABLE, super::SINGLE_LINE])
        ));
    }
    assert!(!super::is_one_line(
        PASSWORD_TEXT,
        &holding(&[super::SINGLE_LINE])
    ));
}

/// A window holding one field and, beside it, a list of `rows` empty rows the
/// search meets first.
struct ALongList {
    rows: usize,
}

/// The application the long list is in.
const LISTED: &str = "org.example.Listed";

fn node(object: &str) -> crate::NodeAt {
    crate::NodeAt {
        holder: ":1.7".to_owned(),
        object: object.to_owned(),
    }
}

impl crate::AccessibilityTree for ALongList {
    fn applications(&self) -> Result<Vec<crate::Running>, crate::TreeFault> {
        Ok(vec![crate::Running {
            at: node("/root"),
            is: Some(LISTED.to_owned()),
        }])
    }

    fn facts(&self, at: &crate::NodeAt) -> Result<crate::Facts, crate::TreeFault> {
        let facts = |role, states, children| crate::Facts {
            role,
            name: String::new(),
            states: vec![states, 0],
            children,
            has_text: false,
            unlisted_children: false,
        };
        let showing = 1 << super::SHOWING;
        Ok(match at.object.as_str() {
            "/root" => facts(75, showing, vec![node("/window")]),
            "/window" => facts(23, showing, vec![node("/field"), node("/list")]),
            "/field" => facts(61, showing | (1 << super::EDITABLE), Vec::new()),
            "/list" => facts(
                20,
                showing,
                (0..self.rows)
                    .map(|row| node(&format!("/row/{row}")))
                    .collect(),
            ),
            _ => facts(20, 0, Vec::new()),
        })
    }

    fn text(&self, _: &crate::NodeAt) -> Result<String, crate::TreeFault> {
        Ok(String::new())
    }

    fn actions(&self, _: &crate::NodeAt) -> Result<Vec<String>, crate::TreeFault> {
        Ok(Vec::new())
    }

    fn act(&self, _: &crate::NodeAt, _: usize) -> Result<bool, crate::TreeFault> {
        Ok(false)
    }
}

/// **The search for a field asks about at most `MOST_ASKED` things, exactly.**
/// It meets the application, the window, the list and every row before the
/// field, so the field is the `rows + 4`th thing asked about: found at
/// `MOST_ASKED`, and not one past it.
#[test]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
fn the_search_for_a_field_stops_at_most_asked_exactly() {
    let most = crate::walking::MOST_ASKED;
    let found = |rows| {
        super::the_places_for_text(&ALongList { rows }, LISTED)
            .unwrap()
            .len()
    };
    assert_eq!(found(most - 4), 1, "the field at the bound was not found");
    assert_eq!(found(most - 3), 0, "the search went past its bound");
}
