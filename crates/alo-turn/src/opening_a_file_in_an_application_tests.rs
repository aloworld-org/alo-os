//! **`open_file_in_application`, through a whole turn**: proposed, approved,
//! asked of the door once, and written down — and every refusal written down
//! too, with the door never asked.
//!
//! This is also where the verb's two card sentences are held to what it does
//! (`tests/every_change_this_machine_offers_says_what_it_does.rs`): *opens it
//! in alo's own copy of the application, or in your own window if you chose
//! that* — the door is asked to open that one file in that one application and
//! answers where — and *the file is not changed* — its bytes are the same after.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use alo_acting::{Answer, Opened, Request, ToOpenAFile};
use alo_applications::{Application, Installed, OPEN_FILE_IN_APPLICATION};
use alo_capability::{Given, Grant, Grantee, Grants, Reach};
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_nearby::Pairings;
use alo_record::{Asking, Record};
use alo_software::Bound;

use crate::how_an_application_starts::{NoRoad, SaysHowAnApplicationStarts, TheRoad};
use crate::machine::{Machine, WhatASoftwareVerbNeeds, WhatAnApplicationVerbNeeds};
use crate::refusing::NotDone;
use crate::testing::{hour, in_english, noon, offering};
use crate::the_door_the_compositor_serves::{NotAtTheDoor, TheDoorTheCompositorServes};
use crate::{Arriving, Turning, WhatAVerbDid};

/// The editor every test here opens a file in.
const EDITOR: &str = "com.vscodium.codium";

/// The agent these turns belong to.
const AGENT: &str = "@opening";

/// What is in the file, so a test can say it is the same afterwards.
const WHAT_IT_SAYS: &str = "hello\n";

/// A tool nothing here reaches: the whole machine offers installing too.
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
        panic!("nothing in this file installs anything")
    }
    fn updates(&self, _: &alo_software::SourceName) -> Result<Vec<String>, alo_software::Failed> {
        Ok(Vec::new())
    }
    fn update(&self, _: &Application) -> Result<(), alo_software::Failed> {
        panic!("nothing in this file updates anything")
    }
    fn remove(&self, _: &Application) -> Result<(), alo_software::Failed> {
        panic!("nothing in this file removes anything")
    }
}

/// Nothing here starts an application by `open_application`.
struct NothingStarts;

impl SaysHowAnApplicationStarts for NothingStarts {
    fn how_it_starts(&self, _: &str) -> Result<TheRoad, NoRoad> {
        Err(NoRoad::NoEntry)
    }
}

/// A door that answers `open-file` with one answer, and remembers each ask —
/// because *one approval causes exactly one execution* breaks here as a door
/// asked twice.
struct ADoor {
    /// What it answers.
    answering: Result<Opened, NotAtTheDoor>,
    /// Every `open-file` it was asked, readable while the machine holds it.
    asked: Rc<RefCell<Vec<ToOpenAFile>>>,
}

impl TheDoorTheCompositorServes for ADoor {
    fn asking(&mut self, _: &Request) -> Result<Answer, NotAtTheDoor> {
        panic!("opening a file asks the door to open a file, never an act")
    }

    fn opening_a_file(&mut self, asked: &ToOpenAFile) -> Result<Opened, NotAtTheDoor> {
        self.asked.borrow_mut().push(asked.clone());
        self.answering.clone()
    }
}

/// How a test arranges the machine and the grants.
struct Arranged {
    /// What the door answers.
    answering: Result<Opened, NotAtTheDoor>,
    /// What is installed.
    installed: Installed,
    /// Whether the editor is granted.
    the_editor: bool,
    /// Whether the folder the file is in is granted.
    the_folder: bool,
    /// Whether the person chose full access.
    full_access: bool,
}

impl Arranged {
    /// The editor installed and granted, the file's folder granted, and the
    /// door answering `answering`.
    fn answering(answering: Result<Opened, NotAtTheDoor>) -> Self {
        Self {
            answering,
            installed: Installed::holding([Application::called(EDITOR, "VSCodium").unwrap()]),
            the_editor: true,
            the_folder: true,
            full_access: false,
        }
    }
}

/// What a turn did, what the door was asked, what the record holds, and
/// whether the file is as it was.
struct Ran {
    /// What the turn answered.
    answered: Result<Option<WhatAVerbDid>, NotDone>,
    /// Every `open-file` the door was asked.
    asked: Vec<ToOpenAFile>,
    /// The sentences the record's entries were written with.
    told: Vec<String>,
    /// How many entries the record holds.
    entries: usize,
    /// Whether the file still says what it said.
    unchanged: bool,
    /// The file, as it really is on this machine.
    file: PathBuf,
}

