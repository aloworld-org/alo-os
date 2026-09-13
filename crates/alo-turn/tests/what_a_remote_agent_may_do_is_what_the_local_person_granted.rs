//! *An agent on machine A that reaches machine B is bound by the grants made on
//! B, by B's person. A's grants confer nothing on B. Pairing lets A ask; it
//! never lets A act.* — ADR 0003, and this file is that sentence held up from
//! B's side.
//!
//! | The acceptance | The test |
//! |---|---|
//! | a verb from a paired machine is evaluated against the receiving machine's grants | [`a_verb_from_a_paired_machine_is_evaluated_against_this_machines_grants`] |
//! | shown to the receiving machine's person for approval | [`a_change_from_a_paired_machine_waits_for_the_person_on_this_machine`] |
//! | recorded there with the origin machine named | [`it_is_recorded_here_with_the_origin_machine_named`] |
//! | refused where the receiving machine's person has not granted it, even where the asking machine's person did | [`a_verb_this_machines_person_has_not_granted_is_refused_even_where_the_asking_machines_person_granted_it`] |
//! | the refusal says the grant was not made here rather than blaming the person who asked | [`the_refusal_says_the_grant_was_not_made_here_rather_than_blaming_the_person_who_asked`] |
//! | the kernel boundary applies exactly as for a local turn | [`a_remote_turn_that_cannot_be_bounded_still_does_not_run`] |
//!
//! And the refusals beside them, each named for the thing it refuses: a grant
//! made to this machine's own agent, a machine nobody paired with, a pairing
//! that ended during the turn, a grant taken back during the turn, and
//! anything of this machine's screen.
//!
//! # Two machines, on one host
//!
//! There is one `Machine` in every test here and it is B — the machine that
//! was asked. A is present as an identity in B's pairings and, in the one test
//! that needs it, as a list of grants its own person made: a list that is
//! built, shown to permit the very thing B refuses, and handed to nothing,
//! because there is nowhere on B's door to hand it. That absence is the test.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{AnswerError, Given, Grant, Grantee, Grants, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Reaching, Resolving as _, file_words};
use std::sync::OnceLock;

use alo_nearby::{
    Deliberating, Keying, MachineId, MayAskIts, NotProven, Origin, Pairing, Pairings, Proof,
    Proposal, Seen, Side,
};
use alo_record::{Asking, Happened, Only, Record, Stopped};
use alo_strings::{Strings, Vocabulary};
use alo_turn::{Arriving, Bounding, Doing, Done, Machine, NoBoundary, NotDone, Turning};

/// This machine — B, the one being asked.
fn here() -> MachineId {
    MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
}

/// The machine down the corridor — A, the one asking.
fn the_reception() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// What B's person called A when they paired.
const CALLED: &str = "the reception machine";

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a turn, a grant, a pairing and a question stand here.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The one pairing two people made in this file, as each machine keeps it:
/// A's row naming B first, B's row naming A second, the same key on both
/// (ADR 0031). Made once, because every proof A makes has to be made with the
/// key B checks it against.
fn the_pairing() -> &'static (Pairing, Pairing) {
    static THE_PAIRING: OnceLock<(Pairing, Pairing)> = OnceLock::new();
    THE_PAIRING.get_or_init(|| {
        let at_a = Keying::fresh().unwrap();
        let at_b = Keying::fresh().unwrap();
        let proposal = Proposal::checked(
            the_reception(),
            here(),
            &[MayAskIts::Models],
            hour(),
            at_a.offer().clone(),
        )
        .unwrap();
        let on_b = Deliberating::asked(proposal.clone(), at_b);
        let on_a = Deliberating::asking(proposal, at_a)
            .unwrap()
            .answered_with(on_b.answered().unwrap().clone())
            .unwrap();
        assert_eq!(on_a.code(), on_b.code());
        (
            on_a.agreed_at(Side::TheOneAsking)
                .agreed_at(Side::TheOneAsked)
                .agreed(noon())
                .unwrap(),
            on_b.agreed_at(Side::TheOneAsking)
                .agreed_at(Side::TheOneAsked)
                .agreed(noon())
                .unwrap(),
        )
    })
}

