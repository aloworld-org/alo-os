//! The whole turn the approval surface's tests are written against.
//!
//! What `alo-agentd` stands up, minus the socket: the machine's file verbs on
//! a real disk, its indicator, its record, grants over two real folders and a
//! turn begun from an invocation. A change approved through it really moves a
//! file, because the question this surface exists to answer — *did one answer
//! carry one change out, and did silence carry out none* — cannot be answered
//! against a machine that does nothing.
//!
//! The words are the machine's one vocabulary as `alo-saying` collects it, so a
//! sentence nobody collected fails here rather than on somebody's screen. The
//! shape is `alo-approving`'s own fixture, copied rather than re-decided.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_capability::{Given, Grant, Grants, ProposalId, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Reaching, Resolving};
use alo_keeping::{Keeping, NotKept};
use alo_record::{Entry, Record};
use alo_strings::Strings;
use alo_turn::{Doing, Done, Kept, Machine, NoBoundary, Shortened, Shortening, Turning};

/// Everything this machine can say, with nothing translated.
pub(crate) fn words() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// The moment every test here is written against.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a grant, a turn and a question stand.
pub(crate) fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The folders and files a turn in these tests really moves.
pub(crate) struct Places {
    /// Where both folders live, removed when the test is done.
    _held: tempfile::TempDir,
    /// The folder the invoices are in.
    pub(crate) invoices: PathBuf,
    /// The folder they are moved into.
    pub(crate) archive: PathBuf,
}

impl Places {
    /// Two folders of this test's own, with three invoices in one of them.
    fn made() -> Self {
        let held = tempfile::tempdir().unwrap();
        let resolved = Resolving::real(&OnThisMachine, held.path())
            .unwrap()
            .into_path_buf();
        let invoices = resolved.join("invoices");
        let archive = resolved.join("archive");
        fs::create_dir_all(&invoices).unwrap();
        fs::create_dir_all(&archive).unwrap();
        for (month, total) in [("march", "4180.00"), ("april", "2260.00"), ("may", "90.10")] {
            fs::write(invoices.join(format!("{month}.pdf")), total).unwrap();
        }
        Self {
            _held: held,
            invoices,
            archive,
        }
    }

    /// One month's invoice where it started.
    pub(crate) fn invoice(&self, month: &str) -> PathBuf {
        self.invoices.join(format!("{month}.pdf"))
    }

    /// The same invoice where an approved move puts it.
    pub(crate) fn archived(&self, month: &str) -> PathBuf {
        self.archive.join(format!("{month}.pdf"))
    }
}

/// Stand a whole machine up, run one turn on it, and hand back the record it
/// wrote with the places it wrote about.
pub(crate) fn on_a_machine(
    body: impl FnOnce(&mut Turning<'_, '_>, &Grants, &Places),
) -> (Record, Places) {
    let mut record = Record::default();
    let places = keeping_in(&mut record, body);
    (record, places)
}

/// The same machine, writing what happened into `kept` — which for one test is
/// a disk with no space left on it.
pub(crate) fn keeping_in(
    kept: &mut dyn Shortening,
    body: impl FnOnce(&mut Turning<'_, '_>, &Grants, &Places),
) -> Places {
    let places = Places::made();
    let strings = words();
    let mut grants = Grants::default();
    for folder in [&places.invoices, &places.archive] {
        grants.grant(
            Grant::checked("@files", Reach::Folder(folder.clone()), noon(), hour()).unwrap(),
        );
    }
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    {
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            kept,
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
        body(&mut turning, &grants, &places);
        let _ = turning.ending(&mut grants);
    }
    places
}

/// A record on a disk with no space left: every entry is refused, so the first
/// thing a turn tries to write down closes it.
pub(crate) struct ANoSpaceLeftDisk;

impl Kept for ANoSpaceLeftDisk {
    fn keep(&mut self, _entry: Entry) -> Result<(), NotKept> {
        Err(NotKept::NotAddedTo {
            path: "/var/lib/alo/record.jsonl".to_owned(),
            why: "no space left on device".to_owned(),
        })
    }
}

impl Shortening for ANoSpaceLeftDisk {
    fn shorten(&mut self, _keeping: Keeping, _now: SystemTime) -> Result<Shortened, NotKept> {
        Ok(Shortened::NotOnADisk)
    }
}

/// One month's invoice, proposed for the archive and waiting for an answer.
pub(crate) fn archiving(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    places: &Places,
    month: &str,
) -> ProposalId {
    archiving_for(turning, grants, places, month, hour())
}

/// The same, as a question that stands for `standing` from noon.
pub(crate) fn archiving_for(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    places: &Places,
    month: &str,
    standing: Duration,
) -> ProposalId {
    turning
        .proposing(
            "move_file",
            &[
                ("file", as_given(&places.invoice(month))),
                ("into", as_given(&places.archive)),
            ],
            grants,
            standing,
            noon(),
        )
        .unwrap()
}

/// A path as a verb's argument arrives: text.
fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// A machine with nothing in front of a turn, which is not a machine alo OS
/// ships: `alo_turn::bounding` says why no library here offers one, and why a
/// test writes these few lines where its reader can see them.
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
