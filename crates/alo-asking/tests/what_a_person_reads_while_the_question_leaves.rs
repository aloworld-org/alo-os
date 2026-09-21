//! **What a person reads while their question is leaving the building** — who
//! answered, and where they run.
//!
//! [ADR 0008](../../../docs/decisions/0008-where-inference-happens.md) names
//! three places inference can happen, and says of the third:
//!
//! > The indicator fires, and says **who** and **where**: a provider that will
//! > not say where it runs is reported as unknown rather than assumed to be
//! > nearby.
//!
//! The road itself has been built since 2026-09-03 and is measured from several
//! directions: `from_a_question_to_what_left.rs` walks a question out to a real
//! service and back and writes the record, `a_key_reaches_one_provider_only.rs`
//! holds the credential to one endpoint, and `a_day_that_never_left.rs` counts
//! what a working day sends. **What none of them reads is the line itself.**
//! They assert the indicator is quiet once the answer is home, which is the
//! other half of law 1 and not this one.
//!
//! So this reads the sentence **while the question is still out**, against a
//! real HTTP service on a real socket, in the two states ADR 0008
//! distinguishes:
//!
//! | The provider | What a person reads |
//! |---|---|
//! | says where it runs | `Mistral, in the EU` |
//! | will not say | `Aurora, which has not said where it runs` |
//!
//! # Why the second one is the test that matters
//!
//! The first sentence is pleasant and the second is the promise. A machine that
//! filled an unstated region with something plausible — the country the address
//! resolves in, the region a name suggests, or nothing at all — would be
//! telling somebody their contract went somewhere it may not have gone, at the
//! one moment they could still stop it. So the second case asserts what the
//! line **does not** say as carefully as what it does.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, Read as _, Write as _};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, SystemTime};

use alo_answering::Answering;
use alo_asking::{Asking, Hosted, Question};
use alo_capability::Grantee;
use alo_egress::Indicator;
use alo_models::{Provider, Region, SourcePolicy};
use alo_strings::{Strings, Vocabulary};

/// One answer, in the shape every OpenAI-compatible provider replies with.
const AN_ANSWER: &str = r#"{"choices":[{"message":{"role":"assistant","content":"No, not without written consent."}}]}"#;

/// A real HTTP service on a real socket, which hands back everything it was
/// sent so the exchange can be read rather than described.
fn serving(response_body: &'static str) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port");
    let port = listener.local_addr().expect("an address").port();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("one request");
        let mut reader = std::io::BufReader::new(stream.try_clone().expect("the same socket"));
        let mut received = String::new();
        let mut length = 0usize;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).expect("a line") == 0 {
                break;
            }
            received.push_str(&line);
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
            received.push_str(&String::from_utf8_lossy(&body));
        }
        let reply = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        );
        stream.write_all(reply.as_bytes()).expect("a reply");
        stream.flush().expect("a flush");
        received
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

fn question() -> Question {
    Question::asked("may the tenant sublet?", "mistral-small-latest").expect("a question")
}

/// Where a host and port resolve to on this machine.
fn resolved(where_to: Option<(String, u16)>) -> Vec<std::net::SocketAddr> {
    use std::net::ToSocketAddrs as _;
    where_to
        .into_iter()
        .flat_map(|(host, port)| {
            (host.as_str(), port)
                .to_socket_addrs()
                .into_iter()
                .flatten()
        })
        .collect()
}

/// Ask one provider a question, and read the indicator **while it is still
/// out** — which is the only moment the sentence in question is on screen.
///
/// The answer and the whole exchange come back beside the line, so a test can
/// hold all three to each other.
fn asked_while_watching(provider: &Provider) -> (String, String) {
    let hosted = Hosted::provider(provider, None);
    let policy = SourcePolicy::Anywhere;
    let mut indicator = Indicator::default();

    let answering = Answering::chosen(hosted.named_source(), &policy).expect("nothing forbids it");
    let asked = Asking::by(&Grantee::named("@mail"), answering, &[], &policy)
        .to_a_provider(
            &question(),
            &hosted,
            &mut indicator,
            noon(),
            &resolved(hosted.where_it_would_connect()),
        )
        .expect("the service answered");

    // Read the line before it comes off. `ended` is what takes it off, and
    // every other test in this crate calls that first and then asserts the
    // indicator is quiet — which is why the sentence itself has never been
    // read.
    let strings = strings();
    let shown: Vec<String> = indicator
        .showing()
        .iter()
        .map(|line| line.said(&strings).text().to_owned())
        .collect();
    assert_eq!(shown.len(), 1, "one question is one line: {shown:?}");
    let line = shown.into_iter().next().unwrap_or_default();

    let answer = asked.ended(&mut indicator);
    assert!(indicator.is_quiet(), "the line stayed up after the answer");
    (line, answer.text().to_owned())
}

