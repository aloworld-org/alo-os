//! **Opening a thing's menu, through a whole turn**, at both levels a person
//! can choose — against a tree that counts every time it is opened and every
//! action it is asked to do.
//!
//! - with no invocation the tree is never opened;
//! - at *ask each time* the menu waits for one approval, and one approval opens
//!   it once; a declined one does nothing;
//! - at full access it opens at once, once, with nothing left waiting;
//! - a grant revoked between the approval and the act stops it before any tree
//!   is opened;
//! - a terminal the person has not allowed their agent is never acted in, even
//!   at full access;
//! - every act and every refusal leaves exactly one entry.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

use alo_adapters::{AccessibilityTree, Facts, NodeAt, OPEN_MENU, Running, TreeFault, WorkingFully};
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

/// A Qt file manager.
const FILES: &str = "org.kde.dolphin";

/// A terminal, which alo reaches only where the person allowed it (ADR 0111).
const TERMINAL: &str = "app.devsuite.Ptyxis";

/// The agent.
const AGENT: &str = "@menus";

/// On the screen.
const ON: u32 = (1 << 25) | (1 << 30);

/// How many times a tree was opened, and how many actions were done.
#[derive(Debug, Default, Clone)]
struct Counted {
    opened: Rc<Cell<usize>>,
    done: Rc<Cell<usize>>,
    /// Whether the application gives no answer to the menu action.
    silent: Rc<Cell<bool>>,
}

/// One window showing one file with a menu.
struct AFolder {
    application: &'static str,
    counted: Counted,
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.7".to_owned(),
        object: object.to_owned(),
    }
}

impl AccessibilityTree for AFolder {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: at("/root"),
            is: Some(self.application.to_owned()),
        }])
    }

    fn facts(&self, place: &NodeAt) -> Result<Facts, TreeFault> {
        let thing = |role, name: &str, children: Vec<NodeAt>| Facts {
            role,
            name: name.to_owned(),
            states: vec![ON, 0],
            children,
            has_text: false,
            unlisted_children: false,
        };
        BTreeMap::from([
            (at("/root"), thing(75, "", vec![at("/win")])),
            (at("/win"), thing(23, "Home", vec![at("/notes")])),
            (at("/notes"), thing(56, "notes.txt", Vec::new())),
        ])
        .remove(place)
        .ok_or(TreeFault::Vanished)
    }

    fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
        panic!("opening a menu never reads a text")
    }

    fn actions(&self, place: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(if place.object == "/notes" {
            vec!["Press".to_owned(), "ShowMenu".to_owned()]
        } else {
            Vec::new()
        })
    }

    fn act(&self, place: &NodeAt, action: usize) -> Result<bool, TreeFault> {
        assert_eq!(
            (place.object.as_str(), action),
            ("/notes", 1),
            "only ShowMenu"
        );
        self.counted.done.set(self.counted.done.get() + 1);
        if self.counted.silent.get() {
            return Err(TreeFault::DidNotAnswer);
        }
        Ok(true)
    }
}

/// The person's windows.
struct TheWindows {
    application: &'static str,
    counted: Counted,
}

impl TheWindowsOnTheScreen for TheWindows {
    fn opening(&mut self) -> Result<Box<dyn AccessibilityTree>, TreeFault> {
        self.counted.opened.set(self.counted.opened.get() + 1);
        Ok(Box::new(AFolder {
            application: self.application,
            counted: self.counted.clone(),
        }))
    }

    fn working_fully(&self, _: &str) -> WorkingFully {
        WorkingFully::NothingMoreToTurnOn
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

/// A compositor's door nothing here knocks on: a menu opens by the tree.
struct NoDoor;

impl TheDoorTheCompositorServes for NoDoor {
    fn asking(&mut self, _: &alo_acting::Request) -> Result<alo_acting::Answer, NotAtTheDoor> {
        panic!("opening a menu never knocks on the compositor's door")
    }
}

/// Nothing here starts an application.
struct NothingStarts;

impl SaysHowAnApplicationStarts for NothingStarts {
    fn how_it_starts(&self, _: &str) -> Result<TheRoad, NoRoad> {
        Err(NoRoad::NoEntry)
    }
}

/// Run `doing` in a turn on a machine with these windows, the file manager and
/// a terminal installed, and the grants `granting` makes.
fn on_a_machine<T>(
    windows: &mut TheWindows,
    granting: impl FnOnce(&mut Grants),
    doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants) -> T,
) -> (T, Record) {
    let strings = in_english();
    let tool = NoTool;
    let publisher = crate::testing::APublisher::that_publishes();
    let mut door = NoDoor;
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let mut bounding = crate::testing::NothingIsBounded;
    let folder = crate::testing::a_folder_of_our_own("menus");
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
                installed: Installed::holding([
                    Application::identified(FILES).unwrap(),
                    Application::identified(TERMINAL).unwrap(),
                ])
                .into(),
                door: &mut door,
                starting: &NothingStarts,
                windows,
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
    (answered, record)
}

fn the_files_are_granted(grants: &mut Grants) {
    grants.grant(
        Grant::checked(AGENT, Reach::Application(FILES.to_owned()), noon(), hour()).unwrap(),
    );
}

fn windows_of(application: &'static str) -> TheWindows {
    TheWindows {
        application,
        counted: Counted::default(),
    }
}

fn the_notes_menu(application: &str) -> Vec<(&'static str, Given)> {
    vec![
        ("application", Given::text(application)),
        ("thing", Given::text("notes.txt")),
    ]
}

fn entries(record: &Record) -> usize {
    record.answering(&Asking::anything()).count()
}

#[test]
fn with_no_invocation_the_tree_is_never_opened() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let ((), record) = on_a_machine(&mut windows, the_files_are_granted, |_, _| ());
    assert_eq!(counted.opened.get(), 0);
    assert_eq!(entries(&record), 0);
}

