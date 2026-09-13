//! The asking side reaches the wire through a turn and through nothing
//! else, and learns what became of a change it proposed.
//!
//! Task 9 of the local-network plan, the asking side: a turn on reception
//! crosses a read and a change to the studio the way `Turning::asking` puts
//! a question — from inside a boundary permitting the studio's one address,
//! with the departure written into reception's record whether or not an
//! answer came back — and then asks, on the additive outcome path, what the
//! studio's person did about the change. Both machines are one process on
//! one host over real sockets, as in every test of this plan.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::net::{SocketAddr, TcpListener, UdpSocket};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

use alo_capability::{Given, Grant, Grants, Reach};
use alo_corridor::{Crossing, Doorway, Heard, NotThrough, Outcome, Receiving};
use alo_egress::{EgressPolicy, Indicator};
use alo_files::{OnThisMachine, Reaching, Resolving as _};
use alo_models::SourcePolicy;
use alo_nearby::{
    Answering, Deliberating, Found, Keying, Looking, MachineId, MayAskIts, Pairing, Pairings,
    Presence, Proposal, Side,
};
use alo_record::{Asking, Happened, Record};
use alo_strings::{Strings, Vocabulary};
use alo_turn::{Bounding, Doing, Done, Machine, NoAnswer, NoBoundary, Places, Turning};

fn reception() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

fn the_studio() -> MachineId {
    MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
}

const THE_STUDIO: &str = "the studio machine";

const RECEPTION: &str = "the reception machine";

const RECEPTIONS_PRINCIPAL: &str = "machine:0f1e2d3c4b5a69788796a5b4c3d2e1f0";

fn a_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

fn the_moment_of(nth: usize) -> SystemTime {
    a_moment() + Duration::from_secs(u64::try_from(nth).unwrap())
}

fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

fn paired_between(asking: MachineId, asked: MachineId) -> (Pairing, Pairing) {
    let at_asking = Keying::fresh().unwrap();
    let at_asked = Keying::fresh().unwrap();
    let proposal = Proposal::checked(
        asking,
        asked,
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
        at_asking.offer().clone(),
    )
    .unwrap();
    let asked_side = Deliberating::asked(proposal.clone(), at_asked);
    let asking_side = Deliberating::asking(proposal, at_asking)
        .unwrap()
        .answered_with(asked_side.answered().unwrap().clone())
        .unwrap();
    (
        asking_side
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(a_moment())
            .unwrap(),
        asked_side
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(a_moment())
            .unwrap(),
    )
}

fn paired() -> (Pairings, Pairings) {
    let (on_reception, on_studio) = paired_between(reception(), the_studio());
    let mut at_reception = Pairings::none();
    at_reception.keep(on_reception);
    let mut at_studio = Pairings::none();
    at_studio.keep(on_studio);
    (at_reception, at_studio)
}

