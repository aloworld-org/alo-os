//! **`choose_on_the_file_chooser`, through a whole turn**: proposed,
//! approved, asked of the door once with the folder's real path, and written
//! down — and every refusal written down too, with the door never asked.
//!
//! This is also where the verb's two card sentences are held to what it does
//! (`tests/every_change_this_machine_offers_says_what_it_does.rs`): *answers
//! the file chooser with this file* — the door is asked once, with that one
//! file — and *nothing outside the folders you granted can be chosen* — a
//! folder nobody granted, and a link out of a granted one, never reach the
//! door.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use alo_acting::{Answer, AnsweredTheChooser, Request, ToAnswerAFileChooser};
use alo_applications::{Application, CHOOSE_ON_THE_FILE_CHOOSER, Installed};
use alo_capability::{Given, Grant, Grantee, Grants, Reach};
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_record::{Asking, Record};
use alo_software::Bound;

use crate::how_an_application_starts::{NoRoad, SaysHowAnApplicationStarts, TheRoad};
use crate::machine::{Machine, WhatASoftwareVerbNeeds, WhatAnApplicationVerbNeeds};
use crate::refusing::NotDone;
use crate::testing::{hour, in_english, noon, offering};
use crate::the_door_the_compositor_serves::{NotAtTheDoor, TheDoorTheCompositorServes};
use crate::{Turning, WhatAVerbDid};

/// The application whose file chooser is up.
const EDITOR: &str = "org.gnome.TextEditor";

/// The agent these turns belong to.
const AGENT: &str = "@choosing";

/// What is in the file, so a test can say it is the same afterwards.
const WHAT_IT_SAYS: &str = "march\n";

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

/// Nothing here starts an application.
struct NothingStarts;

impl SaysHowAnApplicationStarts for NothingStarts {
    fn how_it_starts(&self, _: &str) -> Result<TheRoad, NoRoad> {
        Err(NoRoad::NoEntry)
    }
}

/// A door that answers `alo-chooses-file` with one answer and remembers each
/// ask — *one approval causes exactly one execution* breaks here as a door
/// asked twice.
struct ADoor {
    /// What it answers.
    answering: Result<AnsweredTheChooser, NotAtTheDoor>,
    /// Every answer it was asked to give.
    asked: Rc<RefCell<Vec<ToAnswerAFileChooser>>>,
}

impl TheDoorTheCompositorServes for ADoor {
    fn asking(&mut self, _: &Request) -> Result<Answer, NotAtTheDoor> {
        panic!("answering the file chooser never asks an act")
    }

    fn answering_the_file_chooser(
        &mut self,
        asked: &ToAnswerAFileChooser,
    ) -> Result<AnsweredTheChooser, NotAtTheDoor> {
        self.asked.borrow_mut().push(asked.clone());
        self.answering.clone()
    }
}

/// The answer a door gives when the sheet took the file.
fn chosen() -> Result<AnsweredTheChooser, NotAtTheDoor> {
    Ok(AnsweredTheChooser::Chosen(
        Application::identified(EDITOR).unwrap(),
    ))
}

/// How a test arranges the grants.
struct Arranged {
    /// What the door answers.
    answering: Result<AnsweredTheChooser, NotAtTheDoor>,
    /// Whether the folder the file is in is granted.
    the_folder: bool,
    /// Whether the person chose full access.
    full_access: bool,
}

impl Arranged {
    /// The folder granted, and the door answering `answering`.
    fn answering(answering: Result<AnsweredTheChooser, NotAtTheDoor>) -> Self {
        Self {
            answering,
            the_folder: true,
            full_access: false,
        }
    }
}

/// What a turn did, what the door was asked, and what the record holds.
struct Ran {
    /// What the turn answered.
    answered: Result<Option<WhatAVerbDid>, NotDone>,
    /// Every answer the door was asked to give.
    asked: Vec<ToAnswerAFileChooser>,
    /// The sentences the record's entries were written with.
    told: Vec<String>,
    /// How many entries the record holds.
    entries: usize,
    /// Whether the file still says what it said.
    unchanged: bool,
    /// The folder, as it really is.
    folder: PathBuf,
}

