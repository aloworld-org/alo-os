//! The plan's acceptance for *one approval, and the sentence a person
//! approves*, one test per criterion.
//!
//! `docs/autonomy/v0-01-delivery-plan.md`, task 7: *a proposed change is shown
//! as the sentence `alo-turn` renders, approved once, carried out, and refused
//! after its proposal has expired — with the record carrying what was approved
//! and by whom.*
//!
//! These tests reach the machine the way a shell does, and not through a seam
//! of the test's own: the change is proposed through `alo_turn::Turning`
//! against the closed list of verbs, the question reaches a compositor through
//! the port `alo-shell` will implement, the file really moves on a real disk,
//! the record is written to a file and read back by the crate that reads
//! records, and the strings come from
//! `alo_saying::everything_this_machine_can_say` — the vocabulary a shell
//! really holds — so a string this crate declares and nobody collects fails
//! here rather than on somebody's screen.
//!
//! Nothing here draws anything, and nothing asserts a pixel. `ROADMAP.md`'s
//! exit gate is about a certified machine, and a green suite on a developer's
//! laptop is not that.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being               reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_approving::words::EVERY_WORD;
use alo_approving::{
    Answered, Approving, Asked, Asks, Compositor, NotAnswered, NotAsked, SurfaceRefused,
};
use alo_capability::{AnswerError, Given, Grant, Grants, ProposalId, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Reaching, Resolving};
use alo_keeping::{Reading, Writing};
use alo_record::{Asking, Only};
use alo_saying::everything_this_machine_can_say;
use alo_strings::Strings;
use alo_turn::{Bounding, Doing, Done, Machine, NoBoundary, NotDone, Turning};

/// The moment the person pressed the key.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the turn, the grants and the question stand.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The words this machine reads: every crate's, in one vocabulary, which is the
/// arrangement a shell is really in.
fn what_this_machine_says() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// A folder of this test's own, resolved — a grant is over a place, and on
/// Windows the resolved spelling is the one a grant has to be made with.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-approving-acceptance-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// A path as a verb's argument arrives: text.
fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// The invoice, the archive it is proposed for, and the file the record is kept
/// in.
struct Places {
    /// The folder the invoice is in.
    invoices: PathBuf,
    /// The folder it would be moved into.
    archive: PathBuf,
    /// The invoice itself.
    march: PathBuf,
    /// Where the record is written.
    kept_at: PathBuf,
}

impl Places {
    /// Two folders, one invoice, and somewhere to keep the record.
    fn made(what: &str) -> Self {
        let invoices = a_folder_of_our_own(&format!("{what}-invoices"));
        let archive = a_folder_of_our_own(&format!("{what}-archive"));
        let march = invoices.join("march.pdf");
        fs::write(&march, "March, 4180.00").unwrap();
        Self {
            invoices,
            archive,
            march,
            kept_at: a_folder_of_our_own(&format!("{what}-record")).join("record.jsonl"),
        }
    }

    /// Grants to `@files` over both folders, made at noon and lasting an hour.
    fn granting(&self) -> Grants {
        let mut grants = Grants::default();
        for folder in [&self.invoices, &self.archive] {
            grants.grant(
                Grant::checked("@files", Reach::Folder(folder.clone()), noon(), hour()).unwrap(),
            );
        }
        grants
    }

    /// Where the invoice would end up.
    fn archived(&self) -> PathBuf {
        self.archive.join("march.pdf")
    }
}

/// Stand a whole machine up, run one turn on it against a record on a real
/// disk, and close the record afterwards so it can be read back.
fn on_this_machine(places: &Places, body: impl FnOnce(&mut Turning<'_, '_>, &Grants, &Strings)) {
    let strings = what_this_machine_says();
    let mut grants = places.granting();
    let mut writing = Writing::opening(&places.kept_at).unwrap();
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        &mut writing,
    )
    .unwrap();
    let mut turning = Turning::beginning(
        Context::at_invocation(noon()),
        "@files",
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    body(&mut turning, &grants, &strings);
    assert!(!turning.is_closed(), "the machine stopped keeping evidence");
    let _ = turning.ending(&mut grants);
}

/// Moving the invoice into the archive, put to the person.
fn archiving(turning: &mut Turning<'_, '_>, grants: &Grants, places: &Places) -> ProposalId {
    turning
        .proposing(
            "move_file",
            &[
                ("file", as_given(&places.march)),
                ("into", as_given(&places.archive)),
            ],
            grants,
            hour(),
            noon(),
        )
        .unwrap()
}

/// What the sentence about this change reads as, in English.
fn the_sentence(places: &Places) -> String {
    format!(
        "move {} into {}",
        places.march.display(),
        places.archive.display()
    )
}

