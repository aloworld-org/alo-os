//! What this crate's own tests are written against.
//!
//! Everything here is *this machine* rather than a description of one: a
//! directory on the disk the tests are running on, real sockets, real files,
//! and the two sides as this process could really have them.
//!
//! **The person is whoever is running the tests.** Everything here touches a
//! real filesystem and a real socket, so a fixture that named a person this
//! process is not would be a fixture whose every happy path is a refusal. The
//! agent is a login beside it, chosen so that it is neither root nor the person
//! whoever runs these tests happens to be.
//!
//! # The one thing one process cannot arrange, and what stands in for it
//!
//! Which door a connection is on is decided by **which user made it**, and a
//! test process is one user — so a test that connected to the real socket twice
//! would get the person's door twice, and everything [`crate::serving`] decides
//! could not be written down as a test at all. [`Pretending`] is what stands in
//! for that, and it is a real socket with real connections on it: the only
//! thing it does differently is that it is *told* which login each connection
//! would have come from. Nothing about the mapping is faked —
//! [`crate::Sides`] is still the only thing that decides a door on a running
//! machine, [`crate::Listening`] is still the only [`Knocking`] that ships, and
//! there is a test in [`crate::knocking`] that the real one answers the same
//! shape.
//!
//! # The vocabulary is the machine's, and it is no longer a list written here
//!
//! [`in_english`] used to name eight crates and leave three out, which was true
//! of the service and became a second answer the day there was a process: item
//! 21f loads `alo_saying::everything_this_machine_can_say` and declares this
//! crate's three on top, so the fixture is what a machine really has rather than
//! a shorter list that happens to be enough. A test asking whether a sentence is
//! translated is then asking about the vocabulary that ships.
//!
//! The three that were absent are `alo-models`, `alo-asking` and
//! `alo-answering`, and they are here now — which changes nothing about what
//! this service does with a question put to a model. [`crate::doing`] still
//! refuses one before it reaches `alo_turn::Turning::asking`, because nothing
//! tells this service what a machine has been set to answer with; the words for
//! *when it does* are loaded and unused, which is the honest way round.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::os::fd::{AsFd as _, BorrowedFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Grant, Grants, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Resolving as _};
use alo_record::Record;
use alo_strings::{Strings, Vocabulary};
use alo_turn::{Machine, Turning};

use crate::caller::{Caller, Uid};
use crate::knocking::Knocking;
use crate::refusing::NotACaller;
use crate::side::{Side, Sides};
use crate::unix::{our_group, us};

/// The login the fixture gives the agent when the person is not it.
const AN_AGENT: u32 = 989;

/// The one it gives the agent when the person happens to be [`AN_AGENT`].
const ANOTHER_AGENT: u32 = 990;

/// This machine, with whoever is running the tests as the person.
pub(crate) fn ourselves() -> Sides {
    let person = us().unwrap();
    let agent = if person.raw() == AN_AGENT {
        ANOTHER_AGENT
    } else {
        AN_AGENT
    };
    Sides::of(person, Uid::of(agent).unwrap(), our_group().unwrap()).unwrap()
}

/// A caller running as the user this process is, which is the person.
pub(crate) fn calling_as_the_person() -> Caller {
    Caller::known(
        i32::try_from(std::process::id()).unwrap(),
        us().unwrap(),
        our_group().unwrap(),
    )
}

