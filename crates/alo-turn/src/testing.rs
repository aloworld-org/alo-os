//! The machine, the folders, the places and the strings this crate's tests are
//! written against.
//!
//! A turn reaches nine crates, so its fixture is the arrangement a real machine
//! is in rather than a smaller one: **every crate's words in one vocabulary**,
//! because a refusal met here can have been worded by the capability model, by
//! the file half, by the record, by the places a question may go or by this
//! crate, and a fixture holding only some of them would make a missing string
//! look like a passing test.
//!
//! The folders are real, for the reason `alo-files`' fixture gives: there is
//! one thing that acts, and abstracting it would be inventing a second answer
//! to *what happened when the machine was asked*.
//!
//! **The provider is a real socket and the runtime is a stub of the trait**,
//! which is `alo-asking`'s division and ADR 0006's reasoning rather than a
//! shortcut: what is worth testing about a provider is what really went out on
//! the wire, and what is worth testing about a runtime is the order around it,
//! which is the same whatever is behind the trait.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::io::{BufRead as _, Read as _, Write as _};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

use alo_answering::{Answering, WentWrong};
use alo_asking::{Miswired, NotAQuestion};
use alo_capability::{
    AnswerError, Authorised, Call, CallError, Given, Grant, Grantee, Grants, ProposalError, Reach,
    Verbs,
};
use alo_context::{Context, Document};
use alo_egress::{EgressPolicy, Indicator, Leaving, NotPermitted};
use alo_files::{Failed, OnThisMachine, Resolving, file_verbs, file_words};
use alo_keeping::NotKept;
use alo_models::{
    InferenceSource, Installed, Loaded, ModelRuntime, ProgressSink, Provider, Region, RuntimeError,
    SourcePolicy,
};
use alo_strings::{Language, Strings, Translation, Word};

use crate::refusing::NotDone;
use crate::unanswered::NoAnswer;

/// A fixed moment, so that expiry is arithmetic rather than a wait.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a turn, a grant and a question stand in these tests.
pub(crate) fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent these turns belong to.
pub(crate) fn files() -> Grantee {
    Grantee::named("@files")
}

/// A folder of this test's own, resolved.
///
/// Resolved because a grant is over a place: on Windows a resolved path carries
/// a `\\?\` prefix the typed one does not, and a grant made over the other
/// spelling would match nothing. `docs/quirks.md` records it.
pub(crate) fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-turn-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// Grants to `@files` over these folders, made at noon and lasting an hour.
pub(crate) fn granting(folders: &[&Path]) -> Grants {
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

/// A path as a verb's argument arrives: text.
pub(crate) fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// What an invocation offered, with this document open.
pub(crate) fn offering(document: &Path) -> Context {
    Context::at_invocation(noon()).and_document(Document::open(document).unwrap())
}

/// The words this machine reads, with nothing translated.
///
/// **Four crates' lists**, which is the arrangement a shell is really in: one
/// vocabulary, every crate declaring into it under its own area.
pub(crate) fn in_english() -> Strings {
    Strings::of(everything_this_machine_says())
}

/// The same, with the given words translated into German and German preferred.
///
/// German because what is read during a turn is sentences rather than labels,
/// and German moves the verb — so a translation that came out reading like
/// English with the words swapped would not be exercising anything.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = everything_this_machine_says();
    let mut german = Translation::into_language(german_language());
    for (word, says) in words {
        german = german.says(word.key(), *says);
    }
    let speaking = vocabulary.check(german).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german_language()]);
    strings
}

/// German, as `alo-strings` names a language.
pub(crate) fn german_language() -> Language {
    Language::written("de").unwrap()
}

