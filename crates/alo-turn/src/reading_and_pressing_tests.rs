//! **The two accessibility verbs, through a whole turn** — each capability
//! guarantee as a test, against a tree that counts every time it is opened and
//! every press it is asked for.
//!
//! - with no invocation the tree is never opened;
//! - a read answers inside the turn, opens the tree once, is written down, and
//!   never carries a password field's contents;
//! - a press waits for one approval, and one approval presses once;
//! - a grant revoked between the approval and the press stops it before any
//!   tree is opened;
//! - every read, every press and every refusal leaves exactly one entry;
//! - a terminal the person has not allowed their agent is neither read nor
//!   pressed, even at full access;
//! - where a switch the person can turn on would help, the refusal names it.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

use alo_adapters::{
    ACTIVATE_CONTROL, AccessibilityTree, Facts, NodeAt, READ_WINDOW, Running, States, TreeFault,
    WorkingFully,
};
use alo_applications::{Application, Installed};
use alo_capability::{Given, Grant, Grantee, Grants, Reach};
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

/// The application every test reads and presses in.
const MAIL: &str = "org.example.Mail";

/// A terminal, which alo reaches only where the person allowed it (ADR 0111).
const TERMINAL: &str = "app.devsuite.Ptyxis";

/// What is typed into the mail application's password field.
const SECRET: &str = "correct-horse-battery-staple";

/// The agent.
const AGENT: &str = "@windows";

// ---------------------------------------------------------------------------
// A tree that is counted.
// ---------------------------------------------------------------------------

/// What every tree a test opens shares: how many times one was opened, and how
/// many presses were asked of all of them.
#[derive(Debug, Default, Clone)]
struct Counted {
    opened: Rc<Cell<usize>>,
    presses: Rc<Cell<usize>>,
    texts_of_a_password: Rc<Cell<usize>>,
}

/// One application's window: the name it shows, a button, and — where
/// `with_text` — words and a password field.
struct OneWindow {
    application: &'static str,
    with_text: bool,
    actions: &'static [&'static str],
    counted: Counted,
}

impl OneWindow {
    fn at(object: &str) -> NodeAt {
        NodeAt {
            holder: ":1.5".to_owned(),
            object: object.to_owned(),
        }
    }

    fn facts_of(&self) -> BTreeMap<NodeAt, Facts> {
        let showing = States::showing_and_usable().as_the_tree_says().to_vec();
        let thing = |role, name: &str, children: Vec<NodeAt>, has_text| Facts {
            role,
            name: name.to_owned(),
            states: showing.clone(),
            children,
            has_text,
            unlisted_children: false,
        };
        let mut inside = vec![Self::at("/sign-in")];
        if self.with_text {
            inside.push(Self::at("/hello"));
            inside.push(Self::at("/password"));
        }
        BTreeMap::from([
            (
                Self::at("/root"),
                thing(75, "", vec![Self::at("/win")], false),
            ),
            (Self::at("/win"), thing(23, "Inbox", inside, false)),
            (
                Self::at("/sign-in"),
                thing(43, "Sign in", Vec::new(), false),
            ),
            (Self::at("/hello"), thing(29, "Hello", Vec::new(), false)),
            (
                Self::at("/password"),
                thing(40, "Password", Vec::new(), true),
            ),
        ])
    }
}

impl AccessibilityTree for OneWindow {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(vec![Running {
            at: Self::at("/root"),
            is: Some(self.application.to_owned()),
        }])
    }

    fn facts(&self, at: &NodeAt) -> Result<Facts, TreeFault> {
        self.facts_of().remove(at).ok_or(TreeFault::Vanished)
    }

    fn text(&self, at: &NodeAt) -> Result<String, TreeFault> {
        if at.object == "/password" {
            self.counted
                .texts_of_a_password
                .set(self.counted.texts_of_a_password.get() + 1);
            return Ok(SECRET.to_owned());
        }
        Ok(String::new())
    }

    fn actions(&self, at: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(if at.object == "/sign-in" {
            self.actions
                .iter()
                .map(|action| (*action).to_owned())
                .collect()
        } else {
            Vec::new()
        })
    }

    fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
        self.counted.presses.set(self.counted.presses.get() + 1);
        Ok(true)
    }
}

