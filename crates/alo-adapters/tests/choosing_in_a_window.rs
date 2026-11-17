//! **Choosing an option of a drop-down and selecting an item of a list**
//! (`accessible.choose_option`, `accessible.select_item`, 2026-11-17), against
//! a stand-in tree — every refusal beside what it refuses.
//!
//! The tree is shaped as GTK and Qt describe a settings window, by the
//! published roles: a closed drop-down whose options are inside it and not on
//! the screen, and a list of files whose items can be selected. **No real
//! toolkit was measured for this file**; `docs/quirks/` says which were.

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
    AccessibilityTree, Adapters, CHOOSE_OPTION, Choosing, Facts, NodeAt, Running, SELECT_ITEM,
    States, TreeFault, fallback_verbs,
};
use alo_applications::{Application, Installed};
use alo_capability::{
    Approvals, Authorised, Call, Effect, Given, Grant, Grantee, Grants, Proposal, Refused,
};
use alo_strings::{Strings, Vocabulary};

/// The application chosen in.
const FILES: &str = "org.kde.dolphin";

/// `ATSPI_ROLE_FRAME`, `_COMBO_BOX`, `_MENU`, `_MENU_ITEM`, `_LIST`,
/// `_LIST_ITEM`, `_PASSWORD_TEXT`.
const FRAME: u32 = 23;
const COMBO_BOX: u32 = 11;
const MENU: u32 = 33;
const MENU_ITEM: u32 = 35;
const LIST: u32 = 31;
const LIST_ITEM: u32 = 32;
const PASSWORD: u32 = 40;

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

fn a_call(verb: &str, given: &[(&str, Given)]) -> Call {
    fallback_verbs().unwrap().call(verb, given).unwrap()
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

fn choosing_option(drop_down: &str, option: &str) -> Choosing {
    let grants = granted();
    let call = a_call(
        CHOOSE_OPTION,
        &[
            ("application", Given::text(FILES)),
            ("name", Given::text(drop_down)),
            ("option", Given::text(option)),
        ],
    );
    Choosing::of(
        approved(&call, &grants),
        &Adapters::default(),
        &grants,
        &installed(),
        &in_english(),
    )
    .expect("granted, installed, no adapter")
}

fn selecting(item: &str) -> Choosing {
    let grants = granted();
    let call = a_call(
        SELECT_ITEM,
        &[
            ("application", Given::text(FILES)),
            ("name", Given::text(item)),
        ],
    );
    Choosing::of(
        approved(&call, &grants),
        &Adapters::default(),
        &grants,
        &installed(),
        &in_english(),
    )
    .expect("granted, installed, no adapter")
}

fn showing() -> u32 {
    States::showing_and_usable().as_the_tree_says()[0]
}

fn out_of_sight() -> u32 {
    // Enabled, sensitive and visible, not showing: an option of a closed
    // drop-down.
    (1 << 8) | (1 << 24) | (1 << 30)
}

fn selectable(states: u32) -> u32 {
    states | (1 << 22)
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.5".to_owned(),
        object: object.to_owned(),
    }
}

/// A window with a drop-down *Sort by* holding three options, and a list of
/// two files.
struct ATree {
    facts: RefCell<BTreeMap<NodeAt, Facts>>,
    texts: BTreeMap<NodeAt, String>,
    /// Which lists offer a selection.
    lists_select: bool,
    /// What a selection answers.
    selection_answers: bool,
    /// What each thing's actions are.
    actions: BTreeMap<NodeAt, Vec<String>>,
    /// Every selection asked: the list and the thing.
    selected: RefCell<Vec<(NodeAt, NodeAt)>>,
    /// Every action done: the thing and the action's name.
    acted: RefCell<Vec<(NodeAt, String)>>,
    /// Whether choosing marks the thing selected, as an honest tree would.
    marks_it: bool,
    /// Whether the application does not answer a selection.
    does_not_answer: bool,
}

fn thing(role: u32, name: &str, states: u32, children: &[&str]) -> Facts {
    Facts {
        role,
        name: name.to_owned(),
        states: vec![states, 0],
        children: children.iter().map(|child| at(child)).collect(),
        has_text: false,
        unlisted_children: false,
    }
}

