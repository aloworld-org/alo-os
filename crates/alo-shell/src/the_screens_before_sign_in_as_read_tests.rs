//! The screens before sign-in as a reader is told about them: each one a
//! dialogue in front, every row of its list with its place and whether it is
//! selected, the keyboard where the keys go — and **nothing secret**, ever.

use alo_strings::{Filling, Strings, Word};

use super::*;
use crate::SignInKey;
use crate::access_events::{AnEvent, what_happened};
use crate::the_first_account::TheFirstAccount;

/// The machine's words, the sign-in screen's and the first start's among them.
fn words() -> Strings {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::sign_in_words::declare_sign_in_words(&mut vocabulary).unwrap();
    crate::first_start_words::declare_first_start_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// One word, said.
fn word(strings: &Strings, word: Word) -> String {
    strings
        .say(&word.key(), &Filling::nothing())
        .text()
        .to_owned()
}

/// Everything under `at`, in reading order, `at` itself first.
fn everything_under(tree: &ReadAloudTree, at: usize) -> Vec<&Node> {
    let node = &tree.nodes()[at];
    let mut all = vec![node];
    for child in &node.children {
        all.extend(everything_under(tree, *child));
    }
    all
}

/// The screen in front, and every node on it.
fn the_screen(tree: &ReadAloudTree) -> (&Node, Vec<&Node>) {
    let sheet = tree.in_front().unwrap();
    (&tree.nodes()[sheet.at], everything_under(tree, sheet.at))
}

/// The node the keyboard is on.
fn the_keyboard(tree: &ReadAloudTree) -> &Node {
    &tree.nodes()[tree.in_front().unwrap().keyboard]
}

/// A node's attribute, by key.
fn attribute<'n>(node: &'n Node, key: &str) -> Option<&'n str> {
    node.attributes
        .iter()
        .find_map(|(k, v)| (k == key).then_some(v.as_str()))
}

/// Whether a node is told as the one selected.
fn selected(node: &Node) -> bool {
    node.states[0] & (1 << access_roles::SELECTED) != 0
}

/// The tree for the first start as it is now.
fn as_read(strings: &Strings, first_start: &TheFirstStart) -> ReadAloudTree {
    ReadAloudTree::before_sign_in(strings, SignInShows::FirstStart(first_start))
}

/// **The first screen is read as it is drawn**: a dialogue in front, named by
/// its question; the three rows in order, each with its place; the keyboard on
/// *Language*, where the ring is; and *Accessibility*.
#[test]
fn the_first_screen_is_read_as_it_is_drawn() {
    let strings = words();
    let first_start = TheFirstStart::arriving(None);
    let tree = as_read(&strings, &first_start);
    assert_eq!(
        tree.in_front().unwrap().which,
        TheSheet::BeforeSignIn(ScreenBeforeSignIn::Overview)
    );
    let (screen, on_it) = the_screen(&tree);
    assert_eq!(screen.role, access_roles::number_of(Role::Dialogue));
    assert_eq!(screen.name, word(&strings, say::OVERVIEW_TITLE));
    assert_eq!(screen.parent, 0, "the screen hangs under the shell itself");

    let rows: Vec<&&Node> = on_it
        .iter()
        .filter(|node| node.role == access_roles::number_of(Role::ListItem))
        .collect();
    assert_eq!(rows.len(), Row::ALL.len());
    for (at, (row, which)) in rows.iter().zip(Row::ALL).enumerate() {
        let (label, set_to) =
            crate::overview_sheet::what_the_row_says(&first_start, which, &strings);
        assert_eq!(row.name, label);
        assert_eq!(row.description, set_to);
        assert_eq!(
            attribute(row, "posinset"),
            Some((at + 1).to_string().as_str())
        );
        assert_eq!(attribute(row, "setsize"), Some("3"));
    }

    let keyboard = the_keyboard(&tree);
    assert_eq!(keyboard.name, word(&strings, say::ROW_LANGUAGE));
    assert!(access_roles::has_the_keyboard(keyboard.states));
    assert_eq!(
        on_it
            .iter()
            .filter(|node| access_roles::has_the_keyboard(node.states))
            .count(),
        1,
        "a reader is told of one place the keys go, never two"
    );
    let accessibility = word(&strings, crate::sign_in_words::ACCESSIBILITY);
    assert!(
        on_it
            .iter()
            .any(|node| node.name == accessibility
                && node.role == access_roles::number_of(Role::Button)),
        "the Accessibility button is on the first screen"
    );
}

