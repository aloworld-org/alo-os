//! **Scrolling in an application's window** (`accessible.scroll_to` and
//! `accessible.scroll_page`, 2026-11-17), against a stand-in tree — every
//! refusal beside what it refuses.
//!
//! The tree here answers as list views were found to on 2026-11-16 (#1337): a
//! row scrolled away is described, visible and not showing. Whether every
//! toolkit says so is not measured; the read-back after a scroll is what says
//! when one does not.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

use alo_adapters::fallback_verbs::{self, APPLICATION, NAME, SCROLL_PAGE, SCROLL_TO, TOWARD};
use alo_adapters::scrolling_words as words;
use alo_adapters::{
    AccessibilityTree, Adapters, Edge, Facts, NodeAt, Running, Scrolling, TYPE_TEXT, TreeFault,
};
use alo_applications::{Application, Installed};
use alo_capability::{
    Approvals, Authorised, Call, Effect, Given, Grant, Grantee, Grants, Proposal, Refused,
};
use alo_strings::{Filling, Strings, Vocabulary};

/// The application scrolled in.
const READER: &str = "org.gnome.Papers";

/// `ATSPI_ROLE_FRAME`, `_PARAGRAPH`, `_DOCUMENT_FRAME`, `_LIST_ITEM`,
/// `_PASSWORD_TEXT`, `_HEADING`.
const FRAME: u32 = 23;
const PARAGRAPH: u32 = 73;
const DOCUMENT: u32 = 82;
const LIST_ITEM: u32 = 56;
const PASSWORD: u32 = 40;
const HEADING: u32 = 83;

/// On the screen; described but scrolled away; hidden altogether.
const ON: u32 = (1 << 25) | (1 << 30);
const OFF: u32 = 1 << 30;
const HIDDEN: u32 = 0;

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
    Installed::holding([Application::called(READER, "Papers").unwrap()])
}

fn to_call(name: &str) -> Call {
    fallback_verbs::fallback_verbs()
        .unwrap()
        .call(
            SCROLL_TO,
            &[
                (APPLICATION, Given::text(READER)),
                (NAME, Given::text(name)),
            ],
        )
        .unwrap()
}

fn page_call(toward: &str) -> Call {
    fallback_verbs::fallback_verbs()
        .unwrap()
        .call(
            SCROLL_PAGE,
            &[
                (APPLICATION, Given::text(READER)),
                (TOWARD, Given::text(toward)),
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

fn scrolling(call: &Call) -> Scrolling {
    let grants = granting(&[READER]);
    Scrolling::of(
        approved(call, &grants),
        &Adapters::default(),
        &grants,
        &installed(),
        &in_english(),
    )
    .expect("granted, installed, no adapter")
}

/// One thing in the stand-in tree, with the things inside it.
struct Thing {
    role: u32,
    name: &'static str,
    states: u32,
    inside: Vec<Thing>,
}

fn a(role: u32, name: &'static str, states: u32) -> Thing {
    Thing {
        role,
        name,
        states,
        inside: Vec::new(),
    }
}

fn holding(role: u32, name: &'static str, states: u32, inside: Vec<Thing>) -> Thing {
    Thing {
        role,
        name,
        states,
        inside,
    }
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.5".to_owned(),
        object: object.to_owned(),
    }
}

/// A tree with one window holding these things, for `is`.
struct ATree {
    is: &'static str,
    facts: RefCell<BTreeMap<NodeAt, Facts>>,
    /// Every scroll asked, in order.
    scrolled: RefCell<Vec<(String, Edge)>>,
    /// What a scroll answers.
    answers: bool,
    /// Whether a scroll the application said yes to puts the thing on the
    /// screen.
    moves: bool,
}

impl ATree {
    fn of(things: Vec<Thing>) -> Self {
        let tree = Self {
            is: READER,
            facts: RefCell::new(BTreeMap::new()),
            scrolled: RefCell::new(Vec::new()),
            answers: true,
            moves: true,
        };
        tree.put(
            "/root",
            &holding(75, "", ON, vec![holding(FRAME, "Papers", ON, things)]),
        );
        tree
    }

    fn put(&self, object: &str, thing: &Thing) {
        let children: Vec<NodeAt> = (0..thing.inside.len())
            .map(|n| at(&format!("{object}/{n}")))
            .collect();
        self.facts.borrow_mut().insert(
            at(object),
            Facts {
                role: thing.role,
                name: thing.name.to_owned(),
                states: vec![thing.states, 0],
                children: children.clone(),
                has_text: false,
                unlisted_children: false,
            },
        );
        for (inside, child) in thing.inside.iter().zip(children) {
            self.put(&child.object, inside);
        }
    }

    fn scrolled(&self) -> Vec<(String, Edge)> {
        self.scrolled.borrow().clone()
    }
}

impl AccessibilityTree for ATree {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: at("/root"),
            is: Some(self.is.to_owned()),
        }])
    }

    fn facts(&self, place: &NodeAt) -> Result<Facts, TreeFault> {
        self.facts
            .borrow()
            .get(place)
            .cloned()
            .ok_or(TreeFault::Vanished)
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

    fn scroll_to(&self, place: &NodeAt, edge: Edge) -> Result<bool, TreeFault> {
        let name = self.facts(place)?.name;
        self.scrolled.borrow_mut().push((name, edge));
        if self.answers && self.moves {
            let mut facts = self.facts.borrow_mut();
            let held = facts.get_mut(place).ok_or(TreeFault::Vanished)?;
            held.states = vec![ON, 0];
        }
        Ok(self.answers)
    }
}

