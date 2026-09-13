//! *B's person approves any change, on B, from B's own approval surface.* —
//! ADR 0003, and this is that sentence at the surface: a change a paired
//! machine asked for reaches the compositor on the machine that was asked,
//! worded exactly as the turn renders it, and the only answer that runs it is
//! given here.
//!
//! The surface is unchanged by this. It reads a question off a turn, and a
//! remote turn lends its turn for reading — so a change from another machine
//! goes in front of a person by the one door every change goes through, and
//! there is no second door for the network.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_approving::{Approving, Asked, Asks, Compositor, NotAsked, SurfaceRefused};
use alo_capability::{Given, Grant, Grants, Reach};
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Reaching, Resolving as _};
use alo_nearby::{Deliberating, MachineId, MayAskIts, Origin, Pairings, Proposal, Side};
use alo_record::Record;
use alo_saying::everything_this_machine_can_say;
use alo_strings::Strings;
use alo_turn::{Arriving, Bounding, Doing, Done, Machine, NoBoundary};

/// A screen, keeping whatever question it was last given.
#[derive(Default)]
struct Screen {
    /// What is in front of the person.
    showing: Option<Asked>,
}

impl Compositor for Screen {
    fn ask(&mut self, asked: Asked) -> Result<(), SurfaceRefused> {
        self.showing = Some(asked);
        Ok(())
    }
}

/// A machine with nothing in front of a turn, which is not a machine alo OS
/// ships — written here where a reader of the test can see that is what it is.
struct NothingIsBounded;

impl Bounding for NothingIsBounded {
    fn carrying_out(&mut self, _reaching: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary> {
        Ok(doing.done())
    }
}

/// A fixed moment.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long everything here stands.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The machine that asked, as the machine being asked keeps its pairing.
fn paired_with_the_reception() -> (Pairings, MachineId) {
    let here = MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap();
    let reception = MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap();
    let mut pairings = Pairings::none();
    pairings.keep(
        Deliberating::of(
            Proposal::checked(reception.clone(), here, &[MayAskIts::Models], hour()).unwrap(),
        )
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(Side::TheOneAsked, noon())
        .unwrap(),
    );
    (pairings, reception)
}

/// A folder of this test's own with one file in it, resolved.
fn a_folder_with_an_invoice() -> (PathBuf, PathBuf) {
    let folder = std::env::temp_dir().join(format!("alo-approving-remote-{}", std::process::id()));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    let folder = OnThisMachine.real(&folder).unwrap().into_path_buf();
    let invoice = folder.join("march.pdf");
    fs::write(&invoice, "March, 4180.00").unwrap();
    (folder, invoice)
}

/// **The change is shown to the person on this machine**, as the sentence the
/// turn renders, marked as the paired machine's, and it happens once when they
/// approve it here — and not at all when there is nothing to show it on.
#[test]
fn a_change_from_a_paired_machine_is_shown_to_the_person_on_this_machine() {
    let strings = Strings::of(everything_this_machine_can_say().unwrap());
    let (folder, invoice) = a_folder_with_an_invoice();
    let mut record = Record::default();
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        &mut record,
    )
    .unwrap();
    let (pairings, reception) = paired_with_the_reception();
    let origin = Origin::paired(&pairings, &reception, "the reception machine", noon()).unwrap();
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            &origin.principal(),
            Reach::Folder(folder.clone()),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    let mut arriving =
        Arriving::beginning(&origin, hour(), noon(), &mut grants, &mut machine).unwrap();

    let id = arriving
        .proposing(
            "rename_file",
            &[
                ("file", Given::text(invoice.to_string_lossy().into_owned())),
                ("name", Given::text("march-final.pdf")),
            ],
            &pairings,
            &grants,
            hour(),
            noon(),
        )
        .unwrap();

    // With no compositor there is nothing to put it in front of, and the
    // answer is a sentence rather than silence.
    let mut approving = Approving::nothing_to_answer();
    assert_eq!(
        approving.ask(None, arriving.turning(), id, noon()),
        Asks::Refused(NotAsked::NoCompositor)
    );

    // With one, the question reaches the screen: the turn's own sentence, and
    // whose it is — the paired machine's principal on this machine's grants.
    let mut screen = Screen::default();
    let Asks::Asked(asked) = approving.ask(Some(&mut screen), arriving.turning(), id, noon())
    else {
        unreachable!("a change that was waiting was not put to anybody")
    };
    assert_eq!(screen.showing.as_ref(), Some(&asked));
    assert!(
        asked.sentence().text().ends_with("to march-final.pdf"),
        "{}",
        asked.sentence()
    );
    assert!(!asked.sentence().is_a_bug());
    assert_eq!(asked.agent(), arriving.grantee());
    assert_eq!(asked.agent().as_str(), origin.principal());
    assert_eq!(asked.id(), id);
    assert!(invoice.is_file(), "a change ran before anybody answered");

    // The person on this machine approves it, on this machine, and it happens
    // once.
    let answer = arriving.approving(id, &pairings, &grants, noon()).unwrap();
    assert!(answer.now_at().unwrap().ends_with("march-final.pdf"));
    assert!(!invoice.is_file());
    assert!(arriving.approving(id, &pairings, &grants, noon()).is_err());
    arriving.ending(&mut grants);

    assert_eq!(record.len(), 1);
    let entry = record.everything().next().unwrap();
    assert!(entry.happened().ran());
    assert_eq!(entry.happened().from_approval(), Some(id.as_u64()));
    assert!(
        entry
            .origin()
            .is_some_and(|origin| origin.is("the reception machine"))
    );
}
