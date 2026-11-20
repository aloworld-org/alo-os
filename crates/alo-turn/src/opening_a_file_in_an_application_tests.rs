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

use alo_acting::{Answer, Opened, Request, ToOpenAFile, ToType, Typed};
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
    /// How many `may-type-into` asks it answers *not in front* before *may*:
    /// how many looks the opened window takes to come.
    not_in_front_for: usize,
    /// How many `may-type-into` asks it was asked.
    checks: Rc<RefCell<usize>>,
}

impl TheDoorTheCompositorServes for ADoor {
    fn asking(&mut self, _: &Request) -> Result<Answer, NotAtTheDoor> {
        panic!("opening a file asks the door to open a file, never an act")
    }

    fn opening_a_file(&mut self, asked: &ToOpenAFile) -> Result<Opened, NotAtTheDoor> {
        self.asked.borrow_mut().push(asked.clone());
        self.answering.clone()
    }

    fn typing(&mut self, asked: &ToType, _: &str) -> Result<Typed, NotAtTheDoor> {
        assert!(
            matches!(asked, ToType::Check { .. }),
            "opening a file typed or saved: {asked:?}"
        );
        let mut checks = self.checks.borrow_mut();
        *checks += 1;
        Ok(if *checks > self.not_in_front_for {
            Typed::MayType
        } else {
            Typed::NotInFront
        })
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
    /// How many looks the opened window takes to come in front.
    not_in_front_for: usize,
    /// What this machine says a file opens with, where it was told.
    opening: Option<AsThisMachineSays>,
    /// The other applications granted, by identifier.
    granted_too: Vec<&'static str>,
}

impl Arranged {
    /// The editor installed and granted, the file's folder granted, and the
    /// door answering `answering`.
    fn answering(answering: Result<Opened, NotAtTheDoor>) -> Self {
        Self {
            answering,
            installed: Installed::holding([
                Application::called(EDITOR, "VSCodium").unwrap(),
                Application::called(FILES, "Dolphin").unwrap(),
                Application::called(PAPERS, "Papers").unwrap(),
            ]),
            the_editor: true,
            the_folder: true,
            full_access: false,
            not_in_front_for: 0,
            opening: None,
            granted_too: Vec::new(),
        }
    }

    /// The same, on a machine that says what a file opens with: the file
    /// manager granted, and opening it with `opens_with`.
    fn in_the_file_manager(opens_with: Result<&'static str, &'static str>) -> Self {
        Self {
            opening: Some(AsThisMachineSays {
                opens_with,
                asked: Rc::new(RefCell::new(0)),
            }),
            granted_too: vec![FILES, PAPERS],
            ..Self::answering(Ok(Opened::InAlosCopy))
        }
    }
}

/// The file manager the person names.
const FILES: &str = "org.kde.dolphin";

/// What the file opens with, as a double-click in it would.
const PAPERS: &str = "org.gnome.Papers";

/// What this machine says about what opens what: [`FILES`] opens folders, and
/// a file opens with `opens_with` — or nothing opens it, for that reason.
#[derive(Clone)]
struct AsThisMachineSays {
    opens_with: Result<&'static str, &'static str>,
    /// How many times what a file opens with was asked.
    asked: Rc<RefCell<usize>>,
}

impl crate::SaysWhatAFileOpensWith for AsThisMachineSays {
    fn opens_folders(&self, identifier: &str) -> bool {
        identifier == FILES
    }