fn said(refused: &Refused) -> String {
    refused.said(&in_english()).text().to_owned()
}

fn expected(word: alo_strings::Word, name: &str) -> String {
    in_english()
        .say(
            &word.key(),
            &Filling::of("application", READER.to_owned()).and("name", name.to_owned()),
        )
        .text()
        .to_owned()
}

/// A long document, its top on the screen.
fn a_long_document() -> Vec<Thing> {
    vec![holding(
        DOCUMENT,
        "report.pdf",
        ON,
        vec![
            a(HEADING, "Summary", ON),
            a(PARAGRAPH, "The first page", ON),
            a(HEADING, "Chapter 3", OFF),
            a(PARAGRAPH, "Further on", OFF),
        ],
    )]
}

#[test]
fn both_verbs_are_changes_over_a_granted_application_and_take_no_position() {
    let verbs = fallback_verbs::fallback_verbs().unwrap();
    for (verb, takes) in [
        (SCROLL_TO, [APPLICATION, NAME]),
        (SCROLL_PAGE, [APPLICATION, TOWARD]),
    ] {
        let verb = verbs.of(verb).unwrap();
        assert_eq!(verb.effect(), Effect::Change);
        assert!(verb.what_it_changes().is_some() && verb.what_it_leaves().is_some());
        let names: Vec<&str> = verb.args().iter().map(alo_capability::Arg::name).collect();
        assert_eq!(names, takes);
    }
    // **Down and up, and nothing else**: there is no distance to give.
    assert!(
        verbs
            .call(
                SCROLL_PAGE,
                &[
                    (APPLICATION, Given::text(READER)),
                    (TOWARD, Given::text("left")),
                ],
            )
            .is_err()
    );
}

#[test]
fn a_thing_off_the_screen_is_scrolled_to_once_and_reads_as_on_it_after() {
    let tree = ATree::of(a_long_document());
    let scrolled = scrolling(&to_call("Chapter 3"))
        .scroll(&tree, &in_english())
        .unwrap();
    assert_eq!(tree.scrolled(), [("Chapter 3".to_owned(), Edge::Top)]);
    assert!(scrolled.now_showing());
    assert!(scrolled.said().is_none());
    assert_eq!(scrolled.application(), READER);
}

#[test]
fn nothing_of_that_name_is_refused_and_nothing_is_scrolled() {
    let tree = ATree::of(a_long_document());
    let refused = scrolling(&to_call("Chapter 9"))
        .scroll(&tree, &in_english())
        .unwrap_err();
    assert_eq!(said(&refused), expected(words::NOTHING_NAMED, "Chapter 9"));
    assert!(tree.scrolled().is_empty());
}