/// A pairing two people made, as B keeps it: naming A.
fn paired_with_the_reception() -> Pairings {
    let mut pairings = Pairings::none();
    pairings.keep(the_pairing().1.clone());
    pairings
}

/// A proof, made at A with A's row, that a verb arriving at B is A's.
fn a_proof_from_the_reception(about: &[u8], at: SystemTime) -> Proof {
    Proof::made(&the_pairing().0, &the_reception(), about, at)
}

/// A, as a place a verb arrives at B from — which B accepts only because the
/// verb proved it (ADR 0031).
fn from_the_reception(pairings: &Pairings) -> Origin {
    Origin::proven(
        pairings,
        &here(),
        &a_proof_from_the_reception(b"a turn", noon()),
        b"a turn",
        CALLED,
        noon(),
        &mut Seen::nothing(),
    )
    .unwrap()
}

/// Every word B has loaded: the nine lists a turn can hand back a refusal
/// worded by, so that a missing string is a failing test and not a passing one.
fn everything_b_says() -> Vocabulary {
    let mut vocabulary = file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_keeping::declare_into(&mut vocabulary).unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_answering::declare_into(&mut vocabulary).unwrap();
    alo_asking::declare_into(&mut vocabulary).unwrap();
    alo_nearby::words::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// A machine with nothing in front of a turn, which is not a machine alo OS
/// ships — `alo_turn::bounding` says why no library here has one, and why a
/// test writes these lines where its reader can see them.
struct NothingIsBounded;

impl Bounding for NothingIsBounded {
    fn carrying_out(&mut self, _reaching: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary> {
        Ok(doing.done())
    }
}

/// A machine that cannot put a boundary around anything at all.
struct NoBoundaryAtAll;

impl Bounding for NoBoundaryAtAll {
    fn carrying_out(
        &mut self,
        _reaching: &Reaching,
        _doing: Doing<'_>,
    ) -> Result<Done, NoBoundary> {
        Err(NoBoundary::because(
            "the kernel would not take an entry for the boundary".to_owned(),
        ))
    }
}

/// A folder of this test's own with one file in it, both resolved — a grant is
/// over a place, and on Windows a resolved path carries a prefix the typed one
/// does not (`docs/quirks.md`).
fn a_folder_with_an_invoice(what: &str) -> (PathBuf, PathBuf) {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-remote-{}-{what}-{}",
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

/// A path as a verb's argument arrives: text.
fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// What the one change these tests make is asked for with.
fn renaming(invoice: &Path) -> Vec<(&'static str, Given)> {
    vec![
        ("file", as_given(invoice)),
        ("name", Given::text("march-final.pdf")),
    ]
}

/// A grant on B, made by B's person, to `whom`, over this folder.
fn b_grants(whom: &str, folder: &Path) -> Grants {
    let mut grants = Grants::default();
    grants
        .grant(Grant::checked(whom, Reach::Folder(folder.to_path_buf()), noon(), hour()).unwrap());
    grants
}

/// B, with a turn from A under way on it.
///
/// A closure rather than a function returning the turn, because an
/// [`Arriving`] borrows the [`Machine`] and the machine borrows the record —
/// so all three live in the caller's frame. `grants` are B's, made by B's
/// person to whoever the test says.
fn on_b<T>(
    what: &str,
    record: &mut Record,
    bounding: &mut dyn Bounding,
    granting: impl FnOnce(&Origin, &Path) -> Grants,
    doing: impl FnOnce(&mut Arriving<'_, '_>, &mut Grants, &Pairings, &Path, &Path) -> T,
) -> T {
    let strings = Strings::of(everything_b_says());
    let (folder, invoice) = a_folder_with_an_invoice(what);
    let mut indicator = Indicator::default();
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        bounding,
        &mut indicator,
        record,
    )
    .unwrap();
    let pairings = paired_with_the_reception();
    let origin = from_the_reception(&pairings);
    let mut grants = granting(&origin, &folder);
    let mut arriving =
        Arriving::beginning(&origin, hour(), noon(), &mut grants, &mut machine).unwrap();
    doing(&mut arriving, &mut grants, &pairings, &folder, &invoice)
}

/// **A verb from a paired machine is evaluated against the receiving
/// machine's grants.** B's person granted the reception machine this folder,
/// on B; a read arriving from it runs, and the record names B's grant as the
/// one it ran against and the machine's principal as whose authority it was.
#[test]
fn a_verb_from_a_paired_machine_is_evaluated_against_this_machines_grants() {
    let mut record = Record::default();
    let principal = on_b(
        "evaluated-here",
        &mut record,
        &mut NothingIsBounded,
        |origin, folder| b_grants(&origin.principal(), folder),
        |arriving, grants, pairings, folder, _| {
            let held: Vec<_> = grants
                .active_at(noon())
                .map(|held| held.id.as_u64())
                .collect();
            let answer = arriving
                .reading(
                    "list_folder",
                    &[("folder", as_given(folder))],
                    pairings,
                    grants,
                    noon(),
                )
                .unwrap();
            assert_eq!(answer.listed().unwrap().things().len(), 1);
            assert_eq!(held.len(), 1);
            (arriving.grantee().clone(), held)
        },
    );

    assert_eq!(record.len(), 1);
    let entry = record.everything().next().unwrap();
    assert!(entry.happened().ran());
    assert_eq!(entry.happened().against(), principal.1);
    assert_eq!(
        entry.agent(),
        Some(&alo_record::Line::of(principal.0.as_str()))
    );
    assert!(
        principal.0.as_str().starts_with("machine:"),
        "the grants were asked for something other than the machine's principal: {}",
        principal.0.as_str()
    );
}

/// **The test this task exists for.** A's person granted A's `@files` this
/// folder — on A. B's person granted nothing. The verb arrives at B and is
/// refused: as a read, at the moment it would have run, and as a change,
/// before anybody was asked. A's grants are built, shown to permit exactly
/// this, and handed to nothing, because B's door has nowhere to take them.
#[test]
fn a_verb_this_machines_person_has_not_granted_is_refused_even_where_the_asking_machines_person_granted_it()
 {
    let mut record = Record::default();
    let still_there = on_b(
        "granted-elsewhere",
        &mut record,
        &mut NothingIsBounded,
        |_, _| Grants::default(),
        |arriving, grants, pairings, folder, invoice| {
            // On A, the person granted it. That is a real grant, and it permits
            // the very call B is about to refuse.
            let on_a = b_grants("@files", folder);
            let listing = alo_files::file_verbs()
                .unwrap()
                .call("list_folder", &[("folder", as_given(folder))])
                .unwrap();
            assert!(
                listing.permitted_by(&on_a, &Grantee::named("@files"), noon()),
                "the asking machine's grant does not even permit it there"
            );

            // On B, nothing was granted — and `on_a` goes nowhere, because
            // there is no parameter for it.
            let refused = arriving
                .reading(
                    "list_folder",
                    &[("folder", as_given(folder))],
                    pairings,
                    grants,
                    noon(),
                )
                .unwrap_err();
            assert!(matches!(refused, NotDone::Refused(_)), "{refused:?}");
            assert!(refused.was_refused());

            let never_asked = arriving
                .proposing(
                    "rename_file",
                    &renaming(invoice),
                    pairings,
                    grants,
                    hour(),
                    noon(),
                )
                .unwrap_err();
            assert!(
                matches!(never_asked, NotDone::NeverAsked(_)),
                "{never_asked:?}"
            );
            assert_eq!(
                arriving.waiting_at(noon()).count(),
                0,
                "B's person was interrupted about a change B's grants never permitted"
            );
            invoice.is_file()
        },
    );

    assert!(still_there, "a change ran on B under a grant made on A");
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Refusals))
            .count(),
        2,
        "two refusals happened and the record does not hold two"
    );
    assert!(
        record
            .everything()
            .all(|entry| entry.origin().is_some_and(|origin| origin.is(CALLED))),
        "a refusal of the reception machine's verb was recorded as though this machine's own"
    );
}

