//! **A question to another computer on the person's own network, over plain
//! http, reaches it — and still leaves this machine as far as alo OS says.**
//!
//! The owner decided on 2026-10-30 (ADR 0021) that a person may add a provider
//! such as `http://192.168.1.20:11434` — an AI service on another computer at
//! home — after Settings has told them that the question and the key cross
//! that network unencrypted. `alo_models::Provider::checked` accepts it. This
//! file is what that acceptance has to mean at the other end:
//!
//! 1. **the question really reaches the service**, with the key, over plain
//!    http — asserted from what the server read off its socket;
//! 2. **an agent's question to it is on the indicator while it is away and in
//!    the record afterwards**, exactly like any provider's (law 1);
//! 3. **the door that shows nothing will not carry it**: `Served`, which is for
//!    a service on this machine, refuses it, so there is no road to such an
//!    address that skips the indicator.
//!
//! A person's own question, typed by them, goes on no indicator for any
//! provider (ADR 0126); that is unchanged, and the person's road is driven here
//! only to show the question reaches the service.
//!
//! # Why the server is on loopback and the address is not
//!
//! The provider's address is written as `192.168.1.20`, and the connection is
//! made to a stub on `127.0.0.1`. That is not a trick played on the code: every
//! road here connects only to the addresses its caller resolved and handed in
//! (ADR 0020), so handing in the stub's address is what the daemon does with
//! whatever the address resolves to. What is under test is everything the
//! address decides — that it is accepted, that it is *somewhere else*, that it
//! goes the provider's road — and a test that needed a second computer on a
//! real network to decide those would not run in the gate.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, Read as _, Write as _};
use std::net::{SocketAddr, TcpListener};
use std::thread;
use std::time::{Duration, SystemTime};

use alo_answering::Answering;
use alo_asking::{Asking, Hosted, Miswired, Question, Served, a_question_nobody_is_showing};
use alo_capability::Grantee;
use alo_egress::Indicator;
use alo_models::{InferenceSource, Provider, Region, Secret, SecretRef, SourcePolicy};
use alo_record::{Entry, Only, Record};

/// What the service at home answers.
const AN_ANSWER: &str = r#"{"choices":[{"message":{"role":"assistant","content":"No, not without written consent."}}]}"#;

/// A stub on loopback that answers one request and hands back what it was sent.
fn serving() -> (u16, thread::JoinHandle<String>) {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).expect("a port");
    let port = listener.local_addr().expect("an address").port();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("one request");
        let mut reader = std::io::BufReader::new(stream.try_clone().expect("the same socket"));
        let mut head = String::new();
        let mut length = 0usize;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).expect("a line") == 0 {
                break;
            }
            if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                length = v.trim().parse().unwrap_or(0);
            }
            let done = line == "\r\n" || line == "\n";
            head.push_str(&line);
            if done {
                break;
            }
        }
        let mut body = vec![0u8; length];
        if length > 0 {
            reader.read_exact(&mut body).expect("the body");
        }
        let reply = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\
             Connection: close\r\n\r\n{AN_ANSWER}",
            AN_ANSWER.len()
        );
        stream.write_all(reply.as_bytes()).expect("a reply");
        stream.flush().expect("a flush");
        head + &String::from_utf8_lossy(&body)
    });
    (port, handle)
}

/// A provider at another computer on the person's network, made the way
/// Settings makes one.
fn at_home(port: u16) -> Provider {
    Provider::checked(
        "At home",
        &format!("http://192.168.1.20:{port}"),
        Region::Unknown,
        Some(SecretRef::named("provider/At home")),
    )
    .expect("another computer on the person's own network is accepted over http")
}

/// Where the stub really is, which is what the caller resolves and hands in.
fn the_stub(port: u16) -> [SocketAddr; 1] {
    [SocketAddr::from(([127, 0, 0, 1], port))]
}

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

fn question() -> Question {
    Question::asked("may the tenant sublet?", "llama3").expect("a question")
}