/// Every word a machine running a turn has loaded.
///
/// **Eight crates' lists.** A turn can hand back a refusal worded by any of
/// them, and a fixture that held only the ones this crate's oldest tests needed
/// would answer a missing string with a passing test.
fn everything_this_machine_says() -> alo_strings::Vocabulary {
    let mut vocabulary = file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_keeping::declare_into(&mut vocabulary).unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_answering::declare_into(&mut vocabulary).unwrap();
    alo_asking::declare_into(&mut vocabulary).unwrap();
    crate::words::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// A provider somebody added, at this address.
pub(crate) fn a_provider(endpoint: &str) -> Provider {
    Provider::checked(
        "Mistral",
        endpoint,
        Region::Declared("the EU".to_owned()),
        None,
    )
    .unwrap()
}

/// Where an answer from that provider says it came from.
pub(crate) fn mistral_source() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "Mistral".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// A service somebody runs on this machine, at this address.
///
/// Its region is unknown because nobody declared one: an answer from it says
/// *on this machine*, which is what item 18a decided `ThisMachine` means.
pub(crate) fn a_service(endpoint: &str) -> Provider {
    Provider::checked("vLLM", endpoint, Region::Unknown, None).unwrap()
}

/// A provider that is genuinely somewhere else, at an address nothing is
/// listening on.
///
/// Not `127.0.0.1`: that address **is** this machine as far as
/// `alo_models::Provider::source` is concerned, so a test that wants a question
/// to leave and fail would get one that never left. `alo-asking`'s fixture uses
/// the same address for the same reason.
pub(crate) fn far_away() -> Provider {
    a_provider("https://127.0.0.2:1")
}

/// A model runtime on this machine, which answers one way or fails one way.
///
/// It records what it was asked, so a test can assert that a question reached
/// the runtime as it was written — and, more usefully, that it reached nothing
/// at all when the permission named somewhere else.
///
/// The six methods a turn never calls answer [`RuntimeError::Unreachable`]
/// rather than panicking: a fixture that panicked would turn a door reaching
/// for the wrong method into a crash report instead of a test failure with a
/// name on it.
#[derive(Debug)]
pub(crate) struct Stub {
    /// What it says when asked, or why it will not.
    says: Result<String, RuntimeError>,
    /// Every question put to it, with the model it was put to.
    asked: Mutex<Vec<(String, String)>>,
}

impl Stub {
    /// A runtime that answers with this.
    pub(crate) fn answering(said: &str) -> Self {
        Self {
            says: Ok(said.to_owned()),
            asked: Mutex::new(Vec::new()),
        }
    }

    /// A runtime that fails this way.
    pub(crate) fn failing(why: RuntimeError) -> Self {
        Self {
            says: Err(why),
            asked: Mutex::new(Vec::new()),
        }
    }

    /// How many questions reached it.
    pub(crate) fn times_asked(&self) -> usize {
        self.asked.lock().unwrap().len()
    }

    /// The one question it was asked, if it was asked exactly one.
    pub(crate) fn asked(&self) -> Option<(String, String)> {
        match self.asked.lock().unwrap().as_slice() {
            [only] => Some(only.clone()),
            _ => None,
        }
    }
}

impl ModelRuntime for Stub {
    fn installed(&self) -> Result<Vec<Installed>, RuntimeError> {
        Err(RuntimeError::Unreachable)
    }

    fn loaded(&self) -> Result<Vec<Loaded>, RuntimeError> {
        Err(RuntimeError::Unreachable)
    }

    fn fetch(&self, _id: &str, _progress: &mut dyn ProgressSink) -> Result<(), RuntimeError> {
        Err(RuntimeError::Unreachable)
    }

    fn remove(&self, _id: &str) -> Result<(), RuntimeError> {
        Err(RuntimeError::Unreachable)
    }

    fn load(&self, _id: &str) -> Result<(), RuntimeError> {
        Err(RuntimeError::Unreachable)
    }

    fn unload(&self, _id: &str) -> Result<(), RuntimeError> {
        Err(RuntimeError::Unreachable)
    }

    fn answers(&self, question: &str, of_model: &str) -> Result<String, RuntimeError> {
        self.asked
            .lock()
            .unwrap()
            .push((question.to_owned(), of_model.to_owned()));
        self.says.clone()
    }
}

/// One answer, in the shape every OpenAI-compatible provider replies with.
pub(crate) const AN_ANSWER: &str = r#"{"choices":[{"index":0,"message":{"role":"assistant","content":"No, not without written consent."}}]}"#;

/// One request, one canned reply.
///
/// Answers with the address to point a provider at, and a handle that yields
/// everything the client sent — head **and** body — so a test can assert on
/// what really went out rather than on what was intended. A real socket rather
/// than a mocked client, for `alo-asking`'s reason: the thing worth testing is
/// what crossed the wire.
pub(crate) fn serving(
    response_body: &'static str,
    status: u16,
) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
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
        if length > 0 {
            reader.read_exact(&mut body).unwrap();
        }
        let reply = format!(
            "HTTP/1.1 {status} OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        );
        stream.write_all(reply.as_bytes()).unwrap();
        stream.flush().unwrap();
        head + &String::from_utf8_lossy(&body)
    });
    (format!("http://127.0.0.1:{port}"), handle)
}

/// The permission a person's setting makes, for a place nothing forbids.
pub(crate) fn permitting(source: InferenceSource) -> Answering {
    Answering::chosen(source, &SourcePolicy::Anywhere).unwrap()
}