/// A screen, as this surface sees one: it keeps every question it was given, so
/// *one question, one surface* is a number rather than an impression.
#[derive(Default)]
struct Screen {
    /// Every question that arrived, in order.
    asked: Vec<Asked>,
}

impl Compositor for Screen {
    fn ask(&mut self, asked: Asked) -> Result<(), SurfaceRefused> {
        self.asked.push(asked);
        Ok(())
    }
}

/// A compositor with nothing to put a question on.
#[derive(Default)]
struct NoScreen {
    /// How many questions were offered to it and refused.
    offered: usize,
}

impl Compositor for NoScreen {
    fn ask(&mut self, _asked: Asked) -> Result<(), SurfaceRefused> {
        self.offered += 1;
        Err(SurfaceRefused::NothingToShowOn)
    }
}

/// A machine with nothing in front of a turn, which is not a machine alo OS
/// ships. `alo_turn::bounding` says why there is no such implementation in any
/// library here, and why what a test needs is these few lines.
struct NothingIsBounded;

impl Bounding for NothingIsBounded {
    fn carrying_out(&mut self, _reaching: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary> {
        Ok(doing.done())
    }

    fn carrying_out_a_departure(
        &mut self,
        _to: &[std::net::SocketAddr],
        doing: &mut dyn FnMut(),
    ) -> Result<(), NoBoundary> {
        doing();
        Ok(())
    }
}

/// **A proposed change is shown as the sentence `alo-turn` renders.**
///
/// The words on the surface are the ones the machine generated from the
/// arguments the call was validated with — not a description of the verb, not a
/// summary, and above all not something a model wrote. The compositor is handed
/// that sentence rather than left to compose one, so there is no point between
/// the capability model and the person's eyes at which the two could differ.
#[test]
fn a_proposed_change_is_shown_as_the_sentence_the_turn_renders() {
    let places = Places::made("shown");
    on_this_machine(&places, |turning, grants, strings| {
        let id = archiving(turning, grants, &places);
        let mut screen = Screen::default();
        let mut approving = Approving::nothing_to_answer();

        let Asks::Asked(asked) = approving.ask(Some(&mut screen), turning, id, noon()) else {
            panic!("a change that was waiting was not put to anybody");
        };

        // What reached the compositor is what the turn renders, and it says the
        // whole of what will happen.
        assert_eq!(screen.asked.len(), 1);
        assert_eq!(screen.asked.first().unwrap(), &asked);
        assert_eq!(asked.sentence().text(), the_sentence(&places));
        assert_eq!(
            asked.sentence(),
            &turning.proposed(id).unwrap().proposal.sentence(strings),
            "the screen and the turn worded one change differently"
        );
        assert!(!asked.sentence().is_a_bug(), "{}", asked.sentence());

        // Whose change it is, and how long is left to answer, come off the same
        // moment as the sentence.
        assert_eq!(asked.agent().as_str(), "@files");
        assert_eq!(asked.id(), id);
        assert_eq!(asked.lapses_in(), hour());

        // The two answers under it read on the machine's own vocabulary, and so
        // does every other string this crate declares — a sentence nobody
        // collected would reach a shell as a key where an answer belongs.
        assert!(!asked.approve_said(strings).is_a_bug());
        assert!(!asked.no_said(strings).is_a_bug());
        let vocabulary = everything_this_machine_can_say().unwrap();
        for word in EVERY_WORD {
            assert!(
                vocabulary.phrase(&word.key()).is_some(),
                "the machine cannot say {}",
                word.named()
            );
        }
    });
}

/// **Approved once**, and once means once.
///
/// The question comes off the surface as it is answered, so the second answer
/// never reaches the turn; the list underneath has taken the proposal off in the
/// same act, so an answer that arrives another way finds nothing either. Both
/// are asserted, because the surface's rule is a convenience and the list's rule
/// is the guarantee — and one invoice arrives in the archive.
#[test]
fn one_approval_carries_the_change_out_exactly_once() {
    let places = Places::made("once");
    on_this_machine(&places, |turning, grants, _| {
        let id = archiving(turning, grants, &places);
        let mut screen = Screen::default();
        let mut approving = Approving::nothing_to_answer();
        assert!(matches!(
            approving.ask(Some(&mut screen), turning, id, noon()),
            Asks::Asked(_)
        ));

        assert!(matches!(
            approving.approve(turning, grants, noon()),
            Answered::Carried(_)
        ));

        // A second answer on the same surface runs nothing.
        assert_eq!(
            approving.approve(turning, grants, noon()),
            Answered::Refused(NotAnswered::NothingToAnswer)
        );

        // And so does an answer that never came through the surface at all.
        assert!(matches!(
            turning.approving(id, grants, noon()),
            Err(NotDone::NotAnswered(AnswerError::NothingWaiting { .. }))
        ));
        assert!(turning.proposed(id).is_none());
    });

    // One file arrived, and it arrived once.
    assert!(places.archived().is_file());
    assert!(!places.march.exists());
    assert_eq!(fs::read_dir(&places.archive).unwrap().count(), 1);
}

/// **Carried out** — on a real disk, and not in a value that says it was.
///
/// What comes back from the answer is the verb's own, so a shell can say where
/// the file went without asking the machine a second question, and the disk
/// agrees with it. This is the middle of `ROADMAP.md`'s exit gate: *approve the
/// sentence, see it happen.*
#[test]
fn an_approved_change_really_happens() {
    let places = Places::made("carried");
    assert!(places.march.is_file());
    on_this_machine(&places, |turning, grants, _| {
        let id = archiving(turning, grants, &places);
        let mut screen = Screen::default();
        let mut approving = Approving::nothing_to_answer();
        assert!(matches!(
            approving.ask(Some(&mut screen), turning, id, noon()),
            Asks::Asked(_)
        ));

        // Nothing has happened yet: a question is not a thing that happened.
        assert!(places.march.is_file());
        assert!(!places.archived().exists());

        let Answered::Carried(answer) = approving.approve(turning, grants, noon()) else {
            panic!("an approved change did not happen");
        };
        assert_eq!(answer.now_at(), Some(places.archived().as_path()));
    });

    assert!(places.archived().is_file());
    assert!(!places.march.exists());
    assert_eq!(
        fs::read_to_string(places.archived()).unwrap(),
        "March, 4180.00",
        "the invoice that arrived is not the invoice that left"
    );
}

/// **Refused after its proposal has expired**, at both moments it can be met.
///
/// A change proposed this morning describes a machine that has moved on, so the
/// question stops standing — and after that it is neither put in front of
/// anybody nor carried out for anybody who still has it on a screen. The refusal
/// is `alo-capability`'s own, quoting the change so a person can ask for it
/// again, and the file has not moved.
#[test]
fn a_proposal_that_expired_is_refused_rather_than_carried_out() {
    let places = Places::made("expired");
    on_this_machine(&places, |turning, grants, strings| {
        let id = archiving(turning, grants, &places);
        let mut screen = Screen::default();
        let mut approving = Approving::nothing_to_answer();
        assert!(matches!(
            approving.ask(Some(&mut screen), turning, id, noon()),
            Asks::Asked(_)
        ));

        // An hour later, somebody who was still looking at it answers.
        let late = noon() + hour();
        let Answered::Refused(why) = approving.approve(turning, grants, late) else {
            panic!("a question that had stood too long was carried out");
        };
        assert!(matches!(
            why,
            NotAnswered::Turn(NotDone::NotAnswered(AnswerError::Lapsed { .. }))
        ));
        let said = why.said(strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("ask again"), "{said}");
        assert!(said.text().contains(&the_sentence(&places)), "{said}");

        // And a change proposed again is not put up once it has lapsed either,
        // so nobody is ever offered an answer the machine will refuse.
        let again = archiving(turning, grants, &places);
        let Asks::Refused(not_asked) = approving.ask(Some(&mut screen), turning, again, late)
        else {
            panic!("a question that had stood too long was put to somebody");
        };
        assert!(matches!(
            not_asked,
            NotAsked::NotWaiting(AnswerError::Lapsed { .. })
        ));
        assert!(not_asked.said(strings).text().contains("ask again"));
        assert_eq!(screen.asked.len(), 1, "a lapsed question reached a screen");
        assert!(!approving.is_asking());
    });

    // Nothing moved.
    assert!(places.march.is_file());
    assert!(!places.archived().exists());
}

/// **The record carries what was approved and by whom.**
///
/// Read back off the disk by the crate that reads records, not from the memory
/// of the session that wrote it. What it holds is the sentence the person was
/// shown — the same value, not a second rendering of it — the agent whose
/// authority it ran under, and the number of the approval that let it happen, so
/// *one approval, one execution* is answerable from the evidence and not only
/// from the code.
///
/// The person is not named on the entry, and that is this machine's shape rather
/// than an omission: a personal machine has one signed-in account
/// (`alo-accounts`), and repeating it on every line would be a record that
/// tracked the person rather than the agent.
#[test]
fn the_record_says_what_was_approved_and_by_whom() {
    let places = Places::made("record");
    let mut approval = None;
    on_this_machine(&places, |turning, grants, _| {
        let id = archiving(turning, grants, &places);
        let mut screen = Screen::default();
        let mut approving = Approving::nothing_to_answer();
        assert!(matches!(
            approving.ask(Some(&mut screen), turning, id, noon()),
            Asks::Asked(_)
        ));
        assert!(matches!(
            approving.approve(turning, grants, noon()),
            Answered::Carried(_)
        ));
        approval = Some(id.as_u64());
    });

    let read = Reading::at(&places.kept_at).unwrap();
    assert!(read.damage().nothing_wrong(), "{:?}", read.damage());
    let record = read.record();

    let executions = Asking::anything().only(Only::Executions);
    let ran: Vec<_> = record.answering(&executions).collect();
    assert_eq!(ran.len(), 1, "one approval, one thing that happened");
    let entry = ran.first().unwrap();

    // What was approved: the verb, and the sentence somebody read.
    assert!(entry.what().is_some_and(|what| what.verb().is("move_file")));
    assert_eq!(
        entry.what().unwrap().sentence().as_str(),
        the_sentence(&places),
        "the record does not hold the sentence the person was shown"
    );

    // By whom: the agent it ran under, and the approval that let it.
    assert!(entry.agent().is_some_and(|agent| agent.is("@files")));
    assert_eq!(entry.happened().from_approval(), approval);
    assert!(
        !entry.happened().against().is_empty(),
        "nothing says which grant permitted it"
    );

    // And a change nobody approved leaves nothing behind that says it ran.
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Refusals))
            .count(),
        0
    );
}