#[test]
fn at_ask_each_time_one_approval_opens_the_menu_once_and_nothing_before_it() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let (did, record) = on_a_machine(&mut windows, the_files_are_granted, |turning, grants| {
        let id = turning
            .proposing(OPEN_MENU, &the_notes_menu(FILES), grants, hour(), noon())
            .unwrap();
        assert_eq!(counted.opened.get(), 0, "opened before the approval");
        let did = turning.approving(id, grants, noon()).unwrap();
        assert!(turning.approving(id, grants, noon()).is_err());
        did
    });
    let opened = did.menu_opened().unwrap();
    assert_eq!((opened.application(), opened.name()), (FILES, "notes.txt"));
    assert!(!opened.unanswered());
    assert_eq!((counted.opened.get(), counted.done.get()), (1, 1));
    assert_eq!(entries(&record), 1);
}

#[test]
fn a_declined_menu_does_nothing_and_is_recorded() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let ((), record) = on_a_machine(&mut windows, the_files_are_granted, |turning, grants| {
        let id = turning
            .proposing(OPEN_MENU, &the_notes_menu(FILES), grants, hour(), noon())
            .unwrap();
        turning.declining(id, noon()).unwrap();
    });
    assert_eq!((counted.opened.get(), counted.done.get()), (0, 0));
    assert_eq!(entries(&record), 1);
}

#[test]
fn at_full_access_the_menu_opens_at_once_and_once() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let (changed, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let at = grants.at_full_access_for(&Grantee::named(AGENT));
            let changed = turning
                .changing(
                    OPEN_MENU,
                    &the_notes_menu(FILES),
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
        panic!("a menu waited at full access")
    };
    assert!(did.menu_opened().is_some());
    assert!(sentence.text().contains("notes.txt"), "{sentence}");
    assert_eq!(counted.done.get(), 1);
    assert_eq!(entries(&record), 1);
}

#[test]
fn a_revoked_grant_stops_it_before_any_window_is_opened() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let (did, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let granted = grants.grant(
                Grant::checked(AGENT, Reach::Application(FILES.to_owned()), noon(), hour())
                    .unwrap(),
            );
            let id =
                turning.proposing(OPEN_MENU, &the_notes_menu(FILES), grants, hour(), noon())?;
            assert!(grants.revoke(granted));
            turning.approving(id, grants, noon())
        },
    );
    assert!(matches!(did, Err(NotDone::Refused(_))), "{did:?}");
    assert_eq!((counted.opened.get(), counted.done.get()), (0, 0));
    assert_eq!(entries(&record), 1);
}

#[test]
fn a_terminal_nobody_allowed_is_never_acted_in_even_at_full_access() {
    let mut windows = windows_of(TERMINAL);
    let counted = windows.counted.clone();
    let (refused, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let at = grants.at_full_access_for(&Grantee::named(AGENT));
            turning
                .changing(
                    OPEN_MENU,
                    &the_notes_menu(TERMINAL),
                    &at,
                    hour(),
                    &mut |_: &Said| None,
                    noon(),
                )
                .map(|_| ())
        },
    );
    assert!(matches!(refused, Err(NotDone::Refused(_))), "{refused:?}");
    assert_eq!((counted.opened.get(), counted.done.get()), (0, 0));
    assert_eq!(entries(&record), 1);
}

/// **An application that does not answer** may have opened the menu: the act
/// ran, is recorded once, and says that it is not known.
#[test]
fn an_application_that_does_not_answer_ran_and_says_it_is_not_known() {
    let mut windows = windows_of(FILES);
    windows.counted.silent.set(true);
    let (did, record) = on_a_machine(&mut windows, the_files_are_granted, |turning, grants| {
        let id = turning
            .proposing(OPEN_MENU, &the_notes_menu(FILES), grants, hour(), noon())
            .unwrap();
        turning.approving(id, grants, noon()).unwrap()
    });
    assert!(did.menu_opened().unwrap().unanswered());
    assert_eq!(entries(&record), 1);
}
