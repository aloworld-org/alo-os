//! **Opening one item of a list** (`accessible.open_item`, 2026-11-17),
//! against a stand-in tree — every refusal beside what it refuses.
//!
//! The tree is a file manager's folder by the published roles: a list whose
//! items offer *activate*. **No real toolkit was measured for this file.**

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

use alo_adapters::{
    AccessibilityTree, Facts, NodeAt, OPEN_ITEM, Opening, Running, States, TreeFault,
    fallback_verbs,
};
use alo_applications::{Application, Installed};
use alo_capability::{
    Approvals, Authorised, Call, Effect, Given, Grant, Grantee, Grants, Proposal, Refused,
};
use alo_strings::{Strings, Vocabulary};

/// The application opened in.
const FILES: &str = "org.kde.dolphin";

/// `ATSPI_ROLE_FRAME`, `_LIST`, `_LIST_ITEM`.
const FRAME: u32 = 23;
const LIST: u32 = 31;
const LIST_ITEM: u32 = 32;

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
            alo_capability::Reach::Application(FILES.to_owned()),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    grants
}

fn installed() -> Installed {
    Installed::holding([Application::called(FILES, "Dolphin").unwrap()])
}

fn a_call(item: &str) -> Call {
    fallback_verbs()
        .unwrap()
        .call(
            OPEN_ITEM,
            &[
                ("application", Given::text(FILES)),
                ("name", Given::text(item)),
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

fn opening(item: &str) -> Opening {
    let grants = granted();
    Opening::of(
        approved(&a_call(item), &grants),
        &grants,
        &installed(),
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

/// A folder *Downloads* listing these items, each offering `actions`.
struct ATree {
    facts: BTreeMap<NodeAt, Facts>,
    actions: Vec<String>,
    answers: Result<bool, TreeFault>,
    acted: RefCell<Vec<(NodeAt, String)>>,
}

impl ATree {
    fn of(items: &[&str], actions: &[&str]) -> Self {
        let showing = States::showing_and_usable().as_the_tree_says().to_vec();
        let fact = |role, name: &str, children: Vec<NodeAt>| Facts {
            role,
            name: name.to_owned(),
            states: showing.clone(),
            children,
            has_text: false,
            unlisted_children: false,
        };
        let places: Vec<NodeAt> = (0..items.len()).map(|n| at(&format!("/i/{n}"))).collect();
        let mut facts = BTreeMap::from([
            (at("/root"), fact(75, "", vec![at("/window")])),
            (at("/window"), fact(FRAME, "Downloads", vec![at("/list")])),
            (at("/list"), fact(LIST, "", places.clone())),
        ]);
        for (item, place) in items.iter().zip(places) {
            facts.insert(place, fact(LIST_ITEM, item, Vec::new()));
        }
        Self {
            facts,
            actions: actions.iter().map(|action| (*action).to_owned()).collect(),
            answers: Ok(true),
            acted: RefCell::new(Vec::new()),
        }
    }
}

impl AccessibilityTree for ATree {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: at("/root"),
            is: Some(FILES.to_owned()),
        }])
    }

    fn facts(&self, place: &NodeAt) -> Result<Facts, TreeFault> {
        self.facts.get(place).cloned().ok_or(TreeFault::Vanished)
    }

    fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
        Ok(String::new())
    }

    fn actions(&self, place: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(if place.object.starts_with("/i/") {
            self.actions.clone()
        } else {
            Vec::new()
        })
    }

    fn act(&self, place: &NodeAt, action: usize) -> Result<bool, TreeFault> {
        self.acted
            .borrow_mut()
            .push((place.clone(), self.actions[action].clone()));
        self.answers
    }
}

fn said(refused: &Refused) -> String {
    refused.said(&in_english()).text().to_owned()
}

#[test]
fn the_verb_is_a_change_over_a_granted_application_and_its_card_says_what_may_start() {
    let verbs = fallback_verbs().unwrap();
    let verb = verbs.of(OPEN_ITEM).unwrap();
    assert_eq!(verb.effect(), Effect::Change);
    assert_eq!(
        a_call("report.pdf").sentence(&in_english()).text(),
        "open “report.pdf” in org.kde.dolphin"
    );
    let strings = in_english();
    let say = |word: Option<&alo_strings::Word>| {
        strings
            .say(&word.unwrap().key(), &alo_strings::Filling::nothing())
            .text()
            .to_owned()
    };
    let changes = say(verb.what_it_changes());
    assert!(changes.contains("Opens that one item"), "{changes}");
    assert!(changes.contains("a program may start"), "{changes}");
    assert!(say(verb.what_it_leaves()).contains("No other item is opened"));
}