/// **A provider that says where it runs is named with the place.**
///
/// The pleasant half, and it still has to be true: the name a person gave the
/// provider when they added it, and the region in the provider's own words.
#[test]
fn a_provider_that_says_where_it_runs_is_named_with_the_place() {
    let (url, server) = serving(AN_ANSWER);
    let provider = Provider::checked("Mistral", &url, Region::Declared("the EU".to_owned()), None)
        .expect("a provider somebody added");

    let (line, answer) = asked_while_watching(&provider);
    let exchange = server.join().expect("the service finished");

    println!("--- what went out on the socket ---\n{exchange}");
    println!("--- what the person read while it was out ---\n{line}");

    assert!(
        line.contains("Mistral"),
        "the line did not say who answered: {line}"
    );
    assert!(
        line.contains("the EU"),
        "the line did not say where they run: {line}"
    );
    assert_eq!(answer, "No, not without written consent.");

    // The question really went out on the socket, so the line is about an
    // egress that happened rather than one that was described.
    assert!(
        exchange.contains("POST /v1/chat/completions"),
        "the question did not go out as a request: {exchange}"
    );
    assert!(
        exchange.contains("may the tenant sublet?"),
        "the question's own words did not go out: {exchange}"
    );
}

/// **A provider that will not say where it runs is shown as unknown, and never
/// as nearby.**
///
/// ADR 0008's sentence, and the one worth measuring. What the line must not do
/// is fill the gap: not with the country the address happens to resolve in, not
/// with a region the name suggests, and not by leaving the place out so the
/// sentence reads like an ordinary one.
#[test]
fn a_provider_that_will_not_say_is_shown_as_unknown_and_never_as_nearby() {
    let (url, server) = serving(AN_ANSWER);
    let provider = Provider::checked("Aurora", &url, Region::Unknown, None)
        .expect("a provider somebody added");

    let (line, answer) = asked_while_watching(&provider);
    let exchange = server.join().expect("the service finished");

    println!("--- what went out on the socket ---\n{exchange}");
    println!("--- what the person read while it was out ---\n{line}");

    assert!(
        line.contains("Aurora"),
        "the line did not say who answered: {line}"
    );
    assert!(
        line.contains("has not said where it runs"),
        "the line did not say the place is unknown: {line}"
    );
    assert_eq!(answer, "No, not without written consent.");

    // And it is not quietly placed. `127.0.0.1` is where this service really
    // is, and a machine that read an address as a region would put this
    // provider next door — which is exactly the assumption ADR 0008 forbids.
    for nearby in [
        "127.0.0.1",
        "localhost",
        "this machine",
        "on this network",
        "the EU",
    ] {
        assert!(
            !line.contains(nearby),
            "the line placed a provider that said nothing: it claimed {nearby:?} in {line}"
        );
    }
}

/// **The two sentences differ only in the place**, so the unknown one cannot be
/// mistaken for the ordinary one at a glance.
///
/// A person reads this while deciding whether to let the question go. If the
/// two lines looked alike, the decision would be made on a difference nobody
/// notices.
#[test]
fn the_unknown_place_does_not_read_like_an_ordinary_line() {
    let (stated_url, stated) = serving(AN_ANSWER);
    let stated_provider = Provider::checked(
        "Mistral",
        &stated_url,
        Region::Declared("the EU".to_owned()),
        None,
    )
    .expect("a provider somebody added");
    let (stated_line, _) = asked_while_watching(&stated_provider);
    stated.join().expect("the service finished");

    let (silent_url, silent) = serving(AN_ANSWER);
    let silent_provider = Provider::checked("Aurora", &silent_url, Region::Unknown, None)
        .expect("a provider somebody added");
    let (silent_line, _) = asked_while_watching(&silent_provider);
    silent.join().expect("the service finished");

    assert_ne!(stated_line, silent_line);
    println!("said where it runs : {stated_line}");
    println!("would not say      : {silent_line}");
}
