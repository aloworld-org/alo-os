//! The same day, answered by a service that forwards — and alo OS says the
//! same thing about it.
//!
//! **This file documents a gap. Nothing in it is a guarantee.**
//! `a_day_that_never_left.rs` walks a day of questions put to a service on this
//! machine and asserts what a person is told: the answer says *on this machine*,
//! the indicator is quiet, and the record's *what left this machine* is empty.
//! Every one of those assertions is true and worth having.
//!
//! This file makes **exactly the same assertions** with one thing changed: the
//! service at `127.0.0.1` is a relay, and the question is answered somewhere
//! else. They all still pass. That is the whole point — the claims are about the
//! address alo OS connected to, and they are read as claims about where the
//! question was processed.
//!
//! `docs/features.md`, on 2026-09-08: *a loopback address establishes where a
//! service is contacted, not where it performs inference.* This is that sentence
//! with a socket under it.
//!
//! # What it is not
//!
//! - **Not a claim that the door is broken.** `Served::at` still refuses every
//!   address that is not this machine, a redirect is still refused
//!   (`a_local_service_cannot_redirect_a_question_off_this_machine`), and a
//!   local failure still never becomes an API call. Those are guarantees and
//!   they are tested where they belong. A transparent relay is not a redirect:
//!   it answers in its own voice, so nothing in the HTTP exchange gives it away.
//! - **Not a proposal.** [ADR 0021](../../../docs/decisions/0021-what-a-service-on-this-machine-vouches-for.md)
//!   is PROPOSED and unaccepted, and this file asserts today's behaviour so that
//!   whoever changes it has to come here and say what they changed it to.
//! - **Not a judgement about the person's choice.** A third-party runtime on
//!   this machine is a legitimate configuration and stays one. What is at issue
//!   is only what alo OS may *claim* about it.
//!
//! # Nothing here leaves this machine
//!
//! The relay and the far service are listeners this test owns, on ports the
//! operating system chose. The far one binds this machine's own interface where
//! there is one, so that *the question left the loopback address the person
//! configured* is literal rather than illustrative, and loopback otherwise.
//! Nothing resolves a name, no credential is used, and no external provider is
//! contacted.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::thread;
use std::time::{Duration, SystemTime};

use alo_answering::Answering;
use alo_asking::{Asking, Question, Served};
use alo_capability::Grantee;
use alo_egress::Indicator;
use alo_models::{InferenceSource, Provider, Region, SourcePolicy};
use alo_record::{Entry, Only, Record};
use alo_strings::{Strings, Vocabulary};

/// One answer, in the shape every OpenAI-compatible service replies with.
const AN_ANSWER: &str = r#"{"choices":[{"message":{"role":"assistant","content":"No, not without written consent."}}]}"#;

/// Long enough for a loaded machine, short enough that a hang is a failure.
const NOT_FOREVER: Duration = Duration::from_secs(10);

/// The moment the person pressed the key.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// The words this machine reads, with nothing translated.
fn strings() -> Strings {
    let mut vocabulary = Vocabulary::default();
    alo_models::declare_into(&mut vocabulary).expect("the words alo-models says");
    Strings::of(vocabulary)
}

/// The question, which is the same one every test in this crate asks.
fn question() -> Question {
    Question::asked("may the tenant sublet?", "a-model-on-this-machine").expect("a question")
}

/// Somewhere that is not the loopback address the person configured.
///
/// This machine's own interface where it has one, so that *the question left
/// the address that was vouched for* is literal. Loopback on a machine with no
/// route, where the test still says what it says — alo OS cannot tell where the
/// relay went either way — with less to look at.
fn somewhere_else() -> IpAddr {
    let asking = UdpSocket::bind("0.0.0.0:0").expect("a socket of our own");
    match asking
        .connect("192.0.2.1:9")
        .and_then(|()| asking.local_addr())
    {
        Ok(ours) => ours.ip(),
        Err(_) => IpAddr::from([127, 0, 0, 1]),
    }
}

/// Read one HTTP request from this stream: the head, and the body it declares.
fn one_request(from: &mut BufReader<TcpStream>) -> Vec<u8> {
    let mut head = Vec::new();
    let mut length = 0usize;
    loop {
        let mut line = String::new();
        if from.read_line(&mut line).expect("the request is readable") == 0 {
            break;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse().unwrap_or(0);
        }
        let done = line == "\r\n" || line == "\n";
        head.extend_from_slice(line.as_bytes());
        if done {
            break;
        }
    }
    let mut body = vec![0u8; length];
    if length > 0 {
        from.read_exact(&mut body).expect("the body is readable");
    }
    head.extend_from_slice(&body);
    head
}

/// The service that really answers, somewhere the person never configured.
///
/// Hands back what it was asked, so a test can say the question arrived here
/// rather than inferring it from the answer.
fn the_far_service() -> (SocketAddr, thread::JoinHandle<Option<String>>) {
    let listener =
        TcpListener::bind(SocketAddr::new(somewhere_else(), 0)).expect("a listener of our own");
    let at = listener.local_addr().expect("a listener knows where it is");
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().ok()?;
        stream.set_read_timeout(Some(NOT_FOREVER)).ok()?;
        let mut reading = BufReader::new(stream.try_clone().ok()?);
        let asked = one_request(&mut reading);
        let mut answering = stream;
        answering
            .write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\
                     Connection: close\r\n\r\n{AN_ANSWER}",
                    AN_ANSWER.len()
                )
                .as_bytes(),
            )
            .ok()?;
        answering.flush().ok()?;
        Some(String::from_utf8_lossy(&asked).into_owned())
    });
    (at, handle)
}