/// **Every language is read, not only the ones that fit**, with its place;
/// the keyboard follows the arrow; and the one chosen is told as selected,
/// with the keyboard then on *Use this language*.
#[test]
fn every_language_is_read_and_the_chosen_one_is_selected() {
    let strings = words();
    let mut first_start = TheFirstStart::arriving(None);
    first_start.pressed(SignInKey::Enter);
    let tree = as_read(&strings, &first_start);
    assert_eq!(
        tree.in_front().unwrap().which,
        TheSheet::BeforeSignIn(ScreenBeforeSignIn::Language)
    );
    let (_, on_it) = the_screen(&tree);
    let rows: Vec<&&Node> = on_it
        .iter()
        .filter(|node| node.role == access_roles::number_of(Role::ListItem))
        .collect();
    let how_many = first_start.languages().how_many();
    assert_eq!(rows.len(), how_many, "every language, not a window of them");
    assert!(rows.iter().all(|row| !selected(row)), "nothing chosen yet");
    let setsize = how_many.to_string();
    assert!(
        rows.iter()
            .all(|row| attribute(row, "setsize") == Some(setsize.as_str()))
    );

    let before = attribute(the_keyboard(&tree), "posinset")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    first_start.pressed(SignInKey::Later);
    let moved = as_read(&strings, &first_start);
    let after = attribute(the_keyboard(&moved), "posinset")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    assert_eq!(
        after,
        before + 1,
        "the keyboard is told one row further down"
    );

    first_start.pressed(SignInKey::Enter);
    let chosen = as_read(&strings, &first_start);
    let (_, on_it) = the_screen(&chosen);
    let selected_rows: Vec<&&Node> = on_it.iter().filter(|node| selected(node)).collect();
    assert_eq!(selected_rows.len(), 1, "one language is selected");
    assert_eq!(
        attribute(selected_rows[0], "posinset"),
        Some(after.to_string().as_str())
    );
    assert_eq!(
        the_keyboard(&chosen).name,
        word(&strings, say::USE_THIS_LANGUAGE),
        "after a choice the keyboard is on the way on"
    );
}

/// **A PIN is never told**: not what was typed, not how much, not whether
/// anything was — and the name, which is drawn in full, is.
#[test]
fn a_secret_is_never_told_and_the_name_is() {
    let strings = words();
    let mut account = TheFirstAccount::arriving();
    for letter in "Ada".chars() {
        account.pressed(SignInKey::Letter(letter));
    }
    let named = ReadAloudTree::before_sign_in(
        &strings,
        SignInShows::FirstAccount {
            account: account.shown(),
            said: None,
        },
    );
    assert_eq!(
        named.in_front().unwrap().which,
        TheSheet::BeforeSignIn(ScreenBeforeSignIn::Name)
    );
    let field = the_keyboard(&named);
    assert_eq!(field.role, access_roles::number_of(Role::Entry));
    assert_eq!(field.name, word(&strings, say::NAME_CHOICE));
    assert_eq!(field.description, "Ada", "the name typed is heard");

    account.pressed(SignInKey::Enter);
    let empty = ReadAloudTree::before_sign_in(
        &strings,
        SignInShows::FirstAccount {
            account: account.shown(),
            said: None,
        },
    );
    for digit in "4821".chars() {
        account.pressed(SignInKey::Letter(digit));
    }
    let typed = ReadAloudTree::before_sign_in(
        &strings,
        SignInShows::FirstAccount {
            account: account.shown(),
            said: None,
        },
    );
    assert_eq!(
        typed.in_front().unwrap().which,
        TheSheet::BeforeSignIn(ScreenBeforeSignIn::Pin)
    );
    let keyboard = the_keyboard(&typed);
    assert_eq!(keyboard.role, access_roles::number_of(Role::PasswordEntry));
    assert_eq!(keyboard.name, word(&strings, say::PIN_CHOICE));
    for node in typed.nodes() {
        assert!(
            !node.name.contains("4821") && !node.description.contains("4821"),
            "the PIN reached the tree: {node:?}"
        );
    }
    assert_eq!(
        typed.nodes(),
        empty.nodes(),
        "a reader can tell nothing about a PIN, not even that one is typed"
    );
}

