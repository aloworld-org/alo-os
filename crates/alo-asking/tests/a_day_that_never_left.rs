//! The other journey: a question, an answer, and a record with no egress in it.
//!
//! `docs/features.md` promises that **a working day with a local model produces
//! zero inference egress**, measured at the network boundary. The measurement
//! is a machine's and this repository does not have one, but the half that is
//! code can be walked here: a day of questions answered on this machine puts
//! nothing on the indicator, makes no departure, and leaves a record whose
//! *what left this machine* is empty while its *what happened* is not.
//!
//! It is `from_a_question_to_what_left.rs` from the other side. That file
//! proves an egress cannot happen without evidence; this one proves the absence
//! of an egress is not the absence of a day.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, Read as _, Write as _};
use std::net::TcpListener;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime};

use alo_answering::{Answering, WentWrong};
use alo_asking::{Asking, Miswired, NotAnswered, Question, Served};
use alo_capability::Grantee;
use alo_egress::Indicator;
use alo_models::{
    InferenceSource, Installed, Loaded, ModelRuntime, ProgressSink, Provider, Region, RuntimeError,
    SourcePolicy,
};
use alo_record::{Entry, Only, Record};
use alo_strings::{Strings, Vocabulary};

/// One answer, in the shape every OpenAI-compatible service replies with.
const AN_ANSWER: &str = r#"{"choices":[{"message":{"role":"assistant","content":"No, not without written consent."}}]}"#;

/// A stub that answers one request.
///
/// A fourth copy of this fixture, for the reason the others give: what is worth
/// testing is what goes out on a socket, and a crate's `cfg(test)` helpers are
/// not reachable from its integration tests. This one is here because item 18b
/// put a **socket** on the path that never leaves — a service on this machine is
/// reached over http on loopback, not through a trait.
fn serving(response_body: &'static str, status: u16) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port");
    let port = listener.local_addr().expect("an address").port();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("one request");
        let mut reader = std::io::BufReader::new(stream.try_clone().expect("the same socket"));
        let mut length = 0usize;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).expect("a line") == 0 {
                break;
            }
            if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                length = v.trim().parse().unwrap_or(0);
            }
            if line == "\r\n" || line == "\n" {
                break;
            }
        }
        let mut body = vec![0u8; length];
        if length > 0 {
            reader.read_exact(&mut body).expect("the body");
        }
        let reply = format!(
            "HTTP/1.1 {status} OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        );
        stream.write_all(reply.as_bytes()).expect("a reply");
        stream.flush().expect("a flush");
    });
    (format!("http://127.0.0.1:{port}"), handle)
}

/// A model runtime on this machine.
///
/// A second copy of this fixture, for the reason the socket stub next door is a
/// third: a crate's `cfg(test)` helpers are not reachable from its integration
/// tests. It counts what it was asked, which is what the *never a silent
/// fallback* tests here actually assert on.
#[derive(Debug)]
struct Local {
    /// What it says when asked, or why it will not.
    says: Result<String, RuntimeError>,
    /// How many questions reached it.
    asked: Mutex<usize>,
}

impl Local {
    /// A runtime that answers with this.
    fn answering(said: &str) -> Self {
        Self {
            says: Ok(said.to_owned()),
            asked: Mutex::new(0),
        }
    }

    /// A runtime that fails this way.
    fn failing(why: RuntimeError) -> Self {
        Self {
            says: Err(why),
            asked: Mutex::new(0),
        }
    }

    /// How many questions reached it.
    fn times_asked(&self) -> usize {
        *self.asked.lock().expect("the count")
    }
}

impl ModelRuntime for Local {
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

    fn answers(&self, _question: &str, _of_model: &str) -> Result<String, RuntimeError> {
        *self.asked.lock().expect("the count") += 1;
        self.says.clone()
    }
}

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

fn strings() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_models::declare_into(&mut vocabulary).expect("the model words");
    alo_egress::declare_into(&mut vocabulary).expect("the egress words");
    alo_answering::declare_into(&mut vocabulary).expect("the answering words");
    alo_asking::declare_into(&mut vocabulary).expect("this crate's words");
    Strings::of(vocabulary)
}

