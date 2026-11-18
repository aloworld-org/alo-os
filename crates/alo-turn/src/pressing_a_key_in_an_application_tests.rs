//! **Pressing a key, through a whole turn**, at both levels a person can
//! choose, against a tree that counts every time it is opened and a door that
//! counts every key it is asked for.
//!
//! - with no invocation nothing is opened and no key goes;
//! - at *ask each time* a key waits for one approval, and one approval presses
//!   once; a declined one presses nothing;
//! - at full access it presses at once, once, with nothing left waiting;
//! - a password field or a terminal with the keyboard is never sent a key;
//! - a window that is not the application's, or is the person's own at the
//!   default, is refused by the compositor and written down;
//! - every press and every refusal leaves exactly one entry.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;

use alo_acting::{AKey, ToType, Typed};
use alo_adapters::{AccessibilityTree, Facts, NodeAt, PRESS_KEY, Running, TreeFault, WorkingFully};
use alo_applications::{Application, Installed};
use alo_capability::{Given, Grant, Grantee, Grants, Reach};
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_record::{Asking, Record};
use alo_software::Bound;
use alo_strings::Said;

use crate::how_an_application_starts::{NoRoad, SaysHowAnApplicationStarts, TheRoad};
use crate::machine::{Machine, WhatASoftwareVerbNeeds, WhatAnApplicationVerbNeeds};
use crate::refusing::NotDone;
use crate::testing::{hour, in_english, noon, offering};
use crate::the_door_the_compositor_serves::{NotAtTheDoor, TheDoorTheCompositorServes};
use crate::the_windows_on_the_screen::TheWindowsOnTheScreen;
use crate::{Changed, Turning};

/// The application every test presses a key in.
const BROWSER: &str = "org.mozilla.firefox";

/// The agent.
const AGENT: &str = "@keys";

/// `ATSPI_ROLE_ENTRY`, `_PASSWORD_TEXT`, `_TERMINAL`.
const ENTRY: u32 = 79;
const PASSWORD: u32 = 40;
const TERMINAL: u32 = 60;

/// Showing and editable, with the keyboard.
const FOCUSED: u32 = (1 << 25) | (1 << 7) | (1 << 12);

/// How many times a tree was opened.
#[derive(Debug, Default, Clone)]
struct Counted {
    opened: Rc<Cell<usize>>,
}

/// One window whose one field has the keyboard.
struct OneField {
    role: u32,
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.9".to_owned(),
        object: object.to_owned(),
    }
}

impl AccessibilityTree for OneField {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: at("/root"),
            is: Some(BROWSER.to_owned()),
        }])
    }

    fn facts(&self, place: &NodeAt) -> Result<Facts, TreeFault> {
        let thing = |role, name: &str, children: Vec<NodeAt>| Facts {
            role,
            name: name.to_owned(),
            states: vec![FOCUSED, 0],
            children,
            has_text: true,
            unlisted_children: false,
        };
        BTreeMap::from([
            (at("/root"), thing(75, "", vec![at("/win")])),
            (at("/win"), thing(23, "Mozilla Firefox", vec![at("/field")])),
            (at("/field"), thing(self.role, "Search", Vec::new())),
        ])
        .remove(place)
        .ok_or(TreeFault::Vanished)
    }

    fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
        panic!("pressing a key never reads a text")
    }

    fn actions(&self, _: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(Vec::new())
    }

    fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
        panic!("pressing a key never presses a control")
    }
}

/// The person's windows.
struct TheWindows {
    role: u32,
    counted: Counted,
}

impl TheWindowsOnTheScreen for TheWindows {
    fn opening(&mut self) -> Result<Box<dyn AccessibilityTree>, TreeFault> {
        self.counted.opened.set(self.counted.opened.get() + 1);
        Ok(Box::new(OneField { role: self.role }))
    }

    fn working_fully(&self, _: &str) -> WorkingFully {
        WorkingFully::NothingMoreToTurnOn
    }
}

/// A compositor's door that answers every key as `answers`, and keeps what it
/// was asked.
struct ADoor {
    answers: Typed,
    asked: Rc<RefCell<Vec<ToType>>>,
}