/// **A change from a paired machine is put to this machine's person**, on
/// this machine's list, and runs once when they approve it — and not again.
/// Nobody on the asking machine is asked anything.
#[test]
fn a_change_from_a_paired_machine_waits_for_the_person_on_this_machine() {
    let mut record = Record::default();
    let renamed = on_b(
        "waits-here",
        &mut record,
        &mut NothingIsBounded,
        |origin, folder| b_grants(&origin.principal(), folder),
        |arriving, grants, pairings, _, invoice| {
            let id = arriving
                .proposing(
                    "rename_file",
                    &renaming(invoice),
                    pairings,
                    grants,
                    hour(),
                    noon(),
                )
                .unwrap();

            // The question is on B's list, worded as B's person reads it, and
            // whose question it is is the machine's principal on B.
            let waiting: Vec<_> = arriving.waiting_at(noon()).collect();
            assert_eq!(waiting.len(), 1);
            let asked = arriving.proposed(id).unwrap();
            assert_eq!(asked.proposal.grantee(), arriving.grantee());
            assert!(
                asked
                    .proposal
                    .sentence(arriving.turning().strings())
                    .text()
                    .contains("march-final.pdf")
            );
            assert!(invoice.is_file(), "a change ran before anybody approved it");

            // B's person approves it, on B. It runs once.
            let answer = arriving.approving(id, pairings, grants, noon()).unwrap();
            let now_at = answer.now_at().unwrap().to_path_buf();
            let again = arriving
                .approving(id, pairings, grants, noon())
                .unwrap_err();
            assert!(matches!(
                again,
                NotDone::NotAnswered(AnswerError::NothingWaiting { .. })
            ));
            now_at
        },
    );

    assert!(renamed.ends_with("march-final.pdf"));
    assert!(renamed.is_file(), "the file did not move on the disk");
    assert_eq!(
        record.len(),
        1,
        "either a question was recorded, or an approval ran twice"
    );
    let entry = record.everything().next().unwrap();
    assert!(entry.happened().ran());
    assert!(entry.happened().from_approval().is_some());
    assert!(entry.origin().is_some_and(|origin| origin.is(CALLED)));
}

