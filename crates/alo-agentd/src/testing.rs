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
use alo_models::{Catalogue, Installed, Loaded, ModelRuntime, ProgressSink, RuntimeError};
use alo_record::Record;
use alo_strings::{Strings, Vocabulary};
use alo_turn::{Machine, Turning};

use crate::caller::{Caller, Uid};
use crate::knocking::Knocking;
use crate::place::Place;
use crate::questions::{Questions, TheBound, WhoseKeyring};
use crate::refusing::NotACaller;
use crate::rereading::Remembering;
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

/// A door of this test's own, in a root of this test's own.
///
/// `/run/alo` is the image's and a test may not write in it (ADR 0017), so the
/// root is a directory on the disk the tests are running on and the person is
/// whoever is running them. What is being tested is every rule beneath the
/// root — which is where all of them are.
pub(crate) fn a_place_of_our_own(what: &str) -> Place {
    Place::beneath(&a_directory_of_our_own(what), ourselves().person())
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
/// `alo-saying`'s fifteen lists and this crate's three on top. See this file's
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
    doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants, &Strings, &Path, &Path) -> T,
) -> T {
    let strings = in_english();
    let (folder, invoice) = a_folder_with_an_invoice(what);
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let mut record = Record::default();
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        &mut record,
    )
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
    doing(&mut turning, &mut grants, &strings, &folder, &invoice)
}

/// The same machine with **no turn under way**, which is what the person's
/// shell talks to for all but a few seconds of a working day.
///
/// A turn is an agent's connection, and a person is signed in for as long as
/// they are signed in — so every question on the person's door has to have an
/// answer here as well, and the record has to be reachable without one.
pub(crate) fn on_a_machine_with_no_turn<T>(
    what: &str,
    record: &mut dyn alo_turn::Shortening,
    doing: impl FnOnce(&mut Machine<'_>, &mut Grants, &Strings, &Path, &Path) -> T,
) -> T {
    let strings = in_english();
    let (folder, invoice) = a_folder_with_an_invoice(what);
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        record,
    )
    .unwrap();
    let mut grants = granting(&folder, noon());
    doing(&mut machine, &mut grants, &strings, &folder, &invoice)
}

/// A machine on which nobody has ever granted anything, and there is no file.
///
/// What the tests that are not about the grants file are handed: reading it
/// again answers *nothing has been granted here*, which is what a machine on
/// its first morning really says.
#[derive(Debug)]
pub(crate) struct NothingIsRemembered;

impl Remembering for NothingIsRemembered {
    fn read_again(&self, _now: SystemTime) -> Result<Grants, alo_remembering::NotRemembered> {
        Err(alo_remembering::NotRemembered::NotThere {
            at: PathBuf::from(alo_remembering::THE_GRANTS),
        })
    }
}

