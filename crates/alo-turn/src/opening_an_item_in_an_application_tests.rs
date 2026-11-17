//! **Opening an item, through a whole turn** — at ask each time and at full
//! access, as the owner decided (*"Yes but it should be without okay on full
//! access"*), and on every refusal.
//!
//! - with no invocation the tree is never opened;
//! - at ask each time an opening waits for one approval, and one approval
//!   opens once; a declined one opens nothing;
//! - at full access it runs with no card, once, and is recorded;
//! - a grant revoked before the opening stops it before any tree is opened;
//! - a terminal nobody allowed is refused at full access too;
//! - every opening and every refusal leaves exactly one entry.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::Cell;
use std::rc::Rc;

use alo_adapters::{
    AccessibilityTree, Facts, NodeAt, OPEN_ITEM, Running, States, TreeFault, WorkingFully,
};
use alo_applications::{Application, Installed};
use alo_capability::{Given, Grant, Grantee, Grants, Reach};
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_record::{Asking, Record};
use alo_software::Bound;

use crate::Turning;
use crate::a_change_at_full_access::Changed;
use crate::how_an_application_starts::{NoRoad, SaysHowAnApplicationStarts, TheRoad};
use crate::machine::{Machine, WhatASoftwareVerbNeeds, WhatAnApplicationVerbNeeds};
use crate::refusing::NotDone;
use crate::testing::{hour, in_english, noon, offering};
use crate::the_door_the_compositor_serves::{NotAtTheDoor, TheDoorTheCompositorServes};
use crate::the_windows_on_the_screen::TheWindowsOnTheScreen;

/// The application every test opens in.
const FILES: &str = "org.kde.dolphin";

/// A terminal, which alo reaches only where the person allowed it (ADR 0111).
const TERMINAL: &str = "app.devsuite.Ptyxis";

/// The agent.
const AGENT: &str = "@opening";

/// What every tree a test opens shares.
#[derive(Debug, Default, Clone)]
struct Counted {
    trees: Rc<Cell<usize>>,
    openings: Rc<Cell<usize>>,
    does_not_answer: Rc<Cell<bool>>,
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.5".to_owned(),
        object: object.to_owned(),
    }
}

/// A folder *Downloads* listing *report.pdf*, which offers *activate*.
struct AFolder {
    application: &'static str,
    counted: Counted,
}

impl AccessibilityTree for AFolder {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: at("/root"),
            is: Some(self.application.to_owned()),
        }])
    }

    fn facts(&self, place: &NodeAt) -> Result<Facts, TreeFault> {
        let thing = |role, name: &str, children: &[&str]| Facts {
            role,
            name: name.to_owned(),
            states: States::showing_and_usable().as_the_tree_says().to_vec(),
            children: children.iter().map(|child| at(child)).collect(),
            has_text: false,
            unlisted_children: false,
        };
        match place.object.as_str() {
            "/root" => Ok(thing(75, "", &["/win"])),
            "/win" => Ok(thing(23, "Downloads", &["/list"])),
            "/list" => Ok(thing(31, "", &["/report"])),
            "/report" => Ok(thing(32, "report.pdf", &[])),
            _ => Err(TreeFault::Vanished),
        }
    }

    fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
        Ok(String::new())
    }

    fn actions(&self, _: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(vec!["activate".to_owned()])
    }

    fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
        self.counted.openings.set(self.counted.openings.get() + 1);
        if self.counted.does_not_answer.get() {
            return Err(TreeFault::DidNotAnswer);
        }
        Ok(true)
    }
}

/// The person's windows, as a test arranges them.
struct TheWindows {
    application: &'static str,
    counted: Counted,
}

impl TheWindowsOnTheScreen for TheWindows {
    fn opening(&mut self) -> Result<Box<dyn AccessibilityTree>, TreeFault> {
        self.counted.trees.set(self.counted.trees.get() + 1);
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

/// A compositor's door nothing here knocks on.
struct NoDoor;

impl TheDoorTheCompositorServes for NoDoor {
    fn asking(&mut self, _: &alo_acting::Request) -> Result<alo_acting::Answer, NotAtTheDoor> {
        panic!("nothing here acts through the compositor")
    }
}

/// Nothing here starts an application.
struct NothingStarts;

impl SaysHowAnApplicationStarts for NothingStarts {
    fn how_it_starts(&self, _: &str) -> Result<TheRoad, NoRoad> {
        Err(NoRoad::NoEntry)
    }
}

/// Run `doing` in a turn on a machine with these windows; answer what it
/// answered and the record.
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
    let folder = crate::testing::a_folder_of_our_own("opening-an-item");
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

/// The arguments for opening `item` in `application`.
fn opening(application: &str, item: &str) -> Vec<(&'static str, Given)> {
    vec![
        ("application", Given::text(application)),
        ("name", Given::text(item)),
    ]
}

fn entries(record: &Record) -> usize {
    record.answering(&Asking::anything()).count()
}

#[test]
fn with_no_invocation_the_tree_is_never_opened() {
    let mut windows = windows_of(FILES);
    let trees = Rc::clone(&windows.counted.trees);
    let ((), _) = on_a_machine(&mut windows, the_files_are_granted, |_, _| ());
    assert_eq!(trees.get(), 0);
}

/// **At ask each time an opening waits, and one approval opens once.**
#[test]
fn at_ask_each_time_one_approval_opens_once() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let (did, record) = on_a_machine(&mut windows, the_files_are_granted, |turning, grants| {
        let id = turning.proposing(
            OPEN_ITEM,
            &opening(FILES, "report.pdf"),
            grants,
            hour(),
            noon(),
        )?;
        assert_eq!(counted.trees.get(), 0, "a proposal opened the tree");
        turning.approving(id, grants, noon())
    });
    let did = did.unwrap();
    let opened = did.item_opened().expect("an item was opened");
    assert_eq!((opened.application(), opened.name()), (FILES, "report.pdf"));
    assert!(!opened.unanswered());
    assert_eq!(counted.trees.get(), 1);
    assert_eq!(counted.openings.get(), 1, "one approval, one opening");
    assert_eq!(entries(&record), 1);
}

/// **At full access it runs without a card, once, and is recorded** — the
/// owner's "without okay on full access".
#[test]
fn at_full_access_an_item_opens_once_without_a_card() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let (changed, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let at_full_access = grants.at_full_access_for(&Grantee::named(AGENT));
            let changed = turning.changing(
                OPEN_ITEM,
                &opening(FILES, "report.pdf"),
                &at_full_access,
                hour(),
                &mut |_| None,
                noon(),
            );
            assert_eq!(turning.waiting_at(noon()).count(), 0, "a card was left");
            changed
        },
    );
    let Ok(Changed::Done { did, sentence }) = changed else {
        panic!("an opening at full access did not run: {changed:?}");
    };
    assert_eq!(sentence.text(), "open “report.pdf” in org.kde.dolphin");
    assert!(did.item_opened().is_some());
    assert_eq!(counted.openings.get(), 1);
    assert_eq!(entries(&record), 1);
}

