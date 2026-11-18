//! **Pressing one key of the closed list in an application's window**
//! (`accessible.press_key`, 2026-11-17), against a stand-in tree — every
//! refusal beside what it refuses.
//!
//! The adapter's part is the asking before any key goes: the grants, the
//! application installed with no adapter of its own, and the place with the
//! keyboard never a password field or a terminal. Delivering the key is the
//! compositor's, and `alo-turn`'s tests hold the order around both.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

use alo_acting::AKey;
use alo_adapters::fallback_verbs::{self, APPLICATION, KEY, PRESS_KEY};
use alo_adapters::keys_words as words;
use alo_adapters::{AccessibilityTree, Adapters, Facts, NodeAt, PressingAKey, Running, TreeFault};
use alo_applications::{Application, Installed};
use alo_capability::{
    Approvals, Authorised, Call, Effect, Given, Grant, Grantee, Grants, Proposal, Refused,
};
use alo_strings::{Filling, Strings, Vocabulary};

/// The application keys are pressed in.
const BROWSER: &str = "org.mozilla.firefox";

/// `ATSPI_ROLE_FRAME`, `_ENTRY`, `_PASSWORD_TEXT`, `_TERMINAL`, `_PUSH_BUTTON`.
const FRAME: u32 = 23;
const ENTRY: u32 = 79;
const PASSWORD: u32 = 40;
const TERMINAL: u32 = 60;
const BUTTON: u32 = 43;

/// Showing and editable; and with the keyboard.
const SHOWING: u32 = (1 << 25) | (1 << 7);
const FOCUSED: u32 = SHOWING | (1 << 12);

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
    Installed::holding([Application::called(BROWSER, "Firefox").unwrap()])
}

fn a_key_call(key: &str) -> Result<Call, alo_capability::CallError> {
    fallback_verbs::fallback_verbs().unwrap().call(
        PRESS_KEY,
        &[(APPLICATION, Given::text(BROWSER)), (KEY, Given::text(key))],
    )
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

fn pressing(key: &str) -> PressingAKey {
    let grants = granting(&[BROWSER]);
    PressingAKey::of(
        approved(&a_key_call(key).unwrap(), &grants),
        &Adapters::default(),
        &grants,
        &installed(),
        &in_english(),
    )
    .expect("granted, installed, no adapter")
}

/// A tree with one window holding things of these roles, names and states.
struct ATree {
    facts: BTreeMap<NodeAt, Facts>,
    classes: BTreeMap<NodeAt, String>,
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.9".to_owned(),
        object: object.to_owned(),
    }
}

impl ATree {
    fn of(things: &[(u32, &str, u32, &str)]) -> Self {
        let children: Vec<NodeAt> = (0..things.len()).map(|n| at(&format!("/t/{n}"))).collect();
        let facts_of = |role, name: &str, states: u32, children| Facts {
            role,
            name: name.to_owned(),
            states: vec![states, 0],
            children,
            has_text: true,
            unlisted_children: false,
        };
        let mut facts = BTreeMap::from([
            (at("/root"), facts_of(75, "", SHOWING, vec![at("/window")])),
            (
                at("/window"),
                facts_of(FRAME, "Mozilla Firefox", SHOWING, children.clone()),
            ),
        ]);
        let mut classes = BTreeMap::new();
        for ((role, name, states, class), place) in things.iter().zip(children) {
            facts.insert(place.clone(), facts_of(*role, name, *states, Vec::new()));
            classes.insert(place, (*class).to_owned());
        }
        Self { facts, classes }
    }
}

impl AccessibilityTree for ATree {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: at("/root"),
            is: Some(BROWSER.to_owned()),
        }])
    }

    fn facts(&self, place: &NodeAt) -> Result<Facts, TreeFault> {
        self.facts.get(place).cloned().ok_or(TreeFault::Vanished)
    }

    fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
        Ok(String::new())
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
}

fn said(refused: &Refused) -> String {
    refused.said(&in_english()).text().to_owned()
}

fn expected(word: alo_strings::Word) -> String {
    in_english()
        .say(&word.key(), &Filling::of("application", BROWSER.to_owned()))
        .text()
        .to_owned()
}

#[test]
fn the_verb_is_a_change_over_a_granted_application_offering_seven_keys_and_nothing_else() {
    let verbs = fallback_verbs::fallback_verbs().unwrap();
    let verb = verbs.of(PRESS_KEY).unwrap();
    assert_eq!(verb.effect(), Effect::Change);
    assert_eq!(verb.what_it_changes(), Some(&words::KEY_CHANGES));
    assert_eq!(verb.what_it_leaves(), Some(&words::KEY_LEAVES));
    for key in AKey::ALL {
        assert!(a_key_call(key.named()).is_ok(), "{key:?}");
    }
    for not_a_key in ["ctrl+s", "Enter", "alt+f4", "a", "page_down", "enter enter"] {
        assert!(a_key_call(not_a_key).is_err(), "{not_a_key} was taken");
    }
}

#[test]
fn the_key_approved_is_the_key_pressed() {
    assert_eq!(pressing("down").key(), AKey::Down);
    assert_eq!(pressing("enter").key(), AKey::Enter);
    assert_eq!(pressing("enter").application(), BROWSER);
}

#[test]
fn a_key_goes_where_an_ordinary_field_or_a_button_has_the_keyboard() {
    for tree in [
        ATree::of(&[(ENTRY, "Search", FOCUSED, "")]),
        ATree::of(&[
            (BUTTON, "Reload", FOCUSED, ""),
            (PASSWORD, "Password", SHOWING, ""),
        ]),
        ATree::of(&[]),
    ] {
        pressing("enter")
            .the_keyboard_is_safe(&tree, &in_english())
            .unwrap();
    }
}

#[test]
fn a_password_field_with_the_keyboard_is_refused() {
    let tree = ATree::of(&[(PASSWORD, "Password", FOCUSED, "")]);
    let refused = pressing("enter")
        .the_keyboard_is_safe(&tree, &in_english())
        .unwrap_err();
    assert_eq!(
        said(&refused),
        expected(words::A_PASSWORD_FIELD_HAS_THE_KEYBOARD)
    );
}

#[test]
fn a_terminal_with_the_keyboard_is_refused_by_its_role_or_its_class() {
    for tree in [
        ATree::of(&[(TERMINAL, "Terminal", FOCUSED, "")]),
        ATree::of(&[(ENTRY, "Terminal 1", FOCUSED, "xterm-helper-textarea")]),
    ] {
        let refused = pressing("enter")
            .the_keyboard_is_safe(&tree, &in_english())
            .unwrap_err();
        assert_eq!(said(&refused), expected(words::A_TERMINAL_HAS_THE_KEYBOARD));
    }
}

#[test]
fn an_application_not_granted_is_refused_before_any_window_is_asked() {
    let granted = granting(&[BROWSER]);
    let authorised = approved(&a_key_call("enter").unwrap(), &granted);
    assert!(
        PressingAKey::of(
            authorised,
            &Adapters::default(),
            &Grants::default(),
            &installed(),
            &in_english(),
        )
        .is_err()
    );
}