/// A service on loopback that carries the question somewhere else and brings
/// the answer back in its own voice.
///
/// **Eleven lines and no cleverness**, which is the point: this is what anybody
/// can put at `127.0.0.1` and configure alo OS to use, and it is indistinguishable
/// from vLLM by anything alo OS can ask.
fn a_service_that_forwards_to(far: SocketAddr) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener can be made");
    let at = listener.local_addr().expect("a listener knows where it is");
    let handle = thread::spawn(move || {
        let Ok((from, _)) = listener.accept() else {
            return;
        };
        drop(from.set_read_timeout(Some(NOT_FOREVER)));
        let Ok(mut reading) = from.try_clone().map(BufReader::new) else {
            return;
        };
        let asked = one_request(&mut reading);

        let Ok(mut onward) = TcpStream::connect_timeout(&far, NOT_FOREVER) else {
            return;
        };
        drop(onward.set_read_timeout(Some(NOT_FOREVER)));
        if onward.write_all(&asked).is_err() {
            return;
        }
        drop(onward.flush());
        let mut answered = Vec::new();
        drop(onward.read_to_end(&mut answered));

        let mut back = from;
        drop(back.write_all(&answered));
        drop(back.flush());
    });
    (format!("http://{at}"), handle)
}

/// **A day answered by a service that forwards reads exactly like a day that
/// never left.**
///
/// Every assertion here is copied from
/// `a_day_that_never_left.rs::a_day_of_questions_answered_by_a_local_service_puts_no_egress_in_the_record`
/// on purpose, and every one of them still passes — while the far service holds
/// the person's question in its hands.
///
/// **This documents a gap.** The day it fails is the day somebody makes alo OS
/// say something truthful here, and they should come to this file and say what.
#[test]
fn a_service_that_forwards_is_answered_as_though_it_never_left() {
    let (far, answering_far_away) = the_far_service();
    let (url, relaying) = a_service_that_forwards_to(far);

    let mail = Grantee::named("@mail");
    let policy = SourcePolicy::Anywhere;
    let indicator = Indicator::default();
    let mut record = Record::default();

    let service = Provider::checked("A service on this machine", &url, Region::Unknown, None)
        .expect("a service somebody runs here");
    let served = Served::at(&service, None).expect("its address is on this machine");
    let permitted =
        Answering::chosen(served.source(), &policy).expect("no rule stops a local answer");

    let answer = Asking::by(&mail, permitted, &[], &policy)
        .to_a_service_on_this_machine(&question(), &served, &resolved(&served))
        .expect("the service answered");
    record.keep(Entry::answered_here(&mail, noon()));

    // What the person is told, and each line of it is what the honest case is
    // told too.
    assert_eq!(answer.source(), &InferenceSource::ThisMachine);
    assert_eq!(answer.came_from(&strings()).text(), "on this machine");
    assert!(indicator.is_quiet(), "the indicator showed something");
    assert_eq!(
        record
            .answering(&alo_record::Asking::anything().only(Only::Egress))
            .count(),
        0,
        "the record says nothing left this machine"
    );

    // And what actually happened, which none of the above can see.
    relaying.join().expect("the relay finished");
    let asked = answering_far_away
        .join()
        .expect("the far service thread finishes")
        .expect("the question reached the far service");
    assert!(
        asked.contains("may the tenant sublet?"),
        "the far service did not receive the question, so this test proves nothing: {asked}"
    );
}

/// **And `ThisMachineOnly` permits it**, which is the policy question rather
/// than a bug.
///
/// A person who set *keep my questions on this machine* and configured a service
/// at `127.0.0.1` is permitted, and would be permitted identically if that
/// service forwarded every word to a continent of somebody else's choosing.
///
/// `a_day_that_never_left.rs::no_rule_can_stop_this_machine_answering_its_own_question`
/// asserts the rule's behaviour for an honest service. This asserts it for one
/// alo OS cannot verify, which is the case ADR 0021 asks the owner about and the
/// one nobody has decided. **Documenting, not endorsing.**
#[test]
fn this_machine_only_still_permits_a_service_that_cannot_be_verified() {
    let (far, answering_far_away) = the_far_service();
    let (url, relaying) = a_service_that_forwards_to(far);

    let policy = SourcePolicy::ThisMachineOnly;
    let service = Provider::checked("A service on this machine", &url, Region::Unknown, None)
        .expect("a service somebody runs here");
    let served = Served::at(&service, None).expect("its address is on this machine");

    // The rule is asked, and permits it, because the source is `ThisMachine`
    // and the source is decided by the address.
    let permitted = Answering::chosen(served.source(), &policy)
        .expect("`this machine only` permits a service whose address is this machine");

    let answer = Asking::by(&Grantee::named("@mail"), permitted, &[], &policy)
        .to_a_service_on_this_machine(&question(), &served, &resolved(&served))
        .expect("the service answered");
    assert_eq!(answer.source(), &InferenceSource::ThisMachine);

    relaying.join().expect("the relay finished");
    assert!(
        answering_far_away
            .join()
            .expect("the far service thread finishes")
            .expect("the question reached the far service")
            .contains("may the tenant sublet?"),
        "the strictest rule this machine has permitted a question it could not follow"
    );
}

/// Where a request would connect, resolved the way `alo-turn` resolves it:
/// before anything is bounded, and handed to the door rather than looked up
/// inside it (ADR 0020).
fn resolved(served: &Served<'_>) -> Vec<SocketAddr> {
    use std::net::ToSocketAddrs as _;
    served
        .where_it_would_connect()
        .into_iter()
        .flat_map(|(host, port)| {
            (host.as_str(), port)
                .to_socket_addrs()
                .into_iter()
                .flatten()
        })
        .collect()
}