#[test]
fn a_declined_opening_opens_nothing_and_is_recorded() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let (declined, record) =
        on_a_machine(&mut windows, the_files_are_granted, |turning, grants| {
            let id = turning.proposing(
                OPEN_ITEM,
                &opening(FILES, "report.pdf"),
                grants,
                hour(),
                noon(),
            )?;
            turning.declining(id, noon())
        });
    assert!(declined.is_ok(), "{declined:?}");
    assert_eq!(counted.trees.get(), 0);
    assert_eq!(counted.openings.get(), 0);
    assert_eq!(entries(&record), 1);
}

#[test]
fn a_revoked_grant_stops_the_opening_before_the_tree() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let (did, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let id = grants.grant(
                Grant::checked(AGENT, Reach::Application(FILES.to_owned()), noon(), hour())
                    .unwrap(),
            );
            let proposed = turning.proposing(
                OPEN_ITEM,
                &opening(FILES, "report.pdf"),
                grants,
                hour(),
                noon(),
            )?;
            assert!(grants.revoke(id));
            turning.approving(proposed, grants, noon())
        },
    );
    assert!(matches!(did, Err(NotDone::Refused(_))), "{did:?}");
    assert_eq!(counted.trees.get(), 0);
    assert_eq!(counted.openings.get(), 0);
    assert_eq!(entries(&record), 1);
}

/// **A terminal nobody allowed is refused even at full access** (ADR 0111),
/// before any tree is opened.
#[test]
fn a_terminal_nobody_allowed_is_never_opened_in_even_at_full_access() {
    let mut windows = windows_of(TERMINAL);
    let counted = windows.counted.clone();
    let (changed, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let at_full_access = grants.at_full_access_for(&Grantee::named(AGENT));
            turning.changing(
                OPEN_ITEM,
                &opening(TERMINAL, "report.pdf"),
                &at_full_access,
                hour(),
                &mut |_| None,
                noon(),
            )
        },
    );
    assert!(changed.is_err(), "a terminal was opened in: {changed:?}");
    assert_eq!(counted.trees.get(), 0);
    assert_eq!(counted.openings.get(), 0);
    assert_eq!(entries(&record), 1);
}

/// **An item not on the screen is refused at both levels**, nothing opened,
/// one entry each.
#[test]
fn an_item_not_there_is_refused_at_both_levels_and_recorded() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let ((asked, at_full), record) =
        on_a_machine(&mut windows, the_files_are_granted, |turning, grants| {
            let given = opening(FILES, "missing.txt");
            let asked = turning
                .proposing(OPEN_ITEM, &given, grants, hour(), noon())
                .and_then(|id| turning.approving(id, grants, noon()));
            let at_full_access = grants.at_full_access_for(&Grantee::named(AGENT));
            let at_full = turning.changing(
                OPEN_ITEM,
                &given,
                &at_full_access,
                hour(),
                &mut |_| None,
                noon(),
            );
            (asked, at_full.map(|_| ()))
        });
    let Err(NotDone::Refused(refused)) = asked else {
        panic!("a missing item was opened: {asked:?}");
    };
    assert!(
        refused
            .said(&in_english())
            .text()
            .contains("no item named “missing.txt”"),
        "{}",
        refused.said(&in_english())
    );
    assert!(matches!(at_full, Err(NotDone::Refused(_))), "{at_full:?}");
    assert_eq!(counted.openings.get(), 0);
    assert_eq!(entries(&record), 2);
}

/// **An application that does not answer may have opened it**: recorded as
/// run, said as not known.
#[test]
fn an_application_that_does_not_answer_is_run_and_said_as_not_known() {
    let mut windows = windows_of(FILES);
    windows.counted.does_not_answer.set(true);
    let (did, record) = on_a_machine(&mut windows, the_files_are_granted, |turning, grants| {
        let id = turning.proposing(
            OPEN_ITEM,
            &opening(FILES, "report.pdf"),
            grants,
            hour(),
            noon(),
        )?;
        turning.approving(id, grants, noon())
    });
    assert!(did.unwrap().item_opened().unwrap().unanswered());
    let asking = Asking::anything();
    let entry = record.answering(&asking).next().unwrap();
    assert!(format!("{entry:?}").contains("is not known"), "{entry:?}");
}