/// **The sign-in screen greets the person and the keys go to the one field it
/// draws**, whatever field the entry thinks it is on.
#[test]
fn the_sign_in_screen_greets_and_its_one_field_has_the_keyboard() {
    let strings = words();
    let tree = ReadAloudTree::before_sign_in(
        &strings,
        SignInShows::Fields {
            whose: "disan",
            name: "",
            password_typed: true,
            field: SignInField::Name,
            said: None,
        },
    );
    assert_eq!(
        tree.in_front().unwrap().which,
        TheSheet::BeforeSignIn(ScreenBeforeSignIn::SignIn)
    );
    let (screen, on_it) = the_screen(&tree);
    assert_eq!(screen.role, access_roles::number_of(Role::Window));
    assert!(
        on_it
            .iter()
            .any(|node| node.role == access_roles::number_of(Role::Heading)
                && node.name.contains("disan")),
        "the greeting names whose machine it is"
    );
    let keyboard = the_keyboard(&tree);
    assert_eq!(keyboard.role, access_roles::number_of(Role::PasswordEntry));
    assert_eq!(
        keyboard.name,
        word(&strings, crate::sign_in_words::ENTER_YOUR_PIN)
    );
    assert!(keyboard.description.is_empty());
}

/// **A refusal is announced as it appears**, so somebody who cannot see the
/// line under the field hears that the PIN was not right.
#[test]
fn a_refusal_is_announced() {
    let strings = words();
    let refused = strings.say(&say::THEY_DIFFER.key(), &Filling::nothing());
    let tree = ReadAloudTree::before_sign_in(
        &strings,
        SignInShows::Fields {
            whose: "disan",
            name: "",
            password_typed: false,
            field: SignInField::Password,
            said: Some(&refused),
        },
    );
    let (_, on_it) = the_screen(&tree);
    assert!(
        on_it
            .iter()
            .any(|node| node.name == refused.text() && node.announced),
        "the refusal is on the tree and announced"
    );
}

/// **A screen with only a sentence** is read as that sentence, with nowhere
/// else for the keyboard to be and no Accessibility, which it does not draw.
#[test]
fn a_sentence_is_read_and_holds_the_keyboard() {
    let strings = words();
    let said = strings.say(&say::THEY_DIFFER.key(), &Filling::nothing());
    let tree = ReadAloudTree::before_sign_in(&strings, SignInShows::Sentence(&said));
    assert_eq!(the_keyboard(&tree).name, said.text());
    let accessibility = word(&strings, crate::sign_in_words::ACCESSIBILITY);
    assert!(tree.nodes().iter().all(|node| node.name != accessibility));
}

/// **Going on to the next screen is heard as a new screen**, not as the
/// keyboard moving on the old one — which is why each is named apart.
#[test]
fn the_next_screen_is_heard_as_a_new_one() {
    let strings = words();
    let mut first_start = TheFirstStart::arriving(None);
    let (told, was) = what_happened(None, &as_read(&strings, &first_start));
    assert!(matches!(told.first(), Some(AnEvent::InFront { .. })));

    first_start.pressed(SignInKey::Later);
    let (told, was) = what_happened(was.as_ref(), &as_read(&strings, &first_start));
    assert!(
        matches!(told.as_slice(), [AnEvent::TheKeyboardIsHere { .. }]),
        "an arrow on the same screen moves the keyboard: {told:?}"
    );

    first_start.pressed(SignInKey::Enter);
    let (told, _) = what_happened(was.as_ref(), &as_read(&strings, &first_start));
    assert!(
        matches!(told.first(), Some(AnEvent::InFront { .. })),
        "opening the region's list is a new screen: {told:?}"
    );
}