impl ATree {
    fn of(options: &[&str], items: &[&str]) -> Self {
        let mut facts = BTreeMap::new();
        facts.insert(at("/root"), thing(75, "", showing(), &["/window"]));
        facts.insert(
            at("/window"),
            thing(FRAME, "Documents", showing(), &["/sort", "/list", "/pin"]),
        );
        facts.insert(
            at("/sort"),
            thing(COMBO_BOX, "Sort by", showing(), &["/menu"]),
        );
        let option_places: Vec<String> = (0..options.len()).map(|n| format!("/o/{n}")).collect();
        let option_refs: Vec<&str> = option_places.iter().map(String::as_str).collect();
        facts.insert(at("/menu"), thing(MENU, "", out_of_sight(), &option_refs));
        for (option, place) in options.iter().zip(&option_places) {
            facts.insert(
                at(place),
                thing(MENU_ITEM, option, selectable(out_of_sight()), &[]),
            );
        }
        let item_places: Vec<String> = (0..items.len()).map(|n| format!("/i/{n}")).collect();
        let item_refs: Vec<&str> = item_places.iter().map(String::as_str).collect();
        facts.insert(at("/list"), thing(LIST, "", showing(), &item_refs));
        for (item, place) in items.iter().zip(&item_places) {
            facts.insert(
                at(place),
                thing(LIST_ITEM, item, selectable(showing()), &[]),
            );
        }
        facts.insert(at("/pin"), thing(PASSWORD, "Password", showing(), &[]));
        Self {
            facts: RefCell::new(facts),
            texts: BTreeMap::new(),
            lists_select: true,
            selection_answers: true,
            does_not_answer: false,
            actions: BTreeMap::new(),
            selected: RefCell::new(Vec::new()),
            acted: RefCell::new(Vec::new()),
            marks_it: true,
        }
    }

    fn mark(&self, place: &NodeAt) {
        if self.marks_it
            && let Some(facts) = self.facts.borrow_mut().get_mut(place)
        {
            facts.states[0] |= 1 << 23;
        }
    }

    fn greyed(self, place: &str) -> Self {
        if let Some(facts) = self.facts.borrow_mut().get_mut(&at(place)) {
            facts.states[0] &= !(1 << 24);
        }
        self
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
        self.facts
            .borrow()
            .get(place)
            .cloned()
            .ok_or(TreeFault::Vanished)
    }

    fn text(&self, place: &NodeAt) -> Result<String, TreeFault> {
        assert_ne!(place.object, "/pin", "a password field was asked its text");
        Ok(self.texts.get(place).cloned().unwrap_or_default())
    }

    fn actions(&self, place: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(self.actions.get(place).cloned().unwrap_or_default())
    }

    fn act(&self, place: &NodeAt, action: usize) -> Result<bool, TreeFault> {
        let name = self.actions[place][action].clone();
        self.acted.borrow_mut().push((place.clone(), name));
        self.mark(place);
        Ok(true)
    }

    fn select_in(&self, list: &NodeAt, place: &NodeAt) -> Result<Option<bool>, TreeFault> {
        if !self.lists_select {
            return Ok(None);
        }
        if self.does_not_answer {
            return Err(TreeFault::DidNotAnswer);
        }
        self.selected
            .borrow_mut()
            .push((list.clone(), place.clone()));
        if self.selection_answers {
            self.mark(place);
        }
        Ok(Some(self.selection_answers))
    }
}

fn said(refused: &Refused) -> String {
    refused.said(&in_english()).text().to_owned()
}

#[test]
fn both_verbs_are_changes_over_a_granted_application() {
    let verbs = fallback_verbs().unwrap();
    for verb in [CHOOSE_OPTION, SELECT_ITEM] {
        let verb = verbs.of(verb).unwrap();
        assert_eq!(verb.effect(), Effect::Change);
        assert!(matches!(
            verb.requires(),
            alo_capability::Requires::Grants(over) if over == &["application".to_owned()]
        ));
    }
    let sentence = a_call(
        CHOOSE_OPTION,
        &[
            ("application", Given::text(FILES)),
            ("name", Given::text("Sort by")),
            ("option", Given::text("Size")),
        ],
    )
    .sentence(&in_english());
    assert_eq!(
        sentence.text(),
        "choose “Size” in the drop-down “Sort by” in org.kde.dolphin"
    );
}

