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
        vec![a(ENTRY, "index.html", FOCUSED)],
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

#[test]
fn no_field_of_that_name_is_refused() {
    let tree = ATree::of("x", vec![a(ENTRY, "other.html", FOCUSED)]);
    let refused = typing("index.html", "x")
        .the_field(&tree, &in_english())
        .unwrap_err();
    assert!(said(&refused).contains("no field"), "{}", said(&refused));
}

#[test]
fn two_fields_of_that_name_are_refused_rather_than_guessed() {
    let tree = ATree::of(
        "x",
        vec![
            a(ENTRY, "index.html", FOCUSED),
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
    let mut editor = a(ENTRY, "index.html", FOCUSED);
    // What an editor that indents and closes brackets on its own shows.
    editor.text = "function f() {\n        x\n}}";
    let tree = ATree::of("index.html - VSCodium", vec![editor]);
    let typing = typing("index.html", "function f() {\n    x\n}");
    let strings = in_english();
    let field = typing.the_field(&tree, &strings).unwrap();
    let refused = typing.read_back(&tree, &field, &strings).unwrap_err();
    assert!(said(&refused).contains("not saved"), "{}", said(&refused));
}