fn question() -> Question {
    Question::asked("may the tenant sublet?", "mistral-7b-instruct").expect("a question")
}

fn mistral() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "Mistral".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// The failure, if it was asked here and did not answer.
fn did_not_answer(not_answered: NotAnswered) -> Option<Box<alo_answering::Failed>> {
    match not_answered {
        NotAnswered::DidNotAnswer(failed) => Some(failed),
        NotAnswered::Miswired(_) => None,
    }
}

/// The wiring mistake, if that is what this was.
fn miswired(not_answered: NotAnswered) -> Option<Miswired> {
    match not_answered {
        NotAnswered::Miswired(miswired) => Some(miswired),
        NotAnswered::DidNotAnswer(_) => None,
    }
}

/// **A working day, and nothing left.** Eight questions answered on this
/// machine: eight entries in the record, no egress in it at all, and an
/// indicator that was never given anything to show.
#[test]
fn a_day_of_questions_answered_here_puts_no_egress_in_the_record() {
    let runtime = Local::answering("No, not without written consent.");
    let mail = Grantee::named("@mail");
    let policy = SourcePolicy::ThisMachineOnly;
    let indicator = Indicator::default();
    let mut record = Record::default();

    for hour in 0..8 {
        let answering = Answering::chosen(InferenceSource::ThisMachine, &policy)
            .expect("no rule stops a machine answering itself");
        let answer = Asking::by(&mail, answering, &[mistral()], &policy)
            .to_this_machine(&question(), &runtime)
            .expect("the runtime answered");

        assert_eq!(answer.source(), &InferenceSource::ThisMachine);
        assert_eq!(
            answer.came_from(&strings()).text(),
            "on this machine",
            "the answer says where it came from, every time"
        );
        record.keep(Entry::answered_here(
            &mail,
            noon() + Duration::from_secs(60 * 60 * hour),
        ));
    }

    assert_eq!(runtime.times_asked(), 8);
    assert!(indicator.is_quiet());
    // The whole of the zero-egress claim that code can carry.
    assert_eq!(
        record
            .answering(&alo_record::Asking::anything().only(Only::Egress))
            .count(),
        0,
        "nothing left this machine"
    );
    // And the day still happened, which is the half that makes the first half
    // worth anything: an empty record would say the same thing about a machine
    // nobody used.
    assert_eq!(record.answering(&alo_record::Asking::anything()).count(), 8);
}

/// **Never a silent fallback, in the direction ADR 0008 was written without.**
/// The local model fails, the machine has a provider it could ask, and it asks
/// nothing: the record has no egress, the offer is a sentence somebody has to
/// answer, and the person is told outright that nothing was sent.
#[test]
fn a_local_model_that_fails_never_becomes_an_api_call() {
    let runtime = Local::failing(RuntimeError::Unreachable);
    let mail = Grantee::named("@mail");
    let policy = SourcePolicy::Anywhere;
    let indicator = Indicator::default();
    let record = Record::default();

    let answering = Answering::chosen(InferenceSource::ThisMachine, &policy)
        .expect("no rule stops a machine answering itself");
    let not_answered = Asking::by(&mail, answering, &[mistral()], &policy)
        .to_this_machine(&question(), &runtime)
        .expect_err("the runtime was not running");

    let failed = did_not_answer(not_answered).expect("it was asked here and did not answer");
    assert_eq!(failed.why(), WentWrong::NothingAnswered);
    assert_eq!(runtime.times_asked(), 1);
    assert!(indicator.is_quiet());
    assert_eq!(
        record
            .answering(&alo_record::Asking::anything().only(Only::Egress))
            .count(),
        0,
        "the provider was never asked"
    );

    // The provider is an offer nobody has taken, and there is no way to take it
    // that is not a person saying so.
    assert_eq!(failed.elsewhere().offers().len(), 1);
    assert_eq!(
        failed
            .elsewhere()
            .offers()
            .first()
            .map(alo_answering::Offer::source),
        Some(&mistral())
    );
    assert_eq!(
        failed.nothing_was_sent(&strings()).text(),
        "nothing was sent anywhere, and nothing will be unless you say so"
    );
    assert_eq!(
        failed.said(&strings()).text(),
        "nothing answered on this machine"
    );
}