#[test]
fn a_hidden_thing_is_not_found_even_by_its_exact_name() {
    let mut document = a_long_document();
    document[0].inside.push(a(HEADING, "Hidden tab", HIDDEN));
    let tree = ATree::of(document);
    let refused = scrolling(&to_call("Hidden tab"))
        .scroll(&tree, &in_english())
        .unwrap_err();
    assert_eq!(said(&refused), expected(words::NOTHING_NAMED, "Hidden tab"));
    assert!(tree.scrolled().is_empty());
}

#[test]
fn two_things_of_that_name_are_refused_rather_than_guessed() {
    let mut document = a_long_document();
    document[0].inside.push(a(HEADING, "Chapter 3", OFF));
    let tree = ATree::of(document);
    let refused = scrolling(&to_call("Chapter 3"))
        .scroll(&tree, &in_english())
        .unwrap_err();
    assert_eq!(said(&refused), expected(words::MORE_THAN_ONE, "Chapter 3"));
    assert!(tree.scrolled().is_empty());
}

#[test]
fn a_password_field_is_never_scrolled_to() {
    let mut document = a_long_document();
    document[0].inside.push(a(PASSWORD, "Password", OFF));
    let tree = ATree::of(document);
    let refused = scrolling(&to_call("Password"))
        .scroll(&tree, &in_english())
        .unwrap_err();
    assert_eq!(
        said(&refused),
        expected(words::A_PASSWORD_FIELD, "Password")
    );
    assert!(tree.scrolled().is_empty());
}

#[test]
fn an_application_that_says_no_is_refused_in_its_own_words() {
    let mut tree = ATree::of(a_long_document());
    tree.answers = false;
    let refused = scrolling(&to_call("Chapter 3"))
        .scroll(&tree, &in_english())
        .unwrap_err();
    assert_eq!(said(&refused), expected(words::DID_NOT_SCROLL, "Chapter 3"));
    assert_eq!(tree.scrolled().len(), 1, "asked once, and only once");
}

#[test]
fn a_yes_that_leaves_the_thing_off_the_screen_ran_and_says_so() {
    let mut tree = ATree::of(a_long_document());
    tree.moves = false;
    let scrolled = scrolling(&to_call("Chapter 3"))
        .scroll(&tree, &in_english())
        .unwrap();
    assert!(!scrolled.now_showing());
    assert_eq!(
        scrolled.said().unwrap().text(),
        expected(words::NOT_SEEN_TO_MOVE, "Chapter 3")
    );
}

#[test]
fn a_thing_already_on_the_screen_that_does_not_move_is_said_as_already_there() {
    let mut tree = ATree::of(a_long_document());
    tree.answers = false;
    let scrolled = scrolling(&to_call("Summary"))
        .scroll(&tree, &in_english())
        .unwrap();
    assert!(scrolled.now_showing());
    assert_eq!(
        scrolled.said().unwrap().text(),
        expected(words::ALREADY_SHOWING, "Summary")
    );
}

#[test]
fn a_page_down_brings_the_first_thing_below_the_screen_to_its_top() {
    let tree = ATree::of(a_long_document());
    let scrolled = scrolling(&page_call("down"))
        .scroll(&tree, &in_english())
        .unwrap();
    assert_eq!(tree.scrolled(), [("Chapter 3".to_owned(), Edge::Top)]);
    assert!(scrolled.now_showing());
}

#[test]
fn a_page_up_brings_the_last_thing_above_the_screen_to_its_bottom() {
    let tree = ATree::of(vec![holding(
        DOCUMENT,
        "report.pdf",
        ON,
        vec![
            a(HEADING, "Summary", OFF),
            a(PARAGRAPH, "The first page", OFF),
            a(HEADING, "Chapter 3", ON),
        ],
    )]);
    scrolling(&page_call("up"))
        .scroll(&tree, &in_english())
        .unwrap();
    assert_eq!(
        tree.scrolled(),
        [("The first page".to_owned(), Edge::Bottom)]
    );
}