/// **Saying no is a whole answer**, and the record keeps it.
///
/// Not one of the four criteria, and here because the refusal path of a surface
/// that asks for consent is the path that decides whether the consent means
/// anything. Nothing happens, nobody is asked why, the file has not moved, and
/// the evidence says a person declined it rather than saying nothing at all.
#[test]
fn saying_no_is_a_whole_answer_and_the_record_keeps_it() {
    let places = Places::made("declined");
    on_this_machine(&places, |turning, grants, _| {
        let id = archiving(turning, grants, &places);
        let mut screen = Screen::default();
        let mut approving = Approving::nothing_to_answer();
        assert!(matches!(
            approving.ask(Some(&mut screen), turning, id, noon()),
            Asks::Asked(_)
        ));
        assert_eq!(approving.decline(turning, noon()), Answered::Declined);
        assert!(!approving.is_asking());
        assert_eq!(
            approving.approve(turning, grants, noon()),
            Answered::Refused(NotAnswered::NothingToAnswer),
            "a change that was declined could still be approved"
        );
    });

    assert!(places.march.is_file());
    assert!(!places.archived().exists());

    let read = Reading::at(&places.kept_at).unwrap();
    let record = read.record();
    assert_eq!(record.len(), 1, "saying no left no evidence");
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Executions))
            .count(),
        0,
        "a change nobody approved is recorded as having run"
    );
}

