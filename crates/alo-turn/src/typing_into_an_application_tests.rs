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
use alo_capability::{ATerminalForTheAgent, Given, Grant, Grantee, Grants, Reach};
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
    /// Whether the field answers *yes* to text put in through the tree and
    /// keeps none of it, as Firefox does.
    ignores_what_is_put_in: bool,
    /// Whether the field keeps what is put in through the tree, changed — an
    /// editor that indents again.
    rewrites_what_is_put_in: bool,
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
            ignores_what_is_put_in: false,
            rewrites_what_is_put_in: false,
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
        if self.0.ignores_what_is_put_in {
            return Ok(true);
        }
        let text = if self.0.rewrites_what_is_put_in {
            rewritten(text)
        } else {
            text.to_owned()
        };
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
            ToType::Key { .. } => Typed::PressedTheKey,
        })
    }

    /// Every file is opened in alo's own copy, and the line is kept in the
    /// same list as the typing's, so their order can be read.
    fn opening_a_file(
        &mut self,
        asked: &alo_acting::ToOpenAFile,
    ) -> Result<alo_acting::Opened, NotAtTheDoor> {
        self.asked
            .borrow_mut()
            .push(asked.written().unwrap_or_default());
        Ok(alo_acting::Opened::InAlosCopy)
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
    typing_once_in(
        EDITOR,
        ATerminalForTheAgent::NotAtAll,
        field,
        whose,
        rewriting,
        approve,
    )
}

/// As [`typing_once`], into `application`, granted to the agent while the
/// person's terminal choice was `terminal` and asked against a list carrying
/// it.
fn typing_once_in(
    application: &str,
    terminal: ATerminalForTheAgent,
    field: TheField,
    whose: Typed,
    rewriting: fn(&str) -> String,
    approve: bool,
) -> Ran {
    let answering = if approve {
        Answering::Approving
    } else {
        Answering::Declining
    };
    typing_once_as(
        TYPE_TEXT,
        application,
        terminal,
        field,
        whose,
        rewriting,
        answering,
    )
}

/// How a typing is answered: approved or declined at *ask each time*, or run
/// at full access with no card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Answering {
    Approving,
    Declining,
    AtFullAccess,
}

/// [`typing_once`], by either typing verb, answered either way.
fn typing_once_by(
    verb: &str,
    field: TheField,
    whose: Typed,
    rewriting: fn(&str) -> String,
    approve: Answering,
) -> Ran {
    typing_once_as(
        verb,
        EDITOR,
        ATerminalForTheAgent::NotAtAll,
        field,
        whose,
        rewriting,
        approve,
    )
}