    fn is_a_folder(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn called(&self, identifier: &str) -> Option<String> {
        match identifier {
            FILES => Some("Dolphin".to_owned()),
            PAPERS => Some("Papers".to_owned()),
            _ => None,
        }
    }

    fn opens_with(&self, _: &Path) -> Result<String, crate::NothingOpensIt> {
        *self.asked.borrow_mut() += 1;
        self.opens_with.map(str::to_owned).map_err(|why| {
            crate::NothingOpensIt::because(in_english().say(
                &alo_applications::words::NOTHING_OPENS.key(),
                &alo_strings::Filling::of("what", why),
            ))
        })
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
    /// How many times the door was asked whether the window is in front.
    checks: usize,
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
    let checks = Rc::new(RefCell::new(0));
    let mut door = ADoor {
        answering: arranged.answering,
        asked: Rc::clone(&asked),
        not_in_front_for: arranged.not_in_front_for,
        checks: Rc::clone(&checks),
    };
    let mut no_tree = crate::testing::NoTreeHere;
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let mut bounding = crate::testing::NothingIsBounded;
    let folder = crate::testing::a_folder_of_our_own("opening");
    let file = folder.join("hello.txt");
    std::fs::write(&file, WHAT_IT_SAYS).unwrap();
    let opening = arranged.opening.clone();
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
        let mut machine = match &opening {
            Some(opening) => machine.knowing_what_files_open_with(opening),
            None => machine,
        };
        let mut grants = Grants::default();
        for application in &arranged.granted_too {
            grants.grant(
                Grant::checked(
                    AGENT,
                    Reach::Application((*application).to_owned()),
                    noon(),
                    hour(),
                )
                .unwrap(),
            );
        }
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
        checks: *checks.borrow(),
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

/// **The opening is answered once its window is in front**, so the typing
/// that follows finds the editor rather than a window not yet drawn — and the
/// wait asks only whether it may type, never types.
///
/// What would have to be true for this to pass while broken: the desktop
/// answering `may` before the window it opened has the keyboard, which
/// `alo_shell`'s `typed_into_the_window_in_front` does not — it answers from
/// the window that holds the keyboard now.
#[test]
fn an_opening_is_answered_once_its_window_is_in_front() {
    let mut arranged = Arranged::answering(Ok(Opened::InAlosCopy));
    arranged.not_in_front_for = 3;
    let ran = on_a_machine(arranged, |turning, grants, file| {
        proposed_and_approved(turning, grants, EDITOR, file)
    });
    assert!(
        ran.answered
            .unwrap()
            .unwrap()
            .opened()
            .unwrap()
            .in_alos_copy()
    );
    assert_eq!(
        ran.checks, 4,
        "it did not wait for the window, or kept asking"
    );
}

/// **An opening that opened nothing waits for nothing.**
#[test]
fn a_refused_opening_does_not_wait_for_a_window() {
    let ran = on_a_machine(
        Arranged::answering(Ok(Opened::CouldNotStart)),
        |turning, grants, file| proposed_and_approved(turning, grants, EDITOR, file),
    );
    assert!(ran.answered.is_err());
    assert_eq!(ran.checks, 0);
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
        not_in_front_for: 0,
        checks: Rc::new(RefCell::new(0)),
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

/// **An application nobody granted is noted as wanted, once, for the person's
/// sheet — and one that is granted is not.** Asked twice in one turn, it is
/// still one entry (the owner's decision of 2026-11-16).
#[test]
fn an_application_nobody_granted_is_wanted_once_and_a_granted_one_is_not() {
    for (granted, expected) in [(false, vec![EDITOR.to_owned()]), (true, Vec::new())] {
        let wanted: RefCell<Vec<String>> = RefCell::new(Vec::new());
        let arranged = Arranged {
            the_editor: granted,
            ..Arranged::answering(Ok(Opened::InAlosCopy))
        };
        let _ran = on_a_machine(arranged, |turning, grants, file| {
            assert!(turning.applications_wanted().is_empty(), "the control");
            for _ in 0..2 {
                let _ = turning.proposing(
                    OPEN_FILE_IN_APPLICATION,
                    &given(EDITOR, file),
                    grants,
                    hour(),
                    noon(),
                );
            }
            *wanted.borrow_mut() = turning.applications_wanted().to_vec();
            Ok(None)
        });
        assert_eq!(*wanted.borrow(), expected, "granted: {granted}");
    }
}

/// How many times the wait asks in `within`, counting the first ask at once.
fn asks_within(within: std::time::Duration) -> u32 {
    let mut waited = std::time::Duration::ZERO;
    let mut asks = 1;
    loop {
        waited += crate::opening_a_file_in_an_application::the_wait_after(asks);
        if waited > within {
            return asks;
        }
        asks += 1;
    }
}

/// **Waiting for an opened file's window backs off**, and still answers a
/// quick window quickly. A fixed quarter-second asked about forty times while
/// Loupe's window took ten seconds to come, each ask a line on the desktop's
/// journal (main's walk at 313ca239, run 44069105568: 59 lines from 888 s to
/// 898 s). The first wait is still a quarter of a second, no wait is longer
/// than two, and the bound of a minute is unchanged.
#[test]
fn waiting_for_an_opened_window_backs_off() {
    use std::time::Duration;
    assert_eq!(
        crate::opening_a_file_in_an_application::the_wait_after(1),
        Duration::from_millis(250),
        "a window that comes at once is waited for longer than before"
    );
    for asks in 1..1000 {
        assert!(
            crate::opening_a_file_in_an_application::the_wait_after(asks) <= Duration::from_secs(2),
            "a window that comes is noticed more than two seconds late"
        );
    }
    assert!(asks_within(Duration::from_secs(10)) <= 8);
    assert!(asks_within(Duration::from_secs(60)) <= 35);
}

/// The sentence of the one card waiting in `turning`.
fn the_card(turning: &Turning<'_, '_>) -> String {
    let waiting: Vec<_> = turning.waiting_at(noon()).collect();
    assert_eq!(waiting.len(), 1, "one card");
    waiting
        .first()
        .map(|it| it.proposal.sentence(&in_english()).into_text())
        .unwrap_or_default()
}

/// **A file opened in a file manager opens as a double-click there would, at
/// ask each time**: the card names the application the file opens with and
/// the file manager it was asked of, and one approval asks the door once, to
/// open the file in that application.
#[test]
fn a_file_opened_in_a_file_manager_opens_where_a_double_click_would() {
    let card = RefCell::new(String::new());
    let ran = on_a_machine(
        Arranged::in_the_file_manager(Ok(PAPERS)),
        |turning, grants, file| {
            let id = turning.proposing(
                OPEN_FILE_IN_APPLICATION,
                &given(FILES, file),
                grants,
                hour(),
                noon(),
            )?;
            *card.borrow_mut() = the_card(turning);
            turning.approving(id, grants, noon()).map(Some)
        },
    );
    assert!(ran.answered.is_ok(), "{:?}", ran.answered);
    assert_eq!(
        *card.borrow(),
        format!("open {} in Papers, as Dolphin would", ran.file.display())
    );
    assert_eq!(ran.asked.len(), 1, "one approval, one opening");
    assert_eq!(
        ran.asked.first().map(|it| it.application().identifier()),
        Some(PAPERS)
    );
    assert!(ran.unchanged);
}

/// **And at full access, with no card**, in the same words and to the same
/// application.
#[test]
fn at_full_access_a_file_opened_in_a_file_manager_opens_where_a_double_click_would() {
    let arranged = Arranged {
        full_access: true,
        granted_too: Vec::new(),
        the_editor: false,
        the_folder: false,
        ..Arranged::in_the_file_manager(Ok(PAPERS))
    };
    let sentence = RefCell::new(String::new());
    let ran = on_a_machine(arranged, |turning, grants, file| {
        let changed = turning.changing(
            OPEN_FILE_IN_APPLICATION,
            &given(FILES, file),
            grants,
            hour(),
            &mut |_| None,
            noon(),
        )?;
        let crate::Changed::Done {
            did,
            sentence: said,
        } = changed
        else {
            panic!("a change at full access waited: {changed:?}");
        };
        *sentence.borrow_mut() = said.into_text();
        Ok(Some(did))
    });
    assert!(ran.answered.is_ok(), "{:?}", ran.answered);
    assert_eq!(
        *sentence.borrow(),
        format!("open {} in Papers, as Dolphin would", ran.file.display())
    );
    assert_eq!(
        ran.asked.first().map(|it| it.application().identifier()),
        Some(PAPERS)
    );
    assert_eq!(ran.asked.len(), 1);
}

/// **A folder still opens in the file manager**, which is what a
/// double-click on a folder does there — and what a file opens with is never
/// asked.
#[test]
fn a_folder_opened_in_a_file_manager_opens_in_it() {
    let arranged = Arranged::in_the_file_manager(Ok(PAPERS));
    let asked = arranged.opening.as_ref().map(|it| Rc::clone(&it.asked));
    let ran = on_a_machine(arranged, |turning, grants, file| {
        let folder = file.parent().unwrap().to_path_buf();
        proposed_and_approved(turning, grants, FILES, &folder)
    });
    assert!(ran.answered.is_ok(), "{:?}", ran.answered);
    assert_eq!(
        ran.asked.first().map(|it| it.application().identifier()),
        Some(FILES)
    );
    assert_eq!(asked.map(|it| *it.borrow()), Some(0));
}

/// **Nothing opens a file of that kind**: refused in the machine's words,
/// written down, and nothing is opened.
#[test]
fn a_file_nothing_opens_is_refused_and_recorded() {
    let ran = on_a_machine(
        Arranged::in_the_file_manager(Err("a file of this kind")),
        |turning, grants, file| proposed_and_approved(turning, grants, FILES, file),
    );
    let Err(NotDone::Refused(refused)) = &ran.answered else {
        panic!("a file nothing opens was opened: {:?}", ran.answered);
    };
    assert!(
        refused
            .said(&in_english())
            .text()
            .starts_with("nothing on this machine opens a file of this kind"),
        "{refused:?}"
    );
    assert!(ran.asked.is_empty(), "the door was asked");
    assert_eq!(ran.entries, 1, "the refusal is recorded");
}

/// **A file outside the grants is refused**, and nothing is opened.
#[test]
fn a_file_outside_the_grants_opened_in_a_file_manager_is_refused() {
    let arranged = Arranged {
        the_folder: false,
        ..Arranged::in_the_file_manager(Ok(PAPERS))
    };
    let ran = on_a_machine(arranged, |turning, grants, file| {
        proposed_and_approved(turning, grants, FILES, file)
    });
    assert!(ran.answered.is_err(), "{:?}", ran.answered);
    assert!(ran.asked.is_empty(), "the door was asked");
    assert_eq!(ran.entries, 1, "the refusal is recorded");
}

/// **What it opens with, not allowed for alo, is wanted for the person's
/// sheet** — named, so the shell asks whether alo may use it — and nothing
/// is opened meanwhile. The file manager, allowed, is not wanted.
#[test]
fn what_it_opens_with_not_allowed_is_wanted_for_the_sheet() {
    let wanted = RefCell::new(Vec::new());
    let arranged = Arranged {
        granted_too: vec![FILES],
        ..Arranged::in_the_file_manager(Ok(PAPERS))
    };
    let ran = on_a_machine(arranged, |turning, grants, file| {
        let proposed = turning.proposing(
            OPEN_FILE_IN_APPLICATION,
            &given(FILES, file),
            grants,
            hour(),
            noon(),
        );
        *wanted.borrow_mut() = turning.applications_wanted().to_vec();
        proposed.map(|_| None)
    });
    assert!(ran.answered.is_err(), "{:?}", ran.answered);
    assert_eq!(*wanted.borrow(), vec![PAPERS.to_owned()]);
    assert!(ran.asked.is_empty());
}

/// **A file manager nobody allowed is left to the grants**, and what the file
/// opens with is never read for a call that could not reach it.
#[test]
fn a_file_manager_nobody_allowed_is_refused_before_anything_is_read() {
    let arranged = Arranged {
        granted_too: Vec::new(),
        ..Arranged::in_the_file_manager(Ok(PAPERS))
    };
    let asked = arranged.opening.as_ref().map(|it| Rc::clone(&it.asked));
    let ran = on_a_machine(arranged, |turning, grants, file| {
        proposed_and_approved(turning, grants, FILES, file)
    });
    assert!(ran.answered.is_err(), "{:?}", ran.answered);
    assert_eq!(asked.map(|it| *it.borrow()), Some(0), "a kind was read");
    assert!(ran.asked.is_empty());
}

/// **A machine that was not told what files open with opens the file in the
/// application named**, as before.
#[test]
fn a_machine_not_told_opens_in_the_application_named() {
    let arranged = Arranged {
        granted_too: vec![FILES],
        ..Arranged::answering(Ok(Opened::InAlosCopy))
    };
    let ran = on_a_machine(arranged, |turning, grants, file| {
        proposed_and_approved(turning, grants, FILES, file)
    });
    assert!(ran.answered.is_ok(), "{:?}", ran.answered);
    assert_eq!(
        ran.asked.first().map(|it| it.application().identifier()),
        Some(FILES)
    );
}

/// **No card names an application by its identifier where the application
/// has a name** — at both levels, the sentence a person approves or reads
/// holds no reverse-DNS identifier, only the names the entries give.
#[test]
fn no_card_names_an_application_by_its_identifier_where_it_has_a_name() {
    let reverse_dns = |said: &str| {
        said.split(|it: char| it.is_whitespace() || it == ',')
            .any(|word| word.matches('.').count() >= 2 && word.starts_with(char::is_alphabetic))
    };
    let card = RefCell::new(String::new());
    let _ = on_a_machine(
        Arranged::in_the_file_manager(Ok(PAPERS)),
        |turning, grants, file| {
            turning.proposing(
                OPEN_FILE_IN_APPLICATION,
                &given(FILES, file),
                grants,
                hour(),
                noon(),
            )?;
            *card.borrow_mut() = the_card(turning);
            Ok(None)
        },
    );
    let card = card.into_inner();
    assert!(
        card.contains("Papers") && card.contains("Dolphin"),
        "{card}"
    );
    assert!(!reverse_dns(&card), "a card names an identifier: {card}");
    // **And the check itself can fail**: an identifier is caught.
    assert!(reverse_dns(
        "open a.pdf in org.gnome.Papers, as Dolphin would"
    ));
}
