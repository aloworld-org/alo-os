//! **Does a person's own question actually reach a provider and come back?**
//!
//! [ADR 0125](../../../docs/decisions/0125-a-command-does-not-report-an-egress-it-asks-a-service-that-can-show-one.md)
//! names the test that decides whether this road works: *a person with a
//! provider in their settings types a question at a shell, and the answer comes
//! back… and with the daemon stopped, the same person is refused in words and
//! nothing leaves.* It says the second half is the one worth writing first, and
//! that half is held elsewhere — `alo-ask`'s own
//! `what_a_person_who_typed_it_sees.rs`, measured as a person on 2026-10-20.
//!
//! **This is the first half, and nothing was holding it.** Measured the same
//! day: `a_question_nobody_is_showing` — the function a person's whole question
//! runs through — had **no tests of its own**, none of this crate's eleven other
//! test files drove it, and the only other caller that reaches it in a test is
//! `alo-egress`'s *a machine that lets nothing leave*, which is a refusal. So
//! every test of this road ended before anything was asked:
//! `alo-agentd`'s handler tests use a keyring holding nothing, so each of them
//! stops at the absent key.
//!
//! # Why this can run in the gate, where the agent's equivalent cannot
//!
//! `alo-agentd`'s `a_question_is_bounded_by_the_kernel.rs` proves the agent's
//! road against a loopback server, and it needs a BPF boundary pinned under
//! `/sys/fs/bpf` — so it cannot run on a machine without one.
//!
//! **A person's road imposes no boundary**, deliberately: ADR 0126 put it
//! beside the agent's road rather than through it, and the function above
//! checks that the place is the provider and asks the organisation's rule about
//! the resolved destination, and then asks. That makes this road *easier* to
//! prove than the agent's, not harder — no boundary, no keyring, no daemon, no
//! network and no money.
//!
//! # Loopback, for the reason the sibling gives
//!
//! `Provider::checked` carried a key over `http://` only to this machine when
//! this was written (`docs/features.md` refused an unencrypted address that is
//! not), so the test that reads a whole answer back is the loopback one. A
//! computer on the person's own network over `http://` is accepted since the
//! owner's decision of 2026-10-30, and
//! `a_question_to_a_computer_on_the_persons_own_network_leaves_and_is_shown.rs`
//! drives this same road to one. What this proves is that
//! **the production path runs and works** — the request really crosses a
//! socket and what the server received is asserted, so it is not a mock
//! agreeing with itself. It proves nothing about a kernel permitting anything,
//! and does not claim to.
//!
//! # One mutation these cannot catch, and why that is right
//!
//! Building the answer from `hosted.named_source()` instead of the source the
//! rule permitted passes all three. That is not a gap: on the answering path
//! the two are **provably equal**, because `the_place_is_this_provider` returns
//! `Miswired::AnotherPlace` when they differ. The guard makes the later line's
//! choice immaterial, so no test can tell the two apart — which is the guard
//! doing its job rather than a test failing to.
//!
//! *Written down because the next person to run a mutation over this file will
//! find it, and the honest answer is shorter than the investigation.*

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, Read as _, Write as _};
use std::net::{SocketAddr, TcpListener};
use std::thread;
use std::time::Duration;

use alo_answering::Answering;
use alo_asking::{Hosted, NotPut, Question, a_question_nobody_is_showing};
use alo_models::{InferenceSource, Provider, Region, Secret, SourcePolicy};

/// What a provider answers, in the shape the reader parses.
const AN_ANSWER: &str =
    r#"{"choices":[{"message":{"content":"No, not without written consent."}}]}"#;

/// Where an answer says it came from.
fn a_provider_in_the_eu() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "Mistral".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// A server on loopback that answers once, and hands back what it was sent.
///
/// **The two callers want different waits and the difference is the point.**
/// A test that expects a request has to wait long enough for a slow machine to
/// deliver one, or it reports *nothing arrived* about a machine under load. A
/// test that expects **nothing** needs only long enough that a bug would have
/// shown: the rule refuses synchronously, so if a socket were going to be
/// opened it would already have been, and waiting longer adds nothing but gate
/// time. Ten seconds and two.
fn serving_for(at_most: Duration) -> (SocketAddr, thread::JoinHandle<Option<String>>) {
    let listener =
        TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).expect("a listener on loopback");
    let at = listener.local_addr().expect("a listener knows where it is");
    let handle = thread::spawn(move || {
        listener
            .set_nonblocking(true)
            .expect("a listener can be made not to wait");
        let until = std::time::Instant::now() + at_most;
        let mut accepted = None;
        while std::time::Instant::now() < until {
            match listener.accept() {
                Ok((stream, _)) => {
                    accepted = Some(stream);
                    break;
                }
                Err(_) => thread::sleep(Duration::from_millis(20)),
            }
        }
        let mut stream = accepted?;
        let mut reader = std::io::BufReader::new(stream.try_clone().ok()?);
        let mut head = String::new();
        let mut length = 0usize;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).ok()? == 0 {
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
            reader.read_exact(&mut body).ok()?;
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\
             Connection: close\r\n\r\n{AN_ANSWER}",
            AN_ANSWER.len()
        );
        stream.write_all(response.as_bytes()).ok()?;
        stream.flush().ok()?;
        Some(head + &String::from_utf8_lossy(&body))
    });
    (at, handle)
}