/// **And it runs the other way too.** A person who chose a provider does not
/// get answered by the model on their laptop: the runtime is asked nothing at
/// all, and the refusal names the door that provider is behind rather than
/// offering this one instead.
#[test]
fn a_question_bound_for_a_provider_never_reaches_the_model_on_this_machine() {
    let runtime = Local::answering("a smaller model, wearing the same face");
    let mail = Grantee::named("@mail");
    let policy = SourcePolicy::Anywhere;

    let answering = Answering::chosen(mistral(), &policy).expect("nothing forbids it");
    let not_answered = Asking::by(&mail, answering, &[], &policy)
        .to_this_machine(&question(), &runtime)
        .expect_err("the permission was for a provider");

    assert_eq!(miswired(not_answered), Some(Miswired::NotOnThisMachine));
    assert_eq!(runtime.times_asked(), 0, "nothing was asked here");
}

/// **The same day, on a service somebody runs here themselves.** vLLM or
/// LM Studio on loopback is this machine in the only sense law 1 cares about,
/// so a day of questions put to it is a day with no egress in the record —
/// which is item 18b's whole claim, walked rather than asserted about a type.
#[test]
fn a_day_of_questions_answered_by_a_local_service_puts_no_egress_in_the_record() {
    let mail = Grantee::named("@mail");
    let policy = SourcePolicy::ThisMachineOnly;
    let indicator = Indicator::default();
    let mut record = Record::default();

    for hour in 0..4 {
        let (url, server) = serving(AN_ANSWER, 200);
        let service = Provider::checked("vLLM", &url, Region::Unknown, None)
            .expect("a service somebody runs here");
        let served = Served::at(&service, None).expect("it is on this machine");
        let answering = Answering::chosen(served.source(), &policy)
            .expect("no rule stops a machine answering itself");
        let answer = Asking::by(&mail, answering, &[mistral()], &policy)
            .to_a_service_on_this_machine(&question(), &served)
            .expect("the service answered");
        server.join().expect("the stub finished");

        assert_eq!(answer.source(), &InferenceSource::ThisMachine);
        assert_eq!(answer.came_from(&strings()).text(), "on this machine");
        record.keep(Entry::answered_here(
            &mail,
            noon() + Duration::from_secs(60 * 60 * hour),
        ));
    }

    assert!(indicator.is_quiet());
    assert_eq!(
        record
            .answering(&alo_record::Asking::anything().only(Only::Egress))
            .count(),
        0,
        "nothing left this machine"
    );
    assert_eq!(record.answering(&alo_record::Asking::anything()).count(), 4);
}

/// **And the address that would have made that claim a lie cannot be reached at
/// all.** The door with no indicator in it is the one place a question could
/// have left unseen, so it takes a value that only exists for an address on
/// this machine — including the three that read as loopback and are not.
#[test]
fn a_service_that_is_not_on_this_machine_never_becomes_a_door() {
    for endpoint in [
        "https://api.mistral.ai",
        "https://localhost.attacker.example",
        "https://127.0.0.1.attacker.example/v1",
        "https://127.0.0.1@attacker.example/",
    ] {
        let elsewhere =
            Provider::checked("Not here", endpoint, Region::Unknown, None).expect("an address");
        assert_eq!(
            Served::at(&elsewhere, None).expect_err("it is not on this machine"),
            Miswired::ReachesOffThisMachine,
            "{endpoint}"
        );
        // And the same address, asked the way it should be, does cause a
        // departure — so the refusal above is about where it goes rather than
        // about the address being unusable.
        assert!(elsewhere.source().causes_egress(), "{endpoint}");
    }
}