/// **It is recorded here, with the origin machine named.** Every entry a
/// remote turn writes — what ran and what was refused — says which machine it
/// came from, by the name B's person gave it; a local turn's entry on the same
/// machine says nothing of the kind; and the record answers *what did other
/// machines cause here* as one question.
#[test]
fn it_is_recorded_here_with_the_origin_machine_named() {
    let mut record = Record::default();
    let (folder, invoice, grants) = {
        let strings = Strings::of(everything_b_says());
        let (folder, invoice) = a_folder_with_an_invoice("recorded-here");
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
        let pairings = paired_with_the_reception();
        let origin = from_the_reception(&pairings);
        let mut grants = b_grants(&origin.principal(), &folder);
        grants.grant(
            Grant::checked("@files", Reach::Folder(folder.clone()), noon(), hour()).unwrap(),
        );

        // From the reception machine: one read that runs, one that is refused.
        let mut arriving =
            Arriving::beginning(&origin, hour(), noon(), &mut grants, &mut machine).unwrap();
        assert!(
            arriving
                .reading(
                    "list_folder",
                    &[("folder", as_given(&folder))],
                    &pairings,
                    &grants,
                    noon()
                )
                .is_ok()
        );
        let elsewhere = a_folder_with_an_invoice("recorded-elsewhere").0;
        assert!(
            arriving
                .reading(
                    "list_folder",
                    &[("folder", as_given(&elsewhere))],
                    &pairings,
                    &grants,
                    noon()
                )
                .is_err()
        );
        arriving.ending(&mut grants);

        // And from this machine's own agent: one read that runs.
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@files",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        assert!(
            turning
                .reading(
                    "list_folder",
                    &[("folder", as_given(&folder))],
                    &grants,
                    noon()
                )
                .is_ok()
        );
        assert!(!turning.ending(&mut grants));
        (folder, invoice, grants)
    };

    assert_eq!(record.len(), 3);
    let entries: Vec<_> = record.everything().collect();
    let [ran_from_elsewhere, refused_from_elsewhere, ours] = entries.as_slice() else {
        unreachable!("three things happened and the record holds three")
    };
    for from_elsewhere in [ran_from_elsewhere, refused_from_elsewhere] {
        assert!(
            from_elsewhere
                .origin()
                .is_some_and(|origin| origin.is(CALLED)),
            "{from_elsewhere:?}"
        );
    }
    assert_eq!(
        ours.origin(),
        None,
        "this machine's own agent was recorded as another machine"
    );
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::FromAnotherMachine))
            .count(),
        2
    );
    // The origin is not the authority: whose grants it ran under is the
    // machine's principal, and this machine's own agent is findable by name
    // and finds nothing of the reception machine's.
    assert_eq!(
        record.answering(&Asking::anything().by("@files")).count(),
        1
    );
    assert!(
        record
            .answering(&Asking::anything().by(CALLED))
            .next()
            .is_none(),
        "the name a person gave a machine was written down as an authority"
    );
    assert!(invoice.is_file());
    assert_eq!(grants.active_at(noon()).count(), 2);
    drop(folder);
}

