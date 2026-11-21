//! **Typing one approved text into one field** (`accessible.type_text`, ADR
//! 0153), against a stand-in tree — every refusal beside what it refuses.
//!
//! The tree here answers as the measurement of 2026-11-13 found real ones do:
//! a GTK document takes text through the tree; a Chromium editor does not and
//! is named after its file; a web terminal is a field of class
//! `xterm-helper-textarea`.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

use alo_adapters::{
    AccessibilityTree, Adapters, Facts, NodeAt, Running, TYPE_TEXT, TreeFault, Typing,
    fallback_verbs,
};
use alo_applications::{Application, Installed};
use alo_capability::{
    Approvals, Authorised, Call, Effect, Given, Grant, Grantee, Grants, Proposal, Refused,
};
use alo_strings::{Strings, Vocabulary};

/// The editor alo types into.
const EDITOR: &str = "com.vscodium.codium";

/// `ATSPI_ROLE_FRAME`, `_ENTRY`, `_TEXT`, `_PASSWORD_TEXT`, `_TERMINAL`.
const FRAME: u32 = 23;
const ENTRY: u32 = 79;
const TEXT: u32 = 61;
const PASSWORD: u32 = 40;
const TERMINAL: u32 = 60;

/// Showing and editable; and with the keyboard.
const EDITABLE: u32 = (1 << 25) | (1 << 7);
const FOCUSED: u32 = EDITABLE | (1 << 12);

/// `ATSPI_STATE_MULTI_LINE` and `_SINGLE_LINE`: VSCodium's editor is an
/// `ENTRY` that says `MULTI_LINE`, its search box one that says `SINGLE_LINE`
/// (`docs/autonomy/updates/how-alo-can-type-into-an-application.md`).
const MULTI_LINE: u32 = 1 << 17;
const SINGLE_LINE: u32 = 1 << 26;

/// VSCodium's editor, as the tree shows it: an entry of many lines.
const AN_EDITOR: u32 = FOCUSED | MULTI_LINE;

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_applications::words::declare_into(&mut vocabulary).unwrap();
    alo_adapters::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