/// Arrange the machine, then do `doing` with the turn, the grants and the file
/// named as `name` gives it — the file's own path where `name` is [`None`].
fn on_a_machine(
    arranged: Arranged,
    doing: impl FnOnce(&mut Turning<'_, '_>, &Grants, &Path) -> Result<Option<WhatAVerbDid>, NotDone>,
) -> Ran {
    let strings = in_english();
    let tool = NoTool;
    let publisher = crate::testing::APublisher::that_publishes();
    let asked = Rc::new(RefCell::new(Vec::new()));
    let mut door = ADoor {
        answering: arranged.answering,
        asked: Rc::clone(&asked),
    };
    let mut no_tree = crate::testing::NoTreeHere;
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let mut bounding = crate::testing::NothingIsBounded;
    let folder = crate::testing::a_folder_of_our_own("opening");
    let file = folder.join("hello.txt");
    std::fs::write(&file, WHAT_IT_SAYS).unwrap();
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
                installed: arranged.installed.into(),
                door: &mut door,
                starting: &NothingStarts,
                windows: &mut no_tree,
            },
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let mut grants = Grants::default();
        if arranged.the_editor {
            grants.grant(
                Grant::checked(AGENT, Reach::Application(EDITOR.to_owned()), noon(), hour())
                    .unwrap(),
            );
        }
        if arranged.the_folder {
            grants.grant(
                Grant::checked(AGENT, Reach::Folder(folder.clone()), noon(), hour()).unwrap(),
            );
        }
        let mut grants = if arranged.full_access {
            grants.at_full_access_for(&Grantee::named(AGENT))
        } else {
            grants
        };
        // **What the person offered is a note somewhere else**, so the
        // document a turn is offered never grants the file being opened.
        let elsewhere = crate::testing::a_folder_of_our_own("offered").join("a-note.txt");
        std::fs::write(&elsewhere, "anything").unwrap();
        let mut turning = Turning::beginning(
            offering(&elsewhere),
            AGENT,
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        doing(&mut turning, &grants, &file)
    };
    let anything = Asking::anything();
    let told = record
        .answering(&anything)
        .flat_map(|entry| entry.told().iter().map(ToString::to_string))
        .collect();
    let asked = asked.borrow().clone();
    Ran {
        answered,
        asked,
        told,
        entries: record.answering(&anything).count(),
        unchanged: std::fs::read_to_string(&file).is_ok_and(|now| now == WHAT_IT_SAYS),
        file,
    }
}

/// What a model sends: this application, and this file.
fn given(application: &str, file: &Path) -> Vec<(&'static str, Given)> {
    vec![
        ("application", Given::text(application)),
        ("path", Given::text(file.to_string_lossy().into_owned())),
    ]
}

/// Propose opening `file` in `application`, and approve it once. A proposal
/// that reached the door would fail `ADoor`'s count, which every test reads.
fn proposed_and_approved(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    application: &str,
    file: &Path,
) -> Result<Option<WhatAVerbDid>, NotDone> {
    let id = turning.proposing(
        OPEN_FILE_IN_APPLICATION,
        &given(application, file),
        grants,
        hour(),
        noon(),
    )?;
    turning.approving(id, grants, noon()).map(Some)
}

/// **The whole road: proposed, approved, asked once, opened in alo's own copy,
/// and written down with where** — and the file is not changed.
#[test]
fn an_approved_opening_asks_the_door_once_and_is_recorded_with_where_it_opened() {
    let ran = on_a_machine(
        Arranged::answering(Ok(Opened::InAlosCopy)),
        |turning, grants, file| proposed_and_approved(turning, grants, EDITOR, file),
    );
    let did = ran.answered.unwrap().unwrap();
    let opened = did.opened().unwrap();
    assert_eq!(opened.application(), EDITOR);
    assert!(opened.in_alos_copy());
    assert_eq!(
        ran.asked
            .iter()
            .map(|asked| (
                asked.application().identifier().to_owned(),
                asked.file().to_owned()
            ))
            .collect::<Vec<_>>(),
        [(EDITOR.to_owned(), ran.file.clone())],
        "one approval, one ask, about the real file"
    );
    assert_eq!(ran.entries, 1);
    assert_eq!(
        ran.told,
        [format!(
            "{} was opened in alo's own copy of {EDITOR}",
            ran.file.display()
        )]
    );
    assert!(ran.unchanged, "opening a file changed it");
}

