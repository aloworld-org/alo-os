//! **Moving a slider or a number box to a number** (`accessible.set_value`,
//! 2026-11-17), against a stand-in tree — every refusal beside what it
//! refuses.
//!
//! The tree is built from the published roles and AT-SPI's `Value`: a slider
//! from 0 to 100 in steps of 1, a number box from 1 to 99, and a progress bar.
//! **No real toolkit was measured for this file.**

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

use alo_adapters::{
    AccessibilityTree, Bounds, Facts, NodeAt, Running, SET_VALUE, Setting, States, TreeFault,
    fallback_verbs,
};
use alo_applications::{Application, Installed};
use alo_capability::{
    Approvals, Authorised, Call, Effect, Given, Grant, Grantee, Grants, Proposal, Refused,
};
use alo_strings::{Strings, Vocabulary};

/// The application moved in.
const MIXER: &str = "org.example.Mixer";

/// `ATSPI_ROLE_FRAME`, `_SLIDER`, `_SPIN_BUTTON`, `_PROGRESS_BAR`.
const FRAME: u32 = 23;
const SLIDER: u32 = 51;
const SPIN_BUTTON: u32 = 52;
const PROGRESS_BAR: u32 = 42;

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

fn granted() -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@alo",
            alo_capability::Reach::Application(MIXER.to_owned()),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    grants
}