/// **An option of a closed drop-down is chosen through the list holding it,
/// once**, and read back as chosen.
#[test]
fn an_option_out_of_sight_is_chosen_through_its_list_once() {
    let tree = ATree::of(&["Name", "Size", "Date"], &["report.pdf"]);
    let choosing = choosing_option("Sort by", "Size");
    assert_eq!(choosing.application(), FILES);
    let chosen = choosing.choose(&tree, &in_english()).unwrap();
    assert_eq!(*tree.selected.borrow(), [(at("/menu"), at("/o/1"))]);
    assert!(tree.acted.borrow().is_empty());
    assert_eq!(chosen.application(), FILES);
    assert_eq!(chosen.authorised().verb(), CHOOSE_OPTION);
    assert!(!chosen.unanswered());
    assert!(chosen.shows_chosen(&tree));
    assert!(chosen.said(true, &in_english()).is_none());
}

/// **A drop-down that shows the option in its text reads as chosen — but only
/// where it has text to show**: words the tree keeps for a thing without text
/// are not read as what it shows.
#[test]
fn a_drop_down_showing_the_option_as_its_text_reads_as_chosen() {
    let mut tree = ATree::of(&["Name", "Size"], &[]);
    tree.marks_it = false;
    tree.texts.insert(at("/sort"), "Size".to_owned());
    let chosen = choosing_option("Sort by", "Size")
        .choose(&tree, &in_english())
        .unwrap();
    assert!(!chosen.shows_chosen(&tree), "a thing without text was read");
    tree.facts
        .borrow_mut()
        .get_mut(&at("/sort"))
        .unwrap()
        .has_text = true;
    assert!(chosen.shows_chosen(&tree));
    tree.texts.insert(at("/sort"), "Name".to_owned());
    assert!(
        !chosen.shows_chosen(&tree),
        "another option was read as this one"
    );
}

/// **An application that does not answer may have chosen**: nothing is
/// refused, and what alo says is that it is not known.
#[test]
fn an_application_that_does_not_answer_may_have_chosen() {
    let mut tree = ATree::of(&[], &["notes.txt"]);
    tree.does_not_answer = true;
    let chosen = selecting("notes.txt").choose(&tree, &in_english()).unwrap();
    assert!(chosen.unanswered());
    let told = chosen.said(false, &in_english()).unwrap();
    assert!(told.text().contains("is not known"), "{told}");
}

/// **Where the list offers no selection, the option's own action chooses
/// it** — and a drop-down that then shows the option reads as chosen.
#[test]
fn without_a_selection_the_options_own_action_chooses_it() {
    let mut tree = ATree::of(&["Name", "Size"], &[]);
    tree.lists_select = false;
    tree.marks_it = false;
    tree.actions.insert(at("/o/0"), vec!["click".to_owned()]);
    let chosen = choosing_option("Sort by", "Name")
        .choose(&tree, &in_english())
        .unwrap();
    assert_eq!(*tree.acted.borrow(), [(at("/o/0"), "click".to_owned())]);
    assert!(!chosen.shows_chosen(&tree), "nothing shows it chosen yet");
    let told = chosen.said(false, &in_english()).unwrap();
    assert!(told.text().contains("does not show it chosen"), "{told}");
    // The drop-down now shows the option by its name, as GTK's does.
    tree.facts.borrow_mut().get_mut(&at("/sort")).unwrap().name = "Name".to_owned();
    assert!(chosen.shows_chosen(&tree));
}

#[test]
fn an_option_the_drop_down_does_not_list_is_refused() {
    let tree = ATree::of(&["Name", "Size"], &[]);
    let refused = choosing_option("Sort by", "Colour")
        .choose(&tree, &in_english())
        .unwrap_err();
    assert!(
        said(&refused).contains("lists no option “Colour”"),
        "{}",
        said(&refused)
    );
    assert!(tree.selected.borrow().is_empty());
}

#[test]
fn an_option_listed_twice_is_refused_rather_than_guessed() {
    let tree = ATree::of(&["Size", "Size"], &[]);
    let refused = choosing_option("Sort by", "Size")
        .choose(&tree, &in_english())
        .unwrap_err();
    assert!(
        said(&refused).contains("more than once"),
        "{}",
        said(&refused)
    );
    assert!(tree.selected.borrow().is_empty());
}