/// **It is somewhere else**, however close: the answer is a provider's, and a
/// question to it is an egress.
#[test]
fn a_computer_on_the_persons_own_network_is_somewhere_else() {
    let provider = at_home(11434);
    assert_eq!(
        provider.source(),
        InferenceSource::Hosted {
            provider: "At home".to_owned(),
            region: Region::Unknown,
        }
    );
    assert!(provider.source().causes_egress());
}

/// **An agent's question reaches it, is on the indicator while it is away, and
/// is one departure in the record afterwards** — with the question and the key
/// read off the socket by the server.
#[test]
fn an_agents_question_reaches_it_over_plain_http_and_is_shown_and_recorded() {
    let (port, server) = serving();
    let provider = at_home(port);
    let key = Secret::typed("sk-at-home-0123456789").expect("a key");
    let hosted = Hosted::provider(&provider, Some(&key));
    let mail = Grantee::named("@mail");
    let policy = SourcePolicy::Anywhere;
    let mut indicator = Indicator::default();
    let mut record = Record::default();

    let answering = Answering::chosen(hosted.named_source(), &policy).expect("nothing forbids it");
    let asked = Asking::by(&mail, answering, &[], &policy)
        .to_a_provider(
            &question(),
            &hosted,
            &mut indicator,
            noon(),
            &the_stub(port),
        )
        .expect("the service at home answered");
    let sent = server.join().expect("the stub finished");

    // Shown while it was away: the line is still up until the caller ends it.
    assert_eq!(
        indicator.showing().len(),
        1,
        "a question left for another computer and the indicator did not show it"
    );
    record.keep(Entry::left(asked.departing()));
    let answer = asked.ended(&mut indicator);
    assert!(indicator.is_quiet());
    assert_eq!(answer.text(), "No, not without written consent.");

    let egress = alo_record::Asking::anything().only(Only::Egress);
    assert_eq!(record.answering(&egress).count(), 1);

    // And it really went, in clear, with the key: what the person was cautioned
    // about is exactly what happened.
    assert!(sent.starts_with("POST "), "{sent}");
    assert!(sent.contains("may the tenant sublet?"), "{sent}");
    assert!(
        sent.to_ascii_lowercase()
            .contains("authorization: bearer sk-at-home-0123456789"),
        "the key did not travel with the request: {sent}"
    );
}

/// **A person's own question reaches it too**, through the road a person's
/// question takes, and the answer says it came from that provider.
#[test]
fn a_persons_own_question_reaches_it_over_plain_http() {
    let (port, server) = serving();
    let provider = at_home(port);
    let key = Secret::typed("sk-at-home-0123456789").expect("a key");
    let source = provider.source();
    let answering =
        Answering::chosen(source.clone(), &SourcePolicy::Anywhere).expect("nothing forbids it");

    let answer = a_question_nobody_is_showing(
        answering,
        &[],
        &SourcePolicy::Anywhere,
        &question(),
        &Hosted::provider(&provider, Some(&key)),
        &the_stub(port),
    )
    .expect("the question was put and answered");
    let sent = server.join().expect("the stub finished");

    assert_eq!(answer.text(), "No, not without written consent.");
    assert_eq!(answer.source(), &source);
    assert!(sent.contains("may the tenant sublet?"), "{sent}");
}

/// **The door that shows nothing will not carry it.** `Served` is for a service
/// on this machine and goes on no indicator, so if it took this address a
/// question could leave for another computer with nothing shown.
#[test]
fn the_door_for_a_service_on_this_machine_refuses_it() {
    let provider = at_home(11434);
    assert!(matches!(
        Served::at(&provider, None),
        Err(Miswired::ReachesOffThisMachine)
    ));
}

/// **An organisation's rule that keeps questions on this machine still refuses
/// it** — the person's own network is not this machine — and nothing is sent.
#[test]
fn a_rule_that_keeps_questions_on_this_machine_refuses_it() {
    let provider = at_home(11434);
    let refused = Answering::chosen(provider.source(), &SourcePolicy::ThisMachineOnly);
    assert!(
        refused.is_err(),
        "a rule permitting only this machine permitted another computer"
    );
}