fn granting(applications: &[&str]) -> Grants {
    let mut grants = Grants::default();
    for application in applications {
        grants.grant(
            Grant::checked(
                "@alo",
                alo_capability::Reach::Application((*application).to_owned()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
    }
    grants
}

fn installed() -> Installed {
    Installed::holding([
        Application::called(EDITOR, "VSCodium").unwrap(),
        Application::called("org.example.Payroll", "Payroll").unwrap(),
    ])
}

fn typing_call(name: &str, text: &str) -> Call {
    fallback_verbs()
        .unwrap()
        .call(
            TYPE_TEXT,
            &[
                ("application", Given::text(EDITOR)),
                ("name", Given::text(name)),
                ("text", Given::text(text)),
            ],
        )
        .unwrap()
}

fn approved(call: &Call, grants: &Grants) -> Authorised {
    let mut approvals = Approvals::default();
    let id = approvals
        .propose(Proposal::checked(call, &Grantee::named("@alo"), grants, noon(), hour()).unwrap());
    approvals
        .approve(id, noon())
        .unwrap()
        .redeem(grants, noon())
        .unwrap()
}

fn typing(name: &str, text: &str) -> Typing {
    let grants = granting(&[EDITOR]);
    Typing::of(
        approved(&typing_call(name, text), &grants),
        &Adapters::default(),
        &grants,
        &installed(),
        &in_english(),
    )
    .expect("granted, installed, no adapter")
}

/// One thing in the stand-in tree.
struct Thing {
    role: u32,
    name: &'static str,
    states: u32,
    text: &'static str,
    takes_text: bool,
    class: &'static str,
}

fn a(role: u32, name: &'static str, states: u32) -> Thing {
    Thing {
        role,
        name,
        states,
        text: "",
        takes_text: false,
        class: "",
    }
}

/// A tree with one window titled `title` holding these things, for `EDITOR`.
struct ATree {
    facts: BTreeMap<NodeAt, Facts>,
    texts: RefCell<BTreeMap<NodeAt, String>>,
    takes: BTreeMap<NodeAt, bool>,
    classes: BTreeMap<NodeAt, String>,
    inserted: RefCell<usize>,
    /// What an insert answers, and whether it changes the text.
    insert_answers: bool,
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.5".to_owned(),
        object: object.to_owned(),
    }
}

impl ATree {
    fn of(title: &'static str, things: Vec<Thing>) -> Self {
        let mut tree = Self {
            facts: BTreeMap::new(),
            texts: RefCell::new(BTreeMap::new()),
            takes: BTreeMap::new(),
            classes: BTreeMap::new(),
            inserted: RefCell::new(0),
            insert_answers: true,
        };
        let children: Vec<NodeAt> = (0..things.len()).map(|n| at(&format!("/t/{n}"))).collect();
        tree.facts.insert(
            at("/root"),
            Facts {
                role: 75,
                name: String::new(),
                states: vec![1 << 25, 0],
                children: vec![at("/window")],
                has_text: false,
                unlisted_children: false,
            },
        );
        tree.facts.insert(
            at("/window"),
            Facts {
                role: FRAME,
                name: title.to_owned(),
                states: vec![1 << 25, 0],
                children: children.clone(),
                has_text: false,
                unlisted_children: false,
            },
        );
        for (thing, place) in things.into_iter().zip(children) {
            tree.facts.insert(
                place.clone(),
                Facts {
                    role: thing.role,
                    name: thing.name.to_owned(),
                    states: vec![thing.states, 0],
                    children: Vec::new(),
                    has_text: true,
                    unlisted_children: false,
                },
            );
            tree.texts
                .borrow_mut()
                .insert(place.clone(), thing.text.to_owned());
            tree.takes.insert(place.clone(), thing.takes_text);
            tree.classes.insert(place, thing.class.to_owned());
        }
        tree
    }
}

impl AccessibilityTree for ATree {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: at("/root"),
            is: Some(EDITOR.to_owned()),
        }])
    }

    fn facts(&self, place: &NodeAt) -> Result<Facts, TreeFault> {
        self.facts.get(place).cloned().ok_or(TreeFault::Vanished)
    }

    fn text(&self, place: &NodeAt) -> Result<String, TreeFault> {
        self.texts
            .borrow()
            .get(place)
            .cloned()
            .ok_or(TreeFault::Vanished)
    }

    fn actions(&self, _: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(Vec::new())
    }

    fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
        Ok(false)
    }

    fn attributes(&self, place: &NodeAt) -> Result<Vec<(String, String)>, TreeFault> {
        Ok(self
            .classes
            .get(place)
            .filter(|class| !class.is_empty())
            .map(|class| vec![("class".to_owned(), class.clone())])
            .unwrap_or_default())
    }

    fn takes_text(&self, place: &NodeAt) -> Result<bool, TreeFault> {
        Ok(self.takes.get(place).copied().unwrap_or(false))
    }

    fn insert_text(&self, place: &NodeAt, position: usize, text: &str) -> Result<bool, TreeFault> {
        *self.inserted.borrow_mut() += 1;
        if self.insert_answers {
            let mut texts = self.texts.borrow_mut();
            let held = texts.get_mut(place).ok_or(TreeFault::Vanished)?;
            let split: String = held.chars().take(position).collect();
            let rest: String = held.chars().skip(position).collect();
            *held = format!("{split}{text}{rest}");
        }
        Ok(self.insert_answers)
    }
}

fn said(refused: &Refused) -> String {
    refused.said(&in_english()).text().to_owned()
}

#[test]
fn the_verb_is_a_change_over_a_granted_application_and_takes_text() {
    let verbs = fallback_verbs().unwrap();
    let verb = verbs.of(TYPE_TEXT).unwrap();
    assert_eq!(verb.effect(), Effect::Change);
    let call = typing_call("index.html", "<p>\n\thi\n</p>\n");
    let texts = call.texts();
    assert_eq!(texts.len(), 1);
}

#[test]
fn the_named_editor_with_the_keyboard_is_found_and_says_it_does_not_take_text_directly() {
    let tree = ATree::of(
        "index.html - VSCodium",
        vec![a(ENTRY, "index.html", AN_EDITOR)],
    );
    let field = typing("index.html", "x")
        .the_field(&tree, &in_english())
        .unwrap();
    assert!(field.has_the_keyboard);
    assert!(!field.takes_text);
}