/// **The refusal says the grant was not made here, rather than blaming the
/// person who asked.** On both roads a remote verb is refused on — before
/// anybody was asked, and at the moment — the sentence names the machine by
/// the name B's person gave it, says *on this machine*, and does not tell
/// anybody to go and pick a folder; the record keeps that same sentence. And
/// a grant that ran out on B says it ran out here.
#[test]
fn the_refusal_says_the_grant_was_not_made_here_rather_than_blaming_the_person_who_asked() {
    let strings = Strings::of(everything_b_says());
    let mut record = Record::default();
    on_b(
        "not-made-here",
        &mut record,
        &mut NothingIsBounded,
        |_, _| Grants::default(),
        |arriving, grants, pairings, folder, invoice| {
            let at_the_moment = arriving
                .reading(
                    "list_folder",
                    &[("folder", as_given(folder))],
                    pairings,
                    grants,
                    noon(),
                )
                .unwrap_err();
            let before_anybody_was_asked = arriving
                .proposing(
                    "rename_file",
                    &renaming(invoice),
                    pairings,
                    grants,
                    hour(),
                    noon(),
                )
                .unwrap_err();

            for refused in [at_the_moment, before_anybody_was_asked] {
                let said = refused.said(&strings);
                assert!(!said.is_a_bug(), "{said}");
                assert!(said.text().contains(CALLED), "{said}");
                assert!(said.text().contains("on this machine"), "{said}");
                assert!(said.text().contains("not been granted"), "{said}");
                assert!(
                    !said.text().contains("picking a folder"),
                    "the person who asked was told to pick a folder on the wrong machine: {said}"
                );
                assert!(
                    !said.text().contains("machine:"),
                    "a machine's identity reached a sentence a person reads: {said}"
                );
            }
        },
    );

    // The record keeps the sentence the person was shown, on both roads.
    let stopped: Vec<_> = record
        .everything()
        .filter_map(|entry| entry.happened().stopped())
        .collect();
    assert!(
        matches!(
            stopped.as_slice(),
            [Stopped::AtTheMoment(_), Stopped::BeforeAnybodyWasAsked(_)]
        ),
        "{stopped:?}"
    );
    for how in stopped {
        let why = how.why().unwrap();
        assert!(why.as_str().contains("on this machine"), "{why}");
        assert!(why.as_str().contains(CALLED), "{why}");
    }

    // And a grant B's person made and that has run out says so, here: made
    // for half an hour, while the pairing stands for the whole one.
    let mut record = Record::default();
    on_b(
        "expired-here",
        &mut record,
        &mut NothingIsBounded,
        |origin, folder| {
            let mut grants = Grants::default();
            grants.grant(
                Grant::checked(
                    &origin.principal(),
                    Reach::Folder(folder.to_path_buf()),
                    noon(),
                    Duration::from_secs(30 * 60),
                )
                .unwrap(),
            );
            grants
        },
        |arriving, grants, pairings, folder, _| {
            let expired = arriving
                .reading(
                    "list_folder",
                    &[("folder", as_given(folder))],
                    pairings,
                    grants,
                    noon() + Duration::from_secs(30 * 60),
                )
                .unwrap_err();
            let said = expired.said(&strings);
            assert!(said.text().contains("has expired"), "{said}");
            assert!(said.text().contains("on this machine"), "{said}");
            assert!(said.text().contains(CALLED), "{said}");
            assert!(!said.text().contains("grant it again"), "{said}");
        },
    );
}

