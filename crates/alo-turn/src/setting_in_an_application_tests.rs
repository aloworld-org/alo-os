//! **Moving a slider or a number box, through a whole turn** — at ask each
//! time and at full access, and on every refusal — against a tree that counts
//! every time it is opened and every move it is asked for.
//!
//! - with no invocation the tree is never opened;
//! - at ask each time a move waits for one approval, and one approval moves
//!   once; a declined one moves nothing;
//! - at full access it runs with no card, once;
//! - a grant revoked before the move stops it before any tree is opened;
//! - a terminal nobody allowed is refused at full access too;
//! - a number outside the control's range is refused, recorded, at both
//!   levels;
//! - a number that reads back differently is recorded as run with what alo
//!   saw beside it.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use alo_adapters::{
    AccessibilityTree, Bounds, Facts, NodeAt, Running, SET_VALUE, States, TreeFault, WorkingFully,
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

/// The application every test moves a slider in.
const MIXER: &str = "org.example.Mixer";

/// A terminal, which alo reaches only where the person allowed it (ADR 0111).
const TERMINAL: &str = "app.devsuite.Ptyxis";

/// The agent.
const AGENT: &str = "@setting";

/// What every tree a test opens shares.
#[derive(Debug, Default, Clone)]
struct Counted {
    opened: Rc<Cell<usize>>,
    moves: Rc<Cell<usize>>,
    /// What the slider holds now.
    holds: Rc<RefCell<Option<f64>>>,
    /// What a move leaves it at, where not the number asked for.
    rounds_to: Rc<Cell<Option<f64>>>,
    /// Whether the application shows the number only on the third look.
    slow: Rc<Cell<bool>>,
    looks: Rc<Cell<usize>>,
    /// Whether the application does not answer the move.
    does_not_answer: Rc<Cell<bool>>,
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.5".to_owned(),
        object: object.to_owned(),
    }
}

/// A window *Mixer* with a slider *Volume* from 0 to 100.
struct OneWindow {
    application: &'static str,
    counted: Counted,
}

