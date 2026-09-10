//! The changes, the grants, the strings and the whole turn this crate's own
//! tests are written against.
//!
//! Every file here that shows something has the same two questions to answer —
//! *what does this say on a machine with no translations* and *what does it say
//! when somebody has translated it* — and answering them from one fixture is
//! what stops the files inventing vocabularies that resemble the real one. The
//! shape is `alo-indicator`'s `testing.rs`, copied rather than re-decided.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! # The questions are real questions
//!
//! A change below is made the way a turn makes one: through
//! `alo_capability::Verbs::call` against the closed list `alo-files` declares,
//! and then through `alo_capability::Proposal::checked` against grants that
//! really permit it. There is deliberately no fixture for *a question that was
//! never a proposal*, because there is no way to write one — which is the
//! guarantee [`crate::Asked`] is shaped around.
//!
//! # It is not only this crate's words
//!
//! The sentence on this surface is worded by whoever declared the verb, so a
//! fixture holding only this crate's list would answer every question with its
//! own key in guillemets and the tests about sentences would still pass — a
//! fixture proving the tests rather than the code. So the vocabulary here is
//! this crate's, `alo-files`', `alo-capability`'s and `alo-turn`'s, which is
//! what a shell showing one of these really holds.
//!
//! # And the turn is a real turn, on a real disk
//!
//! [`on_a_machine`] stands up what `alo-agentd` stands up: the machine's verbs,
//! its indicator, its record, a grant over two folders that exist, and a turn
//! begun from an invocation. A change approved through it really moves a file,
//! because the question this crate exists to answer — *did one approval carry
//! one change out* — cannot be answered against a machine that does nothing.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Approvals, Call, Given, Grant, Grantee, Grants, Proposal, ProposalId, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Reaching};
use alo_record::Record;
use alo_strings::{Language, Strings, Translation, Vocabulary};
use alo_turn::{Doing, Done, Machine, NoBoundary, Turning};

use crate::asked::Asked;
use crate::surface::{Compositor, SurfaceRefused};
use crate::words::declare_into;

/// The moment every test here is written against.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a grant, a turn and a question stand.
pub(crate) fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent every question here belongs to.
pub(crate) fn files() -> Grantee {
    Grantee::named("@files")
}

/// Everything this crate says, and everything it puts a sentence beside.
fn its_own_and_what_it_quotes() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).unwrap();
    alo_files::words::declare_into(&mut vocabulary).unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// This crate's own words with nothing translated: what a machine that has no
/// translations shows, which is what most of these tests are about.
pub(crate) fn in_english() -> Strings {
    Strings::of(its_own_and_what_it_quotes())
}

/// The same, with the sentence and the two answers translated into German and
/// German preferred — German because it is the language the rest of this
/// repository tests translation with.
pub(crate) fn translated() -> Strings {
    let vocabulary = its_own_and_what_it_quotes();
    let translation = Translation::into_language(german())
        .says(
            alo_files::words::MOVE_FILE_SENTENCE.key(),
            "{file} nach {into} verschieben",
        )
        .says(crate::words::APPROVE.key(), "Genehmigen")
        .says(crate::words::NO.key(), "Nein");
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german()]);
    strings
}

/// German, as `alo-strings` names a language.
fn german() -> Language {
    Language::written("de").unwrap()
}

/// A path as a verb's argument arrives: text.
pub(crate) fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

// ---------------------------------------------------------------------------
// Changes that need no disk, for the files that only word one.
// ---------------------------------------------------------------------------

/// Moving March's invoice into the archive, validated against the closed list
/// of verbs this machine offers.
pub(crate) fn archiving_march() -> Call {
    moving(Path::new("/home/anna/Invoices/march.pdf"))
}

/// The same for April, so a test about one question can show it is a different
/// value from another.
pub(crate) fn archiving_april() -> Call {
    moving(Path::new("/home/anna/Invoices/april.pdf"))
}

/// One file, into the archive.
fn moving(file: &Path) -> Call {
    alo_files::file_verbs()
        .unwrap()
        .call(
            "move_file",
            &[
                ("file", as_given(file)),
                ("into", as_given(Path::new("/home/anna/Archive"))),
            ],
        )
        .unwrap()
}

/// Grants to `@files` over the two folders those changes reach.
fn granting_both() -> Grants {
    granting(&[
        Path::new("/home/anna/Invoices"),
        Path::new("/home/anna/Archive"),
    ])
}