/// **Where the person chose their own window, the record says that.**
#[test]
fn an_opening_in_the_persons_own_window_is_recorded_as_that() {
    let ran = on_a_machine(
        Arranged::answering(Ok(Opened::InTheirWindow)),
        |turning, grants, file| proposed_and_approved(turning, grants, EDITOR, file),
    );
    let did = ran.answered.unwrap().unwrap();
    assert!(!did.opened().unwrap().in_alos_copy());
    assert_eq!(
        ran.told,
        [format!(
            "{} was opened in your own window of {EDITOR}, as you chose in Settings",
            ran.file.display()
        )]
    );
}

/// **Named by the name it goes by, it is opened by its identifier** — the walk
/// of 2026-11-14's *vscodium*.
#[test]
fn an_application_named_by_its_name_is_asked_for_by_its_identifier() {
    let ran = on_a_machine(
        Arranged::answering(Ok(Opened::InAlosCopy)),
        |turning, grants, file| proposed_and_approved(turning, grants, "vscodium", file),
    );
    assert!(ran.answered.is_ok(), "{:?}", ran.answered);
    assert_eq!(
        ran.asked
            .first()
            .map(|asked| asked.application().identifier().to_owned()),
        Some(EDITOR.to_owned())
    );
}

/// **Each refusal the desktop can answer is the person's own sentence, and is
/// written down** — and nothing is said to have opened.
#[test]
fn each_refusal_the_desktop_answers_is_a_sentence_and_is_recorded() {
    for (answer, expected) in [
        (
            Ok(Opened::NotInstalled),
            format!("nothing installed on this machine is {EDITOR}"),
        ),
        (
            Ok(Opened::TakesNoFile),
            format!("{EDITOR}'s entry gives this machine no way to hand it a file"),
        ),
        (
            Ok(Opened::NoCopyOfItsOwn),
            "cannot open a separate copy of its own for alo".to_owned(),
        ),
        (
            Ok(Opened::CouldNotStart),
            format!("{EDITOR} could not be started with"),
        ),
        (
            Err(NotAtTheDoor::NoCompositor),
            format!("this machine could not reach {EDITOR}"),
        ),
    ] {
        let ran = on_a_machine(
            Arranged::answering(answer.clone()),
            |turning, grants, file| proposed_and_approved(turning, grants, EDITOR, file),
        );
        let Err(NotDone::Refused(refused)) = ran.answered else {
            panic!("{answer:?} was not a refusal: {:?}", ran.answered);
        };
        let said = refused.said(&in_english()).text().to_owned();
        assert!(said.contains(&expected), "{answer:?} said {said:?}");
        assert_eq!(ran.asked.len(), 1, "{answer:?}");
        assert_eq!(ran.entries, 1, "{answer:?} left no record");
        assert!(ran.unchanged, "{answer:?}");
    }
}

/// **A file outside every grant is refused before anybody is asked**, the door
/// is never reached, and the refusal is written down.
#[test]
fn a_file_outside_the_grants_is_refused_before_anything_is_proposed() {
    let arranged = Arranged {
        the_folder: false,
        ..Arranged::answering(Ok(Opened::InAlosCopy))
    };
    let ran = on_a_machine(arranged, |turning, grants, file| {
        proposed_and_approved(turning, grants, EDITOR, file)
    });
    assert!(
        matches!(ran.answered, Err(NotDone::NeverAsked(_))),
        "{:?}",
        ran.answered
    );
    assert!(
        ran.asked.is_empty(),
        "a file nobody granted reached the door"
    );
    assert_eq!(ran.entries, 1);
}

/// **An application nobody granted is refused the same way.**
#[test]
fn an_application_nobody_granted_is_refused_before_anything_is_proposed() {
    let arranged = Arranged {
        the_editor: false,
        ..Arranged::answering(Ok(Opened::InAlosCopy))
    };
    let ran = on_a_machine(arranged, |turning, grants, file| {
        proposed_and_approved(turning, grants, EDITOR, file)
    });
    assert!(
        matches!(ran.answered, Err(NotDone::NeverAsked(_))),
        "{:?}",
        ran.answered
    );
    assert!(ran.asked.is_empty());
    assert_eq!(ran.entries, 1);
}

