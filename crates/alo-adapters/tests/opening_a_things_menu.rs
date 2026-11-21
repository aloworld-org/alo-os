//! **Opening one thing's right-click menu** (`accessible.open_menu`,
//! 2026-11-17), against a stand-in tree — every refusal beside what it refuses.
//!
//! The tree answers as Qt's does: a thing with a menu offers the action
//! `ShowMenu` beside its others. That other toolkits name it so is not
//! measured; a thing whose application offers no such action is refused in
//! words.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

use alo_adapters::fallback_verbs::{self, APPLICATION, OPEN_MENU, THING};
use alo_adapters::menu_words as words;
use alo_adapters::{AccessibilityTree, Facts, NodeAt, OpeningAMenu, Running, TreeFault};
use alo_applications::{Application, Installed};
use alo_capability::{
    Approvals, Authorised, Call, Effect, Given, Grant, Grantee, Grants, Proposal, Refused,
};
use alo_strings::{Filling, Strings, Vocabulary};

/// A Qt file manager.
const FILES: &str = "org.kde.dolphin";

/// `ATSPI_ROLE_FRAME`, `_LIST_ITEM`, `_PASSWORD_TEXT`.
const FRAME: u32 = 23;
const LIST_ITEM: u32 = 56;
const PASSWORD: u32 = 40;

/// On the screen; and not.
const ON: u32 = (1 << 25) | (1 << 30);
const OFF: u32 = 1 << 30;

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

