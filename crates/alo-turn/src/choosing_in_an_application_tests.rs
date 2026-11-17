//! **Choosing in an application, through a whole turn** — at ask each time and
//! at full access, and on every refusal — against a tree that counts every
//! time it is opened and every choice it is asked for.
//!
//! - with no invocation the tree is never opened;
//! - at ask each time a choice waits for one approval, and one approval
//!   chooses once; a declined one chooses nothing;
//! - at full access it runs with no card, once;
//! - a grant revoked before the choice stops it before any tree is opened;
//! - a terminal nobody allowed is refused at full access too;
//! - every choice and every refusal leaves exactly one entry, and a choice the
//!   window does not show is recorded as run with what alo saw beside it.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;

use alo_adapters::{
    AccessibilityTree, CHOOSE_OPTION, Facts, NodeAt, Running, SELECT_ITEM, States, TreeFault,
    WorkingFully,
};
use alo_applications::{Application, Installed};
use alo_capability::{Given, Grant, Grantee, Grants, Reach};
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_record::{Asking, Record};
use alo_software::Bound;

use crate::a_change_at_full_access::Changed;
use crate::how_an_application_starts::{NoRoad, SaysHowAnApplicationStarts, TheRoad};
use crate::machine::{Machine, WhatASoftwareVerbNeeds, WhatAnApplicationVerbNeeds};
use crate::refusing::NotDone;
use crate::testing::{hour, in_english, noon, offering};
use crate::the_door_the_compositor_serves::{NotAtTheDoor, TheDoorTheCompositorServes};
use crate::the_windows_on_the_screen::TheWindowsOnTheScreen;
use crate::{Turning, WhatAVerbDid};

/// The application every test chooses in.
const FILES: &str = "org.kde.dolphin";

/// A terminal, which alo reaches only where the person allowed it (ADR 0111).
const TERMINAL: &str = "app.devsuite.Ptyxis";

/// The agent.
const AGENT: &str = "@choosing";

/// What every tree a test opens shares.
#[derive(Debug, Default, Clone)]
struct Counted {
    opened: Rc<Cell<usize>>,
    chosen: Rc<Cell<usize>>,
    /// Whether a choice marks the thing selected.
    leaves_it_unmarked: Rc<Cell<bool>>,
    marked: Rc<RefCell<Vec<NodeAt>>>,
    /// Whether the application shows a choice only on the third look.
    slow: Rc<Cell<bool>>,
    pending: Rc<RefCell<Option<NodeAt>>>,
    looks: Rc<Cell<usize>>,
    /// Whether the application does not answer the choice.
    does_not_answer: Rc<Cell<bool>>,
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.5".to_owned(),
        object: object.to_owned(),
    }
}

/// A window *Documents* with a closed drop-down *Sort by* (options *Name* and
/// *Size*) and a list holding *report.pdf*.
struct OneWindow {
    application: &'static str,
    counted: Counted,
}

impl OneWindow {
    fn facts_of(&self) -> BTreeMap<NodeAt, Facts> {
        let showing = States::showing_and_usable();
        let selectable = |states: States, place: &str| {
            let states = states.selectable();
            let marked = self.counted.marked.borrow().contains(&at(place));
            if marked {
                states.and_selected()
            } else {
                states
            }
            .as_the_tree_says()
            .to_vec()
        };
        let thing = |role, name: &str, states: Vec<u32>, children: &[&str]| Facts {
            role,
            name: name.to_owned(),
            states,
            children: children.iter().map(|child| at(child)).collect(),
            has_text: false,
            unlisted_children: false,
        };
        let shown = showing.as_the_tree_says().to_vec();
        let hidden = showing.hidden().as_the_tree_says().to_vec();
        BTreeMap::from([
            (at("/root"), thing(75, "", shown.clone(), &["/win"])),
            (
                at("/win"),
                thing(23, "Documents", shown.clone(), &["/sort", "/list"]),
            ),
            (at("/sort"), thing(11, "Sort by", shown.clone(), &["/menu"])),
            (
                at("/menu"),
                thing(33, "", hidden.clone(), &["/name", "/size"]),
            ),
            (
                at("/name"),
                thing(35, "Name", selectable(showing.hidden(), "/name"), &[]),
            ),
            (
                at("/size"),
                thing(35, "Size", selectable(showing.hidden(), "/size"), &[]),
            ),
            (at("/list"), thing(31, "", shown, &["/report"])),
            (
                at("/report"),
                thing(32, "report.pdf", selectable(showing, "/report"), &[]),
            ),
        ])
    }
}