/// **A person's question reaches a provider and the answer comes back.**
///
/// The half ADR 0125 names and nothing was holding. The whole public road runs:
/// the rule permits the stated source, the place is checked to be this
/// provider, the rule is asked again about the **resolved destination**, and
/// only then is the question put.
///
/// Three assertions, and the third is the one that makes this more than a mock:
/// the answer's text, the source it names, and **what the server actually
/// received** — the question in the body and a key in the headers.
#[test]
fn a_persons_question_reaches_a_provider_and_the_answer_comes_back() {
    // Long, because this one is waiting for a request that must arrive.
    let (at, server) = serving_for(Duration::from_secs(10));
    let provider = Provider::checked(
        "Mistral",
        &format!("http://{at}"),
        Region::Declared("the EU".to_owned()),
        None,
    )
    .expect("a provider at an address of our own");
    let key = Secret::typed("sk-live-0123456789").expect("a key");
    let question = Question::asked("may the tenant sublet?", "mistral-small-latest")
        .expect("a question with a model to put it to");

    let answering = Answering::chosen(a_provider_in_the_eu(), &SourcePolicy::Anywhere)
        .expect("a rule that permits anywhere permits this");

    let answer = a_question_nobody_is_showing(
        answering,
        &[],
        &SourcePolicy::Anywhere,
        &question,
        &Hosted::provider(&provider, Some(&key)),
        &[at],
    )
    .expect("the question was put and answered");

    assert_eq!(answer.text(), "No, not without written consent.");
    assert_eq!(answer.source(), &a_provider_in_the_eu());

    let sent = server
        .join()
        .expect("the server thread finishes")
        .expect("the request really reached the server");
    assert!(sent.contains("may the tenant sublet?"), "{sent}");
    assert!(
        sent.to_ascii_lowercase().contains("authorization:"),
        "the key did not travel with the request: {sent}"
    );
}

/// **And the organisation's rule stops it before anything leaves.**
///
/// The same road, the same reachable server, and a rule that keeps questions on
/// this machine. The assertion worth having is not the refusal — it is that the
/// **server received nothing**, so the rule refused before a socket was opened
/// rather than after.
///
/// This is `alo-egress`'s *a machine that lets nothing leave* asked of the
/// whole road rather than of the policy, and with a server standing by to
/// prove the difference.
#[test]
fn a_rule_that_keeps_questions_here_refuses_before_anything_is_sent() {
    // Short, because this one is waiting to be sure nothing arrives, and the
    // refusal is synchronous: a socket that was going to be opened already was.
    let (at, server) = serving_for(Duration::from_secs(2));
    let provider = Provider::checked(
        "Mistral",
        &format!("http://{at}"),
        Region::Declared("the EU".to_owned()),
        None,
    )
    .expect("a provider at an address of our own");
    let key = Secret::typed("sk-live-0123456789").expect("a key");
    let question = Question::asked("may the tenant sublet?", "mistral-small-latest")
        .expect("a question with a model to put it to");

    // Permitted about the *stated* source, so the refusal below is the second
    // asking — the one about where it is actually going, which is this road's
    // own and is not the one `Answering::chosen` made.
    let answering = Answering::chosen(a_provider_in_the_eu(), &SourcePolicy::Anywhere)
        .expect("a rule that permits anywhere permits this");

    let refused = a_question_nobody_is_showing(
        answering,
        &[],
        &SourcePolicy::ThisMachineOnly,
        &question,
        &Hosted::provider(&provider, Some(&key)),
        &[at],
    )
    .expect_err("a machine that keeps questions here must refuse this");
    assert!(
        matches!(refused, NotPut::HeldBack(_)),
        "refused for the wrong reason: {refused:?}"
    );

    assert_eq!(
        server.join().expect("the server thread finishes"),
        None,
        "the rule refused and something still reached the server"
    );
}

/// **A question permitted for one provider is not put to another**, and nothing
/// is sent.
///
/// `the_place_is_this_provider` is the road's first act, and its own words say
/// why: *two readings of "is this actually a provider, and is it this one" is
/// how one of them one day sends a question to a place nothing decided about.*
///
/// *Added because removing that call passed both tests above.* They permit
/// Mistral and put the question to Mistral, so the check was true either way —
/// the classic shape of a guard that is never exercised. `Miswired::AnotherPlace`
/// was asserted nowhere but in `refusing.rs`, as a property of the variant
/// rather than of the road that produces it.
///
/// **Only the name differs.** The address is the same reachable loopback
/// server, so what is being tested is the name the rule permitted against the
/// name the question is going to — not an unreachable host, which would refuse
/// for a different reason and prove nothing about this one.
#[test]
fn a_question_permitted_for_one_provider_is_not_put_to_another() {
    let (at, server) = serving_for(Duration::from_secs(2));
    // The same address, a different name.
    let somewhere_else = Provider::checked(
        "OpenAI",
        &format!("http://{at}"),
        Region::Declared("the EU".to_owned()),
        None,
    )
    .expect("a provider at an address of our own");
    let key = Secret::typed("sk-live-0123456789").expect("a key");
    let question = Question::asked("may the tenant sublet?", "mistral-small-latest")
        .expect("a question with a model to put it to");

    // The rule permitted Mistral, and nothing has decided about OpenAI.
    let answering = Answering::chosen(a_provider_in_the_eu(), &SourcePolicy::Anywhere)
        .expect("a rule that permits anywhere permits this");

    let refused = a_question_nobody_is_showing(
        answering,
        &[],
        &SourcePolicy::Anywhere,
        &question,
        &Hosted::provider(&somewhere_else, Some(&key)),
        &[at],
    )
    .expect_err("a question permitted for Mistral must not be put to OpenAI");
    assert!(
        matches!(refused, NotPut::Miswired(_)),
        "refused for the wrong reason: {refused:?}"
    );

    assert_eq!(
        server.join().expect("the server thread finishes"),
        None,
        "the question went to a provider nothing decided about"
    );
}