/// **A grant to this machine's own agent does not reach a remote one.** B's
/// person granted `@files` this folder — their own `@files`. A verb from the
/// reception machine, whose agent may well be called `@files` too, is refused,
/// and the grant is real: B's own `@files` reads the folder a moment later.
#[test]
fn a_grant_to_this_machines_own_agent_does_not_reach_a_remote_one() {
    let mut record = Record::default();
    let strings = Strings::of(everything_b_says());
    let (folder, _invoice) = a_folder_with_an_invoice("own-agent");
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
    let pairings = paired_with_the_reception();
    let origin = from_the_reception(&pairings);
    let mut grants = b_grants("@files", &folder);

    let mut arriving =
        Arriving::beginning(&origin, hour(), noon(), &mut grants, &mut machine).unwrap();
    let refused = arriving
        .reading(
            "list_folder",
            &[("folder", as_given(&folder))],
            &pairings,
            &grants,
            noon(),
        )
        .unwrap_err();
    assert!(refused.was_refused(), "{refused:?}");
    arriving.ending(&mut grants);

    let mut turning = Turning::beginning(
        Context::at_invocation(noon()),
        "@files",
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    assert!(
        turning
            .reading(
                "list_folder",
                &[("folder", as_given(&folder))],
                &grants,
                noon()
            )
            .is_ok(),
        "the grant that refused the reception machine is not a real grant"
    );
    assert!(!turning.ending(&mut grants));

    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Refusals))
            .count(),
        1
    );
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Executions))
            .count(),
        1
    );
}

/// **A machine nobody paired with cannot begin a turn here.** There is no
/// origin for it, and a turn cannot begin without one — so nothing on B is
/// asked, and nothing is written, for a machine that was merely seen, one
/// whose pairing ran out, or one whose pairing was undone.
#[test]
fn a_machine_nobody_paired_with_cannot_begin_a_turn_here() {
    let seen_only = Origin::proven(
        &Pairings::none(),
        &here(),
        &a_proof_from_the_reception(b"a turn", noon()),
        b"a turn",
        CALLED,
        noon(),
        &mut Seen::nothing(),
    );
    assert_eq!(seen_only.unwrap_err(), NotProven::NotWithThatMachine);

    let ran_out = Origin::proven(
        &paired_with_the_reception(),
        &here(),
        &a_proof_from_the_reception(b"a turn", noon() + hour()),
        b"a turn",
        CALLED,
        noon() + hour(),
        &mut Seen::nothing(),
    );
    assert_eq!(ran_out.unwrap_err(), NotProven::NotWithThatMachine);

    let mut pairings = paired_with_the_reception();
    assert!(pairings.revoke(&the_reception()));
    let undone = Origin::proven(
        &pairings,
        &here(),
        &a_proof_from_the_reception(b"a turn", noon()),
        b"a turn",
        CALLED,
        noon(),
        &mut Seen::nothing(),
    );
    assert_eq!(undone.unwrap_err(), NotProven::NotWithThatMachine);

    // And the sentence for it says nothing was considered, in words B has.
    let said = NotProven::NotWithThatMachine.said(&Strings::of(everything_b_says()));
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("not paired"), "{said}");
}

