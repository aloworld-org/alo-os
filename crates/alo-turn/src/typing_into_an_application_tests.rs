//! **`accessible.type_text` in a turn** (ADR 0153): one approval types one text
//! once, through the road the field offers, and saves only what reads back —
//! with every refusal beside what it refuses.
//!
//! The tree and the door here share one field's text, so a door that "types"
//! changes what the tree reads back, and a test can make an editor that
//! rewrites what it is given.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use alo_acting::{ToType, Typed};
use alo_adapters::{AccessibilityTree, Facts, NodeAt, Running, TYPE_TEXT, TreeFault, WorkingFully};
use alo_applications::{Application, Installed};
use alo_capability::{Given, Grant, Grants, Reach};
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_record::{Asking, Record};
use alo_software::Bound;

use crate::Turning;
use crate::how_an_application_starts::{NoRoad, SaysHowAnApplicationStarts, TheRoad};
use crate::machine::{Machine, WhatASoftwareVerbNeeds, WhatAnApplicationVerbNeeds};
use crate::refusing::NotDone;
use crate::testing::{hour, in_english, noon, offering};
use crate::the_door_the_compositor_serves::{NotAtTheDoor, TheDoorTheCompositorServes};
use crate::the_windows_on_the_screen::TheWindowsOnTheScreen;

/// The editor typed into.
const EDITOR: &str = "com.vscodium.codium";

/// The agent.
const AGENT: &str = "@typing";

/// What alo is approved to type.
const THE_TEXT: &str = "<p>\n\thello, 日本\n</p>\n";

/// Showing and editable; and with the keyboard.
const EDITABLE: u32 = (1 << 25) | (1 << 7);
const FOCUSED: u32 = EDITABLE | (1 << 12);

/// How the field is, as a test arranges it.
#[derive(Clone)]
struct TheField {
    role: u32,
    states: u32,
    takes_text: bool,
    /// What the field holds — shared with the door.
    text: Rc<RefCell<String>>,
    opened: Rc<Cell<usize>>,
}

impl TheField {
    fn an_editor() -> Self {
        Self {
            role: 79,
            states: FOCUSED,
            takes_text: false,
            text: Rc::new(RefCell::new(String::new())),
            opened: Rc::new(Cell::new(0)),
        }
    }
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.7".to_owned(),
        object: object.to_owned(),
    }
}

/// The tree: an application, one window, one field named `index.html`.
struct ATree(TheField);

impl AccessibilityTree for ATree {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: at("/root"),
            is: Some(EDITOR.to_owned()),
        }])
    }

    fn facts(&self, place: &NodeAt) -> Result<Facts, TreeFault> {
        let facts = |role, name: &str, states, children| Facts {
            role,
            name: name.to_owned(),
            states: vec![states, 0],
            children,
            has_text: true,
            unlisted_children: false,
        };
        match place.object.as_str() {
            "/root" => Ok(facts(75, "", 1 << 25, vec![at("/window")])),
            "/window" => Ok(facts(
                23,
                "index.html - VSCodium",
                1 << 25,
                vec![at("/field")],
            )),
            "/field" => Ok(facts(self.0.role, "index.html", self.0.states, Vec::new())),
            _ => Err(TreeFault::Vanished),
        }
    }

    fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
        Ok(self.0.text.borrow().clone())
    }

    fn actions(&self, _: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(Vec::new())
    }

    fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
        Ok(false)
    }

    fn takes_text(&self, _: &NodeAt) -> Result<bool, TreeFault> {
        Ok(self.0.takes_text)
    }

    fn insert_text(&self, _: &NodeAt, position: usize, text: &str) -> Result<bool, TreeFault> {
        let mut held = self.0.text.borrow_mut();
        let before: String = held.chars().take(position).collect();
        let after: String = held.chars().skip(position).collect();
        *held = format!("{before}{text}{after}");
        Ok(true)
    }
}

struct TheWindows(TheField);

impl TheWindowsOnTheScreen for TheWindows {
    fn opening(&mut self) -> Result<Box<dyn AccessibilityTree>, TreeFault> {
        self.0.opened.set(self.0.opened.get() + 1);
        Ok(Box::new(ATree(self.0.clone())))
    }

    fn working_fully(&self, _: &str) -> WorkingFully {
        WorkingFully::NothingMoreToTurnOn
    }
}

/// The compositor's door, as a test arranges it.
struct ADoor {
    /// What it answers each kind of asking.
    whose: Typed,
    /// What a typing puts into the field: the text, or what an editor that
    /// rewrites it makes of it.
    rewriting: fn(&str) -> String,
    field: Rc<RefCell<String>>,
    asked: Rc<RefCell<Vec<String>>>,
}