/// Arrange the machine — a folder holding `march.txt`, and beside it a folder
/// nobody grants holding `secret.txt` — then do `doing`.
fn on_a_machine(
    arranged: Arranged,
    doing: impl FnOnce(
        &mut Turning<'_, '_>,
        &Grants,
        &Path,
        &Path,
    ) -> Result<Option<WhatAVerbDid>, NotDone>,
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
    let folder = crate::testing::a_folder_of_our_own("choosing");
    std::fs::write(folder.join("march.txt"), WHAT_IT_SAYS).unwrap();
    let elsewhere = crate::testing::a_folder_of_our_own("not-granted");
    std::fs::write(elsewhere.join("secret.txt"), "secret").unwrap();
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
                installed: Installed::nothing().into(),
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
        let offered = crate::testing::a_folder_of_our_own("offered").join("a-note.txt");
        std::fs::write(&offered, "anything").unwrap();
        let mut turning =
            Turning::beginning(offering(&offered), AGENT, hour(), &mut grants, &mut machine)
                .unwrap();
        doing(&mut turning, &grants, &folder, &elsewhere)
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
        unchanged: std::fs::read_to_string(folder.join("march.txt"))
            .is_ok_and(|now| now == WHAT_IT_SAYS),
        folder: std::fs::canonicalize(&folder).unwrap(),
    }
}

/// Propose choosing `name` in `folder`, and approve it once.
fn proposed_and_approved(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    folder: &Path,
    name: &str,
) -> Result<Option<WhatAVerbDid>, NotDone> {
    let id = turning.proposing(
        CHOOSE_ON_THE_FILE_CHOOSER,
        &[
            ("folder", Given::text(folder.to_string_lossy().into_owned())),
            ("name", Given::text(name)),
        ],
        grants,
        hour(),
        noon(),
    )?;
    turning.approving(id, grants, noon()).map(Some)
}

/// **The whole road: proposed, approved, asked once with the real folder,
/// and written down with whom it was chosen for** — and the file not changed.
#[test]
fn an_approved_choice_asks_the_door_once_and_is_recorded() {
    let ran = on_a_machine(
        Arranged::answering(chosen()),
        |turning, grants, folder, _| proposed_and_approved(turning, grants, folder, "march.txt"),
    );
    let did = ran.answered.unwrap().unwrap();
    let chosen = did.chosen_on_the_file_chooser().unwrap();
    assert_eq!(chosen.application(), EDITOR);
    assert_eq!(chosen.file(), ran.folder.join("march.txt"));
    assert_eq!(
        ran.asked
            .iter()
            .map(|asked| (asked.folder().to_path_buf(), asked.name().to_owned()))
            .collect::<Vec<_>>(),
        [(ran.folder.clone(), "march.txt".to_owned())],
        "one approval, one ask, with the folder's real path"
    );
    assert_eq!(ran.entries, 1);
    assert_eq!(
        ran.told,
        [format!(
            "{} was chosen on the file chooser for {EDITOR}",
            ran.folder.join("march.txt").display()
        )]
    );
    assert!(ran.unchanged, "choosing a file changed it");
}

/// **A name to save under, not there yet, is the folder's**: asked once.
#[test]
fn a_name_to_save_under_is_asked_once() {
    let ran = on_a_machine(
        Arranged::answering(chosen()),
        |turning, grants, folder, _| proposed_and_approved(turning, grants, folder, "april.txt"),
    );
    assert!(ran.answered.is_ok(), "{:?}", ran.answered);
    assert_eq!(ran.asked.len(), 1);
}

/// **A folder nobody granted is refused before anything is proposed**, the
/// door never reached, and the refusal written down.
#[test]
fn a_folder_nobody_granted_is_refused_and_recorded() {
    let ran = on_a_machine(
        Arranged::answering(chosen()),
        |turning, grants, _, elsewhere| {
            proposed_and_approved(turning, grants, elsewhere, "secret.txt")
        },
    );
    assert!(
        matches!(ran.answered, Err(NotDone::NeverAsked(_))),
        "{:?}",
        ran.answered
    );
    assert!(
        ran.asked.is_empty(),
        "a folder nobody granted reached the door"
    );
    assert_eq!(ran.entries, 1);
}