impl AccessibilityTree for OneWindow {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: at("/root"),
            is: Some(self.application.to_owned()),
        }])
    }

    fn facts(&self, place: &NodeAt) -> Result<Facts, TreeFault> {
        // **An application slow to show the choice**: marked only on the
        // third look after it was asked.
        let pending = self.counted.pending.borrow().clone();
        if pending.as_ref() == Some(place) {
            self.counted.looks.set(self.counted.looks.get() + 1);
            if self.counted.looks.get() >= 3 {
                self.counted.marked.borrow_mut().push(place.clone());
                self.counted.pending.borrow_mut().take();
            }
        }
        self.facts_of().remove(place).ok_or(TreeFault::Vanished)
    }

    fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
        Ok(String::new())
    }

    fn actions(&self, _: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(Vec::new())
    }

    fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
        panic!("a choice through a list that selects did an action")
    }

    fn select_in(&self, _: &NodeAt, place: &NodeAt) -> Result<Option<bool>, TreeFault> {
        self.counted.chosen.set(self.counted.chosen.get() + 1);
        if self.counted.does_not_answer.get() {
            return Err(TreeFault::DidNotAnswer);
        }
        if self.counted.slow.get() {
            *self.counted.pending.borrow_mut() = Some(place.clone());
        } else if !self.counted.leaves_it_unmarked.get() {
            self.counted.marked.borrow_mut().push(place.clone());
        }
        Ok(Some(true))
    }
}

/// The person's windows, as a test arranges them.
struct TheWindows {
    application: &'static str,
    counted: Counted,
}

impl TheWindowsOnTheScreen for TheWindows {
    fn opening(&mut self) -> Result<Box<dyn AccessibilityTree>, TreeFault> {
        self.counted.opened.set(self.counted.opened.get() + 1);
        Ok(Box::new(OneWindow {
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
    let folder = crate::testing::a_folder_of_our_own("choosing");
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

/// The arguments for choosing *Size* in *Sort by*.
fn choosing_size(application: &str) -> Vec<(&'static str, Given)> {
    vec![
        ("application", Given::text(application)),
        ("name", Given::text("Sort by")),
        ("option", Given::text("Size")),
    ]
}

/// The arguments for selecting `item`.
fn selecting(application: &str, item: &str) -> Vec<(&'static str, Given)> {
    vec![
        ("application", Given::text(application)),
        ("name", Given::text(item)),
    ]
}

fn entries(record: &Record) -> usize {
    record.answering(&Asking::anything()).count()
}

/// Propose, then approve.
fn approving(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    verb: &str,
    given: &[(&str, Given)],
) -> Result<WhatAVerbDid, NotDone> {
    let id = turning.proposing(verb, given, grants, hour(), noon())?;
    turning.approving(id, grants, noon())
}

#[test]
fn with_no_invocation_the_tree_is_never_opened() {
    let mut windows = windows_of(FILES);
    let opened = Rc::clone(&windows.counted.opened);
    let ((), _) = on_a_machine(&mut windows, the_files_are_granted, |_, _| ());
    assert_eq!(opened.get(), 0);
}

/// **At ask each time a choice waits, and one approval chooses once**: the
/// proposal opens nothing, the approval opens the tree once, one choice is
/// asked, it reads back chosen, and one entry is written.
#[test]
fn at_ask_each_time_one_approval_chooses_once() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let (did, record) = on_a_machine(&mut windows, the_files_are_granted, |turning, grants| {
        let given = choosing_size(FILES);
        let id = turning.proposing(CHOOSE_OPTION, &given, grants, hour(), noon())?;
        assert_eq!(counted.opened.get(), 0, "a proposal opened the tree");
        turning.approving(id, grants, noon())
    });
    let did = did.unwrap();
    let chosen = did.chosen().expect("an option was chosen");
    assert_eq!(
        (chosen.application(), chosen.name(), chosen.option()),
        (FILES, "Sort by", Some("Size"))
    );
    assert!(chosen.seen() && !chosen.unanswered());
    assert_eq!(counted.opened.get(), 1);
    assert_eq!(counted.chosen.get(), 1, "one approval, one choice");
    assert_eq!(entries(&record), 1);
}

/// **At full access it runs without a card, once**, and is recorded once.
#[test]
fn at_full_access_a_selection_runs_once_without_a_card() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let (changed, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let at_full_access = grants.at_full_access_for(&Grantee::named(AGENT));
            let changed = turning.changing(
                SELECT_ITEM,
                &selecting(FILES, "report.pdf"),
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
        panic!("a selection at full access did not run: {changed:?}");
    };
    assert_eq!(sentence.text(), "select “report.pdf” in org.kde.dolphin");
    let chosen = did.chosen().unwrap();
    assert_eq!((chosen.name(), chosen.option()), ("report.pdf", None));
    assert_eq!(counted.opened.get(), 1);
    assert_eq!(counted.chosen.get(), 1);
    assert_eq!(entries(&record), 1);
}

#[test]
fn a_declined_choice_chooses_nothing_and_is_recorded() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let (declined, record) =
        on_a_machine(&mut windows, the_files_are_granted, |turning, grants| {
            let id =
                turning.proposing(CHOOSE_OPTION, &choosing_size(FILES), grants, hour(), noon())?;
            turning.declining(id, noon())
        });
    assert!(declined.is_ok(), "{declined:?}");
    assert_eq!(counted.opened.get(), 0);
    assert_eq!(counted.chosen.get(), 0);
    assert_eq!(entries(&record), 1);
}

/// **A grant revoked between the approval and the choice stops it**, before
/// the tree is opened, and the refusal is written down.
#[test]
fn a_revoked_grant_stops_the_choice_before_the_tree() {
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
                SELECT_ITEM,
                &selecting(FILES, "report.pdf"),
                grants,
                hour(),
                noon(),
            )?;
            assert!(grants.revoke(id));
            turning.approving(proposed, grants, noon())
        },
    );
    assert!(matches!(did, Err(NotDone::Refused(_))), "{did:?}");
    assert_eq!(counted.opened.get(), 0);
    assert_eq!(counted.chosen.get(), 0);
    assert_eq!(entries(&record), 1);
}