#[test]
fn a_drop_down_not_on_the_screen_is_refused() {
    let tree = ATree::of(&["Size"], &[]);
    let refused = choosing_option("View", "Size")
        .choose(&tree, &in_english())
        .unwrap_err();
    assert!(
        said(&refused).contains("no drop-down named “View”"),
        "{}",
        said(&refused)
    );
}

#[test]
fn a_greyed_out_drop_down_is_refused() {
    let tree = ATree::of(&["Size"], &[]).greyed("/sort");
    let refused = choosing_option("Sort by", "Size")
        .choose(&tree, &in_english())
        .unwrap_err();
    assert!(said(&refused).contains("greyed out"), "{}", said(&refused));
    assert!(tree.selected.borrow().is_empty());
}

#[test]
fn an_application_that_says_no_is_refused() {
    let mut tree = ATree::of(&["Size"], &[]);
    tree.selection_answers = false;
    let refused = choosing_option("Sort by", "Size")
        .choose(&tree, &in_english())
        .unwrap_err();
    assert!(
        said(&refused).contains("said it did not choose"),
        "{}",
        said(&refused)
    );
}

/// **An item is selected through its list, once**, and reads back selected.
#[test]
fn an_item_is_selected_through_its_list_once() {
    let tree = ATree::of(&[], &["notes.txt", "report.pdf"]);
    let chosen = selecting("report.pdf")
        .choose(&tree, &in_english())
        .unwrap();
    assert_eq!(*tree.selected.borrow(), [(at("/list"), at("/i/1"))]);
    assert!(chosen.shows_chosen(&tree));
}

/// **Selecting never opens**: an item whose list offers no selection and
/// which offers only *activate* — what opens a file — is refused, and the
/// action is never done.
#[test]
fn selecting_never_opens_an_item() {
    let mut tree = ATree::of(&[], &["install.sh"]);
    tree.lists_select = false;
    tree.actions.insert(
        at("/i/0"),
        vec!["activate".to_owned(), "click".to_owned(), "open".to_owned()],
    );
    let refused = selecting("install.sh")
        .choose(&tree, &in_english())
        .unwrap_err();
    assert!(
        said(&refused).contains("does not let “install.sh” be chosen"),
        "{}",
        said(&refused)
    );
    assert!(tree.acted.borrow().is_empty(), "an item was opened");
    // An item that offers *select* is selected by it.
    tree.actions
        .insert(at("/i/0"), vec!["activate".to_owned(), "select".to_owned()]);
    selecting("install.sh")
        .choose(&tree, &in_english())
        .unwrap();
    assert_eq!(*tree.acted.borrow(), [(at("/i/0"), "select".to_owned())]);
}

#[test]
fn two_items_of_one_name_are_refused_and_a_missing_one_is_named() {
    let tree = ATree::of(&[], &["a.txt", "a.txt"]);
    let refused = selecting("a.txt").choose(&tree, &in_english()).unwrap_err();
    assert!(
        said(&refused).contains("more than one “a.txt”"),
        "{}",
        said(&refused)
    );
    let refused = selecting("b.txt").choose(&tree, &in_english()).unwrap_err();
    assert!(
        said(&refused).contains("no item named “b.txt”"),
        "{}",
        said(&refused)
    );
    assert!(tree.selected.borrow().is_empty());
}