impl AccessibilityTree for OneWindow {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: at("/root"),
            is: Some(self.application.to_owned()),
        }])
    }

    fn facts(&self, place: &NodeAt) -> Result<Facts, TreeFault> {
        let showing = States::showing_and_usable().as_the_tree_says().to_vec();
        let thing = |role, name: &str, children: &[&str]| Facts {
            role,
            name: name.to_owned(),
            states: showing.clone(),
            children: children.iter().map(|child| at(child)).collect(),
            has_text: false,
            unlisted_children: false,
        };
        match place.object.as_str() {
            "/root" => Ok(thing(75, "", &["/win"])),
            "/win" => Ok(thing(23, "Mixer", &["/volume"])),
            "/volume" => Ok(thing(51, "Volume", &[])),
            _ => Err(TreeFault::Vanished),
        }
    }

    fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
        Ok(String::new())
    }

    fn actions(&self, _: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(Vec::new())
    }

    fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
        panic!("a slider was pressed rather than moved")
    }

    fn value(&self, _: &NodeAt) -> Result<Option<f64>, TreeFault> {
        if self.counted.slow.get() && self.counted.moves.get() > 0 {
            self.counted.looks.set(self.counted.looks.get() + 1);
            if self.counted.looks.get() < 3 {
                return Ok(Some(30.0));
            }
        }
        Ok(*self.counted.holds.borrow())
    }

    fn range(&self, _: &NodeAt) -> Result<Option<Bounds>, TreeFault> {
        Ok(Some(Bounds {
            least: 0.0,
            most: 100.0,
            step: 1.0,
        }))
    }

    fn set_value(&self, _: &NodeAt, value: f64) -> Result<bool, TreeFault> {
        self.counted.moves.set(self.counted.moves.get() + 1);
        if self.counted.does_not_answer.get() {
            return Err(TreeFault::DidNotAnswer);
        }
        *self.counted.holds.borrow_mut() = Some(self.counted.rounds_to.get().unwrap_or(value));
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
    let folder = crate::testing::a_folder_of_our_own("setting");
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
                    Application::identified(MIXER).unwrap(),
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

fn the_mixer_is_granted(grants: &mut Grants) {
    grants.grant(
        Grant::checked(AGENT, Reach::Application(MIXER.to_owned()), noon(), hour()).unwrap(),
    );
}

fn windows_of(application: &'static str) -> TheWindows {
    TheWindows {
        application,
        counted: Counted::default(),
    }
}

/// The arguments for moving *Volume* in `application` to `value`.
fn volume(application: &str, value: i64) -> Vec<(&'static str, Given)> {
    vec![
        ("application", Given::text(application)),
        ("name", Given::text("Volume")),
        ("value", Given::number(value)),
    ]
}

fn entries(record: &Record) -> usize {
    record.answering(&Asking::anything()).count()
}

fn approving(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    given: &[(&str, Given)],
) -> Result<WhatAVerbDid, NotDone> {
    let id = turning.proposing(SET_VALUE, given, grants, hour(), noon())?;
    turning.approving(id, grants, noon())
}

#[test]
fn with_no_invocation_the_tree_is_never_opened() {
    let mut windows = windows_of(MIXER);
    let opened = Rc::clone(&windows.counted.opened);
    let ((), _) = on_a_machine(&mut windows, the_mixer_is_granted, |_, _| ());
    assert_eq!(opened.get(), 0);
}

/// **At ask each time a move waits, and one approval moves once.**
#[test]
fn at_ask_each_time_one_approval_moves_once() {
    let mut windows = windows_of(MIXER);
    let counted = windows.counted.clone();
    let (did, record) = on_a_machine(&mut windows, the_mixer_is_granted, |turning, grants| {
        let id = turning.proposing(SET_VALUE, &volume(MIXER, 40), grants, hour(), noon())?;
        assert_eq!(counted.opened.get(), 0, "a proposal opened the tree");
        turning.approving(id, grants, noon())
    });
    let did = did.unwrap();
    let set = did.set().expect("a slider was moved");
    assert_eq!((set.application(), set.name(), set.value()), (MIXER, "Volume", 40));
    assert!(set.seen() && !set.unanswered());
    assert_eq!(counted.opened.get(), 1);
    assert_eq!(counted.moves.get(), 1, "one approval, one move");
    assert_eq!(entries(&record), 1);
}

/// **At full access it runs without a card, once.**
#[test]
fn at_full_access_a_move_runs_once_without_a_card() {
    let mut windows = windows_of(MIXER);
    let counted = windows.counted.clone();
    let (changed, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let at_full_access = grants.at_full_access_for(&Grantee::named(AGENT));
            let changed = turning.changing(
                SET_VALUE,
                &volume(MIXER, 70),
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
        panic!("a move at full access did not run: {changed:?}");
    };
    assert_eq!(sentence.text(), "move “Volume” in org.example.Mixer to 70");
    assert_eq!(did.set().map(crate::AValueSet::value), Some(70));
    assert_eq!(counted.moves.get(), 1);
    assert_eq!(entries(&record), 1);
}

#[test]
fn a_declined_move_moves_nothing_and_is_recorded() {
    let mut windows = windows_of(MIXER);
    let counted = windows.counted.clone();
    let (declined, record) = on_a_machine(&mut windows, the_mixer_is_granted, |turning, grants| {
        let id = turning.proposing(SET_VALUE, &volume(MIXER, 40), grants, hour(), noon())?;
        turning.declining(id, noon())
    });
    assert!(declined.is_ok(), "{declined:?}");
    assert_eq!(counted.opened.get(), 0);
    assert_eq!(counted.moves.get(), 0);
    assert_eq!(entries(&record), 1);
}

#[test]
fn a_revoked_grant_stops_the_move_before_the_tree() {
    let mut windows = windows_of(MIXER);
    let counted = windows.counted.clone();
    let (did, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let id = grants.grant(
                Grant::checked(AGENT, Reach::Application(MIXER.to_owned()), noon(), hour())
                    .unwrap(),
            );
            let proposed = turning.proposing(SET_VALUE, &volume(MIXER, 40), grants, hour(), noon())?;
            assert!(grants.revoke(id));
            turning.approving(proposed, grants, noon())
        },
    );
    assert!(matches!(did, Err(NotDone::Refused(_))), "{did:?}");
    assert_eq!(counted.opened.get(), 0);
    assert_eq!(counted.moves.get(), 0);
    assert_eq!(entries(&record), 1);
}

#[test]
fn a_terminal_nobody_allowed_is_never_moved_in_even_at_full_access() {
    let mut windows = windows_of(TERMINAL);
    let counted = windows.counted.clone();
    let (changed, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let at_full_access = grants.at_full_access_for(&Grantee::named(AGENT));
            turning.changing(
                SET_VALUE,
                &volume(TERMINAL, 40),
                &at_full_access,
                hour(),
                &mut |_| None,
                noon(),
            )
        },
    );
    assert!(changed.is_err(), "a terminal was moved in: {changed:?}");
    assert_eq!(counted.opened.get(), 0);
    assert_eq!(counted.moves.get(), 0);
    assert_eq!(entries(&record), 1);
}