#[test]
fn an_unnamed_document_in_a_window_titled_with_the_name_is_found() {
    let mut document = a(TEXT, "", EDITABLE);
    document.takes_text = true;
    let tree = ATree::of("notes.txt - Text Editor", vec![document]);
    let field = typing("notes.txt", "x")
        .the_field(&tree, &in_english())
        .unwrap();
    assert!(field.takes_text);
    assert!(!field.has_the_keyboard);
}

/// **The field with the keyboard, in a window titled with the approved name,
/// is that document** whatever the editor calls the field itself — and one
/// without the keyboard, named otherwise, is not.
///
/// What would have to be true for this to pass while broken: VSCodium putting
/// the keyboard in a field of its window other than the editor — its search
/// box, say — which the read-back then catches: the text would not be in the
/// document, and nothing is saved.
#[test]
fn the_field_with_the_keyboard_in_a_window_titled_with_the_name_is_found() {
    let tree = ATree::of(
        "hello.txt - VSCodium",
        vec![a(ENTRY, "Editor content", AN_EDITOR)],
    );
    let field = typing("hello.txt", "x")
        .the_field(&tree, &in_english())
        .unwrap();
    assert!(field.has_the_keyboard);

    let tree = ATree::of(
        "hello.txt - VSCodium",
        vec![a(ENTRY, "Editor content", EDITABLE | MULTI_LINE)],
    );
    assert!(
        typing("hello.txt", "x")
            .the_field(&tree, &in_english())
            .is_err(),
        "a field without the keyboard, named otherwise, was taken for the document"
    );
}

#[test]
fn no_field_of_that_name_is_refused() {
    let tree = ATree::of("x", vec![a(ENTRY, "other.html", FOCUSED)]);
    let refused = typing("index.html", "x")
        .the_field(&tree, &in_english())
        .unwrap_err();
    assert!(said(&refused).contains("no field"), "{}", said(&refused));
}

/// **Of two fields of that name, the one with the keyboard is the one** —
/// alo's own copy of VSCodium beside the person's, both showing the same file.
///
/// What would have to be true for this to pass while broken: the keyboard
/// being in the person's window rather than alo's copy, which this cannot see
/// and does not need to: the compositor then answers `not-alos-copy` and
/// nothing is typed.
#[test]
fn of_two_fields_of_that_name_the_one_with_the_keyboard_is_found() {
    let tree = ATree::of(
        "x",
        vec![
            a(ENTRY, "index.html", EDITABLE | MULTI_LINE),
            a(ENTRY, "index.html", AN_EDITOR),
        ],
    );
    let field = typing("index.html", "x")
        .the_field(&tree, &in_english())
        .unwrap();
    assert!(field.has_the_keyboard);
    assert_eq!(
        field.at,
        at("/t/1"),
        "the field without the keyboard was chosen"
    );
}

#[test]
fn two_fields_of_that_name_are_refused_rather_than_guessed() {
    let tree = ATree::of(
        "x",
        vec![
            a(ENTRY, "index.html", EDITABLE),
            a(ENTRY, "index.html", EDITABLE),
        ],
    );
    let refused = typing("index.html", "x")
        .the_field(&tree, &in_english())
        .unwrap_err();
    assert!(
        said(&refused).contains("more than one"),
        "{}",
        said(&refused)
    );
}

#[test]
fn a_password_field_is_never_typed_into() {
    let tree = ATree::of("Sign in", vec![a(PASSWORD, "Password", FOCUSED)]);
    let refused = typing("Password", "hunter2")
        .the_field(&tree, &in_english())
        .unwrap_err();
    assert!(
        said(&refused).contains("password field"),
        "{}",
        said(&refused)
    );
}

#[test]
fn a_terminal_by_its_role_or_its_class_is_never_typed_into() {
    let mut web = a(ENTRY, "Terminal 1, sh", FOCUSED);
    web.class = "xterm-helper-textarea";
    for (tree, name) in [
        (ATree::of("x", vec![web]), "Terminal 1, sh"),
        (
            ATree::of("x", vec![a(TERMINAL, "Terminal", FOCUSED)]),
            "Terminal",
        ),
    ] {
        let refused = typing(name, "rm -rf ~")
            .the_field(&tree, &in_english())
            .unwrap_err();
        assert!(said(&refused).contains("terminal"), "{}", said(&refused));
    }
}

