//! The people, the machine, the strings and the whole turn this crate's own
//! tests are written against.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! # The session is a real sign-in
//!
//! [`anna`] is not a value assembled for a test: it is what `alo-accounts`
//! hands back when a password verifies and the machine description agrees
//! about the number, which is the only way an `alo_accounts::Session` exists.
//! A seat locked over anything else would be a lock this crate could not
//! actually be given.
//!
//! # And the turn is a real turn, on a real disk
//!
//! [`on_a_machine`] is `alo-approving`'s fixture, copied rather than shared:
//! the machine's verbs, its indicator, its record, a grant over two folders
//! that exist, and a turn begun from an invocation. *The change was not carried
//! out from the lock screen* is then a file that did not move.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_accounts::{Accounts, Session};
use alo_appearance::DisplayId;
use alo_approving::{Asked, Compositor, SurfaceRefused};
use alo_capability::{Given, Grant, Grants, ProposalId, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Reaching};
use alo_record::Record;
use alo_strings::{Strings, Vocabulary};
use alo_turn::{Doing, Done, Machine, NoBoundary, Turning};

/// The number this machine's person runs as.
const PERSON: u32 = 1000;

/// Anna's password, and it is right.
pub(crate) const ANNAS: &str = "correct horse battery staple";

/// Ben's password, on the same machine, and it is right for Ben.
pub(crate) const BENS: &str = "a different correct password";

/// The moment every test here is written against.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a grant, a turn and a question stand.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// This crate's words, and the words of the two crates whose refusals an
/// unlock carries.
pub(crate) fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    crate::declare_into(&mut vocabulary).unwrap();
    alo_accounts::declare_into(&mut vocabulary).unwrap();
    alo_sessiond::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The machine's accounts: Anna, at this machine's number.
pub(crate) fn the_machine() -> Accounts {
    let mut accounts = Accounts::none().unwrap();
    accounts.created("anna", PERSON, ANNAS).unwrap();
    accounts
}

/// The same machine with a second account on it, at another number.
pub(crate) fn the_machine_with_ben() -> Accounts {
    let mut accounts = the_machine();
    accounts.created("ben", PERSON + 1, BENS).unwrap();
    accounts
}

/// Anna's session, as a sign-in really opens one. Made once: hashing a
/// password is deliberately slow.
pub(crate) fn anna() -> Session {
    static ANNA: OnceLock<Session> = OnceLock::new();
    ANNA.get_or_init(|| {
        let signed_in = the_machine().signs_in("anna", ANNAS).unwrap();
        Session::opened(signed_in, PERSON).unwrap()
    })
    .clone()
}

/// A notification, as whatever delivers them might carry one. The text is
/// the kind a lock screen must never show.
pub(crate) fn a_notification(text: &str) -> String {
    text.to_owned()
}

/// The laptop's own display.
pub(crate) fn the_laptop() -> DisplayId {
    DisplayId::named("eDP-1 Built-in display").unwrap()
}

/// A screen a change can be put on, counting how often it was.
#[derive(Default)]
pub(crate) struct Screen {
    /// How many questions arrived.
    pub(crate) asked: usize,
}

impl Compositor for Screen {
    fn ask(&mut self, _asked: Asked) -> Result<(), SurfaceRefused> {
        self.asked = self.asked.saturating_add(1);
        Ok(())
    }
}

/// The folders and the file a turn in these tests really moves.
pub(crate) struct Places {
    /// The folder the invoice is in.
    invoices: PathBuf,
    /// The folder it is moved into.
    pub(crate) archive: PathBuf,
    /// March's invoice.
    pub(crate) march: PathBuf,
}

/// A folder of this test's own, resolved the way a grant has to be made.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-locking-{}-{what}-{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    alo_files::Resolving::real(&OnThisMachine, &folder)
        .unwrap()
        .into_path_buf()
}

/// Stand a whole machine up, run one turn on it, and hand back the record it
/// wrote and the places it worked in.
pub(crate) fn on_a_machine(
    body: impl FnOnce(&mut Turning<'_, '_>, &Grants, &Places),
) -> (Record, Places) {
    let invoices = a_folder_of_our_own("invoices");
    let archive = a_folder_of_our_own("archive");
    let march = invoices.join("march.pdf");
    fs::write(&march, "March, 4180.00").unwrap();
    let places = Places {
        invoices,
        archive,
        march,
    };

    let mut vocabulary = Vocabulary::empty();
    alo_files::words::declare_into(&mut vocabulary).unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    alo_approving::declare_into(&mut vocabulary).unwrap();
    let strings = Strings::of(vocabulary);

    let mut grants = Grants::default();
    for folder in [&places.invoices, &places.archive] {
        grants.grant(
            Grant::checked("@files", Reach::Folder(folder.clone()), noon(), hour()).unwrap(),
        );
    }
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
        body(&mut turning, &grants, &places);
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

/// A path as a verb's argument arrives: text.
fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// A machine with nothing in front of a turn, which is not a machine alo OS
/// ships — `alo_turn::bounding` says why no library here has one, and why a
/// test needs these few lines where whoever reads it can see them.
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