impl TheDoorTheCompositorServes for ADoor {
    fn asking(&mut self, _: &alo_acting::Request) -> Result<alo_acting::Answer, NotAtTheDoor> {
        Err(NotAtTheDoor::NoCompositor)
    }

    fn typing(&mut self, asked: &ToType, text: &str) -> Result<Typed, NotAtTheDoor> {
        self.asked.borrow_mut().push(asked.written());
        if self.whose != Typed::MayType {
            return Ok(self.whose);
        }
        Ok(match asked {
            ToType::Check { .. } => Typed::MayType,
            ToType::Text { .. } => {
                self.field.borrow_mut().push_str(&(self.rewriting)(text));
                Typed::Typed
            }
            ToType::Save { .. } => Typed::AskedToSave,
        })
    }
}

struct NoTool;

impl alo_software::Tool for NoTool {
    fn sources(&self) -> Result<Vec<alo_software::Configured>, alo_software::Failed> {
        Ok(Vec::new())
    }
    fn installed(&self) -> Result<Vec<String>, alo_software::Failed> {
        Ok(Vec::new())
    }
    fn open(&self) -> Result<Vec<String>, alo_software::Failed> {
        Ok(Vec::new())
    }
    fn install(
        &self,
        _: &alo_software::SourceName,
        _: &Application,
    ) -> Result<(), alo_software::Failed> {
        Ok(())
    }
    fn updates(&self, _: &alo_software::SourceName) -> Result<Vec<String>, alo_software::Failed> {
        Ok(Vec::new())
    }
    fn update(&self, _: &Application) -> Result<(), alo_software::Failed> {
        Ok(())
    }
    fn remove(&self, _: &Application) -> Result<(), alo_software::Failed> {
        Ok(())
    }
}

struct NothingStarts;

impl SaysHowAnApplicationStarts for NothingStarts {
    fn how_it_starts(&self, _: &str) -> Result<TheRoad, NoRoad> {
        Err(NoRoad::NoEntry)
    }
}

/// What a turn answered, what the door was asked, and the record.
struct Ran {
    answered: Result<Option<crate::WhatAVerbDid>, NotDone>,
    asked: Vec<String>,
    entries: usize,
    opened: usize,
}

/// Propose typing `THE_TEXT` into `index.html`, and approve it once — or
/// decline it.
fn typing_once(field: TheField, whose: Typed, rewriting: fn(&str) -> String, approve: bool) -> Ran {
    let strings = in_english();
    let tool = NoTool;
    let publisher = crate::testing::APublisher::that_publishes();
    let asked = Rc::new(RefCell::new(Vec::new()));
    let mut door = ADoor {
        whose,
        rewriting,
        field: field.text.clone(),
        asked: asked.clone(),
    };
    let opened = field.opened.clone();
    let mut windows = TheWindows(field);
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let mut bounding = crate::testing::NothingIsBounded;
    let folder = crate::testing::a_folder_of_our_own("typing");
    let there = folder.join("a-note.txt");
    std::fs::write(&there, "anything").unwrap();
    let answered = {
        let mut machine = Machine::carrying_out_file_software_and_application_verbs(
            &strings,
            &OnThisMachine,
            WhatASoftwareVerbNeeds {
                tool: &tool,
                bound: Bound::Nobodys,
                publishing: crate::testing::publishing_through(&publisher),
            },
            WhatAnApplicationVerbNeeds {
                installed: Installed::holding([Application::identified(EDITOR).unwrap()]).into(),
                door: &mut door,
                starting: &NothingStarts,
                windows: &mut windows,
            },
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(AGENT, Reach::Application(EDITOR.to_owned()), noon(), hour()).unwrap(),
        );
        let mut turning =
            Turning::beginning(offering(&there), AGENT, hour(), &mut grants, &mut machine).unwrap();
        let id = turning
            .proposing(
                TYPE_TEXT,
                &[
                    ("application", Given::text(EDITOR)),
                    ("name", Given::text("index.html")),
                    ("text", Given::text(THE_TEXT)),
                ],
                &grants,
                hour(),
                noon(),
            )
            .unwrap();
        assert_eq!(opened.get(), 0, "a proposal opened the tree");
        assert!(asked.borrow().is_empty(), "a proposal asked the compositor");
        if approve {
            turning.approving(id, &grants, noon()).map(Some)
        } else {
            turning.declining(id, noon()).map(|()| None)
        }
    };
    let entries = record.answering(&Asking::anything()).count();
    let asked = asked.borrow().clone();
    Ran {
        answered,
        asked,
        entries,
        opened: opened.get(),
    }
}