/// Grants to `@files` over these folders, made at noon and lasting an hour.
fn granting(folders: &[&Path]) -> Grants {
    let mut grants = Grants::default();
    for folder in folders {
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder((*folder).to_path_buf()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
    }
    grants
}

/// One change, really proposed, waiting for an answer.
pub(crate) fn one_waiting(call: Call) -> (Approvals, ProposalId) {
    let grants = granting_both();
    let mut approvals = Approvals::default();
    let id =
        approvals.propose(Proposal::checked(&call, &files(), &grants, noon(), hour()).unwrap());
    (approvals, id)
}

// ---------------------------------------------------------------------------
// A whole turn, on a real disk.
// ---------------------------------------------------------------------------

/// The folders and files a turn in this crate's tests really moves.
pub(crate) struct Places {
    /// The folder the invoices are in.
    pub(crate) invoices: PathBuf,
    /// The folder they are moved into.
    pub(crate) archive: PathBuf,
    /// March's invoice.
    pub(crate) march: PathBuf,
    /// April's, for the test that needs two questions at once.
    pub(crate) april: PathBuf,
}

impl Places {
    /// Two folders of this test's own, with two invoices in one of them.
    fn made() -> Self {
        let invoices = a_folder_of_our_own("invoices");
        let archive = a_folder_of_our_own("archive");
        let march = invoices.join("march.pdf");
        let april = invoices.join("april.pdf");
        fs::write(&march, "March, 4180.00").unwrap();
        fs::write(&april, "April, 2260.00").unwrap();
        Self {
            invoices,
            archive,
            march,
            april,
        }
    }
}

/// A folder of this test's own, resolved — a grant is over a place, and on
/// Windows the resolved spelling is the one a grant has to be made with.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-approving-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    alo_files::Resolving::real(&OnThisMachine, &folder)
        .unwrap()
        .into_path_buf()
}

/// Stand a whole machine up, run one turn on it, and hand back the record it
/// wrote.
///
/// What `alo-agentd` stands up, minus the socket: the machine's verbs, its
/// indicator, somewhere to keep the record, grants over two real folders, and a
/// turn begun from an invocation. The record comes back because *what was
/// approved and by whom* is a question about evidence, and evidence is the one
/// thing a test must read rather than assume.
pub(crate) fn on_a_machine(
    body: impl FnOnce(&mut Turning<'_, '_>, &Grants, &Strings, &Places),
) -> (Record, Places) {
    let places = Places::made();
    let strings = in_english();
    let mut grants = granting(&[&places.invoices, &places.archive]);
    let mut record = Record::default();
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    {
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
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
        body(&mut turning, &grants, &strings, &places);
        let _ = turning.ending(&mut grants);
    }
    (record, places)
}

/// March's invoice, proposed for the archive and waiting for an answer.
pub(crate) fn an_invoice(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    places: &Places,
) -> ProposalId {
    proposing(turning, grants, &places.march, &places.archive)
}

/// April's, for the test that needs two questions waiting at once.
pub(crate) fn another_invoice(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    places: &Places,
) -> ProposalId {
    proposing(turning, grants, &places.april, &places.archive)
}

/// One file, proposed for one folder.
fn proposing(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    file: &Path,
    into: &Path,
) -> ProposalId {
    turning
        .proposing(
            "move_file",
            &[("file", as_given(file)), ("into", as_given(into))],
            grants,
            hour(),
            noon(),
        )
        .unwrap()
}

/// A machine with nothing in front of a turn, which is not a machine alo OS
/// ships.
///
/// `alo_turn::bounding` says why there is no such implementation in any library
/// here — it would be ADR 0015's guarantee turned off by default on every host
/// — and why what a test needs is these few lines, written where whoever reads
/// the test can see them.
struct NothingIsBounded;

impl alo_turn::Bounding for NothingIsBounded {
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

/// A screen, as this crate sees one: it keeps whatever it was last given and
/// counts how often it was asked, so *one question, one surface* is a number
/// rather than an impression.
pub(crate) struct Screen {
    /// How many questions arrived.
    pub(crate) asked: usize,
    /// The last one, when it accepted any.
    pub(crate) last: Option<Asked>,
    /// What every request is answered with.
    answer: Result<(), SurfaceRefused>,
}

impl Screen {
    /// A compositor with a screen.
    pub(crate) fn with_a_screen() -> Self {
        Self {
            asked: 0,
            last: None,
            answer: Ok(()),
        }
    }

    /// A compositor with nothing to put a question on.
    pub(crate) fn with_no_screen() -> Self {
        Self {
            asked: 0,
            last: None,
            answer: Err(SurfaceRefused::NothingToShowOn),
        }
    }
}

impl Compositor for Screen {
    fn ask(&mut self, asked: Asked) -> Result<(), SurfaceRefused> {
        self.asked = self.asked.saturating_add(1);
        if self.answer.is_ok() {
            self.last = Some(asked);
        }
        self.answer
    }
}