/// The person's windows, as a test arranges them.
struct TheWindows {
    application: &'static str,
    with_text: bool,
    actions: &'static [&'static str],
    fully: WorkingFully,
    counted: Counted,
}

impl TheWindows {
    fn of(application: &'static str) -> Self {
        Self {
            application,
            with_text: true,
            actions: &["click"],
            fully: WorkingFully::NothingMoreToTurnOn,
            counted: Counted::default(),
        }
    }
}

impl TheWindowsOnTheScreen for TheWindows {
    fn opening(&mut self) -> Result<Box<dyn AccessibilityTree>, TreeFault> {
        self.counted.opened.set(self.counted.opened.get() + 1);
        Ok(Box::new(OneWindow {
            application: self.application,
            with_text: self.with_text,
            actions: self.actions,
            counted: self.counted.clone(),
        }))
    }

    fn working_fully(&self, _: &str) -> WorkingFully {
        self.fully
    }
}

// ---------------------------------------------------------------------------
// A machine to run turns on.
// ---------------------------------------------------------------------------

/// A tool nothing here reaches; every change it could make refuses.
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
        panic!("nothing here acts on an application's window")
    }
}

/// Nothing here starts an application.
struct NothingStarts;

impl SaysHowAnApplicationStarts for NothingStarts {
    fn how_it_starts(&self, _: &str) -> Result<TheRoad, NoRoad> {
        Err(NoRoad::NoEntry)
    }
}