/// A folder of this test's own, on the disk the tests are running on.
///
/// `alo-keeping` and `alo-files` have the same fixture and for the same reason:
/// a test about a real socket has to be about a real directory, and two of them
/// must not meet.
pub(crate) fn a_directory_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-agentd-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// A fixed moment, for the tests that do not run a service.
///
/// [`crate::serving`] cannot use it: a service reads a real clock once a round,
/// so a grant made at a moment in 2025 would have expired before the first
/// message arrived. Those tests use [`granting`] with the moment the service
/// itself will see.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a turn, a grant and a question stand in these tests.
pub(crate) fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// One request, in the envelope it really arrives in.
///
/// Written out rather than composed with `alo_protocol`'s own writer, because
/// what these tests are about is a line somebody else wrote arriving on a
/// socket — and a fixture that produced the line with the same code that reads
/// it would be a test of a round trip rather than of a door.
pub(crate) fn a_message(asks: &str) -> String {
    format!(r#"{{"format":1,"asks":{asks}}}"#)
}

/// A folder with one file in it, resolved, and the file's path.
///
/// Resolved because a grant is over a place and containment is decided
/// lexically: a grant made over the typed spelling of a path the machine
/// resolves differently would match nothing. `alo-turn`'s fixture does the same
/// and for the same reason.
pub(crate) fn a_folder_with_an_invoice(what: &str) -> (PathBuf, PathBuf) {
    let folder = OnThisMachine
        .real(&a_directory_of_our_own(what))
        .unwrap()
        .into_path_buf();
    let invoice = folder.join("march.pdf");
    fs::write(&invoice, "March, 4180.00").unwrap();
    (folder, invoice)
}

/// A grant to `@files` over this folder, made at this moment and lasting an
/// hour.
///
/// The moment is an argument because a service reads a real clock: a fixture
/// that granted at a fixed noon would hand a running service a grant that ran
/// out decades ago, and every test of it would be a test of expiry.
pub(crate) fn granting(folder: &Path, at: SystemTime) -> Grants {
    let mut grants = Grants::default();
    grants
        .grant(Grant::checked("@files", Reach::Folder(folder.to_path_buf()), at, hour()).unwrap());
    grants
}

/// The words this machine reads, with nothing translated.
///
/// The machine's own vocabulary, assembled the way the process assembles it —
/// `alo-saying`'s fourteen lists and this crate's three on top. See this file's
/// header for why it is no longer a list written out here.
pub(crate) fn in_english() -> Strings {
    Strings::of(everything_this_machine_says())
}

/// Every word a machine running this service has loaded.
fn everything_this_machine_says() -> Vocabulary {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::words::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// The arrangement the message tests need: a machine that offers the six,
/// grants over one folder, and a turn under way for `@files`.
///
/// Written as a closure taking the turn rather than a function returning one,
/// because a `Turning` borrows the `Machine`, the machine borrows the record
/// and the strings, and all of them have to live in one frame. `alo-turn`'s
/// fixture has the same shape for the same reason.
///
/// **Nothing is offered at the invocation**, which is what a real turn on this
/// machine begins with: what is in front of the person is answered by Wayland
/// and AT-SPI, and there is no compositor here.
pub(crate) fn on_a_machine<T>(
    what: &str,
    doing: impl FnOnce(&mut Turning<'_, '_>, &Grants, &Strings, &Path, &Path) -> T,
) -> T {
    let strings = in_english();
    let (folder, invoice) = a_folder_with_an_invoice(what);
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let mut machine =
        Machine::carrying_out_file_verbs(&strings, &OnThisMachine, &mut indicator, &mut record)
            .unwrap();
    let mut grants = granting(&folder, noon());
    let mut turning = Turning::beginning(
        Context::at_invocation(noon()),
        "@files",
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    doing(&mut turning, &grants, &strings, &folder, &invoice)
}

/// A door that hands out real connections and is told which side each is on.
///
/// A real `UnixListener`, so the connections a test makes are the connections a
/// service really reads: the same accepting, the same reading, the same
/// closing. What it is told is only the thing one process cannot arrange, which
/// is that a connection came from a second login.
///
/// **Past the end of the list is a stranger.** A fixture that was not told
/// about a connection has nothing true to say about which door it is on, and
/// the honest answer to *who is this* is the same one the socket gives.
#[derive(Debug)]
pub(crate) struct Pretending {
    /// Where the socket is.
    at: PathBuf,
    /// The socket itself.
    listener: UnixListener,
    /// Which side each connection is on, in the order they arrive.
    sides: Vec<Option<Side>>,
    /// How many have arrived.
    so_far: AtomicUsize,
}

impl Pretending {
    /// A socket of this test's own, handing out these sides in this order.
    pub(crate) fn handing_out(what: &str, sides: &[Option<Side>]) -> Self {
        let at = a_directory_of_our_own(what).join("pretending.sock");
        let listener = UnixListener::bind(&at).unwrap();
        Self {
            at,
            listener,
            sides: sides.to_vec(),
            so_far: AtomicUsize::new(0),
        }
    }

    /// Where to connect.
    pub(crate) fn at(&self) -> PathBuf {
        self.at.clone()
    }
}

impl Knocking for Pretending {
    fn waiting_on(&self) -> BorrowedFd<'_> {
        self.listener.as_fd()
    }

    fn next(&self) -> Result<(Side, UnixStream), NotACaller> {
        let (connection, _) = self
            .listener
            .accept()
            .map_err(|why| NotACaller::NotAccepted { why })?;
        let which = self.so_far.fetch_add(1, Ordering::Relaxed);
        match self.sides.get(which) {
            Some(Some(side)) => Ok((*side, connection)),
            // Closed by being dropped, with nothing written on it: the same
            // answer the real socket gives a stranger, for the same reason.
            Some(None) | None => Err(NotACaller::Stranger { uid: 65534 }),
        }
    }
}