#[test]
fn a_field_that_cannot_be_edited_is_not_a_place_to_type() {
    let tree = ATree::of("x", vec![a(ENTRY, "index.html", 1 << 25)]);
    assert!(
        typing("index.html", "x")
            .the_field(&tree, &in_english())
            .is_err()
    );
}

#[test]
fn the_direct_road_puts_the_text_in_once_and_it_reads_back() {
    let mut document = a(TEXT, "", EDITABLE);
    document.takes_text = true;
    document.text = "end";
    let tree = ATree::of("notes.txt - Text Editor", vec![document]);
    let typing = typing("notes.txt", "fn é\n\t日 ");
    let strings = in_english();
    let field = typing.the_field(&tree, &strings).unwrap();
    typing.put_in(&tree, &field, &strings).unwrap();
    assert_eq!(*tree.inserted.borrow(), 1);
    typing.read_back(&tree, &field, &strings).unwrap();
}

#[test]
fn an_application_that_says_it_did_not_take_the_text_is_refused() {
    let mut document = a(TEXT, "", EDITABLE);
    document.takes_text = true;
    let mut tree = ATree::of("notes.txt - Text Editor", vec![document]);
    tree.insert_answers = false;
    let typing = typing("notes.txt", "x");
    let strings = in_english();
    let field = typing.the_field(&tree, &strings).unwrap();
    let refused = typing.put_in(&tree, &field, &strings).unwrap_err();
    assert!(
        said(&refused).contains("did not take"),
        "{}",
        said(&refused)
    );
}

#[test]
fn a_text_that_reads_back_differently_is_not_saved() {
    let mut editor = a(ENTRY, "index.html", AN_EDITOR);
    // What an editor that indents and closes brackets on its own shows.
    editor.text = "function f() {\n        x\n}}";
    let tree = ATree::of("index.html - VSCodium", vec![editor]);
    let typing = typing("index.html", "function f() {\n    x\n}");
    let strings = in_english();
    let field = typing.the_field(&tree, &strings).unwrap();
    let refused = typing.read_back(&tree, &field, &strings).unwrap_err();
    assert!(said(&refused).contains("not saved"), "{}", said(&refused));
}

/// **A name that means no field is refused with what is there** (the owner,
/// 2026-11-15, relayed: option D): the fields' own names, and the window's
/// title for one with none — each a name typing would take.
///
/// Every-try, set 5: after opening hello.txt in VSCodium, `gpt-4o-mini` named
/// the editor `Document` or `TextEditor` in about half its tries.
#[test]
fn a_name_that_means_no_field_is_refused_with_what_is_there() {
    let tree = ATree::of(
        "hello.txt - VSCodium",
        vec![a(TEXT, "", FOCUSED), a(ENTRY, "Search", EDITABLE)],
    );
    let refused = typing("Document", "x")
        .the_field(&tree, &in_english())
        .unwrap_err();
    let said = said(&refused);
    assert!(said.contains("“Document”"), "{said}");
    assert!(said.contains("“hello.txt - VSCodium”"), "{said}");
    // **And every name it says is one this typing takes**: typing that saves
    // does not take the search box, so it is not said; typing that does not
    // save takes both, and says both.
    assert!(
        !said.contains("“Search”"),
        "a one-line field is offered to typing that saves: {said}"
    );
    assert!(
        typing("hello.txt - VSCodium", "x")
            .the_field(&tree, &in_english())
            .is_ok()
    );
    let unsaved = typing_without_saving("Document", "x")
        .the_field(&tree, &in_english())
        .unwrap_err();
    let unsaved = self::said(&unsaved);
    assert!(unsaved.contains("“Search”"), "{unsaved}");
    assert!(unsaved.contains("“hello.txt - VSCodium”"), "{unsaved}");
    assert!(
        typing_without_saving("Search", "x")
            .the_field(&tree, &in_english())
            .is_ok()
    );
}