/// The one body: `verb` into `application`, under `terminal`, answered as
/// `approve`.
fn typing_once_as(
    verb: &str,
    application: &str,
    terminal: ATerminalForTheAgent,
    field: TheField,
    whose: Typed,
    rewriting: fn(&str) -> String,
    approve: Answering,
) -> Ran {
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
                installed: Installed::holding([Application::identified(application).unwrap()])
                    .into(),
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
            Grant::checked_for_when(
                &Grantee::named(AGENT),
                Reach::Application(application.to_owned()),
                noon(),
                hour(),
                terminal,
            )
            .unwrap(),
        );
        let mut turning =
            Turning::beginning(offering(&there), AGENT, hour(), &mut grants, &mut machine).unwrap();
        let grants = grants.with_the_terminal_as(terminal);
        // **The field is `field` for typing that saves nothing**
        // (`alo_adapters::fallback_verbs::FIELD`), `name` for `type_text`.
        let the_field = if verb == alo_adapters::TYPE_WITHOUT_SAVING {
            alo_adapters::fallback_verbs::FIELD
        } else {
            alo_adapters::fallback_verbs::NAME
        };
        let given = [
            ("application", Given::text(application)),
            (the_field, Given::text("index.html")),
            ("text", Given::text(THE_TEXT)),
        ];
        match approve {
            Answering::AtFullAccess => {
                let at = grants.at_full_access_for(&alo_capability::Grantee::named(AGENT));
                turning
                    .changing(
                        verb,
                        &given,
                        &at,
                        hour(),
                        &mut |_: &alo_strings::Said| None,
                        noon(),
                    )
                    .map(|changed| match changed {
                        crate::Changed::Done { did, .. } => Some(did),
                        crate::Changed::Waiting(_) => None,
                    })
            }
            Answering::Approving | Answering::Declining => {
                let id = turning.proposing(verb, &given, &grants, hour(), noon());
                assert_eq!(opened.get(), 0, "a proposal opened the tree");
                assert!(asked.borrow().is_empty(), "a proposal asked the compositor");
                match id {
                    Err(refused) => Err(refused),
                    Ok(id) if approve == Answering::Approving => {
                        turning.approving(id, &grants, noon()).map(Some)
                    }
                    Ok(id) => turning.declining(id, noon()).map(|()| None),
                }
            }
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
    assert!(typed.asked_to_save() && typed.read_back());
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

/// **Letting alo use a terminal does not let it type into one** (ADR 0111;
/// the owner, 2026-11-17): what is typed into a terminal runs, so a terminal
/// application the person allowed is refused before any tree is opened or
/// the compositor asked, the field's own role notwithstanding — and the
/// refusal is written down.
#[test]
fn a_terminal_the_person_allowed_is_still_never_typed_into() {
    let ran = typing_once_in(
        "app.devsuite.Ptyxis",
        ATerminalForTheAgent::MayOpenOne,
        TheField::an_editor(),
        Typed::MayType,
        as_it_is,
        true,
    );
    assert!(
        matches!(&ran.answered, Err(NotDone::Refused(_))),
        "a terminal was typed into: {:?}",
        ran.answered
    );
    let said = match &ran.answered {
        Err(NotDone::Refused(refused)) => refused.said(&in_english()).text().to_owned(),
        _ => String::new(),
    };
    assert!(said.contains("is a terminal"), "{said}");
    assert!(
        !said.contains("only where you have let"),
        "the refusal still says letting alo use a terminal would let it type: {said}"
    );
    assert_eq!(ran.opened, 0, "the tree was opened for a terminal");
    assert!(ran.asked.is_empty(), "the compositor was asked");
    assert_eq!(ran.entries, 1);
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

/// **The whole road, as a person's request makes it: the file opened in alo's
/// own copy, then the text typed into it, then saved — in that order, one
/// record for each approved step.** At the default level, where each change
/// waits for its one approval.
///
/// The executors, the turn, the grants and the record are real; the
/// compositor's door and the accessibility tree are this file's, sharing the
/// field so a typing changes what is read back. The door's real wire is
/// `alo-agentd`'s `at_the_compositors_door`, held by its own tests; the
/// shell's side of it by `alo-shell`'s door tests.
///
/// What would have to be true for this to pass while broken: the real door
/// answering `may` before alo's copy has the keyboard, or the real tree
/// naming VSCodium's editor something other than its file — neither of which a
/// test without the application can see. The walk
/// (`crates/alo-walking/src/writing_in_vscodium.rs`) is where they are.
#[test]
fn a_file_is_opened_then_typed_into_then_saved_with_one_record_each() {
    let field = TheField::an_editor();
    let strings = in_english();
    let tool = NoTool;
    let publisher = crate::testing::APublisher::that_publishes();
    let asked = Rc::new(RefCell::new(Vec::new()));
    let mut door = ADoor {
        whose: Typed::MayType,
        rewriting: as_it_is,
        field: field.text.clone(),
        asked: asked.clone(),
    };
    let mut windows = TheWindows(field.clone());
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let mut bounding = crate::testing::NothingIsBounded;
    let folder = crate::testing::a_folder_of_our_own("writing-in-an-editor");
    let file = folder.join("index.html");
    std::fs::write(&file, "").unwrap();
    let elsewhere = crate::testing::a_folder_of_our_own("writing-offered").join("a-note.txt");
    std::fs::write(&elsewhere, "anything").unwrap();
    {
        let mut machine = Machine::carrying_out_file_software_and_application_verbs(
            &strings,
            &OnThisMachine,
            WhatASoftwareVerbNeeds {
                tool: &tool,
                bound: Bound::Nobodys,
                publishing: crate::testing::publishing_through(&publisher),
            },
            WhatAnApplicationVerbNeeds {
                installed: Installed::holding([Application::called(EDITOR, "VSCodium").unwrap()])
                    .into(),
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
        grants.grant(Grant::checked(AGENT, Reach::Folder(folder.clone()), noon(), hour()).unwrap());
        let mut turning = Turning::beginning(
            offering(&elsewhere),
            AGENT,
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        let opening = turning
            .proposing(
                alo_applications::OPEN_FILE_IN_APPLICATION,
                &[
                    ("application", Given::text(EDITOR)),
                    ("path", Given::text(file.to_string_lossy().into_owned())),
                ],
                &grants,
                hour(),
                noon(),
            )
            .unwrap();
        let opened = turning.approving(opening, &grants, noon()).unwrap();
        assert!(
            opened
                .opened()
                .is_some_and(crate::AFileOpened::in_alos_copy)
        );
        // **And it says where typing into it goes** (option A), read from the
        // window once it was in front — the name the typing below is given.
        let (types_into, in_the_window) = opened
            .opened()
            .and_then(crate::AFileOpened::types_into)
            .expect("the window could be read, so where typing goes is said");
        assert_eq!(types_into, "index.html");
        assert_eq!(in_the_window, "index.html - VSCodium");
        let types_into = types_into.to_owned();
        let typing = turning
            .proposing(
                TYPE_TEXT,
                &[
                    ("application", Given::text(EDITOR)),
                    ("name", Given::text(types_into)),
                    ("text", Given::text(THE_TEXT)),
                ],
                &grants,
                hour(),
                noon(),
            )
            .unwrap();
        let typed = turning.approving(typing, &grants, noon()).unwrap();
        assert!(typed.typed().is_some_and(crate::ATextTyped::saved));
    }
    let real = std::fs::canonicalize(&file).unwrap();
    assert_eq!(
        *asked.borrow(),
        [
            format!("open-file {EDITOR} {}", real.display()),
            format!("may-type-into {EDITOR}"),
            format!("type-into {EDITOR} {}", THE_TEXT.len()),
            format!("save-in {EDITOR}"),
        ],
        "the road's steps were not asked once each, in order"
    );
    assert_eq!(field.text.borrow().as_str(), THE_TEXT);
    assert_eq!(
        record.answering(&Asking::anything()).count(),
        2,
        "one record for the opening and one for the typing"
    );
}

/// **A field that says it took the text and kept none of it is typed by keys
/// instead** — Firefox, on every field it has
/// (`docs/quirks/chromium-offers-no-editable-text-and-firefox-ignores-it.md`).
/// Only once the field reads back exactly as it was, so nothing is typed
/// twice; and only where the field has the keyboard, as for any typing by
/// keys. Then it reads back, and saves, as every typing does.
#[test]
fn a_field_that_ignores_what_is_put_in_is_typed_by_keys_instead() {
    let mut field = TheField::an_editor();
    field.takes_text = true;
    field.ignores_what_is_put_in = true;
    let text = field.text.clone();
    let ran = typing_once(field, Typed::MayType, as_it_is, true);
    let did = ran.answered.unwrap().expect("approved");
    let typed = did.typed().expect("a text was typed");
    assert!(typed.saved());
    assert!(!typed.through_the_tree(), "said as typed through the tree");
    assert_eq!(*text.borrow(), THE_TEXT, "typed twice, or not at all");
    assert_eq!(
        ran.asked,
        [
            format!("may-type-into {EDITOR}"),
            format!("type-into {EDITOR} {}", THE_TEXT.len()),
            format!("save-in {EDITOR}"),
        ]
    );
    assert_eq!(ran.entries, 1);
}

/// **And without the keyboard it is refused**, with nothing typed and nothing
/// saved: the keys would go wherever the keyboard is.
#[test]
fn a_field_that_ignores_what_is_put_in_and_has_no_keyboard_is_refused() {
    let mut field = TheField::an_editor();
    field.takes_text = true;
    field.ignores_what_is_put_in = true;
    field.states = EDITABLE;
    let text = field.text.clone();
    let ran = typing_once(field, Typed::MayType, as_it_is, true);
    assert!(matches!(ran.answered, Err(NotDone::Refused(_))));
    assert_eq!(ran.asked, [format!("may-type-into {EDITOR}")]);
    assert!(text.borrow().is_empty());
    assert_eq!(ran.entries, 1, "the refusal is written down");
}

/// **A field that kept the text changed is never typed again by keys**: it
/// took something, so the keys would put the text in a second time. It is not
/// saved, as any text that read back otherwise is not, and the typing is
/// recorded as run.
#[test]
fn a_field_that_kept_the_text_changed_is_not_typed_again_by_keys() {
    let mut field = TheField::an_editor();
    field.takes_text = true;
    field.rewrites_what_is_put_in = true;
    let text = field.text.clone();
    let ran = typing_once(field, Typed::MayType, as_it_is, true);
    let did = ran.answered.unwrap().expect("approved");
    let typed = did.typed().expect("a text was typed");
    assert!(!typed.saved());
    assert!(typed.through_the_tree());
    assert_eq!(
        ran.asked,
        [format!("may-type-into {EDITOR}")],
        "typed by keys too"
    );
    assert_eq!(*text.borrow(), rewritten(THE_TEXT));
    assert_eq!(ran.entries, 1);
}

/// **Typing without saving types, reads back and never asks to save**, at
/// both levels a person can choose (2026-11-17): the text goes in as
/// `type-text` would put it, and no `save-in` ever reaches the compositor —
/// which in a web page would save the page.
#[test]
fn typing_without_saving_reads_back_and_never_asks_to_save_at_either_level() {
    for answering in [Answering::Approving, Answering::AtFullAccess] {
        let ran = typing_once_by(
            alo_adapters::TYPE_WITHOUT_SAVING,
            TheField::an_editor(),
            Typed::MayType,
            as_it_is,
            answering,
        );
        let did = ran.answered.unwrap().expect("carried out");
        let typed = did.typed().expect("a text was typed");
        assert!(!typed.asked_to_save(), "{answering:?}");
        assert!(typed.read_back(), "{answering:?}");
        assert!(!typed.saved(), "{answering:?}");
        assert_eq!(
            ran.asked,
            [format!("type-into {EDITOR} {}", THE_TEXT.len())],
            "{answering:?}"
        );
        assert_eq!(ran.entries, 1, "{answering:?}");
    }
}

/// **A text that reads back differently is said, and still nothing is
/// saved** — with no word about saving, since none was asked.
#[test]
fn typing_without_saving_says_a_text_that_reads_back_differently() {
    let ran = typing_once_by(
        alo_adapters::TYPE_WITHOUT_SAVING,
        TheField::an_editor(),
        Typed::MayType,
        rewritten,
        Answering::Approving,
    );
    let did = ran.answered.unwrap().expect("carried out");
    let typed = did.typed().expect("a text was typed");
    assert!(!typed.read_back() && !typed.saved());
    assert!(ran.asked.iter().all(|line| !line.starts_with("save-in")));
    assert_eq!(ran.entries, 1);
}

/// **The person's own window at the default refuses typing without saving
/// too**, at both levels, with nothing typed and the refusal written down.
#[test]
fn typing_without_saving_into_the_persons_own_window_is_refused_at_either_level() {
    for answering in [Answering::Approving, Answering::AtFullAccess] {
        let ran = typing_once_by(
            alo_adapters::TYPE_WITHOUT_SAVING,
            TheField::an_editor(),
            Typed::NotAlosCopy,
            as_it_is,
            answering,
        );
        assert!(
            matches!(ran.answered, Err(NotDone::Refused(_))),
            "{answering:?}"
        );
        assert_eq!(ran.entries, 1, "{answering:?}");
    }
}