fn a_call(name: &str, value: i64) -> Call {
    fallback_verbs()
        .unwrap()
        .call(
            SET_VALUE,
            &[
                ("application", Given::text(MIXER)),
                ("name", Given::text(name)),
                ("value", Given::number(value)),
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

fn setting(name: &str, value: i64) -> Setting {
    let grants = granted();
    Setting::of(
        approved(&a_call(name, value), &grants),
        &grants,
        &Installed::holding([Application::called(MIXER, "Mixer").unwrap()]),
        &in_english(),
    )
    .expect("granted, installed, no adapter")
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.5".to_owned(),
        object: object.to_owned(),
    }
}

/// A window *Mixer* with a slider *Volume*, a number box *Copies*, and a
/// progress bar *Download*.
struct ATree {
    facts: BTreeMap<NodeAt, Facts>,
    values: RefCell<BTreeMap<NodeAt, f64>>,
    bounds: BTreeMap<NodeAt, Bounds>,
    /// What a move answers, and what it leaves the value as.
    takes: Result<bool, TreeFault>,
    rounds_to: Option<f64>,
    /// Every move asked: where, and to what.
    moved: RefCell<Vec<(NodeAt, f64)>>,
}

impl ATree {
    fn of(things: &[(u32, &str)]) -> Self {
        let showing = States::showing_and_usable().as_the_tree_says().to_vec();
        let mut facts = BTreeMap::new();
        let places: Vec<String> = (0..things.len()).map(|n| format!("/t/{n}")).collect();
        let fact = |role, name: &str, children: Vec<NodeAt>| Facts {
            role,
            name: name.to_owned(),
            states: showing.clone(),
            children,
            has_text: false,
            unlisted_children: false,
        };
        facts.insert(at("/root"), fact(75, "", vec![at("/window")]));
        facts.insert(
            at("/window"),
            fact(
                FRAME,
                "Mixer",
                places.iter().map(|place| at(place)).collect(),
            ),
        );
        let mut values = BTreeMap::new();
        let mut bounds = BTreeMap::new();
        for ((role, name), place) in things.iter().zip(&places) {
            facts.insert(at(place), fact(*role, name, Vec::new()));
            values.insert(at(place), 30.0);
            let range = if *role == SPIN_BUTTON {
                Bounds {
                    least: 1.0,
                    most: 99.0,
                    step: 1.0,
                }
            } else {
                Bounds {
                    least: 0.0,
                    most: 100.0,
                    step: 1.0,
                }
            };
            bounds.insert(at(place), range);
        }
        Self {
            facts,
            values: RefCell::new(values),
            bounds,
            takes: Ok(true),
            rounds_to: None,
            moved: RefCell::new(Vec::new()),
        }
    }

    fn standard() -> Self {
        Self::of(&[
            (SLIDER, "Volume"),
            (SPIN_BUTTON, "Copies"),
            (PROGRESS_BAR, "Download"),
        ])
    }
}

impl AccessibilityTree for ATree {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: at("/root"),
            is: Some(MIXER.to_owned()),
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
        panic!("a slider was pressed rather than moved")
    }

    fn value(&self, place: &NodeAt) -> Result<Option<f64>, TreeFault> {
        Ok(self.values.borrow().get(place).copied())
    }

    fn range(&self, place: &NodeAt) -> Result<Option<Bounds>, TreeFault> {
        Ok(self.bounds.get(place).copied())
    }

    fn set_value(&self, place: &NodeAt, value: f64) -> Result<bool, TreeFault> {
        self.moved.borrow_mut().push((place.clone(), value));
        if self.takes == Ok(true) {
            self.values
                .borrow_mut()
                .insert(place.clone(), self.rounds_to.unwrap_or(value));
        }
        self.takes
    }
}

fn said(refused: &Refused) -> String {
    refused.said(&in_english()).text().to_owned()
}

#[test]
fn the_verb_is_a_change_over_a_granted_application_with_a_bounded_number() {
    let verbs = fallback_verbs().unwrap();
    let verb = verbs.of(SET_VALUE).unwrap();
    assert_eq!(verb.effect(), Effect::Change);
    assert_eq!(
        a_call("Volume", 40).sentence(&in_english()).text(),
        "move “Volume” in org.example.Mixer to 40"
    );
    // A number no slider could hold is refused by the declaration itself.
    assert!(
        verbs
            .call(
                SET_VALUE,
                &[
                    ("application", Given::text(MIXER)),
                    ("name", Given::text("Volume")),
                    ("value", Given::number(10_000_000_000)),
                ],
            )
            .is_err()
    );
}

/// **A slider is moved once, through its own number, and reads back.**
#[test]
fn a_slider_is_moved_once_and_reads_back() {
    let tree = ATree::standard();
    let moving = setting("Volume", 40);
    assert_eq!(moving.application(), MIXER);
    let set = moving.set(&tree, &in_english()).unwrap();
    assert_eq!(*tree.moved.borrow(), [(at("/t/0"), 40.0)]);
    assert_eq!(
        (set.application(), set.name(), set.value()),
        (MIXER, "Volume", 40)
    );
    assert_eq!(set.authorised().verb(), SET_VALUE);
    assert!(!set.unanswered());
    assert!(set.reads_back(&tree));
    assert!(set.said(true, &in_english()).is_none());
}

#[test]
fn a_number_box_is_moved_too() {
    let tree = ATree::standard();
    setting("Copies", 3).set(&tree, &in_english()).unwrap();
    assert_eq!(*tree.moved.borrow(), [(at("/t/1"), 3.0)]);
}

/// **A number outside the control's own range is refused, naming the range,
/// before anything moves.**
#[test]
fn a_number_outside_its_range_is_refused_before_anything_moves() {
    let tree = ATree::standard();
    let refused = setting("Copies", 0).set(&tree, &in_english()).unwrap_err();
    assert!(
        said(&refused).contains("goes from 1 to 99"),
        "{}",
        said(&refused)
    );
    let refused = setting("Volume", 101)
        .set(&tree, &in_english())
        .unwrap_err();
    assert!(
        said(&refused).contains("101 is outside it"),
        "{}",
        said(&refused)
    );
    assert!(tree.moved.borrow().is_empty());
}

#[test]
fn a_progress_bar_only_shows_a_number_and_is_never_moved() {
    let tree = ATree::standard();
    let refused = setting("Download", 50)
        .set(&tree, &in_english())
        .unwrap_err();
    assert!(
        said(&refused).contains("cannot be moved"),
        "{}",
        said(&refused)
    );
    assert!(tree.moved.borrow().is_empty());
}

#[test]
fn a_control_not_there_or_there_twice_is_refused() {
    let tree = ATree::standard();
    let refused = setting("Balance", 5).set(&tree, &in_english()).unwrap_err();
    assert!(
        said(&refused).contains("no slider or number box named “Balance”"),
        "{}",
        said(&refused)
    );
    let twice = ATree::of(&[(SLIDER, "Volume"), (SLIDER, "Volume")]);
    let refused = setting("Volume", 5).set(&twice, &in_english()).unwrap_err();
    assert!(
        said(&refused).contains("more than one"),
        "{}",
        said(&refused)
    );
    assert!(tree.moved.borrow().is_empty() && twice.moved.borrow().is_empty());
}

#[test]
fn a_control_that_says_no_range_cannot_be_moved() {
    let mut tree = ATree::standard();
    tree.bounds.clear();
    let refused = setting("Volume", 5).set(&tree, &in_english()).unwrap_err();
    assert!(
        said(&refused).contains("does not let “Volume” be moved"),
        "{}",
        said(&refused)
    );
    assert!(tree.moved.borrow().is_empty());
}

#[test]
fn a_greyed_out_slider_is_refused() {
    let mut tree = ATree::standard();
    tree.facts.get_mut(&at("/t/0")).unwrap().states = States::showing_and_usable()
        .greyed_out()
        .as_the_tree_says()
        .to_vec();
    let refused = setting("Volume", 5).set(&tree, &in_english()).unwrap_err();
    assert!(said(&refused).contains("greyed out"), "{}", said(&refused));
    assert!(tree.moved.borrow().is_empty());
}

#[test]
fn an_application_that_refuses_the_number_is_refused() {
    let mut tree = ATree::standard();
    tree.takes = Ok(false);
    let refused = setting("Volume", 5).set(&tree, &in_english()).unwrap_err();
    assert!(
        said(&refused).contains("did not take 5"),
        "{}",
        said(&refused)
    );
}

/// **A number that reads back as another is said plainly**, not claimed as
/// done; one within half a step is the number asked for.
#[test]
fn a_number_that_reads_back_differently_is_said_plainly() {
    let mut tree = ATree::standard();
    tree.rounds_to = Some(45.0);
    let set = setting("Volume", 40).set(&tree, &in_english()).unwrap();
    assert!(!set.reads_back(&tree));
    let told = set.said(false, &in_english()).unwrap();
    assert!(told.text().contains("shows a different number"), "{told}");
    // Within half a step either way, the far edge included, is the number;
    // past it, above or below, is not.
    for (read, seen) in [
        (40.4, true),
        (40.5, true),
        (39.5, true),
        (41.0, false),
        (35.0, false),
    ] {
        tree.rounds_to = Some(read);
        let set = setting("Volume", 40).set(&tree, &in_english()).unwrap();
        assert_eq!(set.reads_back(&tree), seen, "read back as {read}");
    }
}

/// **The ends of the range are inside it**: a number box from 1 to 99 takes
/// 1 and 99.
#[test]
fn the_ends_of_the_range_are_inside_it() {
    let tree = ATree::standard();
    setting("Copies", 1).set(&tree, &in_english()).unwrap();
    setting("Copies", 99).set(&tree, &in_english()).unwrap();
    assert_eq!(
        *tree.moved.borrow(),
        [(at("/t/1"), 1.0), (at("/t/1"), 99.0)]
    );
}

/// **An application that does not answer may have moved it**, and says so.
#[test]
fn an_application_that_does_not_answer_is_said_as_not_known() {
    let mut tree = ATree::standard();
    tree.takes = Err(TreeFault::DidNotAnswer);
    let set = setting("Volume", 40).set(&tree, &in_english()).unwrap();
    assert!(set.unanswered());
    let told = set.said(false, &in_english()).unwrap();
    assert!(told.text().contains("is not known"), "{told}");
}

/// **What the card says is what the tests above hold**: one control moved and
/// read back, and a number outside its range refused.
#[test]
fn the_card_says_what_moving_does_and_leaves() {
    let verbs = fallback_verbs().unwrap();
    let verb = verbs.of(SET_VALUE).unwrap();
    let strings = in_english();
    let say = |word: Option<&alo_strings::Word>| {
        strings
            .say(&word.unwrap().key(), &alo_strings::Filling::nothing())
            .text()
            .to_owned()
    };
    let changes = say(verb.what_it_changes());
    let leaves = say(verb.what_it_leaves());
    assert!(
        changes.contains("that one slider or number box"),
        "{changes}"
    );
    assert!(changes.contains("reads it back"), "{changes}");
    assert!(
        leaves.contains("outside the control's own range is refused"),
        "{leaves}"
    );
}

/// **An application nobody granted is refused before any tree is asked.**
#[test]
fn an_ungranted_application_is_refused_before_the_tree() {
    let grants = granted();
    let authorised = approved(&a_call("Volume", 5), &grants);
    let refused = Setting::of(
        authorised,
        &Grants::default(),
        &Installed::holding([Application::called(MIXER, "Mixer").unwrap()]),
        &in_english(),
    );
    assert!(refused.is_err(), "{refused:?}");
}