impl TheDoorTheCompositorServes for ADoor {
    fn asking(&mut self, _: &alo_acting::Request) -> Result<alo_acting::Answer, NotAtTheDoor> {
        panic!("pressing a key asks nothing but the key")
    }

    fn typing(&mut self, asked: &ToType, text: &str) -> Result<Typed, NotAtTheDoor> {
        assert!(text.is_empty(), "a key carries no text");
        self.asked.borrow_mut().push(asked.clone());
        Ok(self.answers)
    }
}

/// A tool nothing here reaches.
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
        panic!("nothing here installs anything")
    }
    fn updates(&self, _: &alo_software::SourceName) -> Result<Vec<String>, alo_software::Failed> {
        Ok(Vec::new())
    }
    fn update(&self, _: &Application) -> Result<(), alo_software::Failed> {
        panic!("nothing here updates anything")
    }
    fn remove(&self, _: &Application) -> Result<(), alo_software::Failed> {
        panic!("nothing here removes anything")
    }
}

/// Nothing here starts an application.
struct NothingStarts;

impl SaysHowAnApplicationStarts for NothingStarts {
    fn how_it_starts(&self, _: &str) -> Result<TheRoad, NoRoad> {
        Err(NoRoad::NoEntry)
    }
}

/// What one machine was asked.
struct Asked {
    opened: usize,
    keys: Vec<ToType>,
    entries: usize,
}

/// Run `doing` in a turn on a machine whose browser's field has this role and
/// whose door answers every key as `answers`.
fn on_a_machine<T>(
    role: u32,
    answers: Typed,
    granting: impl FnOnce(&mut Grants),
    doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants) -> T,
) -> (T, Asked) {
    let strings = in_english();
    let tool = NoTool;
    let publisher = crate::testing::APublisher::that_publishes();
    let keys = Rc::new(RefCell::new(Vec::new()));
    let mut door = ADoor {
        answers,
        asked: Rc::clone(&keys),
    };
    let counted = Counted::default();
    let mut windows = TheWindows {
        role,
        counted: counted.clone(),
    };
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let mut bounding = crate::testing::NothingIsBounded;
    let folder = crate::testing::a_folder_of_our_own("keys");
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
                installed: Installed::holding([Application::identified(BROWSER).unwrap()]).into(),
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
        granting(&mut grants);
        let mut turning =
            Turning::beginning(offering(&there), AGENT, hour(), &mut grants, &mut machine).unwrap();
        doing(&mut turning, &mut grants)
    };
    let asked = Asked {
        opened: counted.opened.get(),
        keys: keys.borrow().clone(),
        entries: record.answering(&Asking::anything()).count(),
    };
    (answered, asked)
}

fn the_browser_is_granted(grants: &mut Grants) {
    grants.grant(
        Grant::checked(
            AGENT,
            Reach::Application(BROWSER.to_owned()),
            noon(),
            hour(),
        )
        .unwrap(),
    );
}

fn enter() -> Vec<(&'static str, Given)> {
    vec![
        ("application", Given::text(BROWSER)),
        ("key", Given::text("enter")),
    ]
}

/// Propose Enter, approve it, and answer what it did.
fn approving_enter(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
) -> Result<crate::WhatAVerbDid, NotDone> {
    let id = turning.proposing(PRESS_KEY, &enter(), grants, hour(), noon())?;
    turning.approving(id, grants, noon())
}

#[test]
fn with_no_invocation_nothing_is_opened_and_no_key_goes() {
    let ((), asked) = on_a_machine(
        ENTRY,
        Typed::PressedTheKey,
        the_browser_is_granted,
        |_, _| (),
    );
    assert_eq!((asked.opened, asked.keys.len(), asked.entries), (0, 0, 0));
}

#[test]
fn at_ask_each_time_one_approval_presses_one_key_once() {
    let (did, asked) = on_a_machine(
        ENTRY,
        Typed::PressedTheKey,
        the_browser_is_granted,
        |turning, grants| {
            let id = turning
                .proposing(PRESS_KEY, &enter(), grants, hour(), noon())
                .unwrap();
            let did = turning.approving(id, grants, noon()).unwrap();
            assert!(turning.approving(id, grants, noon()).is_err());
            did
        },
    );
    let pressed = did.key_pressed().unwrap();
    assert_eq!(
        (pressed.application(), pressed.key()),
        (BROWSER, AKey::Enter)
    );
    assert_eq!(
        asked.keys,
        [ToType::Key {
            application: Application::identified(BROWSER).unwrap(),
            key: AKey::Enter,
        }]
    );
    assert_eq!(asked.entries, 1);
}