/// **Nowhere to put the question is a sentence a person can read**, in the
/// vocabulary the machine really holds.
///
/// Not one of the four criteria either, and here for the same reason: a change
/// that could not be put to anybody, and that said nothing about it, is an agent
/// that appears to have ignored an instruction — and the person's next act is to
/// ask for the same thing again. Both ways of having nowhere are refused in
/// words, neither answers anything, and the change is still waiting when a
/// screen arrives.
#[test]
fn nowhere_to_put_the_question_is_refused_in_words_rather_than_in_silence() {
    let places = Places::made("nowhere");
    on_this_machine(&places, |turning, grants, strings| {
        let id = archiving(turning, grants, &places);
        let mut approving = Approving::nothing_to_answer();

        // Nothing is drawing a screen at all.
        assert_eq!(
            approving.ask(None, turning, id, noon()),
            Asks::Refused(NotAsked::NoCompositor)
        );
        let said = NotAsked::NoCompositor.said(strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("Sign in to the desktop"), "{said}");

        // Something is drawing, and there is no display to draw on.
        let mut nowhere = NoScreen::default();
        assert_eq!(
            approving.ask(Some(&mut nowhere), turning, id, noon()),
            Asks::Refused(NotAsked::Surface(SurfaceRefused::NothingToShowOn))
        );
        assert_eq!(nowhere.offered, 1);
        let said = NotAsked::Surface(SurfaceRefused::NothingToShowOn).said(strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("Connect a screen"), "{said}");

        // Neither answered anything: a screen arrives and the change is put up.
        assert!(!approving.is_asking());
        let mut screen = Screen::default();
        assert!(matches!(
            approving.ask(Some(&mut screen), turning, id, noon()),
            Asks::Asked(_)
        ));
        assert_eq!(approving.asking(), Some(id));
    });

    // And nothing ran while nobody could be asked.
    assert!(places.march.is_file());
    assert!(!places.archived().exists());
}