#[test]
fn nothing_further_is_refused_and_nothing_is_scrolled() {
    let tree = ATree::of(a_long_document());
    let refused = scrolling(&page_call("up"))
        .scroll(&tree, &in_english())
        .unwrap_err();
    assert_eq!(said(&refused), expected(words::NOTHING_FURTHER_UP, ""));
    let all_on = ATree::of(vec![a(PARAGRAPH, "Short", ON)]);
    let refused = scrolling(&page_call("down"))
        .scroll(&all_on, &in_english())
        .unwrap_err();
    assert_eq!(said(&refused), expected(words::NOTHING_FURTHER_DOWN, ""));
    assert!(tree.scrolled().is_empty() && all_on.scrolled().is_empty());
}

#[test]
fn a_page_is_scrolled_in_the_document_when_one_is_showing() {
    // A side list scrolled away comes first in reading order; the page is the
    // document's.
    let mut things = vec![holding(
        FRAME,
        "Sidebar",
        ON,
        vec![a(LIST_ITEM, "First", ON), a(LIST_ITEM, "Second", OFF)],
    )];
    things.extend(a_long_document());
    let tree = ATree::of(things);
    scrolling(&page_call("down"))
        .scroll(&tree, &in_english())
        .unwrap();
    assert_eq!(tree.scrolled(), [("Chapter 3".to_owned(), Edge::Top)]);
}

#[test]
fn a_page_passes_over_a_password_field() {
    let tree = ATree::of(vec![holding(
        DOCUMENT,
        "Sign in",
        ON,
        vec![
            a(HEADING, "Welcome", ON),
            a(PASSWORD, "Password", OFF),
            a(PARAGRAPH, "Forgotten it?", OFF),
        ],
    )]);
    scrolling(&page_call("down"))
        .scroll(&tree, &in_english())
        .unwrap();
    assert_eq!(tree.scrolled(), [("Forgotten it?".to_owned(), Edge::Top)]);
}

#[test]
fn another_applications_window_is_never_scrolled() {
    let mut tree = ATree::of(a_long_document());
    tree.is = "org.example.Other";
    assert!(
        scrolling(&to_call("Chapter 3"))
            .scroll(&tree, &in_english())
            .is_err()
    );
    assert!(tree.scrolled().is_empty());
}

#[test]
fn an_application_not_granted_is_refused_before_any_window_is_asked() {
    let granted = granting(&[READER]);
    let authorised = approved(&to_call("Chapter 3"), &granted);
    assert!(
        Scrolling::of(
            authorised,
            &Adapters::default(),
            &Grants::default(),
            &installed(),
            &in_english(),
        )
        .is_err()
    );
}

#[test]
fn another_verbs_authority_does_not_scroll() {
    let grants = granting(&[READER]);
    let call = fallback_verbs::fallback_verbs()
        .unwrap()
        .call(
            TYPE_TEXT,
            &[
                (APPLICATION, Given::text(READER)),
                (NAME, Given::text("report.pdf")),
                ("text", Given::text("x")),
            ],
        )
        .unwrap();
    assert!(
        Scrolling::of(
            approved(&call, &grants),
            &Adapters::default(),
            &grants,
            &installed(),
            &in_english(),
        )
        .is_err()
    );
}

#[test]
fn what_scrolling_says_it_changes_is_what_it_did() {
    // The card says the window moves and nothing is typed, pressed or changed:
    // the only thing a scroll asks of the tree is the scroll.
    let tree = ATree::of(a_long_document());
    let before = tree.facts.borrow().len();
    scrolling(&to_call("Chapter 3"))
        .scroll(&tree, &in_english())
        .unwrap();
    assert_eq!(tree.facts.borrow().len(), before);
    assert_eq!(tree.scrolled().len(), 1);
    let verbs = fallback_verbs::fallback_verbs().unwrap();
    let verb = verbs.of(SCROLL_TO).unwrap();
    assert_eq!(verb.what_it_changes(), Some(&words::SCROLL_CHANGES));
    assert_eq!(verb.what_it_leaves(), Some(&words::SCROLL_LEAVES));
}