/// Run `doing` in a turn on a machine with these windows, the mail application
/// and a terminal installed, and the grants `granting` makes; answer what it
/// answered and the record.
fn on_a_machine<T>(
    windows: &mut TheWindows,
    granting: impl FnOnce(&mut Grants),
    doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants) -> T,
) -> (T, Record) {
    let strings = in_english();
    let tool = NoTool;
    let mut door = NoDoor;
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let mut bounding = crate::testing::NothingIsBounded;
    let folder = crate::testing::a_folder_of_our_own("windows");
    let there = folder.join("a-note.txt");
    std::fs::write(&there, "anything").unwrap();
    let answered = {
        let mut machine = Machine::carrying_out_file_software_and_application_verbs(
            &strings,
            &OnThisMachine,
            WhatASoftwareVerbNeeds {
                tool: &tool,
                bound: Bound::Nobodys,
            },
            WhatAnApplicationVerbNeeds {
                installed: Installed::holding([
                    Application::identified(MAIL).unwrap(),
                    Application::identified(TERMINAL).unwrap(),
                ]),
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

/// A grant to the agent over the mail application, for an hour from noon.
fn the_mail_is_granted(grants: &mut Grants) {
    grants
        .grant(Grant::checked(AGENT, Reach::Application(MAIL.to_owned()), noon(), hour()).unwrap());
}

/// Read the application's windows, inside the turn.
fn reading(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    application: &str,
) -> Result<crate::WhatAVerbDid, NotDone> {
    turning.reading(
        READ_WINDOW,
        &[("application", Given::text(application))],
        grants,
        noon(),
    )
}

/// Propose pressing the button named *Sign in* in `application`.
fn proposing(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    application: &str,
) -> Result<alo_capability::ProposalId, NotDone> {
    turning.proposing(
        ACTIVATE_CONTROL,
        &[
            ("application", Given::text(application)),
            ("kind", Given::text("button")),
            ("name", Given::text("Sign in")),
        ],
        grants,
        hour(),
        noon(),
    )
}

/// How many entries the record holds.
fn entries(record: &Record) -> usize {
    record.answering(&Asking::anything()).count()
}

// ---------------------------------------------------------------------------
// The guarantees.
// ---------------------------------------------------------------------------

/// **With no invocation the tree is never opened**: a machine that offers
/// both verbs is made, a turn is begun and ended with the mail application
/// granted — and the person's windows were never asked anything.
#[test]
fn with_no_invocation_the_tree_is_never_opened() {
    let mut windows = TheWindows::of(MAIL);
    let opened = Rc::clone(&windows.counted.opened);
    let ((), _) = on_a_machine(&mut windows, the_mail_is_granted, |_, _| ());
    assert_eq!(opened.get(), 0, "the tree was opened with no read or press");
}

/// **A read answers inside the turn**, opens the tree once, is written down
/// once — and what it answers never holds a password field's contents, which
/// were never even asked for.
#[test]
fn a_read_answers_inside_the_turn_once_and_never_holds_a_password() {
    let mut windows = TheWindows::of(MAIL);
    let counted = windows.counted.clone();
    let (read, record) = on_a_machine(&mut windows, the_mail_is_granted, |turning, grants| {
        reading(turning, grants, MAIL)
    });
    let read = read.unwrap();
    let showed = read.the_windows_showed().expect("the windows were read");
    let titles: Vec<&str> = showed
        .shown()
        .windows()
        .iter()
        .map(alo_adapters::ShownWindow::title)
        .collect();
    assert_eq!(titles, ["Inbox"]);
    assert!(showed.shown().shows_any_text());
    assert!(!format!("{showed:?}").contains(SECRET), "{showed:?}");
    assert_eq!(counted.texts_of_a_password.get(), 0);
    assert_eq!(counted.opened.get(), 1);
    assert_eq!(counted.presses.get(), 0, "a read pressed something");
    assert_eq!(entries(&record), 1);
}

/// **An application nobody granted is refused before the tree is opened**, and
/// the refusal is written down.
#[test]
fn an_ungranted_read_opens_nothing_and_is_recorded() {
    let mut windows = TheWindows::of(MAIL);
    let opened = Rc::clone(&windows.counted.opened);
    let (read, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| reading(turning, grants, MAIL),
    );
    assert!(matches!(read, Err(NotDone::Refused(_))), "{read:?}");
    assert_eq!(opened.get(), 0);
    assert_eq!(entries(&record), 1);
}

/// **A read that finds no text says which switch would help**, and only the
/// one that would.
#[test]
fn a_read_that_finds_no_text_names_the_switch_that_would_help() {
    for (fully, names) in [
        (
            WorkingFully::CouldBeTurnedOn,
            "Let alo work fully inside apps like VSCodium",
        ),
        (
            WorkingFully::NothingMoreToTurnOn,
            "Apps describe themselves",
        ),
    ] {
        let mut windows = TheWindows::of(MAIL);
        windows.with_text = false;
        windows.fully = fully;
        let (read, _) = on_a_machine(&mut windows, the_mail_is_granted, |turning, grants| {
            reading(turning, grants, MAIL)
        });
        let read = read.unwrap();
        let told = read.the_windows_showed().unwrap().told().join("\n");
        assert!(told.contains("“Inbox”"), "{fully:?}: {told}");
        assert!(told.contains(names), "{fully:?}: {told}");
    }
}

/// **A press waits, and one approval presses once**: nothing is opened while
/// the question stands, the tree is opened once at the approval, one press is
/// asked, and one entry is written.
#[test]
fn one_approval_presses_once_and_nothing_before_it() {
    let mut windows = TheWindows::of(MAIL);
    let counted = windows.counted.clone();
    let (did, record) = on_a_machine(&mut windows, the_mail_is_granted, |turning, grants| {
        let id = proposing(turning, grants, MAIL)?;
        assert_eq!(counted.opened.get(), 0, "a proposal opened the tree");
        turning.approving(id, grants, noon())
    });
    let did = did.unwrap();
    let pressed = did.pressed().expect("a control was pressed");
    assert_eq!(
        (pressed.application(), pressed.kind(), pressed.name()),
        (MAIL, "button", "Sign in")
    );
    assert!(!pressed.unanswered());
    assert_eq!(counted.opened.get(), 1);
    assert_eq!(counted.presses.get(), 1, "one approval, one press");
    assert_eq!(entries(&record), 1);
}

/// **A press the person declined presses nothing, and the no is written
/// down.**
#[test]
fn a_declined_press_presses_nothing_and_is_recorded() {
    let mut windows = TheWindows::of(MAIL);
    let counted = windows.counted.clone();
    let (declined, record) = on_a_machine(&mut windows, the_mail_is_granted, |turning, grants| {
        let id = proposing(turning, grants, MAIL)?;
        turning.declining(id, noon())
    });
    assert!(declined.is_ok(), "{declined:?}");
    assert_eq!(counted.opened.get(), 0);
    assert_eq!(counted.presses.get(), 0);
    assert_eq!(entries(&record), 1);
}

/// **A grant revoked between the approval and the press stops it**, before the
/// tree is opened, and the refusal is written down.
#[test]
fn a_revoked_grant_stops_the_next_press() {
    let mut windows = TheWindows::of(MAIL);
    let counted = windows.counted.clone();
    let (did, record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let id = grants.grant(
                Grant::checked(AGENT, Reach::Application(MAIL.to_owned()), noon(), hour()).unwrap(),
            );
            let proposed = proposing(turning, grants, MAIL)?;
            assert!(grants.revoke(id), "the grant was there to revoke");
            turning.approving(proposed, grants, noon())
        },
    );
    assert!(matches!(did, Err(NotDone::Refused(_))), "{did:?}");
    assert_eq!(counted.opened.get(), 0, "a revoked grant opened the tree");
    assert_eq!(counted.presses.get(), 0);
    assert_eq!(entries(&record), 1);
}

/// **A terminal the person has not allowed their agent is neither read nor
/// pressed, even at full access** (ADR 0111): refused before any tree is
/// opened, and each refusal written down.
#[test]
fn a_terminal_nobody_allowed_is_never_read_or_pressed_even_at_full_access() {
    let mut windows = TheWindows::of(TERMINAL);
    let counted = windows.counted.clone();
    let ((read, proposed), record) = on_a_machine(
        &mut windows,
        |_| {},
        |turning, grants| {
            let at_full_access = grants.at_full_access_for(&Grantee::named(AGENT));
            (
                reading(turning, &at_full_access, TERMINAL),
                proposing(turning, &at_full_access, TERMINAL),
            )
        },
    );
    assert!(read.is_err(), "a terminal was read: {read:?}");
    assert!(
        proposed.is_err(),
        "a press in a terminal was put to the person"
    );
    assert_eq!(counted.opened.get(), 0);
    assert_eq!(counted.presses.get(), 0);
    assert_eq!(entries(&record), 2);
}

/// **A control that offers no press names *Let alo work fully inside apps like
/// VSCodium* where turning it on would give it one**, and the plain sentence
/// where it would not; nothing is pressed either way, and each is recorded.
#[test]
fn a_press_that_cannot_be_given_names_the_switch_only_where_it_would_help() {
    for (fully, names_it) in [
        (WorkingFully::CouldBeTurnedOn, true),
        (WorkingFully::NothingMoreToTurnOn, false),
    ] {
        let mut windows = TheWindows::of(MAIL);
        windows.actions = &["doDefault"];
        windows.fully = fully;
        let counted = windows.counted.clone();
        let (did, record) = on_a_machine(&mut windows, the_mail_is_granted, |turning, grants| {
            let id = proposing(turning, grants, MAIL)?;
            turning.approving(id, grants, noon())
        });
        let Err(NotDone::Refused(refused)) = did else {
            panic!("{fully:?}: a control with no press was pressed: {did:?}");
        };
        let said = refused.said(&in_english()).text().to_owned();
        assert_eq!(
            said.contains("“Let alo work fully inside apps like VSCodium”"),
            names_it,
            "{fully:?}: {said}"
        );
        assert_eq!(counted.presses.get(), 0);
        assert_eq!(entries(&record), 1);
    }
}
