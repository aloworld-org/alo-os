//! The whole journey: a question, a departure, an answer, and a record.
//!
//! `alo-asking` writes nothing down — `alo-record` is reachable from none of
//! the crates it observes, and that is the arrangement this test exists to
//! prove is enough. The departure comes back on **both** paths, so both the
//! question that was answered and the one that was not can be written down by
//! whoever holds the record, and neither can be written by anything that did
//! not actually cause an egress.
//!
//! Law 1 has two halves and this walks the second one: *visible at the moment
//! it happens **and afterwards in a record**.*

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, Read as _, Write as _};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, SystemTime};

use alo_answering::{Answering, WentWrong};
use alo_asking::{Asking, DidNotAnswer, Hosted, NotAsked, Question};
use alo_capability::Grantee;
use alo_egress::Indicator;
use alo_models::{InferenceSource, Provider, Region, SourcePolicy};
use alo_record::{Entry, Only, Record};
use alo_strings::{Strings, Vocabulary};

/// One answer, in the shape every OpenAI-compatible provider replies with.
const AN_ANSWER: &str = r#"{"choices":[{"message":{"role":"assistant","content":"No, not without written consent."}}]}"#;

/// A stub that answers one request.
///
/// A third copy of this fixture, and the reason is the same one `alo-models`
/// gives for its own: what is worth testing is what goes out on a socket, and
/// a crate's `cfg(test)` helpers are not reachable from its integration tests.
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

fn mistral(endpoint: &str) -> Provider {
    Provider::checked(
        "Mistral",
        endpoint,
        Region::Declared("the EU".to_owned()),
        None,
    )
    .expect("a provider somebody added")
}

/// The failure, if the question was sent and did not come back.
///
/// A helper rather than a `let … else { panic!(…) }`, because this workspace
/// denies `clippy::panic` everywhere including its tests: an `Option` unwrapped
/// where it is expected says the same thing and obeys the same lint.
fn did_not_answer(not_asked: NotAsked) -> Option<Box<DidNotAnswer>> {
    match not_asked {
        NotAsked::DidNotAnswer(unanswered) => Some(unanswered),
        _ => None,
    }
}

/// The refusal a rule made, if that is what this was.
fn held_back(not_asked: NotAsked) -> Option<alo_egress::NotPermitted> {
    match not_asked {
        NotAsked::HeldBack(refused) => Some(refused),
        _ => None,
    }
}

fn question() -> Question {
    Question::asked("may the tenant sublet?", "mistral-small-latest").expect("a question")
}

/// **A question, out and back, and written down.** The record answers *what
/// left this machine* with one entry, made from the departure this crate hands
/// back and from nothing else.
#[test]
fn a_question_that_was_answered_is_one_departure_in_the_record() {
    let (url, server) = serving(AN_ANSWER, 200);
    let provider = mistral(&url);
    let hosted = Hosted::provider(&provider, None);
    let mail = Grantee::named("@mail");
    let policy = SourcePolicy::Anywhere;
    let mut indicator = Indicator::default();
    let mut record = Record::default();

    let answering = Answering::chosen(hosted.named_source(), &policy).expect("nothing forbids it");
    let asked = Asking::by(&mail, answering, &[], &policy)
        .to_a_provider(&question(), &hosted, &mut indicator, noon())
        .expect("the stub answered");
    server.join().expect("the stub finished");

    // The record is written from the departure, and only then does the line
    // come off the indicator.
    record.keep(Entry::left(asked.departing()));
    let answer = asked.ended(&mut indicator);
    assert!(indicator.is_quiet());
    assert_eq!(answer.text(), "No, not without written consent.");

    let asking = alo_record::Asking::anything().only(Only::Egress);
    assert_eq!(record.answering(&asking).count(), 1);
    let entry = record
        .answering(&asking)
        .next()
        .expect("the one thing that left");
    assert_eq!(entry.at(), noon());
    assert_eq!(
        entry
            .happened()
            .agent()
            .map(|agent| agent.as_str().to_owned()),
        Some("@mail".to_owned())
    );
    // And nothing alo OS did on its own, because it did nothing on its own.
    assert_eq!(
        record
            .answering(&alo_record::Asking::anything().only(Only::OnItsOwn))
            .count(),
        0
    );
}