/// **The refusal case**: with nothing for text showing, nothing is listed and
/// the refusal is the one it always was.
#[test]
fn with_nothing_there_the_refusal_lists_nothing() {
    let tree = ATree::of("hello.txt - VSCodium", vec![a(FRAME, "", EDITABLE)]);
    let refused = typing("Document", "x")
        .the_field(&tree, &in_english())
        .unwrap_err();
    assert!(!said(&refused).contains("It shows"), "{}", said(&refused));
}

/// **After a file is opened, the place typing into it goes is the file's
/// name** (option A), and typing with that name reaches it.
#[test]
fn where_a_file_is_typed_into_is_its_name_and_typing_takes_it() {
    use alo_adapters::places_for_text::where_a_file_is_typed_into;
    let tree = ATree::of("hello.txt - VSCodium", vec![a(TEXT, "", FOCUSED)]);
    let (name, window) =
        where_a_file_is_typed_into(&tree, EDITOR, "hello.txt").expect("one place is meant");
    assert_eq!(name, "hello.txt");
    assert_eq!(window, "hello.txt - VSCodium");
    assert!(typing(&name, "x").the_field(&tree, &in_english()).is_ok());
}

/// **The refusal cases**: no window titled with the file, two editors and
/// neither with the keyboard, a password field, or another application — and
/// nothing is offered.
#[test]
fn where_no_one_place_is_meant_nothing_is_offered() {
    use alo_adapters::places_for_text::where_a_file_is_typed_into;
    let elsewhere = ATree::of("Welcome - VSCodium", vec![a(TEXT, "", FOCUSED)]);
    assert_eq!(
        where_a_file_is_typed_into(&elsewhere, EDITOR, "hello.txt"),
        None
    );
    let two = ATree::of(
        "hello.txt - VSCodium",
        vec![a(TEXT, "", EDITABLE), a(TEXT, "", EDITABLE)],
    );
    assert_eq!(where_a_file_is_typed_into(&two, EDITOR, "hello.txt"), None);
    let secret = ATree::of("hello.txt - VSCodium", vec![a(PASSWORD, "", FOCUSED)]);
    assert_eq!(
        where_a_file_is_typed_into(&secret, EDITOR, "hello.txt"),
        None
    );
    let tree = ATree::of("hello.txt - VSCodium", vec![a(TEXT, "", FOCUSED)]);
    assert_eq!(
        where_a_file_is_typed_into(&tree, "org.example.Payroll", "hello.txt"),
        None
    );
}

/// **A terminal with the keyboard is never offered or listed**: VSCodium's
/// built-in terminal is a web terminal by its class, and typing refuses it.
#[test]
fn a_terminal_with_the_keyboard_is_never_offered_or_listed() {
    use alo_adapters::places_for_text::where_a_file_is_typed_into;
    let web_terminal = Thing {
        class: "xterm-helper-textarea",
        ..a(ENTRY, "", FOCUSED)
    };
    let tree = ATree::of(
        "hello.txt - VSCodium",
        vec![
            web_terminal,
            a(TEXT, "", EDITABLE),
            a(PASSWORD, "Secret", EDITABLE),
        ],
    );
    assert_eq!(where_a_file_is_typed_into(&tree, EDITOR, "hello.txt"), None);
    let refused = typing("Document", "x")
        .the_field(&tree, &in_english())
        .unwrap_err();
    let said = said(&refused);
    assert!(
        !said.contains("Secret"),
        "a password field is listed: {said}"
    );
}

/// **Only the web terminal's own class marks a terminal**: an editor whose
/// class is anything else is still offered — and a field that is not showing
/// is never one.
#[test]
fn another_class_is_no_terminal_and_a_hidden_field_is_no_place() {
    use alo_adapters::places_for_text::where_a_file_is_typed_into;
    let an_editor = Thing {
        class: "monaco-editor",
        ..a(TEXT, "", FOCUSED)
    };
    let tree = ATree::of("hello.txt - VSCodium", vec![an_editor]);
    assert_eq!(
        where_a_file_is_typed_into(&tree, EDITOR, "hello.txt"),
        Some(("hello.txt".to_owned(), "hello.txt - VSCodium".to_owned()))
    );
    // Editable and with the keyboard, but not showing.
    let hidden = ATree::of(
        "hello.txt - VSCodium",
        vec![a(TEXT, "", (1 << 7) | (1 << 12))],
    );
    assert_eq!(
        where_a_file_is_typed_into(&hidden, EDITOR, "hello.txt"),
        None
    );
    // Showing, but no place for text at all.
    let a_label = ATree::of("hello.txt - VSCodium", vec![a(FRAME, "hello.txt", FOCUSED)]);
    assert_eq!(
        where_a_file_is_typed_into(&a_label, EDITOR, "hello.txt"),
        None
    );
}