#[test]
fn a_declined_key_presses_nothing_and_is_recorded() {
    let ((), asked) = on_a_machine(
        ENTRY,
        Typed::PressedTheKey,
        the_browser_is_granted,
        |turning, grants| {
            let id = turning
                .proposing(PRESS_KEY, &enter(), grants, hour(), noon())
                .unwrap();
            turning.declining(id, noon()).unwrap();
        },
    );
    assert_eq!((asked.opened, asked.keys.len(), asked.entries), (0, 0, 1));
}

#[test]
fn at_full_access_a_key_is_pressed_at_once_and_once() {
    let (changed, asked) = on_a_machine(
        ENTRY,
        Typed::PressedTheKey,
        |_| {},
        |turning, grants| {
            let at = grants.at_full_access_for(&Grantee::named(AGENT));
            let changed = turning
                .changing(
                    PRESS_KEY,
                    &enter(),
                    &at,
                    hour(),
                    &mut |_: &Said| None,
                    noon(),
                )
                .unwrap();
            assert_eq!(turning.waiting_at(noon()).count(), 0);
            changed
        },
    );
    let Changed::Done { did, sentence } = changed else {
        panic!("a key waited at full access")
    };
    assert!(did.key_pressed().is_some());
    assert!(sentence.text().contains(BROWSER), "{sentence}");
    assert_eq!((asked.keys.len(), asked.entries), (1, 1));
}

#[test]
fn a_password_field_or_a_terminal_with_the_keyboard_is_never_sent_a_key_at_either_level() {
    for role in [PASSWORD, TERMINAL] {
        let ((asking, at_full_access), asked) = on_a_machine(
            role,
            Typed::PressedTheKey,
            the_browser_is_granted,
            |turning, grants| {
                let asking = approving_enter(turning, grants);
                let at = grants.at_full_access_for(&Grantee::named(AGENT));
                let full = turning
                    .changing(
                        PRESS_KEY,
                        &enter(),
                        &at,
                        hour(),
                        &mut |_: &Said| None,
                        noon(),
                    )
                    .map(|_| ());
                (asking, full)
            },
        );
        assert!(
            matches!(asking, Err(NotDone::Refused(_))),
            "{role}: {asking:?}"
        );
        assert!(
            matches!(at_full_access, Err(NotDone::Refused(_))),
            "{role}: {at_full_access:?}"
        );
        assert!(asked.keys.is_empty(), "{role}: a key went");
        assert_eq!(asked.entries, 2, "{role}: each refusal is written down");
    }
}

#[test]
fn a_window_the_compositor_will_not_press_in_is_refused_and_written_down() {
    for answers in [Typed::NotInFront, Typed::NotAlosCopy, Typed::NoKeyboard] {
        let (did, asked) =
            on_a_machine(ENTRY, answers, the_browser_is_granted, |turning, grants| {
                approving_enter(turning, grants)
            });
        assert!(
            matches!(did, Err(NotDone::Refused(_))),
            "{answers:?}: {did:?}"
        );
        assert_eq!((asked.keys.len(), asked.entries), (1, 1), "{answers:?}");
    }
}

#[test]
fn a_revoked_grant_stops_the_key_before_any_window_is_opened() {
    let (did, asked) = on_a_machine(
        ENTRY,
        Typed::PressedTheKey,
        |_| {},
        |turning, grants| {
            let granted = grants.grant(
                Grant::checked(
                    AGENT,
                    Reach::Application(BROWSER.to_owned()),
                    noon(),
                    hour(),
                )
                .unwrap(),
            );
            let id = turning.proposing(PRESS_KEY, &enter(), grants, hour(), noon())?;
            assert!(grants.revoke(granted));
            turning.approving(id, grants, noon())
        },
    );
    assert!(matches!(did, Err(NotDone::Refused(_))), "{did:?}");
    assert_eq!((asked.opened, asked.keys.len(), asked.entries), (0, 0, 1));
}
