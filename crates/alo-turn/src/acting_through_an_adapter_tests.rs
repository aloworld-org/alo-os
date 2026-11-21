//! **alo asks Text Editor's adapter to open a document**: where alo may work
//! is asked before the card, the desktop is told before the message goes, and
//! one approval sends one message.
//!
//! What a person meets, in the order the coordinator's ruling of 2026-11-21
//! (option A) put it:
//!
//! - at *alo works in its own copy*, Text Editor (which has no copy of its
//!   own) is refused **before any card**, in the sentence every other road
//!   says for it, and nothing is sent;
//! - at the default, the desktop is asked `work-in-their-window` before the
//!   message goes and told `done` after it, so the border alo's work wears is
//!   up while it does;
//! - one approval sends exactly one message, to Text Editor, and is recorded;
//! - a desktop that refuses, or is not there, sends nothing.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::rc::Rc;

use alo_acting::{ToWorkIn, Where, Worked, Working};
use alo_adapters::{Delivers, Message, NotDelivered};
use alo_applications::{Application, Installed};
use alo_capability::{AloInApplications, Given, Grant, Grants, Reach};
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_record::{Asking, Record};
use alo_software::Bound;

use crate::Turning;
use crate::how_an_application_starts::{NoRoad, SaysHowAnApplicationStarts, TheRoad};
use crate::machine::{Machine, WhatASoftwareVerbNeeds, WhatAnApplicationVerbNeeds};
use crate::refusing::NotDone;
use crate::testing::{a_folder_of_our_own, hour, in_english, noon, offering};
use crate::the_door_the_compositor_serves::{
    NotAtTheDoor, TheDoorTheCompositorServes, WhileAloWorks,
};

/// The application, and its adapter's verb.
const EDITOR: &str = "org.gnome.TextEditor";
const OPEN_DOCUMENT: &str = "text_editor.open_document";

/// The agent.
const AGENT: &str = "@adapter";

/// Everything that happened, in order, on the door and the bus.
type Happened = Rc<RefCell<Vec<String>>>;

/// A door that answers `work-in-…` as it is told, and writes down each ask.
struct ADoor {
    answers: Result<Working, NotAtTheDoor>,
    happened: Happened,
}

struct AtWork {
    answered: Working,
    happened: Happened,
}

impl WhileAloWorks for AtWork {
    fn answered(&self) -> Working {
        self.answered
    }

    fn done(self: Box<Self>) -> Result<Worked, NotAtTheDoor> {
        self.happened.borrow_mut().push("done".to_owned());
        Ok(Worked::Uncounted)
    }
}

impl TheDoorTheCompositorServes for ADoor {
    fn asking(&mut self, _: &alo_acting::Request) -> Result<alo_acting::Answer, NotAtTheDoor> {
        panic!("an adapter's verb asks the door nothing but where alo works")
    }

    fn working_in(&mut self, asked: &ToWorkIn) -> Result<Box<dyn WhileAloWorks>, NotAtTheDoor> {
        self.happened.borrow_mut().push(asked.written());
        let answered = self.answers.clone()?;
        Ok(Box::new(AtWork {
            answered,
            happened: Rc::clone(&self.happened),
        }))
    }
}

/// A session bus that writes down what it is sent.
struct ABus {
    happened: Happened,
    sent: RefCell<Vec<Message>>,
}