/// A grants file of a test's own, holding this list.
pub(crate) fn a_file_holding(what: &str, grants: &Grants, at: SystemTime) -> crate::ThePersonsFile {
    let path = a_directory_of_our_own(what).join("grants.toml");
    alo_remembering::kept(&path, grants, at).unwrap();
    crate::ThePersonsFile::at(&path)
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

/// A machine with nothing in front of a turn, which is not a machine this
/// service ever really is.
///
/// `crate::bounding` is the real one and it needs a kernel that started the BPF
/// security module, root, and a control group subtree — and making one moves
/// **this whole process** into a control group of its own, which two tests
/// running at once would fight over. So the tests in this crate, which are about
/// sockets, doors, messages and turns, are run with this; what the real one does
/// is asserted in `alo-bounding`'s own tests against a real kernel, and through
/// this service in `tests/a_turn_is_bounded_by_the_kernel.rs`, which is one test
/// in a binary of its own for exactly that reason.
/// A turn on a machine that has granted nothing, writing into this record.
///
/// [`on_a_machine`] is the same thing with a folder and a grant in it, and it
/// keeps the record to itself. This one lends it out, because what a question
/// to a model leaves behind is the thing being tested — and a machine with no
/// grant is the honest setting for it: asking a model is not a verb and reaches
/// no folder.
///
/// The record is whatever a test hands in, rather than an in-memory one: a
/// question refused by a rule is now written down, and *what reached the disk*
/// is a different question from *what this process remembers*. A file-backed
/// record and one that refuses every write are both `Shortening`, and both are
/// used.
pub(crate) fn on_a_machine_that_answers<T>(
    record: &mut dyn alo_turn::Shortening,
    doing: impl FnOnce(&mut Turning<'_, '_>, &Grants, &Strings) -> T,
) -> T {
    let strings = in_english();
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        record,
    )
    .unwrap();
    let mut grants = Grants::default();
    let mut turning = Turning::beginning(
        Context::at_invocation(noon()),
        "@files",
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    doing(&mut turning, &grants, &strings)
}

/// A model runtime that says one thing, without one being installed anywhere.
///
/// A test that wants what the pinned runtime was *asked* serves one with
/// [`a_runtime_served`]; most do not, and want only a
/// question arriving and an answer coming back, which is the one method below
/// that does anything.
#[derive(Debug)]
pub(crate) struct Saying(Result<String, RuntimeError>);

/// A runtime that answers every question with this.
pub(crate) fn a_runtime_saying(said: Result<String, RuntimeError>) -> Box<dyn ModelRuntime> {
    Box::new(Saying(said))
}

impl ModelRuntime for Saying {
    fn installed(&self) -> Result<Vec<Installed>, RuntimeError> {
        Ok(Vec::new())
    }

    fn loaded(&self) -> Result<Vec<Loaded>, RuntimeError> {
        Ok(Vec::new())
    }

    fn fetch(&self, id: &str, _progress: &mut dyn ProgressSink) -> Result<(), RuntimeError> {
        Err(RuntimeError::NotOffered(id.to_owned()))
    }

    fn remove(&self, id: &str) -> Result<(), RuntimeError> {
        Err(RuntimeError::NotInstalled(id.to_owned()))
    }

    fn load(&self, id: &str) -> Result<(), RuntimeError> {
        Err(RuntimeError::NotInstalled(id.to_owned()))
    }

    fn unload(&self, id: &str) -> Result<(), RuntimeError> {
        Err(RuntimeError::NotInstalled(id.to_owned()))
    }

    fn answers(&self, _question: &str, _of_model: &str) -> Result<String, RuntimeError> {
        self.0.clone()
    }

    /// Nothing is brought: this fixture serves questions rather than weights,
    /// and a test that needs the door refused should use one that refuses.
    fn bring(&self, _weights: &alo_models::Weights) -> Result<(), RuntimeError> {
        Ok(())
    }
}

/// The machine at reception, whose agent asks.
pub(crate) fn reception() -> alo_nearby::MachineId {
    alo_nearby::MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// The studio machine, which is asked — the one this service runs as in
/// these tests.
pub(crate) fn the_studio() -> alo_nearby::MachineId {
    alo_nearby::MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
}

/// Two machines paired the way two machines are, permitting `may`, for a
/// day from `at`: the asking one's row first, the asked one's second.
///
/// The moment is an argument for [`granting`]'s reason: a running service
/// reads a real clock, and a pairing made at a fixed noon would have run
/// out before the first message arrived. `alo-corridor`'s fixture, copied
/// rather than shared, because a fixture is not a public surface.
pub(crate) fn paired_between(
    asking: alo_nearby::MachineId,
    asked: alo_nearby::MachineId,
    may: &[alo_nearby::MayAskIts],
    at: SystemTime,
) -> (alo_nearby::Pairing, alo_nearby::Pairing) {
    use alo_nearby::{Deliberating, Keying, Proposal, Side};
    let at_asking = Keying::fresh().unwrap();
    let at_asked = Keying::fresh().unwrap();
    let proposal = Proposal::checked(
        asking,
        asked,
        may,
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
            .agreed(at)
            .unwrap(),
        asked_side
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(at)
            .unwrap(),
    )
}

/// A network with nobody on it, for the tests on the person's door that are
/// not about proposing to anybody.
#[derive(Debug)]
pub(crate) struct NobodyIsNearby;

impl crate::looking::LookingFor for NobodyIsNearby {
    fn look_for(&self, _machine: &alo_nearby::MachineId) -> Option<alo_nearby::Found> {
        None
    }
}

/// A machine where nobody has chosen anything to answer questions.
pub(crate) fn nothing_has_been_chosen() -> Questions {
    Questions::of_a_session(
        None,
        None,
        Catalogue::built_in().unwrap(),
        TheBound::Nobodys,
        WhoseKeyring::Nobodys,
    )
}

#[derive(Debug)]
pub(crate) struct NothingIsBounded;

impl alo_turn::Bounding for NothingIsBounded {
    fn carrying_out(
        &mut self,
        _reaching: &alo_files::Reaching,
        doing: alo_turn::Doing<'_>,
    ) -> Result<alo_turn::Done, alo_turn::NoBoundary> {
        Ok(doing.done())
    }

    /// And a network request, carried out where it stands.
    ///
    /// The same four lines as above and the same argument: what a test needs is
    /// a boundary that is not one, written where whoever reads the test can see
    /// that is what it is.
    fn carrying_out_a_departure(
        &mut self,
        _to: &[std::net::SocketAddr],
        doing: &mut dyn FnMut(),
    ) -> Result<(), alo_turn::NoBoundary> {
        doing();
        Ok(())
    }
}

/// The pinned runtime's side of one question, served on a socket of this
/// test's own: it answers once with `reply` and hands back everything it was
/// sent, head and body, so a test reads what a question was asked with rather
/// than the code that built it.
pub(crate) fn a_runtime_served(
    reply: &'static str,
) -> (alo_models::Ollama, std::thread::JoinHandle<String>) {
    use std::io::{BufRead as _, Read as _, Write as _};

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
        let mut head = String::new();
        let mut length = 0usize;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap() == 0 {
                break;
            }
            if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                length = value.trim().parse().unwrap_or(0);
            }
            let done = line == "\r\n" || line == "\n";
            head.push_str(&line);
            if done {
                break;
            }
        }
        let mut body = vec![0u8; length];
        reader.read_exact(&mut body).unwrap();
        let written = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{reply}",
            reply.len()
        );
        stream.write_all(written.as_bytes()).unwrap();
        stream.flush().unwrap();
        head + &String::from_utf8_lossy(&body)
    });
    let runtime = alo_models::Ollama::at(
        &format!("http://127.0.0.1:{port}"),
        Catalogue::built_in().unwrap(),
    );
    (runtime, handle)
}