/// **A terminal nobody allowed is refused even at full access**, before any
/// tree is opened, and the refusal is written down.
#[test]
fn a_terminal_nobody_allowed_is_never_chosen_in_even_at_full_access() {
    let mut windows = windows_of(TERMINAL);
    let counted = windows.counted.clone();
    let (changed, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let at_full_access = grants.at_full_access_for(&Grantee::named(AGENT));
            turning.changing(
                CHOOSE_OPTION,
                &choosing_size(TERMINAL),
                &at_full_access,
                hour(),
                &mut |_| None,
                noon(),
            )
        },
    );
    assert!(changed.is_err(), "a terminal was chosen in: {changed:?}");
    assert_eq!(counted.opened.get(), 0);
    assert_eq!(counted.chosen.get(), 0);
    assert_eq!(entries(&record), 1);
}

/// **An item not on the screen is refused after the tree is read**, at both
/// levels, with nothing chosen and one entry each.
#[test]
fn an_item_not_there_is_refused_at_both_levels_and_recorded() {
    let mut windows = windows_of(FILES);
    let counted = windows.counted.clone();
    let ((asked, at_full), record) =
        on_a_machine(&mut windows, the_files_are_granted, |turning, grants| {
            let given = selecting(FILES, "missing.txt");
            let asked = approving(turning, grants, SELECT_ITEM, &given);
            let at_full_access = grants.at_full_access_for(&Grantee::named(AGENT));
            let at_full = turning.changing(
                SELECT_ITEM,
                &given,
                &at_full_access,
                hour(),
                &mut |_| None,
                noon(),
            );
            (asked, at_full.map(|_| ()))
        });
    let Err(NotDone::Refused(refused)) = asked else {
        panic!("a missing item was selected: {asked:?}");
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
    assert_eq!(counted.chosen.get(), 0);
    assert_eq!(entries(&record), 2);
}

/// **A choice the window does not show is recorded as run, with what alo saw
/// beside it** — alo does not claim it worked.
#[test]
fn a_choice_the_window_does_not_show_is_run_and_says_so() {
    let mut windows = windows_of(FILES);
    windows.counted.leaves_it_unmarked.set(true);
    let counted = windows.counted.clone();
    let (did, record) = on_a_machine(&mut windows, the_files_are_granted, |turning, grants| {
        approving(turning, grants, CHOOSE_OPTION, &choosing_size(FILES))
    });
    let did = did.unwrap();
    let chosen = did.chosen().unwrap();
    assert!(!chosen.seen(), "{chosen:?}");
    assert_eq!(counted.chosen.get(), 1);
    assert_eq!(entries(&record), 1);
    let asking = Asking::anything();
    let entry = record.answering(&asking).next().unwrap();
    assert!(
        format!("{entry:?}").contains("does not show it chosen"),
        "{entry:?}"
    );
}

/// **The window is looked at again for a while, not once**: an application
/// that shows the choice only on the third look is seen to have made it.
#[test]
fn a_choice_shown_late_is_still_seen() {
    let mut windows = windows_of(FILES);
    windows.counted.slow.set(true);
    let (did, _) = on_a_machine(&mut windows, the_files_are_granted, |turning, grants| {
        approving(turning, grants, CHOOSE_OPTION, &choosing_size(FILES))
    });
    assert!(did.unwrap().chosen().unwrap().seen());
}

/// **An application that does not answer may have chosen**: recorded as run,
/// said as not known, and never said to be seen.
#[test]
fn an_application_that_does_not_answer_is_run_and_not_seen() {
    let mut windows = windows_of(FILES);
    windows.counted.does_not_answer.set(true);
    let (did, record) = on_a_machine(&mut windows, the_files_are_granted, |turning, grants| {
        approving(turning, grants, SELECT_ITEM, &selecting(FILES, "report.pdf"))
    });
    let did = did.unwrap();
    let chosen = did.chosen().unwrap();
    assert!(chosen.unanswered() && !chosen.seen(), "{chosen:?}");
    let asking = Asking::anything();
    let entry = record.answering(&asking).next().unwrap();
    assert!(format!("{entry:?}").contains("is not known"), "{entry:?}");
}