/// **A stranger presenting A's identity cannot begin a turn here either**, and
/// is refused before any grant is asked (ADR 0031): B is paired with A, the
/// stranger read A's identity off the network and holds a key of its own,
/// and the one door a remote turn begins from takes a proof and not a name.
#[test]
fn a_stranger_presenting_the_receptions_identity_cannot_begin_a_turn_here() {
    let strangers_keying = Keying::fresh().unwrap();
    let a_stranger = MachineId::read("99998888777766665555444433332222").unwrap();
    let proposal = Proposal::checked(
        a_stranger,
        here(),
        &[MayAskIts::Models],
        hour(),
        strangers_keying.offer().clone(),
    )
    .unwrap();
    let somebody_elses = Deliberating::asked(proposal.clone(), Keying::fresh().unwrap());
    let strangers_row = Deliberating::asking(proposal, strangers_keying)
        .unwrap()
        .answered_with(somebody_elses.answered().unwrap().clone())
        .unwrap()
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(noon())
        .unwrap();
    let presenting = Proof::made(&strangers_row, &the_reception(), b"a turn", noon());
    assert_eq!(presenting.from(), &the_reception());

    let refused = Origin::proven(
        &paired_with_the_reception(),
        &here(),
        &presenting,
        b"a turn",
        CALLED,
        noon(),
        &mut Seen::nothing(),
    );
    assert_eq!(refused.unwrap_err(), NotProven::NotFromThatMachine);

    let said = NotProven::NotFromThatMachine.said(&Strings::of(everything_b_says()));
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("could not prove"), "{said}");
}

/// **A pairing undone during a turn stops it at the next door**, taking effect
/// immediately: a change proposed while paired and approved after the pairing
/// was revoked does not run, and is written down as stopped at the moment with
/// the pairing named as why. A read and a proposal after it are refused
/// before the grants are asked.
#[test]
fn a_pairing_undone_during_a_turn_stops_it_at_the_next_door() {
    let strings = Strings::of(everything_b_says());
    let mut record = Record::default();
    let still_there = on_b(
        "pairing-undone",
        &mut record,
        &mut NothingIsBounded,
        |origin, folder| b_grants(&origin.principal(), folder),
        |arriving, grants, pairings, folder, invoice| {
            let id = arriving
                .proposing(
                    "rename_file",
                    &renaming(invoice),
                    pairings,
                    grants,
                    hour(),
                    noon(),
                )
                .unwrap();

            // B's person undoes the pairing between the question and the answer.
            let mut pairings = pairings.clone();
            assert!(pairings.revoke(&the_reception()));

            let stopped = arriving
                .approving(id, &pairings, grants, noon())
                .unwrap_err();
            assert!(matches!(stopped, NotDone::Refused(_)), "{stopped:?}");
            let said = stopped.said(&strings);
            assert!(said.text().contains("no longer paired"), "{said}");
            assert!(said.text().contains(CALLED), "{said}");

            let read = arriving
                .reading(
                    "list_folder",
                    &[("folder", as_given(folder))],
                    &pairings,
                    grants,
                    noon(),
                )
                .unwrap_err();
            assert!(read.was_refused());
            assert!(read.said(&strings).text().contains("no longer paired"));

            let proposed = arriving
                .proposing(
                    "rename_file",
                    &renaming(invoice),
                    &pairings,
                    grants,
                    hour(),
                    noon(),
                )
                .unwrap_err();
            assert!(matches!(proposed, NotDone::NeverAsked(_)), "{proposed:?}");
            assert!(proposed.said(&strings).text().contains("no longer paired"));
            assert_eq!(arriving.waiting_at(noon()).count(), 0);
            invoice.is_file()
        },
    );

    assert!(still_there, "a change ran after the pairing was undone");
    let stopped: Vec<_> = record
        .everything()
        .filter_map(|entry| entry.happened().stopped())
        .collect();
    assert!(
        matches!(
            stopped.as_slice(),
            [
                Stopped::AtTheMoment(_),
                Stopped::AtTheMoment(_),
                Stopped::BeforeAnybodyWasAsked(_)
            ]
        ),
        "{stopped:?}"
    );
    assert!(
        record
            .everything()
            .all(|entry| entry.origin().is_some_and(|origin| origin.is(CALLED)))
    );
}