/// **A number outside the slider's range is refused at both levels**, with
/// nothing moved and each refusal recorded.
#[test]
fn a_number_outside_the_range_is_refused_at_both_levels_and_recorded() {
    let mut windows = windows_of(MIXER);
    let counted = windows.counted.clone();
    let ((asked, at_full), record) =
        on_a_machine(&mut windows, the_mixer_is_granted, |turning, grants| {
            let given = volume(MIXER, 150);
            let asked = approving(turning, grants, &given);
            let at_full_access = grants.at_full_access_for(&Grantee::named(AGENT));
            let at_full = turning.changing(
                SET_VALUE,
                &given,
                &at_full_access,
                hour(),
                &mut |_| None,
                noon(),
            );
            (asked, at_full.map(|_| ()))
        });
    let Err(NotDone::Refused(refused)) = asked else {
        panic!("a number outside the range was set: {asked:?}");
    };
    let said = refused.said(&in_english()).text().to_owned();
    assert!(said.contains("goes from 0 to 100"), "{said}");
    assert!(matches!(at_full, Err(NotDone::Refused(_))), "{at_full:?}");
    assert_eq!(counted.moves.get(), 0);
    assert_eq!(entries(&record), 2);
}

/// **A number that reads back differently is recorded as run, with what alo
/// saw beside it** — alo does not claim it worked.
#[test]
fn a_number_that_reads_back_differently_is_run_and_says_so() {
    let mut windows = windows_of(MIXER);
    windows.counted.rounds_to.set(Some(45.0));
    let counted = windows.counted.clone();
    let (did, record) = on_a_machine(&mut windows, the_mixer_is_granted, |turning, grants| {
        approving(turning, grants, &volume(MIXER, 40))
    });
    let did = did.unwrap();
    assert!(!did.set().unwrap().seen());
    assert_eq!(counted.moves.get(), 1);
    assert_eq!(entries(&record), 1);
    let asking = Asking::anything();
    let entry = record.answering(&asking).next().unwrap();
    assert!(
        format!("{entry:?}").contains("shows a different number"),
        "{entry:?}"
    );
}

/// **The control is read again for a while, not once**: one that shows the
/// number only on the third look is seen to have moved.
#[test]
fn a_number_shown_late_is_still_seen() {
    let mut windows = windows_of(MIXER);
    windows.counted.slow.set(true);
    let (did, _) = on_a_machine(&mut windows, the_mixer_is_granted, |turning, grants| {
        approving(turning, grants, &volume(MIXER, 40))
    });
    assert!(did.unwrap().set().unwrap().seen());
}

/// **An application that does not answer may have moved it**: recorded as
/// run, said as not known, never said to be seen.
#[test]
fn an_application_that_does_not_answer_is_run_and_not_seen() {
    let mut windows = windows_of(MIXER);
    windows.counted.does_not_answer.set(true);
    let (did, record) = on_a_machine(&mut windows, the_mixer_is_granted, |turning, grants| {
        approving(turning, grants, &volume(MIXER, 40))
    });
    let did = did.unwrap();
    let set = did.set().unwrap();
    assert!(set.unanswered() && !set.seen(), "{set:?}");
    let asking = Asking::anything();
    let entry = record.answering(&asking).next().unwrap();
    assert!(format!("{entry:?}").contains("is not known"), "{entry:?}");
}