/// A failure at this place, as `alo-answering` makes one.
fn failing(source: InferenceSource) -> alo_answering::Failed {
    permitting(source)
        .did_not_answer(WentWrong::NothingAnswered, &[], &SourcePolicy::Anywhere)
        .unwrap()
}

/// One example of every way a question a turn asked can come back without an
/// answer.
///
/// Written out by hand, for the reason [`everything_that_can_come_back`] is: a
/// list derived from the variants would be derived from the same thing it is
/// checking. Two of the seven appear twice, because the two questions this type
/// answers — *did anything leave* and *is the turn over* — are answered
/// differently by the two roads through each.
pub(crate) fn every_way_a_question_can_fail() -> Vec<NoAnswer> {
    let mut indicator = Indicator::default();
    // A real refusal from the rule rather than one built by hand: `NotPermitted`
    // has no public constructor, which is what stops a held-back entry from
    // being written about a refusal the policy never made.
    let held_back = refused_by(&mut indicator, &SourcePolicy::ThisMachineOnly);

    vec![
        NoAnswer::NotAQuestion(NotAQuestion::Nothing),
        NoAnswer::CannotBeShown(a_name_that_cannot_be_drawn()),
        NoAnswer::HeldBack(held_back),
        NoAnswer::DidNotAnswer(Box::new(failing(mistral_source()))),
        NoAnswer::DidNotAnswer(Box::new(failing(InferenceSource::ThisMachine))),
        NoAnswer::Miswired(Miswired::NotAProvider),
        NoAnswer::NotRecorded {
            why: no_space_left(),
            after_it_left: true,
        },
        NoAnswer::NotRecorded {
            why: no_space_left(),
            after_it_left: false,
        },
        NoAnswer::TurnClosed,
    ]
}

/// What the rule says when it will not let a question leave.
fn refused_by(indicator: &mut Indicator, policy: &SourcePolicy) -> NotPermitted {
    let leaving = Leaving::asking(&files(), &mistral_source()).unwrap();
    indicator
        .beginning(&EgressPolicy::from(policy), leaving, noon())
        .err()
        .unwrap()
}

/// A provider whose name cannot be put on the indicator.
///
/// `alo_models::Provider` asks only that a name is not empty, so one carrying a
/// line break reaches law 1 — where it is refused rather than drawn onto the
/// one surface a person is expected to trust.
fn a_name_that_cannot_be_drawn() -> alo_egress::DestinationError {
    Leaving::asking(
        &files(),
        &InferenceSource::Hosted {
            provider: "Mistral\nis fine really".to_owned(),
            region: Region::Unknown,
        },
    )
    .err()
    .unwrap()
}

/// What a full disk says.
fn no_space_left() -> NotKept {
    NotKept::NotAddedTo {
        path: "/var/lib/alo/record.jsonl".to_owned(),
        why: "no space left on device".to_owned(),
    }
}

/// The six, as a machine offers them.
pub(crate) fn the_six() -> Verbs {
    file_verbs().unwrap()
}

/// A call over a folder, from the verb that lists one.
pub(crate) fn listing(folder: &Path) -> Call {
    the_six()
        .call("list_folder", &[("folder", as_given(folder))])
        .unwrap()
}

/// One example of every way a turn can answer with something other than what
/// was asked for.
///
/// Written out by hand, because the point of the list is that a variant added
/// without a sentence for it is caught — and a list derived from the variants
/// would be derived from the same thing it is checking.
pub(crate) fn everything_that_can_come_back() -> Vec<NotDone> {
    // A real refusal rather than one built by hand, because `Refused` is made
    // by the crate that refuses and this list is about what a person is told.
    let nothing_granted = Grants::default();
    let refused = Authorised::read(
        &listing(Path::new("/home/anna/Invoices")),
        &files(),
        &nothing_granted,
        noon(),
    )
    .err()
    .unwrap();

    vec![
        NotDone::TurnedAway(CallError::NoSuchVerb {
            name: "delete_everything".to_owned(),
        }),
        NotDone::NeverAsked(ProposalError::ReadDoesNotWait {
            verb: "list_folder".to_owned(),
        }),
        NotDone::NotAnswered(AnswerError::NothingWaiting { number: 7 }),
        NotDone::Refused(refused),
        NotDone::MachineCouldNot(Failed::Gone {
            path: "/home/anna/Invoices/march.pdf".to_owned(),
        }),
        NotDone::NotRecorded(NotKept::NotAddedTo {
            path: "/var/lib/alo/record.jsonl".to_owned(),
            why: "no space left on device".to_owned(),
        }),
        NotDone::TurnClosed,
    ]
}