/// **A link in a granted folder to a file nobody granted is refused at the
/// OK**, the door never reached, and the refusal written down in the file
/// verbs' own words.
#[cfg(unix)]
#[test]
fn a_link_out_of_a_granted_folder_is_refused_and_recorded() {
    let ran = on_a_machine(
        Arranged::answering(chosen()),
        |turning, grants, folder, elsewhere| {
            std::os::unix::fs::symlink(elsewhere.join("secret.txt"), folder.join("a-link.txt"))
                .unwrap();
            proposed_and_approved(turning, grants, folder, "a-link.txt")
        },
    );
    let Err(NotDone::Refused(refused)) = &ran.answered else {
        panic!(
            "a link out of the grants was not refused: {:?}",
            ran.answered
        );
    };
    assert!(
        refused.said(&in_english()).text().contains("secret.txt"),
        "{}",
        refused.said(&in_english()).text()
    );
    assert!(
        ran.asked.is_empty(),
        "a link out of the grants reached the door"
    );
    assert_eq!(ran.entries, 1);
}

/// **A folder that is not there is refused before the door.**
#[test]
fn a_folder_that_is_not_there_is_refused() {
    let ran = on_a_machine(
        Arranged::answering(chosen()),
        |turning, grants, folder, _| {
            proposed_and_approved(turning, grants, &folder.join("gone"), "march.txt")
        },
    );
    assert!(ran.answered.is_err(), "{:?}", ran.answered);
    assert!(ran.asked.is_empty());
    assert_eq!(ran.entries, 1);
}

/// **At full access a folder in no grant is answered with** (ADR 0149) —
/// after the one OK, asked once.
#[test]
fn at_full_access_any_folder_is_answered_with() {
    let arranged = Arranged {
        the_folder: false,
        full_access: true,
        ..Arranged::answering(chosen())
    };
    let ran = on_a_machine(arranged, |turning, grants, _, elsewhere| {
        proposed_and_approved(turning, grants, elsewhere, "secret.txt")
    });
    assert!(ran.answered.is_ok(), "{:?}", ran.answered);
    assert_eq!(ran.asked.len(), 1);
}

/// **Every way the desktop says no is a sentence, and is recorded**: no sheet
/// up, a file the sheet would not take, no desktop at all.
#[test]
fn each_no_from_the_desktop_is_a_sentence_and_is_recorded() {
    for (answering, says) in [
        (Ok(AnsweredTheChooser::NoSheet), "No file chooser was open"),
        (
            Ok(AnsweredTheChooser::NotBelieved),
            "The file chooser could not take",
        ),
        (
            Err(NotAtTheDoor::NoCompositor),
            "The desktop could not be asked",
        ),
    ] {
        let ran = on_a_machine(
            Arranged::answering(answering),
            |turning, grants, folder, _| {
                proposed_and_approved(turning, grants, folder, "march.txt")
            },
        );
        let Err(NotDone::Refused(refused)) = &ran.answered else {
            panic!("{says}: not refused: {:?}", ran.answered);
        };
        let said = refused.said(&in_english());
        assert!(said.text().starts_with(says), "{says}: {said}");
        assert!(!said.is_a_bug(), "{said}");
        assert_eq!(ran.asked.len(), 1, "{says}");
        assert_eq!(ran.entries, 1, "{says}");
    }
}

/// **Declined, nothing is chosen**, and the no is written down.
#[test]
fn a_declined_choice_chooses_nothing_and_is_recorded() {
    let ran = on_a_machine(
        Arranged::answering(chosen()),
        |turning, grants, folder, _| {
            let id = turning.proposing(
                CHOOSE_ON_THE_FILE_CHOOSER,
                &[
                    ("folder", Given::text(folder.to_string_lossy().into_owned())),
                    ("name", Given::text("march.txt")),
                ],
                grants,
                hour(),
                noon(),
            )?;
            turning.declining(id, noon()).map(|()| None)
        },
    );
    assert!(ran.answered.is_ok(), "{:?}", ran.answered);
    assert!(ran.asked.is_empty(), "a declined choice reached the door");
    assert_eq!(ran.entries, 1);
}