fn everything_a_machine_says() -> Vocabulary {
    let mut vocabulary = alo_files::file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_keeping::declare_into(&mut vocabulary).unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_answering::declare_into(&mut vocabulary).unwrap();
    alo_asking::declare_into(&mut vocabulary).unwrap();
    alo_nearby::words::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    alo_corridor::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// A boundary that imposes nothing and runs everything, on both machines:
/// what the kernel does with a boundary is `alo-bounding`'s to prove.
struct NothingIsBounded;

impl Bounding for NothingIsBounded {
    fn carrying_out(&mut self, _reaching: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary> {
        Ok(doing.done())
    }

    fn carrying_out_a_departure(
        &mut self,
        _to: &[SocketAddr],
        doing: &mut dyn FnMut(),
    ) -> Result<(), NoBoundary> {
        doing();
        Ok(())
    }
}

fn a_folder_with_an_invoice(what: &str) -> (PathBuf, PathBuf) {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-corridor-turn-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    let folder = OnThisMachine.real(&folder).unwrap().into_path_buf();
    let invoice = folder.join("march.pdf");
    fs::write(&invoice, "March, 4180.00").unwrap();
    (folder, invoice)
}

fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

fn naming(machine: &MachineId) -> Option<String> {
    (*machine == reception()).then(|| RECEPTION.to_owned())
}

fn found_by_discovery(machine: MachineId, port: u16) -> Found {
    let there = UdpSocket::bind("127.0.0.1:0").unwrap();
    there
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let at = there.local_addr().unwrap();
    let answering = Answering::on(there, Presence::of(machine.clone(), port));
    let answered = thread::spawn(move || answering.answer_one());
    let here = UdpSocket::bind("127.0.0.1:0").unwrap();
    here.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let looking = Looking::from(here);
    looking.ask(at).unwrap();
    let found = looking.found(Duration::from_secs(5)).unwrap();
    assert!(answered.join().unwrap().unwrap().is_some());
    found
        .into_iter()
        .find(|one| one.machine == machine)
        .unwrap()
}

/// What the studio was left holding.
struct Studio {
    heard: Vec<Heard>,
    invoice: PathBuf,
}

/// The studio, granting reception its folder, hearing `how_many`
/// connections, and — after each — answering every change waiting as its
/// person would: approving when `approving`, declining otherwise.
fn a_studio(
    what: &str,
    how_many: usize,
    pairings: Pairings,
    approving: bool,
) -> (Found, PathBuf, JoinHandle<Studio>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (folder, invoice) = a_folder_with_an_invoice(what);
    let studio_folder = folder.clone();
    let handle = thread::spawn(move || {
        let strings = Strings::of(everything_a_machine_says());
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = NothingIsBounded;
        let mut heard = Vec::new();
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let mut doorway = Doorway::at(the_studio(), &mut machine, hour(), hour()).unwrap();
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(
                RECEPTIONS_PRINCIPAL,
                Reach::Folder(studio_folder.clone()),
                a_moment(),
                hour(),
            )
            .unwrap(),
        );
        let receiving = Receiving::on(listener);
        for nth in 0..how_many {
            let arrived = receiving.accept_one().unwrap();
            let now = the_moment_of(nth);
            let what_happened = arrived
                .considered(
                    &mut doorway,
                    &pairings,
                    &mut grants,
                    &naming,
                    &EgressPolicy::InTheBuilding,
                    now,
                )
                .unwrap();
            heard.push(what_happened);
            let waiting: Vec<_> = doorway
                .turn()
                .map(|turn| turn.waiting_at(now).map(|change| change.id).collect())
                .unwrap_or_default();
            for id in waiting {
                let turn = doorway.turn().unwrap();
                if approving {
                    turn.approving(id, &pairings, &grants, now).unwrap();
                } else {
                    turn.declining(id, now).unwrap();
                }
            }
        }
        Studio { heard, invoice }
    });
    (found_by_discovery(the_studio(), port), folder, handle)
}

/// One turn on reception, ending when the closure is done, with the record
/// and the indicator left for the test to read.
fn on_reception<T>(
    record: &mut Record,
    indicator: &mut Indicator,
    doing: impl FnOnce(&mut Turning<'_, '_>) -> T,
) -> T {
    let strings = Strings::of(everything_a_machine_says());
    let mut bounding = NothingIsBounded;
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        indicator,
        record,
    )
    .unwrap();
    let mut grants = Grants::default();
    let mut turning = Turning::beginning(
        alo_context::Context::at_invocation(a_moment()),
        "@files",
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    let outcome = doing(&mut turning);
    assert!(!turning.ending(&mut grants));
    outcome
}

/// How many entries in a record say something left.
fn how_many_left(record: &Record) -> usize {
    record
        .answering(&Asking::anything())
        .filter(|entry| matches!(entry.happened(), Happened::Left { .. }))
        .count()
}

/// **A turn crosses a read and a change, the departure is written down
/// each time, and what became of the change is asked for on the wire.**
/// The studio's person approves it, and reception is told so — and told
/// *nothing waiting* for a number nobody proposed.
#[test]
fn a_turn_crosses_a_verb_and_learns_what_became_of_it() {
    let (at_reception, at_studio) = paired();
    let (found, folder, studio) = a_studio("approved", 4, at_studio, true);
    let mut record = Record::default();
    let mut indicator = Indicator::default();
    let places = Places::under(&SourcePolicy::InTheBuilding);
    let now = a_moment();

    let (listing, waits, became, nothing) = on_reception(&mut record, &mut indicator, |turning| {
        let agent = turning.grantee().clone();
        let crossing =
            || Crossing::to(&at_reception, &reception(), &found, THE_STUDIO, &agent, now).unwrap();
        let listing = crossing()
            .reading_through(
                turning,
                "list_folder",
                &[("folder", as_given(&folder))],
                &places,
                now,
            )
            .unwrap();
        let waits = crossing()
            .changing_through(
                turning,
                "rename_file",
                &[
                    ("file", as_given(&folder.join("march.pdf"))),
                    ("name", Given::text("march-final.pdf")),
                ],
                &places,
                now,
            )
            .unwrap();
        let number = waits.waits().unwrap();
        let became = crossing()
            .asking_after_through(turning, number, &places, now)
            .unwrap();
        let nothing = crossing()
            .asking_after_through(turning, number + 40, &places, now)
            .unwrap();
        (listing, waits, became, nothing)
    });

    let studio = studio.join().unwrap();
    assert!(listing.did().is_some(), "{listing:?}");
    assert!(waits.waits().is_some(), "{waits:?}");
    assert_eq!(became.became(), Some(Outcome::Approved), "{became:?}");
    assert_eq!(
        nothing.became(),
        Some(Outcome::NothingWaiting),
        "{nothing:?}"
    );
    assert!(
        studio.invoice.with_file_name("march-final.pdf").is_file(),
        "the studio's person approved the rename and the file did not move"
    );
    assert_eq!(studio.heard.len(), 4);
    assert!(
        studio
            .heard
            .iter()
            .all(|heard| matches!(heard, Heard::Answered { .. })),
        "{:?}",
        studio.heard
    );
    // Law 1's second half, four times: every crossing is in reception's
    // record as having left, and the indicator is quiet again.
    assert_eq!(how_many_left(&record), 4);
    assert!(indicator.is_quiet());
}

/// **The studio's person declines, and reception is told so.**
#[test]
fn a_change_the_studios_person_declined_comes_back_as_declined() {
    let (at_reception, at_studio) = paired();
    let (found, folder, studio) = a_studio("declined", 2, at_studio, false);
    let mut record = Record::default();
    let mut indicator = Indicator::default();
    let places = Places::under(&SourcePolicy::InTheBuilding);
    let now = a_moment();

    let became = on_reception(&mut record, &mut indicator, |turning| {
        let agent = turning.grantee().clone();
        let waits = Crossing::to(&at_reception, &reception(), &found, THE_STUDIO, &agent, now)
            .unwrap()
            .changing_through(
                turning,
                "rename_file",
                &[
                    ("file", as_given(&folder.join("march.pdf"))),
                    ("name", Given::text("march-final.pdf")),
                ],
                &places,
                now,
            )
            .unwrap();
        Crossing::to(&at_reception, &reception(), &found, THE_STUDIO, &agent, now)
            .unwrap()
            .asking_after_through(turning, waits.waits().unwrap(), &places, now)
            .unwrap()
    });

    let studio = studio.join().unwrap();
    assert_eq!(became.became(), Some(Outcome::Declined), "{became:?}");
    assert!(studio.invoice.is_file(), "a declined rename moved the file");
    assert_eq!(how_many_left(&record), 2);
}

/// **A rule that says nothing leaves holds the crossing back inside the
/// turn**: nothing is dialled, the studio hears nothing, the refusal is in
/// reception's record as held back, and the turn says so.
#[test]
fn a_rule_that_says_nothing_leaves_holds_the_crossing_back_before_it_is_dialled() {
    let (at_reception, at_studio) = paired();
    // The studio hears nothing, so it is told to hear nothing.
    let (found, folder, studio) = a_studio("held-back", 0, at_studio, true);
    let mut record = Record::default();
    let mut indicator = Indicator::default();
    let places = Places::under(&SourcePolicy::ThisMachineOnly);
    let now = a_moment();

    let held = on_reception(&mut record, &mut indicator, |turning| {
        let agent = turning.grantee().clone();
        Crossing::to(&at_reception, &reception(), &found, THE_STUDIO, &agent, now)
            .unwrap()
            .reading_through(
                turning,
                "list_folder",
                &[("folder", as_given(&folder))],
                &places,
                now,
            )
            .unwrap_err()
    });

    let studio = studio.join().unwrap();
    assert!(studio.heard.is_empty());
    assert!(
        matches!(held, NotThrough::TheTurn(NoAnswer::HeldBack(_))),
        "{held:?}"
    );
    assert!(!held.something_left());
    assert_eq!(how_many_left(&record), 0);
    assert_eq!(
        record
            .answering(&Asking::anything())
            .filter(|entry| matches!(entry.happened(), Happened::HeldBack { .. }))
            .count(),
        1
    );
    assert!(indicator.is_quiet());
}

/// **An unpaired machine is not a place a turn may cross to**: nothing
/// leaves, nothing is written, and the turn hands the refusal back.
#[test]
fn a_turn_cannot_cross_to_a_machine_this_one_is_not_paired_with() {
    let (_, at_studio) = paired();
    let (found, _, studio) = a_studio("not-paired", 0, at_studio, true);
    let mut record = Record::default();
    let mut indicator = Indicator::default();
    let now = a_moment();

    let refused = on_reception(&mut record, &mut indicator, |turning| {
        Crossing::to(
            &Pairings::none(),
            &reception(),
            &found,
            THE_STUDIO,
            turning.grantee(),
            now,
        )
        .map(|_| ())
        .unwrap_err()
    });

    let studio = studio.join().unwrap();
    assert!(studio.heard.is_empty());
    assert!(
        matches!(refused, alo_corridor::NotCrossed::NotPairedWithIt),
        "{refused:?}"
    );
    assert_eq!(record.answering(&Asking::anything()).count(), 0);
    assert!(indicator.is_quiet());
}
