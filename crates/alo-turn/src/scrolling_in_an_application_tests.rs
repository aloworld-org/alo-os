//! **Scrolling, through a whole turn**, at both levels a person can choose —
//! each guarantee as a test, against a tree that counts every time it is
//! opened and every scroll it is asked for.
//!
//! - with no invocation the tree is never opened;
//! - at *ask each time* a scroll waits for one approval, and one approval
//!   scrolls once; a declined one scrolls nothing;
//! - at full access it scrolls at once, once, with nothing left waiting;
//! - a grant revoked between the approval and the scroll stops it before any
//!   tree is opened;
//! - a terminal the person has not allowed their agent is never scrolled, even
//!   at full access;
//! - every scroll and every refusal leaves exactly one entry.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

use alo_adapters::{
    AccessibilityTree, Edge, Facts, NodeAt, Running, SCROLL_PAGE, SCROLL_TO, TreeFault,
    WorkingFully,
};
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

/// The application every test scrolls in.
const READER: &str = "org.gnome.Papers";

/// A terminal, which alo reaches only where the person allowed it (ADR 0111).
const TERMINAL: &str = "app.devsuite.Ptyxis";

/// The agent.
const AGENT: &str = "@scrolling";

/// On the screen; described but scrolled away.
const ON: u32 = (1 << 25) | (1 << 30);
const OFF: u32 = 1 << 30;

/// How many times a tree was opened, and how many scrolls were asked.
#[derive(Debug, Default, Clone)]
struct Counted {
    opened: Rc<Cell<usize>>,
    scrolls: Rc<Cell<usize>>,
}

/// One window of a long document: its top on the screen, *Chapter 3* below.
struct ALongDocument {
    application: &'static str,
    counted: Counted,
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.5".to_owned(),
        object: object.to_owned(),
    }
}

impl ALongDocument {
    fn facts_of() -> BTreeMap<NodeAt, Facts> {
        let thing = |role, name: &str, states: u32, children: Vec<NodeAt>| Facts {
            role,
            name: name.to_owned(),
            states: vec![states, 0],
            children,
            has_text: false,
            unlisted_children: false,
        };
        BTreeMap::from([
            (at("/root"), thing(75, "", ON, vec![at("/win")])),
            (at("/win"), thing(23, "report.pdf", ON, vec![at("/doc")])),
            (
                at("/doc"),
                thing(82, "report.pdf", ON, vec![at("/top"), at("/ch3")]),
            ),
            (at("/top"), thing(83, "Summary", ON, Vec::new())),
            (at("/ch3"), thing(83, "Chapter 3", OFF, Vec::new())),
        ])
    }
}