/// What a request a test served was sent as its body, read as JSON.
pub(crate) fn the_body_of(request: &str) -> serde_json::Value {
    serde_json::from_str(request.split_once("\r\n\r\n").map_or("", |(_, body)| body)).unwrap()
}

/// What the studio answers a question with, in the shape the corridor reads.
pub(crate) const THE_STUDIOS_ANSWER: &str = r#"{"object":"chat.completion","model":"the-studios-model","choices":[{"index":0,"message":{"role":"assistant","content":"Three are unpaid."},"finish_reason":"stop"}]}"#;

/// The studio machine answering on a socket of this test's own, for a few
/// seconds, counting every question that reached it.
///
/// Shared by `crate::corridor` and `crate::choosing_to_answer`: a question down
/// the corridor, and a question to the machine the person's door just chose.
pub(crate) fn the_studio_answering() -> (std::net::SocketAddr, std::sync::Arc<AtomicUsize>) {
    let (at, heard, _) = the_studio_answering_and_keeping();
    (at, heard)
}

/// The same studio, keeping every request that reached it — head and body, as
/// the bytes arrived — so a test reads what crossed to the other machine rather
/// than the code that sent it.
pub(crate) fn the_studio_answering_and_keeping() -> (
    std::net::SocketAddr,
    std::sync::Arc<AtomicUsize>,
    std::sync::Arc<std::sync::Mutex<Vec<String>>>,
) {
    use std::io::{BufRead as _, Read as _, Write as _};

    let kept = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let keeping = std::sync::Arc::clone(&kept);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let at = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let heard = std::sync::Arc::new(AtomicUsize::new(0));
    let counting = std::sync::Arc::clone(&heard);
    std::thread::spawn(move || {
        let until = std::time::Instant::now() + Duration::from_secs(5);
        while std::time::Instant::now() < until {
            let Ok((mut stream, _)) = listener.accept() else {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            };
            counting.fetch_add(1, Ordering::SeqCst);
            stream.set_nonblocking(false).unwrap();
            let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
            let mut length = 0usize;
            let mut request = String::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse().unwrap_or(0);
                }
                request.push_str(&line);
                if line == "\r\n" || line == "\n" {
                    break;
                }
            }
            let mut body = vec![0u8; length];
            drop(reader.read_exact(&mut body));
            request.push_str(&String::from_utf8_lossy(&body));
            keeping.lock().unwrap().push(request);
            let written = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{THE_STUDIOS_ANSWER}",
                THE_STUDIOS_ANSWER.len()
            );
            drop(stream.write_all(written.as_bytes()));
        }
    });
    (at, heard, kept)
}

/// The network, where the studio answers discovery at `at` — and how many
/// times anybody looked.
#[derive(Debug)]
pub(crate) struct TheStudioIsAt {
    /// Where it answers.
    pub(crate) at: std::net::SocketAddr,
    /// How many times it was looked for.
    pub(crate) looked: std::cell::Cell<usize>,
}

impl crate::looking::LookingFor for TheStudioIsAt {
    fn look_for(&self, machine: &alo_nearby::MachineId) -> Option<alo_nearby::Found> {
        self.looked.set(self.looked.get() + 1);
        (*machine == the_studio())
            .then(|| alo_nearby::Found::seen(the_studio(), self.at.port(), self.at.ip()))
    }
}