fn granting() -> Grants {
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

fn menu_call(name: &str) -> Call {
    fallback_verbs::fallback_verbs()
        .unwrap()
        .call(
            OPEN_MENU,
            &[
                (APPLICATION, Given::text(FILES)),
                (THING, Given::text(name)),
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

fn opening(name: &str) -> OpeningAMenu {
    let grants = granting();
    OpeningAMenu::of(
        approved(&menu_call(name), &grants),
        &grants,
        &installed(),
        &in_english(),
    )
    .expect("granted, installed, no adapter")
}

/// One thing in the window: its role, name, states and actions.
type Thing = (u32, &'static str, u32, &'static [&'static str]);

/// A tree with one window holding these things, which records every action
/// done and answers each as `answers`.
struct ATree {
    facts: BTreeMap<NodeAt, Facts>,
    actions: BTreeMap<NodeAt, Vec<String>>,
    done: RefCell<Vec<(String, String)>>,
    answers: Result<bool, TreeFault>,
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.7".to_owned(),
        object: object.to_owned(),
    }
}

impl ATree {
    fn of(things: &[Thing]) -> Self {
        let children: Vec<NodeAt> = (0..things.len()).map(|n| at(&format!("/t/{n}"))).collect();
        let facts_of = |role, name: &str, states: u32, children| Facts {
            role,
            name: name.to_owned(),
            states: vec![states, 0],
            children,
            has_text: false,
            unlisted_children: false,
        };
        let mut facts = BTreeMap::from([
            (at("/root"), facts_of(75, "", ON, vec![at("/window")])),
            (at("/window"), facts_of(FRAME, "Home", ON, children.clone())),
        ]);
        let mut actions = BTreeMap::new();
        for ((role, name, states, offers), place) in things.iter().zip(children) {
            facts.insert(place.clone(), facts_of(*role, name, *states, Vec::new()));
            actions.insert(
                place,
                offers.iter().map(|action| (*action).to_owned()).collect(),
            );
        }
        Self {
            facts,
            actions,
            done: RefCell::new(Vec::new()),
            answers: Ok(true),
        }
    }

    fn done(&self) -> Vec<(String, String)> {
        self.done.borrow().clone()
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
        Ok(self.actions.get(place).cloned().unwrap_or_default())
    }

    fn act(&self, place: &NodeAt, action: usize) -> Result<bool, TreeFault> {
        let name = self.facts(place)?.name;
        let action = self
            .actions
            .get(place)
            .and_then(|actions| actions.get(action))
            .cloned()
            .unwrap_or_default();
        self.done.borrow_mut().push((name, action));
        self.answers
    }
}

fn said(refused: &Refused) -> String {
    refused.said(&in_english()).text().to_owned()
}

fn expected(word: alo_strings::Word, name: &str) -> String {
    in_english()
        .say(
            &word.key(),
            &Filling::of("application", FILES.to_owned()).and("name", name.to_owned()),
        )
        .text()
        .to_owned()
}

/// Two files in a list, each with a menu, and a third with none.
fn a_folder() -> Vec<Thing> {
    vec![
        (
            LIST_ITEM,
            "notes.txt",
            ON,
            &["Press", "ShowMenu", "SetFocus"][..],
        ),
        (LIST_ITEM, "report.pdf", ON, &["Press", "ShowMenu"][..]),
        (LIST_ITEM, "plain", ON, &["Press"][..]),
    ]
}

#[test]
fn the_verb_is_a_change_over_a_granted_application_taking_no_position() {
    let verbs = fallback_verbs::fallback_verbs().unwrap();
    let verb = verbs.of(OPEN_MENU).unwrap();
    assert_eq!(verb.effect(), Effect::Change);
    assert_eq!(verb.what_it_changes(), Some(&words::MENU_CHANGES));
    assert_eq!(verb.what_it_leaves(), Some(&words::MENU_LEAVES));
    let names: Vec<&str> = verb.args().iter().map(alo_capability::Arg::name).collect();
    assert_eq!(names, [APPLICATION, THING]);
}

#[test]
fn the_named_things_own_menu_action_is_done_once_and_nothing_else() {
    let tree = ATree::of(&a_folder());
    let opened = opening("notes.txt").open(&tree, &in_english()).unwrap();
    assert_eq!(
        tree.done(),
        [("notes.txt".to_owned(), "ShowMenu".to_owned())]
    );
    assert!(!opened.unanswered());
    assert_eq!(opened.application(), FILES);
}

#[test]
fn nothing_of_that_name_on_the_screen_is_refused_and_nothing_is_done() {
    let mut things = a_folder();
    things.push((LIST_ITEM, "below.txt", OFF, &["ShowMenu"][..]));
    let tree = ATree::of(&things);
    for name in ["missing.txt", "below.txt"] {
        let refused = opening(name).open(&tree, &in_english()).unwrap_err();
        assert_eq!(said(&refused), expected(words::NOTHING_NAMED, name));
    }
    assert!(tree.done().is_empty());
}

#[test]
fn a_thing_with_no_menu_is_refused_in_words() {
    let tree = ATree::of(&a_folder());
    let refused = opening("plain").open(&tree, &in_english()).unwrap_err();
    assert_eq!(said(&refused), expected(words::NO_MENU, "plain"));
    assert!(tree.done().is_empty());
}

#[test]
fn two_of_that_name_with_a_menu_are_refused_rather_than_guessed() {
    let mut things = a_folder();
    things.push((LIST_ITEM, "notes.txt", ON, &["ShowMenu"][..]));
    let tree = ATree::of(&things);
    let refused = opening("notes.txt").open(&tree, &in_english()).unwrap_err();
    assert_eq!(said(&refused), expected(words::MORE_THAN_ONE, "notes.txt"));
    assert!(tree.done().is_empty());
}

#[test]
fn a_password_field_is_never_acted_on() {
    let tree = ATree::of(&[(PASSWORD, "Password", ON, &["ShowMenu"][..])]);
    let refused = opening("Password").open(&tree, &in_english()).unwrap_err();
    assert_eq!(
        said(&refused),
        expected(words::A_PASSWORD_FIELD, "Password")
    );
    assert!(tree.done().is_empty());
}

#[test]
fn an_application_that_says_no_is_refused_and_one_that_does_not_answer_is_said() {
    let mut tree = ATree::of(&a_folder());
    tree.answers = Ok(false);
    let refused = opening("notes.txt").open(&tree, &in_english()).unwrap_err();
    assert_eq!(said(&refused), expected(words::DID_NOT_OPEN, "notes.txt"));

    let mut tree = ATree::of(&a_folder());
    tree.answers = Err(TreeFault::DidNotAnswer);
    let opened = opening("notes.txt").open(&tree, &in_english()).unwrap();
    assert!(opened.unanswered());
    assert_eq!(
        opened.said().unwrap().text(),
        expected(words::DID_NOT_ANSWER, "notes.txt")
    );
    assert_eq!(tree.done().len(), 1, "asked once");
}

#[test]
fn an_application_not_granted_is_refused_before_any_window_is_asked() {
    let authorised = approved(&menu_call("notes.txt"), &granting());
    assert!(
        OpeningAMenu::of(authorised, &Grants::default(), &installed(), &in_english(),).is_err()
    );
}

/// A window holding one thing inside another, `levels` deep, the deepest named
/// `name` and offering a menu.
fn a_chain(levels: usize, name: &str) -> ATree {
    let mut tree = ATree::of(&[]);
    let facts_of = |role, name: &str, children: Vec<NodeAt>| Facts {
        role,
        name: name.to_owned(),
        states: vec![ON, 0],
        children,
        has_text: false,
        unlisted_children: false,
    };
    let place = |level: usize| at(&format!("/chain/{level}"));
    if let Some(window) = tree.facts.get_mut(&at("/window")) {
        window.children = vec![place(1)];
    }
    for level in 1..=levels {
        let (role, named, children) = if level == levels {
            (LIST_ITEM, name, Vec::new())
        } else {
            (FRAME, "", vec![place(level + 1)])
        };
        tree.facts
            .insert(place(level), facts_of(role, named, children));
    }
    tree.actions
        .insert(place(levels), vec!["ShowMenu".to_owned()]);
    tree
}

#[test]
fn the_application_and_the_name_approved_are_the_ones_acted_on() {
    let opening = opening("notes.txt");
    assert_eq!(
        (opening.application(), opening.name()),
        (FILES, "notes.txt")
    );
    let opened = opening
        .open(&ATree::of(&a_folder()), &in_english())
        .unwrap();
    assert_eq!(opened.name(), "notes.txt");
}

/// **A window not wholly listed is never guessed in**, even with one thing of
/// that name found.
#[test]
fn a_window_not_wholly_listed_is_too_much_to_be_sure() {
    for object in ["/window", "/t/0"] {
        let mut tree = ATree::of(&a_folder());
        if let Some(facts) = tree.facts.get_mut(&at(object)) {
            facts.unlisted_children = true;
        }
        let refused = opening("notes.txt").open(&tree, &in_english()).unwrap_err();
        assert_eq!(
            said(&refused),
            expected(words::TOO_MUCH, "notes.txt"),
            "{object}"
        );
        assert!(tree.done().is_empty());
    }
}

/// **Past the bound, a name not found is not *nothing of that name*.**
#[test]
fn a_window_with_more_things_than_are_asked_about_is_too_much_to_be_sure() {
    let many: Vec<Thing> = (0..=alo_adapters::opening_a_menu::MOST_ASKED)
        .map(|_| (LIST_ITEM, "", ON, &[][..]))
        .collect();
    let tree = ATree::of(&many);
    let refused = opening("notes.txt").open(&tree, &in_english()).unwrap_err();
    assert_eq!(said(&refused), expected(words::TOO_MUCH, "notes.txt"));
}

/// **The window itself is never the thing**, even when its title is the name.
#[test]
fn a_window_titled_with_the_name_is_not_the_thing_named() {
    let tree = ATree::of(&a_folder());
    let refused = opening("Home").open(&tree, &in_english()).unwrap_err();
    assert_eq!(said(&refused), expected(words::NOTHING_NAMED, "Home"));
}

/// **As deep as a walk reads and no deeper.**
#[test]
fn a_thing_at_the_deepest_level_is_found_and_one_below_it_is_too_much_to_be_sure() {
    let deepest = alo_adapters::walking::DEEPEST;
    let tree = a_chain(deepest, "At the bottom");
    opening("At the bottom").open(&tree, &in_english()).unwrap();
    assert_eq!(tree.done().len(), 1);

    let tree = a_chain(deepest + 1, "Below it");
    let refused = opening("Below it").open(&tree, &in_english()).unwrap_err();
    assert_eq!(said(&refused), expected(words::TOO_MUCH, "Below it"));
    assert!(tree.done().is_empty());
}