/// **Typing without saving** (2026-11-17) takes `type_text`'s arguments and
/// checks, says on the card that nothing is saved, and is a typing that does
/// not save; `type_text` is one that does.
#[test]
fn typing_without_saving_takes_the_same_text_and_says_nothing_is_saved() {
    let verbs = fallback_verbs().unwrap();
    let verb = verbs.of(alo_adapters::TYPE_WITHOUT_SAVING).unwrap();
    assert_eq!(verb.effect(), Effect::Change);
    assert_eq!(
        verb.what_it_changes(),
        Some(&alo_adapters::typing_without_saving_words::UNSAVED_CHANGES)
    );
    assert_eq!(
        verb.what_it_leaves(),
        Some(&alo_adapters::typing_words::TYPE_LEAVES)
    );
    let call = verbs
        .call(
            alo_adapters::TYPE_WITHOUT_SAVING,
            &[
                ("application", Given::text(EDITOR)),
                ("field", Given::text("Search")),
                ("text", Given::text("alo OS")),
            ],
        )
        .unwrap();
    let grants = granting(&[EDITOR]);
    let unsaved = Typing::of(
        approved(&call, &grants),
        &Adapters::default(),
        &grants,
        &installed(),
        &in_english(),
    )
    .unwrap();
    assert!(!unsaved.saves());
    assert!(typing("index.html", "x").saves());
}

/// A typing of `text` into `name` that saves nothing.
fn typing_without_saving(name: &str, text: &str) -> Typing {
    let call = fallback_verbs()
        .unwrap()
        .call(
            alo_adapters::TYPE_WITHOUT_SAVING,
            &[
                ("application", Given::text(EDITOR)),
                ("field", Given::text(name)),
                ("text", Given::text(text)),
            ],
        )
        .unwrap();
    let grants = granting(&[EDITOR]);
    Typing::of(
        approved(&call, &grants),
        &Adapters::default(),
        &grants,
        &installed(),
        &in_english(),
    )
    .unwrap()
}

/// **Each typing's read-back says what became of it in its own words**: a
/// typing that saves says it was not saved, and one that saves nothing never
/// mentions saving — whether the text differs or cannot be read at all.
#[test]
fn each_typing_says_a_read_back_that_failed_in_its_own_words() {
    let strings = in_english();
    let filling =
        alo_strings::Filling::of("application", EDITOR.to_owned()).and("name", "Search".to_owned());
    let words = |word: alo_strings::Word| strings.say(&word.key(), &filling).text().to_owned();
    // Many lines, so typing that saves takes it as typing that does not does.
    let mut field = a(ENTRY, "Search", AN_EDITOR);
    field.text = "something else";
    let differs = ATree::of("Search - VSCodium", vec![field]);
    let unreadable = ATree::of("Search - VSCodium", vec![a(ENTRY, "Search", AN_EDITOR)]);
    unreadable.texts.borrow_mut().clear();
    for (tree, saving, word) in [
        (
            &differs,
            true,
            alo_adapters::typing_words::READ_BACK_DIFFERS,
        ),
        (
            &unreadable,
            true,
            alo_adapters::typing_words::CANNOT_READ_BACK,
        ),
        (
            &differs,
            false,
            alo_adapters::typing_without_saving_words::UNSAVED_READ_BACK_DIFFERS,
        ),
        (
            &unreadable,
            false,
            alo_adapters::typing_without_saving_words::UNSAVED_CANNOT_READ_BACK,
        ),
    ] {
        let typing = if saving {
            typing("Search", "alo OS")
        } else {
            typing_without_saving("Search", "alo OS")
        };
        let found = typing.the_field(tree, &strings).unwrap();
        let refused = typing.read_back(tree, &found, &strings).unwrap_err();
        assert_eq!(said(&refused), words(word), "saving: {saving}");
    }
}