/// **An application that is not installed is refused at the OK, before the
/// door**, and written down.
#[test]
fn an_application_that_is_not_installed_is_refused_before_the_door() {
    let arranged = Arranged {
        installed: Installed::nothing(),
        ..Arranged::answering(Ok(Opened::InAlosCopy))
    };
    let ran = on_a_machine(arranged, |turning, grants, file| {
        proposed_and_approved(turning, grants, EDITOR, file)
    });
    let Err(NotDone::Refused(refused)) = ran.answered else {
        panic!("not installed was not refused: {:?}", ran.answered);
    };
    assert!(
        refused
            .said(&in_english())
            .text()
            .contains("nothing installed on this machine is"),
        "{:?}",
        refused.said(&in_english()).text()
    );
    assert!(ran.asked.is_empty());
    assert_eq!(ran.entries, 1);
}

/// **A file that is not there is refused before the door**, and written down.
#[test]
fn a_file_that_is_not_there_is_refused_before_the_door() {
    let ran = on_a_machine(
        Arranged::answering(Ok(Opened::InAlosCopy)),
        |turning, grants, file| {
            let gone = file.with_file_name("not-here.txt");
            proposed_and_approved(turning, grants, EDITOR, &gone)
        },
    );
    assert!(
        matches!(ran.answered, Err(NotDone::Refused(_))),
        "{:?}",
        ran.answered
    );
    assert!(ran.asked.is_empty());
    assert_eq!(ran.entries, 1);
}

/// **At full access a file in no granted folder is opened** (ADR 0149) —
/// after the one OK, asked once.
#[test]
fn at_full_access_a_file_in_no_granted_folder_is_opened_after_the_ok() {
    let arranged = Arranged {
        the_folder: false,
        the_editor: false,
        full_access: true,
        ..Arranged::answering(Ok(Opened::InAlosCopy))
    };
    let ran = on_a_machine(arranged, |turning, grants, file| {
        proposed_and_approved(turning, grants, EDITOR, file)
    });
    assert!(ran.answered.is_ok(), "{:?}", ran.answered);
    assert_eq!(ran.asked.len(), 1);
}

/// **Declined, nothing is opened**, and the no is written down.
#[test]
fn a_declined_opening_opens_nothing_and_is_recorded() {
    let ran = on_a_machine(
        Arranged::answering(Ok(Opened::InAlosCopy)),
        |turning, grants, file| {
            let id = turning.proposing(
                OPEN_FILE_IN_APPLICATION,
                &given(EDITOR, file),
                grants,
                hour(),
                noon(),
            )?;
            turning.declining(id, noon()).map(|()| None)
        },
    );
    assert!(ran.answered.is_ok(), "{:?}", ran.answered);
    assert!(
        ran.asked.is_empty(),
        "a proposal nobody approved reached the door"
    );
    assert_eq!(ran.entries, 1);
}

/// **A paired machine cannot ask for a file to be opened here** — refused by
/// name before any grant is asked, even at full access, and written down.
#[test]
fn a_paired_machine_cannot_open_a_file_here() {
    let strings = in_english();
    let tool = NoTool;
    let publisher = crate::testing::APublisher::that_publishes();
    let asked = Rc::new(RefCell::new(Vec::new()));
    let mut door = ADoor {
        answering: Ok(Opened::InAlosCopy),
        asked: Rc::clone(&asked),
    };
    let mut no_tree = crate::testing::NoTreeHere;
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let mut bounding = crate::testing::NothingIsBounded;
    let refused = {
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
                windows: &mut no_tree,
            },
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        assert!(
            machine.verbs().of(OPEN_FILE_IN_APPLICATION).is_some(),
            "the control: this machine offers the verb, so a refusal is the pairing's"
        );
        let origin = crate::arriving::tests::origin();
        let mut grants = Grants::default().at_full_access_for(&crate::testing::files());
        let mut arriving =
            Arriving::beginning(&origin, hour(), noon(), &mut grants, &mut machine).unwrap();
        let refused = arriving.proposing(
            OPEN_FILE_IN_APPLICATION,
            &given(EDITOR, Path::new("/home/anna/hello.txt")),
            &Pairings::none(),
            &grants,
            hour(),
            noon(),
        );
        arriving.ending(&mut grants);
        refused
    };
    assert!(
        matches!(
            refused,
            Err(NotDone::TurnedAway(
                alo_capability::CallError::NoSuchVerb { .. }
            ))
        ),
        "{refused:?}"
    );
    assert!(asked.borrow().is_empty(), "a paired machine opened a file");
    assert_eq!(record.len(), 1, "the refusal was not written down");
}