/// **A grant taken back on this machine stops a remote change at the moment**,
/// exactly as it stops a local one: the grants are asked last, and last is when
/// B's person's approval is spent.
#[test]
fn a_grant_taken_back_here_stops_a_remote_change_at_the_moment() {
    let mut record = Record::default();
    on_b(
        "grant-taken-back",
        &mut record,
        &mut NothingIsBounded,
        |origin, folder| b_grants(&origin.principal(), folder),
        |arriving, grants, pairings, _, invoice| {
            let id = arriving
                .proposing(
                    "rename_file",
                    &renaming(invoice),
                    pairings,
                    grants,
                    hour(),
                    noon(),
                )
                .unwrap();
            let held: Vec<_> = grants.active_at(noon()).map(|held| held.id).collect();
            for grant in held {
                assert!(grants.revoke(grant));
            }
            let stopped = arriving
                .approving(id, pairings, grants, noon())
                .unwrap_err();
            assert!(matches!(stopped, NotDone::Refused(_)), "{stopped:?}");
            assert!(invoice.is_file(), "the file moved after the grant was gone");
        },
    );
    let entry = record.everything().next().unwrap();
    assert!(entry.happened().was_stopped());
    assert!(entry.origin().is_some_and(|origin| origin.is(CALLED)));
}

/// **A remote turn is offered nothing of this machine.** No window, no
/// selection, no document: the context is empty, nothing was granted at the
/// turn's beginning, and ending it takes nothing back.
#[test]
fn a_remote_turn_is_offered_nothing_of_this_machine() {
    let mut record = Record::default();
    on_b(
        "offered-nothing",
        &mut record,
        &mut NothingIsBounded,
        |_, _| Grants::default(),
        |arriving, grants, _, _, _| {
            assert!(arriving.turning().context().is_empty());
            assert_eq!(arriving.turning().granted(), None);
            assert!(
                grants.is_empty(),
                "beginning a remote turn granted something"
            );
            assert_eq!(arriving.turning().ends(), noon() + hour());
            assert!(!arriving.is_closed());
            assert!(!arriving.a_thread_is_lost());
            assert_eq!(arriving.origin().called(), CALLED);
        },
    );
    assert!(
        record.is_empty(),
        "a turn that did nothing wrote something down"
    );
}

/// **A remote turn that cannot be bounded still does not run.** ADR 0013 and
/// ADR 0015 apply on the receiving machine exactly as for a local turn: with
/// no boundary the file is where it was, the machine's own refusal is written
/// down — stamped with where the verb came from — and it is not the grants
/// refusing.
#[test]
fn a_remote_turn_that_cannot_be_bounded_still_does_not_run() {
    let mut record = Record::default();
    let still_there = on_b(
        "no-boundary",
        &mut record,
        &mut NoBoundaryAtAll,
        |origin, folder| b_grants(&origin.principal(), folder),
        |arriving, grants, pairings, folder, invoice| {
            let id = arriving
                .proposing(
                    "rename_file",
                    &renaming(invoice),
                    pairings,
                    grants,
                    hour(),
                    noon(),
                )
                .unwrap();
            let not_bounded = arriving
                .approving(id, pairings, grants, noon())
                .unwrap_err();
            assert!(
                matches!(not_bounded, NotDone::NotBounded(_)),
                "{not_bounded:?}"
            );
            assert!(!not_bounded.was_refused());

            let read = arriving
                .reading(
                    "list_folder",
                    &[("folder", as_given(folder))],
                    pairings,
                    grants,
                    noon(),
                )
                .unwrap_err();
            assert!(matches!(read, NotDone::NotBounded(_)), "{read:?}");
            invoice.is_file()
        },
    );

    assert!(still_there, "a change ran with no boundary around it");
    assert_eq!(record.len(), 2);
    for entry in record.everything() {
        assert!(
            matches!(entry.happened(), Happened::NotBounded { .. }),
            "{:?}",
            entry.happened()
        );
        assert!(!entry.happened().ran());
        assert!(entry.origin().is_some_and(|origin| origin.is(CALLED)));
    }
}