fn as_it_is(text: &str) -> String {
    text.to_owned()
}

/// What VSCodium did with its own settings on 2026-11-13: indented again.
fn rewritten(text: &str) -> String {
    text.replace('\t', "    ")
}

#[test]
fn one_approval_types_once_by_keys_reads_back_and_saves() {
    let ran = typing_once(TheField::an_editor(), Typed::MayType, as_it_is, true);
    let did = ran.answered.unwrap().expect("approved");
    let typed = did.typed().expect("a text was typed");
    assert!(typed.saved());
    assert!(!typed.through_the_tree());
    assert_eq!(typed.characters(), THE_TEXT.chars().count());
    assert_eq!(
        ran.asked,
        [
            format!("type-into {EDITOR} {}", THE_TEXT.len()),
            format!("save-in {EDITOR}"),
        ]
    );
    assert_eq!(ran.opened, 1);
    assert_eq!(ran.entries, 1);
}

#[test]
fn a_field_that_takes_text_is_typed_through_the_tree_after_the_compositor_says_whose_it_is() {
    let mut field = TheField::an_editor();
    field.takes_text = true;
    field.states = EDITABLE;
    let text = field.text.clone();
    let ran = typing_once(field, Typed::MayType, as_it_is, true);
    let typed = ran.answered.unwrap().expect("approved");
    let typed = typed.typed().expect("a text was typed");
    assert!(typed.saved() && typed.through_the_tree());
    assert_eq!(*text.borrow(), THE_TEXT);
    assert_eq!(
        ran.asked,
        [
            format!("may-type-into {EDITOR}"),
            format!("save-in {EDITOR}")
        ]
    );
}

#[test]
fn the_persons_own_window_at_the_default_is_refused_and_nothing_is_saved() {
    let ran = typing_once(TheField::an_editor(), Typed::NotAlosCopy, as_it_is, true);
    assert!(matches!(ran.answered, Err(NotDone::Refused(_))));
    assert_eq!(
        ran.asked.len(),
        1,
        "asked once, never to save: {:?}",
        ran.asked
    );
    assert_eq!(ran.entries, 1, "the refusal is written down");
}

#[test]
fn a_text_the_editor_changed_is_not_saved_and_the_typing_is_recorded_as_run() {
    let ran = typing_once(TheField::an_editor(), Typed::MayType, rewritten, true);
    let did = ran.answered.unwrap().expect("approved");
    let typed = did.typed().expect("a text was typed");
    assert!(!typed.saved());
    assert_eq!(ran.asked.len(), 1, "never asked to save: {:?}", ran.asked);
    assert_eq!(ran.entries, 1);
}

#[test]
fn a_password_field_is_refused_before_the_compositor_is_asked() {
    let mut field = TheField::an_editor();
    field.role = 40;
    let ran = typing_once(field, Typed::MayType, as_it_is, true);
    assert!(matches!(ran.answered, Err(NotDone::Refused(_))));
    assert!(ran.asked.is_empty());
    assert_eq!(ran.entries, 1);
}

#[test]
fn a_terminal_is_refused_before_the_compositor_is_asked() {
    let mut field = TheField::an_editor();
    field.role = 60;
    let ran = typing_once(field, Typed::MayType, as_it_is, true);
    assert!(matches!(ran.answered, Err(NotDone::Refused(_))));
    assert!(ran.asked.is_empty());
}

#[test]
fn keys_are_never_sent_to_a_field_without_the_keyboard() {
    let mut field = TheField::an_editor();
    field.states = EDITABLE;
    let ran = typing_once(field, Typed::MayType, as_it_is, true);
    assert!(matches!(ran.answered, Err(NotDone::Refused(_))));
    assert!(ran.asked.is_empty());
    assert_eq!(ran.entries, 1);
}

#[test]
fn a_declined_typing_opens_nothing_and_asks_nothing() {
    let ran = typing_once(TheField::an_editor(), Typed::MayType, as_it_is, false);
    assert!(ran.answered.is_ok());
    assert_eq!(ran.opened, 0);
    assert!(ran.asked.is_empty());
    assert_eq!(ran.entries, 1);
}

#[test]
fn the_compositor_not_there_types_nothing() {
    let ran = typing_once(TheField::an_editor(), Typed::NoKeyboard, as_it_is, true);
    assert!(matches!(ran.answered, Err(NotDone::Refused(_))));
    assert_eq!(ran.asked.len(), 1);
}