/// What typing that saves into a one-line field is refused in, for `name`.
fn a_one_line_field_refused(name: &str) -> String {
    alo_adapters::a_one_line_field_said(EDITOR, name, &in_english())
        .text()
        .to_owned()
}

/// **Typing that saves is refused for a one-line field** (the owner,
/// 2026-11-20): a search box, an entry that says nothing about its lines, and
/// a `TEXT` that says `SINGLE_LINE` — in words that name the typing that does
/// not save, and with nothing put in. Typing that does not save still takes
/// each of them.
#[test]
fn typing_that_saves_is_refused_for_a_one_line_field_and_typing_without_saving_is_not() {
    for (role, states) in [
        (ENTRY, FOCUSED | SINGLE_LINE),
        (ENTRY, FOCUSED),
        (TEXT, FOCUSED | SINGLE_LINE),
    ] {
        let tree = ATree::of("Welcome - VSCodium", vec![a(role, "Search", states)]);
        let refused = typing("Search", "alo OS")
            .the_field(&tree, &in_english())
            .unwrap_err();
        let said = said(&refused);
        assert_eq!(said, a_one_line_field_refused("Search"), "{role}");
        assert!(said.contains("accessible.type_without_saving"), "{said}");
        assert!(said.contains("nothing was typed"), "{said}");
        assert_eq!(
            alo_adapters::a_one_line_field_to_save(&tree, EDITOR, "Search", &in_english())
                .map(|said| said.text().to_owned()),
            Some(said),
            "the card would be asked where the typing refuses: {role}"
        );
        assert!(
            typing_without_saving("Search", "alo OS")
                .the_field(&tree, &in_english())
                .is_ok(),
            "{role}"
        );
        assert_eq!(*tree.inserted.borrow(), 0, "{role}");
    }
}

/// **A document is typed into as it always was**: VSCodium's editor, an
/// `ENTRY` of many lines, and GNOME Text Editor's `TEXT` that says nothing
/// about its lines — and nothing is refused before the card for either.
#[test]
fn a_document_of_many_lines_is_still_typed_into_and_saved() {
    for (role, states) in [(ENTRY, AN_EDITOR), (TEXT, FOCUSED), (TEXT, AN_EDITOR)] {
        let tree = ATree::of("hello.txt - VSCodium", vec![a(role, "hello.txt", states)]);
        let typing = typing("hello.txt", "x");
        assert!(typing.saves());
        assert!(typing.the_field(&tree, &in_english()).is_ok(), "{role}");
        assert_eq!(
            alo_adapters::a_one_line_field_to_save(&tree, EDITOR, "hello.txt", &in_english()),
            None,
            "{role}"
        );
    }
}

/// **Before the card, only what was seen is refused**: a name meaning no
/// place, or two, a password field — refused in its own words when it runs —
/// and an application that is not there are each left to the typing.
#[test]
fn before_the_card_nothing_unseen_is_refused() {
    let search = || a(ENTRY, "Search", FOCUSED | SINGLE_LINE);
    let strings = in_english();
    let one = ATree::of("Welcome - VSCodium", vec![search()]);
    assert_eq!(
        alo_adapters::a_one_line_field_to_save(&one, EDITOR, "Address", &strings),
        None
    );
    assert_eq!(
        alo_adapters::a_one_line_field_to_save(&one, "org.example.Payroll", "Search", &strings),
        None
    );
    let two = ATree::of(
        "Welcome - VSCodium",
        vec![
            a(ENTRY, "Search", EDITABLE | SINGLE_LINE),
            a(ENTRY, "Search", EDITABLE | SINGLE_LINE),
        ],
    );
    assert_eq!(
        alo_adapters::a_one_line_field_to_save(&two, EDITOR, "Search", &strings),
        None
    );
    let secret = ATree::of(
        "Sign in",
        vec![a(PASSWORD, "Password", FOCUSED | SINGLE_LINE)],
    );
    assert_eq!(
        alo_adapters::a_one_line_field_to_save(&secret, EDITOR, "Password", &strings),
        None
    );
}