/// **One approval opens once, by the item's own *activate*.**
#[test]
fn an_item_is_opened_once_by_its_own_activate() {
    let tree = ATree::of(&["notes.txt", "report.pdf"], &["select", "activate"]);
    let open = opening("report.pdf");
    assert_eq!(open.application(), FILES);
    let opened = open.open(&tree, &in_english()).unwrap();
    assert_eq!(*tree.acted.borrow(), [(at("/i/1"), "activate".to_owned())]);
    assert_eq!((opened.application(), opened.name()), (FILES, "report.pdf"));
    assert_eq!(opened.authorised().verb(), OPEN_ITEM);
    assert!(!opened.unanswered());
    assert!(opened.said(&in_english()).is_none());
}

/// **An item that offers only *select* or *click* is not opened by them**,
/// which in many toolkits only select; it is refused, and nothing is done.
#[test]
fn select_and_click_are_never_taken_for_opening() {
    let tree = ATree::of(&["report.pdf"], &["select", "click", "press"]);
    let refused = opening("report.pdf")
        .open(&tree, &in_english())
        .unwrap_err();
    assert!(
        said(&refused).contains("does not let “report.pdf” be opened"),
        "{}",
        said(&refused)
    );
    assert!(tree.acted.borrow().is_empty());
    // `open` and `invoke` are each an opening.
    for action in ["open", "invoke"] {
        let tree = ATree::of(&["report.pdf"], &[action]);
        opening("report.pdf").open(&tree, &in_english()).unwrap();
        assert_eq!(*tree.acted.borrow(), [(at("/i/0"), action.to_owned())]);
    }
}

#[test]
fn an_item_not_there_there_twice_or_greyed_out_is_refused() {
    let tree = ATree::of(&["a.txt", "a.txt"], &["activate"]);
    let refused = opening("a.txt").open(&tree, &in_english()).unwrap_err();
    assert!(
        said(&refused).contains("more than one “a.txt”"),
        "{}",
        said(&refused)
    );
    let refused = opening("b.txt").open(&tree, &in_english()).unwrap_err();
    assert!(
        said(&refused).contains("no item named “b.txt”"),
        "{}",
        said(&refused)
    );
    let mut greyed = ATree::of(&["a.txt"], &["activate"]);
    greyed.facts.get_mut(&at("/i/0")).unwrap().states = States::showing_and_usable()
        .greyed_out()
        .as_the_tree_says()
        .to_vec();
    let refused = opening("a.txt").open(&greyed, &in_english()).unwrap_err();
    assert!(said(&refused).contains("greyed out"), "{}", said(&refused));
    assert!(tree.acted.borrow().is_empty() && greyed.acted.borrow().is_empty());
}

#[test]
fn an_application_that_says_no_is_refused_and_one_that_does_not_answer_is_not_known() {
    let mut tree = ATree::of(&["a.txt"], &["activate"]);
    tree.answers = Ok(false);
    let refused = opening("a.txt").open(&tree, &in_english()).unwrap_err();
    assert!(
        said(&refused).contains("said it did not open"),
        "{}",
        said(&refused)
    );
    tree.answers = Err(TreeFault::DidNotAnswer);
    let opened = opening("a.txt").open(&tree, &in_english()).unwrap();
    assert!(opened.unanswered());
    assert!(
        opened
            .said(&in_english())
            .unwrap()
            .text()
            .contains("is not known")
    );
}

/// **An application nobody granted is refused before any tree is asked.**
#[test]
fn an_ungranted_application_is_refused_before_the_tree() {
    let grants = granted();
    let authorised = approved(&a_call("a.txt"), &grants);
    let refused = Opening::of(authorised, &Grants::default(), &installed(), &in_english());
    assert!(refused.is_err(), "{refused:?}");
}