/// **A question that failed still left**, so it is in the record exactly like
/// one that did not — a machine that wrote down only the questions that were
/// answered would report a quieter day than it had.
#[test]
fn a_question_that_was_not_answered_is_still_one_departure_in_the_record() {
    let (url, server) = serving(r#"{"error":"upstream capacity"}"#, 503);
    let provider = mistral(&url);
    let hosted = Hosted::provider(&provider, None);
    let mail = Grantee::named("@mail");
    let policy = SourcePolicy::Anywhere;
    let mut indicator = Indicator::default();
    let mut record = Record::default();

    let answering = Answering::chosen(hosted.named_source(), &policy).expect("nothing forbids it");
    let elsewhere = [InferenceSource::ThisMachine];
    let not_asked = Asking::by(&mail, answering, &elsewhere, &policy)
        .to_a_provider(&question(), &hosted, &mut indicator, noon())
        .expect_err("the stub was having trouble");
    server.join().expect("the stub finished");

    assert!(!not_asked.nothing_left());
    let unanswered = did_not_answer(not_asked).expect("a 503 is a failure at the far end");
    record.keep(Entry::left(unanswered.departing()));
    let failed = unanswered.ended(&mut indicator);
    assert!(indicator.is_quiet());

    assert_eq!(failed.why(), WentWrong::HavingTrouble(503));
    assert_eq!(
        record
            .answering(&alo_record::Asking::anything().only(Only::Egress))
            .count(),
        1
    );

    // And the offer is a thing a person answers. Nothing here took it, and the
    // record has nothing about a second attempt because there was not one.
    assert_eq!(failed.elsewhere().offers().len(), 1);
    assert_eq!(
        failed.nothing_was_sent(&strings()).text(),
        "nothing was sent anywhere, and nothing will be unless you say so"
    );
}

/// **A rule that refused it is evidence too**, and it is not egress: the record
/// answers *what left* with nothing and *what was stopped* with one entry, in
/// the words the person read.
#[test]
fn a_question_the_rule_refused_is_written_down_as_a_refusal_and_not_as_egress() {
    // Nothing is listening here, so a question that was sent would come back as
    // a failure rather than as the refusal this test expects.
    let provider = mistral("https://127.0.0.2:1");
    let hosted = Hosted::provider(&provider, None);
    let mail = Grantee::named("@mail");
    let mut indicator = Indicator::default();
    let mut record = Record::default();

    // Chosen when the rule was looser; refused by the rule in force now.
    let answering = Answering::chosen(hosted.named_source(), &SourcePolicy::Anywhere)
        .expect("nothing forbade it then");
    let not_asked = Asking::by(&mail, answering, &[], &SourcePolicy::InTheBuilding)
        .to_a_provider(&question(), &hosted, &mut indicator, noon())
        .expect_err("the rule keeps questions in the building");

    assert!(not_asked.nothing_left());
    let refused = held_back(not_asked).expect("the rule keeps questions in the building");
    record.keep(Entry::held_back(&refused, &strings(), noon()));
    assert!(indicator.is_quiet());

    assert_eq!(
        record
            .answering(&alo_record::Asking::anything().only(Only::Egress))
            .count(),
        0,
        "nothing left the machine"
    );
    let stopped = alo_record::Asking::anything().only(Only::Refusals);
    assert_eq!(record.answering(&stopped).count(), 1);
    assert_eq!(
        refused.said(&strings()).text(),
        "this machine is set to keep everything in the building, and Mistral, in the EU is \
         outside it"
    );
}
