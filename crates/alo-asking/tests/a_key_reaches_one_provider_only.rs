//! Where a credential goes, and everywhere it does not.
//!
//! `openai.rs` already proves the two halves nearest the wire, and neither is
//! repeated here: a service given a key is sent `authorization: bearer …`, and
//! one given none is sent no authorisation at all. Both have one endpoint in
//! them, so neither can answer the question this file asks — **with two
//! configured and both listening, does the key reach only the one the person
//! chose?**
//!
//! # Why this uses the local-service door, and what that is worth
//!
//! A test server here speaks plain HTTP, and **a plain-HTTP provider cannot be
//! constructed**: `Provider::checked` answers `InsecureEndpoint` for `http://`
//! to anywhere that is not this machine, because a key over plain HTTP to
//! somewhere else is a key on the wire in clear. That refusal is a guarantee and
//! this file is not going to route around it by building a `Provider` field by
//! field.
//!
//! So the pairing is exercised through `Served`, which is the same pairing:
//! both doors take a provider and its key **together**, at construction, and
//! hand the pair to the same `openai::put`. What is asserted is that the key
//! goes where the pair says and nowhere else — which is the property, on the
//! door an owned server can actually be.
//!
//! What this therefore does **not** claim is anything about TLS or about a
//! hosted provider's own behaviour. `openai.rs` has the wire, `alo-models` has
//! the refusal above, and `secret.rs` has the rule that a key cannot be
//! rendered at all.
//!
//! # Owned fixtures, a synthetic secret, nothing external
//!
//! Both services are listeners this test owns, on loopback, on ports the
//! operating system chose. The key is invented here and is a credential to
//! nothing. **No external provider is contacted.**

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use alo_answering::Answering;
use alo_asking::{Asking, Question, Served};
use alo_capability::Grantee;
use alo_models::{Provider, Region, Secret, SourcePolicy};

/// One answer, in the shape every OpenAI-compatible service replies with.
const AN_ANSWER: &str = r#"{"choices":[{"message":{"role":"assistant","content":"No."}}]}"#;

/// A key that is not a credential to anything, and never was.
const A_SYNTHETIC_SECRET: &str = "sk-live-DO-NOT-LOG-9f3c1a";

/// Long enough for a loaded machine, short enough that a hang is a failure.
const NOT_FOREVER: Duration = Duration::from_secs(10);

/// A service of this test's own, which answers once and reports what it was
/// sent — or reports that nothing ever arrived.
fn a_service_of_our_own(named: &str) -> (Provider, thread::JoinHandle<Option<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a listener of our own");
    let at = listener.local_addr().expect("a listener knows where it is");
    listener
        .set_nonblocking(true)
        .expect("a listener can be made not to wait");

    let handle = thread::spawn(move || {
        let until = std::time::Instant::now() + Duration::from_secs(3);
        let mut accepted = None;
        while std::time::Instant::now() < until {
            if let Ok((stream, _)) = listener.accept() {
                accepted = Some(stream);
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        let stream = accepted?;
        stream.set_read_timeout(Some(NOT_FOREVER)).ok()?;
        let mut reading = BufReader::new(stream.try_clone().ok()?);
        let mut head = String::new();
        let mut length = 0usize;
        loop {
            let mut line = String::new();
            if reading.read_line(&mut line).ok()? == 0 {
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
            reading.read_exact(&mut body).ok()?;
        }
        let mut answering: TcpStream = stream;
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
        Some(head + &String::from_utf8_lossy(&body))
    });

    let provider = Provider::checked(named, &format!("http://{at}"), Region::Unknown, None)
        .expect("a service on this machine");
    (provider, handle)
}

/// Where a request would connect, resolved the way `alo-turn` resolves it
/// (ADR 0020): before anything is bounded, and handed to the door.
fn resolved(provider: &Provider) -> Vec<SocketAddr> {
    use std::net::ToSocketAddrs as _;
    alo_models::address::where_it_connects(&provider.endpoint)
        .into_iter()
        .flat_map(|(host, port)| {
            (host.as_str(), port)
                .to_socket_addrs()
                .into_iter()
                .flatten()
        })
        .collect()
}

/// **The credential reaches the one the person chose, and no other.**
///
/// Two services are configured and both are listening. The question goes to one
/// of them, with the key. The other is asked nothing at all — not the question,
/// not the key, not a connection.
///
/// The pairing is structural — a door is built from a provider *and* its key
/// together — and this asserts it from the far end of two sockets rather than
/// from the shape of a type.
#[test]
fn the_key_reaches_the_chosen_one_and_the_other_hears_nothing() {
    let (chosen, chosen_heard) = a_service_of_our_own("Chosen");
    let (other, other_heard) = a_service_of_our_own("Other");
    let key = Secret::typed(A_SYNTHETIC_SECRET).expect("a key somebody typed");

    let served = Served::at(&chosen, Some(&key)).expect("it is on this machine");
    let permitted =
        Answering::chosen(served.source(), &SourcePolicy::ThisMachineOnly).expect("permitted");
    let answer = Asking::by(
        &Grantee::named("@mail"),
        permitted,
        &[],
        &SourcePolicy::ThisMachineOnly,
    )
    .to_a_service_on_this_machine(
        &Question::asked("may the tenant sublet?", "a-model").expect("a question"),
        &served,
        &resolved(&chosen),
    )
    .expect("the chosen service answered");
    assert_eq!(answer.text(), "No.");

    let sent = chosen_heard
        .join()
        .expect("the chosen service's thread finishes")
        .expect("the chosen service was asked");
    // The header name and the scheme are matched without case, and the key
    // itself with it — lower-casing the whole line would lower-case the key,
    // and a test that passed on a mangled key would be no test at all.
    assert!(
        sent.to_lowercase().contains("authorization: bearer "),
        "the one the person chose was sent no authorisation: {sent}"
    );
    assert!(
        sent.contains(A_SYNTHETIC_SECRET),
        "the one the person chose was not sent the key it was paired with"
    );

    assert_eq!(
        other_heard
            .join()
            .expect("the other service's thread finishes"),
        None,
        "somewhere the person did not choose was contacted"
    );
    // And the other one was never even given the key to be sent: the pair is
    // made at construction, so there is no moment at which a second door holds
    // this key.
    assert!(Served::at(&other, None).is_ok());
}

/// **A key never travels in clear to somewhere that is not this machine**, which
/// is why the test above uses the local door rather than a hosted provider.
///
/// Asserted here so that the reason this file is shaped as it is cannot quietly
/// stop being true: the day `Provider::checked` stops refusing, this fails and
/// whoever changed it reads the paragraph above.
#[test]
fn a_plain_http_provider_somewhere_else_cannot_be_made_at_all() {
    let refused = Provider::checked(
        "Somewhere else",
        "http://198.51.100.7:8000",
        Region::Unknown,
        None,
    )
    .expect_err("a key over plain http to somewhere else is refused");
    assert_eq!(refused, alo_models::ProviderError::InsecureEndpoint);
}