impl AccessibilityTree for ALongDocument {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: at("/root"),
            is: Some(self.application.to_owned()),
        }])
    }

    fn facts(&self, place: &NodeAt) -> Result<Facts, TreeFault> {
        let mut facts = Self::facts_of().remove(place).ok_or(TreeFault::Vanished)?;
        // Once scrolled, everything reads as on the screen.
        if self.counted.scrolls.get() > 0 {
            facts.states = vec![ON, 0];
        }
        Ok(facts)
    }

    fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
        panic!("a scroll never reads a text")
    }

    fn actions(&self, _: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(Vec::new())
    }

    fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
        panic!("a scroll never presses anything")
    }

    fn scroll_to(&self, _: &NodeAt, _: Edge) -> Result<bool, TreeFault> {
        self.counted.scrolls.set(self.counted.scrolls.get() + 1);
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
        Ok(Box::new(ALongDocument {
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

/// A compositor's door nothing here knocks on: scrolling goes by the tree.
struct NoDoor;

impl TheDoorTheCompositorServes for NoDoor {
    fn asking(&mut self, _: &alo_acting::Request) -> Result<alo_acting::Answer, NotAtTheDoor> {
        panic!("a scroll never knocks on the compositor's door")
    }
}

/// Nothing here starts an application.
struct NothingStarts;

impl SaysHowAnApplicationStarts for NothingStarts {
    fn how_it_starts(&self, _: &str) -> Result<TheRoad, NoRoad> {
        Err(NoRoad::NoEntry)
    }
}

/// Run `doing` in a turn on a machine with these windows, the reader and a
/// terminal installed, and the grants `granting` makes.
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
    let folder = crate::testing::a_folder_of_our_own("scrolling");
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
                    Application::identified(READER).unwrap(),
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

fn the_reader_is_granted(grants: &mut Grants) {
    grants.grant(
        Grant::checked(AGENT, Reach::Application(READER.to_owned()), noon(), hour()).unwrap(),
    );
}

fn windows_of(application: &'static str) -> TheWindows {
    TheWindows {
        application,
        counted: Counted::default(),
    }
}

/// The two scrolls, as a model asks for them.
fn to_chapter_3(application: &str) -> (&'static str, Vec<(&'static str, Given)>) {
    (
        SCROLL_TO,
        vec![
            ("application", Given::text(application)),
            ("name", Given::text("Chapter 3")),
        ],
    )
}

fn a_page_down(application: &str) -> (&'static str, Vec<(&'static str, Given)>) {
    (
        SCROLL_PAGE,
        vec![
            ("application", Given::text(application)),
            ("toward", Given::text("down")),
        ],
    )
}

fn entries(record: &Record) -> usize {
    record.answering(&Asking::anything()).count()
}

#[test]
fn with_no_invocation_the_tree_is_never_opened() {
    let mut windows = windows_of(READER);
    let counted = windows.counted.clone();
    let ((), record) = on_a_machine(&mut windows, the_reader_is_granted, |_, _| ());
    assert_eq!(counted.opened.get(), 0);
    assert_eq!(entries(&record), 0);
}

#[test]
fn at_ask_each_time_one_approval_scrolls_once_and_nothing_before_it() {
    for (verb, given) in [to_chapter_3(READER), a_page_down(READER)] {
        let mut windows = windows_of(READER);
        let counted = windows.counted.clone();
        let (scrolled, record) =
            on_a_machine(&mut windows, the_reader_is_granted, |turning, grants| {
                let id = turning
                    .proposing(verb, &given, grants, hour(), noon())
                    .unwrap();
                assert_eq!(
                    counted.opened.get(),
                    0,
                    "{verb}: opened before the approval"
                );
                let did = turning.approving(id, grants, noon()).unwrap();
                assert!(turning.approving(id, grants, noon()).is_err());
                did
            });
        let scrolled = scrolled.scrolled().unwrap().clone();
        assert_eq!(scrolled.application(), READER);
        assert!(scrolled.now_showing());
        assert_eq!(counted.scrolls.get(), 1, "{verb}");
        assert_eq!(counted.opened.get(), 1, "{verb}");
        assert_eq!(entries(&record), 1, "{verb}");
    }
}

#[test]
fn a_declined_scroll_scrolls_nothing_and_is_recorded() {
    let mut windows = windows_of(READER);
    let counted = windows.counted.clone();
    let ((), record) = on_a_machine(&mut windows, the_reader_is_granted, |turning, grants| {
        let (verb, given) = to_chapter_3(READER);
        let id = turning
            .proposing(verb, &given, grants, hour(), noon())
            .unwrap();
        turning.declining(id, noon()).unwrap();
    });
    assert_eq!(counted.opened.get(), 0);
    assert_eq!(counted.scrolls.get(), 0);
    assert_eq!(entries(&record), 1);
}

#[test]
fn at_full_access_a_scroll_runs_at_once_and_once() {
    for (verb, given) in [to_chapter_3(READER), a_page_down(READER)] {
        let mut windows = windows_of(READER);
        let counted = windows.counted.clone();
        let (changed, record) = on_a_machine(
            &mut windows,
            |_| {},
            |turning, grants| {
                let at = grants.at_full_access_for(&Grantee::named(AGENT));
                let changed = turning
                    .changing(verb, &given, &at, hour(), &mut |_: &Said| None, noon())
                    .unwrap();
                assert_eq!(turning.waiting_at(noon()).count(), 0);
                changed
            },
        );
        let Changed::Done { did, sentence } = changed else {
            panic!("{verb} waited at full access")
        };
        assert!(did.scrolled().is_some(), "{verb}");
        assert!(sentence.text().contains(READER), "{sentence}");
        assert_eq!(counted.scrolls.get(), 1, "{verb}");
        assert_eq!(entries(&record), 1, "{verb}");
    }
}

#[test]
fn a_revoked_grant_stops_the_scroll_before_any_window_is_opened() {
    let mut windows = windows_of(READER);
    let counted = windows.counted.clone();
    let (refused, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let granted = grants.grant(
                Grant::checked(AGENT, Reach::Application(READER.to_owned()), noon(), hour())
                    .unwrap(),
            );
            let (verb, given) = to_chapter_3(READER);
            let id = turning
                .proposing(verb, &given, grants, hour(), noon())
                .unwrap();
            assert!(grants.revoke(granted), "the grant was there to revoke");
            turning.approving(id, grants, noon())
        },
    );
    assert!(matches!(refused, Err(NotDone::Refused(_))), "{refused:?}");
    assert_eq!(counted.opened.get(), 0);
    assert_eq!(counted.scrolls.get(), 0);
    assert_eq!(entries(&record), 1, "the refusal is written down");
}

#[test]
fn a_terminal_nobody_allowed_is_never_scrolled_even_at_full_access() {
    let mut windows = windows_of(TERMINAL);
    let counted = windows.counted.clone();
    let (refused, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let at = grants.at_full_access_for(&Grantee::named(AGENT));
            let (verb, given) = to_chapter_3(TERMINAL);
            turning
                .changing(verb, &given, &at, hour(), &mut |_: &Said| None, noon())
                .map(|_| ())
        },
    );
    assert!(matches!(refused, Err(NotDone::Refused(_))), "{refused:?}");
    assert_eq!(counted.opened.get(), 0);
    assert_eq!(counted.scrolls.get(), 0);
    assert_eq!(entries(&record), 1, "the refusal is written down");
}