impl Delivers for ABus {
    fn deliver(&self, message: &Message) -> Result<(), NotDelivered> {
        self.happened
            .borrow_mut()
            .push(format!("sent {}", message.destination()));
        self.sent.borrow_mut().push(message.clone());
        Ok(())
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

struct NothingStarts;

impl SaysHowAnApplicationStarts for NothingStarts {
    fn how_it_starts(&self, _: &str) -> Result<TheRoad, NoRoad> {
        Err(NoRoad::NoEntry)
    }
}

/// What one machine saw.
struct Saw {
    happened: Vec<String>,
    sent: Vec<Message>,
    entries: usize,
}

/// Run `doing` in a turn, at the setting `chosen`, on a machine whose door
/// answers `answers` and which holds a `notes.txt` granted to the agent.
fn on_a_machine<T>(
    chosen: AloInApplications,
    answers: Result<Working, NotAtTheDoor>,
    doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants, &Given) -> T,
) -> (T, Saw) {
    let strings = in_english();
    let tool = NoTool;
    let publisher = crate::testing::APublisher::that_publishes();
    let happened: Happened = Rc::default();
    let mut door = ADoor {
        answers,
        happened: Rc::clone(&happened),
    };
    let bus = ABus {
        happened: Rc::clone(&happened),
        sent: RefCell::default(),
    };
    let mut windows = crate::testing::NoTreeHere;
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let mut bounding = crate::testing::NothingIsBounded;
    let folder = a_folder_of_our_own("adapter");
    let notes = folder.join("notes.txt");
    std::fs::write(&notes, "a note").unwrap();
    let where_alo_works = move || chosen;
    let answered = {
        let machine = Machine::carrying_out_file_software_and_application_verbs(
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
        let mut machine = machine
            .with_the_persons_session_bus(&bus)
            .reading_where_alo_works(&where_alo_works);
        let mut grants = Grants::default();
        for reach in [
            Reach::Application(EDITOR.to_owned()),
            Reach::Folder(folder.clone()),
        ] {
            grants.grant(Grant::checked(AGENT, reach, noon(), hour()).unwrap());
        }
        let mut turning =
            Turning::beginning(offering(&notes), AGENT, hour(), &mut grants, &mut machine).unwrap();
        doing(&mut turning, &mut grants, &crate::testing::as_given(&notes))
    };
    let saw = Saw {
        happened: happened.borrow().clone(),
        sent: bus.sent.borrow().clone(),
        entries: record.answering(&Asking::anything()).count(),
    };
    (answered, saw)
}

fn the_document(notes: &Given) -> Vec<(&'static str, Given)> {
    vec![(alo_adapters::text_editor::DOCUMENT, notes.clone())]
}

/// **At *alo works in its own copy*, Text Editor is refused before any card**:
/// it has no copy of its own, so alo may not work in it at that setting, and
/// the person is told so in the words every other road uses. Nothing is asked
/// of the desktop and nothing is sent.
#[test]
fn at_its_own_copy_only_an_adapters_verb_is_refused_before_any_card() {
    let (proposed, saw) = on_a_machine(
        AloInApplications::InItsOwnCopy,
        Ok(Working::Showing),
        |turning, grants, notes| {
            turning.proposing(OPEN_DOCUMENT, &the_document(notes), grants, hour(), noon())
        },
    );
    let Err(NotDone::Refused(refused)) = proposed else {
        panic!("a card was proposed at its own copy only: {proposed:?}")
    };
    assert_eq!(
        refused.said(&in_english()).text(),
        "alo cannot work in Text Editor safely at this setting, because it cannot open a \
         separate copy of its own for alo. To let alo work in your own window instead, choose \
         “alo may work in your own window” in Settings → What alo can use"
    );
    assert!(saw.happened.is_empty(), "{:?}", saw.happened);
    assert!(saw.sent.is_empty());
    assert_eq!(saw.entries, 1, "the refusal is written down");
}

/// **In the person's window, the desktop is told before the message goes and
/// after it**: `work-in-their-window` first, which puts up the line the border
/// reads, then the one message to Text Editor, then `done`.
#[test]
fn an_adapters_verb_in_the_persons_window_tells_the_desktop_first() {
    let (did, saw) = on_a_machine(
        AloInApplications::InItsOwnCopyOrTheirWindow,
        Ok(Working::Showing),
        |turning, grants, notes| {
            let id = turning
                .proposing(OPEN_DOCUMENT, &the_document(notes), grants, hour(), noon())
                .unwrap();
            let did = turning.approving(id, grants, noon()).unwrap();
            assert!(turning.approving(id, grants, noon()).is_err());
            did
        },
    );
    assert_eq!(
        saw.happened,
        [
            ToWorkIn::at(
                Application::identified(EDITOR).unwrap(),
                Where::InTheirWindow
            )
            .written(),
            format!("sent {EDITOR}"),
            "done".to_owned(),
        ]
    );
    assert_eq!(saw.sent.len(), 1, "one approval, one message");
    assert_eq!(saw.sent[0].method(), "Open");
    let crate::WhatAVerbDid::AnAdapterVerbRan(ran) = did else {
        panic!("{did:?}")
    };
    assert_eq!((ran.verb(), ran.application()), (OPEN_DOCUMENT, EDITOR));
    assert_eq!(saw.entries, 1, "the run is written down");
}

/// **A desktop that will not let alo work there, or is not there, sends
/// nothing**: the line the border reads could not go up, so the message does
/// not go either, and the refusal is written down.
#[test]
fn a_desktop_that_refuses_or_is_not_there_sends_nothing() {
    for answers in [
        Ok(Working::NotChosen),
        Ok(Working::CannotShow),
        Err(NotAtTheDoor::NoCompositor),
    ] {
        let (ran, saw) = on_a_machine(
            AloInApplications::InItsOwnCopyOrTheirWindow,
            answers.clone(),
            |turning, grants, notes| {
                let id = turning
                    .proposing(OPEN_DOCUMENT, &the_document(notes), grants, hour(), noon())
                    .unwrap();
                turning.approving(id, grants, noon())
            },
        );
        assert!(
            matches!(ran, Err(NotDone::Refused(_))),
            "{answers:?}: {ran:?}"
        );
        assert!(saw.sent.is_empty(), "{answers:?}: something was sent");
        assert_eq!(saw.entries, 1, "{answers:?}: the refusal is written down");
    }
}

/// **With no invocation nothing is asked or sent.**
#[test]
fn with_no_invocation_nothing_is_asked_or_sent() {
    let ((), saw) = on_a_machine(
        AloInApplications::InItsOwnCopyOrTheirWindow,
        Ok(Working::Showing),
        |_, _, _| (),
    );
    assert!(saw.happened.is_empty());
    assert!(saw.sent.is_empty());
    assert_eq!(saw.entries, 0);
}