/// **Looking inside a drop-down is bounded as reading a window is** — at most
/// `MOST_THINGS` things and `DEEPEST` levels — and past either bound, or where
/// the drop-down says it has more than it listed, nothing is chosen, because
/// *the only one* cannot be known.
#[test]
fn looking_inside_a_drop_down_is_bounded_and_past_a_bound_nothing_is_chosen() {
    use alo_adapters::walking::{DEEPEST, MOST_THINGS};
    let too_much = |tree: &ATree| {
        let refused = choosing_option("Sort by", "Deep")
            .choose(tree, &in_english())
            .unwrap_err();
        assert!(
            said(&refused).contains("whether “Sort by” is the only one"),
            "{}",
            said(&refused)
        );
        assert!(tree.selected.borrow().is_empty());
    };
    // An option `levels` deep inside the drop-down: the menu is one level in.
    let deep = |levels: usize| {
        let tree = ATree::of(&[], &[]);
        {
            let mut facts = tree.facts.borrow_mut();
            let mut parent = at("/menu");
            for level in 2..levels {
                let here = at(&format!("/d/{level}"));
                facts.get_mut(&parent).unwrap().children = vec![here.clone()];
                facts.insert(here.clone(), thing(MENU, "", out_of_sight(), &[]));
                parent = here;
            }
            facts.get_mut(&parent).unwrap().children = vec![at("/deep")];
            facts.insert(at("/deep"), thing(MENU_ITEM, "Deep", out_of_sight(), &[]));
        }
        tree
    };
    let reachable = deep(DEEPEST);
    choosing_option("Sort by", "Deep")
        .choose(&reachable, &in_english())
        .unwrap();
    too_much(&deep(DEEPEST + 1));
    // As many options as are read at once, counting the menu, and one more.
    let many = |count: usize| {
        let names: Vec<String> = (0..count).map(|n| format!("Option {n}")).collect();
        let mut names: Vec<&str> = names.iter().map(String::as_str).collect();
        names.push("Deep");
        let names: Vec<&'static str> = names
            .into_iter()
            .map(|name| &*Box::leak(name.to_owned().into_boxed_str()))
            .collect();
        ATree::of(&names, &[])
    };
    choosing_option("Sort by", "Deep")
        .choose(&many(MOST_THINGS - 2), &in_english())
        .unwrap();
    too_much(&many(MOST_THINGS - 1));
    // A menu that says it holds more than it listed.
    let unlisted = ATree::of(&["Deep"], &[]);
    unlisted
        .facts
        .borrow_mut()
        .get_mut(&at("/menu"))
        .unwrap()
        .unlisted_children = true;
    too_much(&unlisted);
}

/// **An option is never looked for outside its drop-down**: an item of the
/// list with the option's name is not chosen in its place.
#[test]
fn an_option_is_looked_for_inside_its_drop_down_alone() {
    let tree = ATree::of(&["Name"], &["Size"]);
    let refused = choosing_option("Sort by", "Size")
        .choose(&tree, &in_english())
        .unwrap_err();
    assert!(
        said(&refused).contains("lists no option"),
        "{}",
        said(&refused)
    );
    assert!(tree.selected.borrow().is_empty());
}

/// **What each card says is what the tests above hold**: one choice, looked
/// at afterwards (`an_option_out_of_sight_is_chosen_through_its_list_once`),
/// and an item selected and never opened (`selecting_never_opens_an_item`).
#[test]
fn each_card_says_what_the_choice_does_and_leaves() {
    let verbs = fallback_verbs().unwrap();
    let strings = in_english();
    let card = |verb: &str| {
        let verb = verbs.of(verb).unwrap();
        let said = |word: Option<&alo_strings::Word>| {
            strings
                .say(&word.unwrap().key(), &alo_strings::Filling::nothing())
                .text()
                .to_owned()
        };
        (said(verb.what_it_changes()), said(verb.what_it_leaves()))
    };
    let (changes, leaves) = card(CHOOSE_OPTION);
    assert!(
        changes.contains("that one option of that one drop-down"),
        "{changes}"
    );
    assert!(changes.contains("looks to see"), "{changes}");
    assert!(leaves.contains("Nothing is typed"), "{leaves}");
    let (changes, leaves) = card(SELECT_ITEM);
    assert!(changes.contains("that one item"), "{changes}");
    assert!(leaves.contains("not opened"), "{leaves}");
}

/// **An application nobody granted is refused before any tree is asked.**
#[test]
fn an_ungranted_application_is_refused_before_the_tree() {
    let grants = granted();
    let call = a_call(
        SELECT_ITEM,
        &[
            ("application", Given::text(FILES)),
            ("name", Given::text("notes.txt")),
        ],
    );
    let authorised = approved(&call, &grants);
    let refused = Choosing::of(
        authorised,
        &Adapters::default(),
        &Grants::default(),
        &installed(),
        &in_english(),
    );
    assert!(refused.is_err(), "{refused:?}");
}
