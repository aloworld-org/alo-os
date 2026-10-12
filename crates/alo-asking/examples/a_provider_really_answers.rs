//! **Does the provider road reach a real provider and come back?** Driven
//! against one, with a real key, over the real network.
//!
//! The other half of the question the local road answered. Every test in this
//! crate drives `to_a_provider` against a loopback server this repository
//! wrote, which proves the shape of the request and cannot prove that a
//! provider accepts it.
//!
//! # What it proves, and the part that matters most
//!
//! **That egress is visible while it happens and recorded afterwards.** Law 1
//! is not *a question reaches a provider* — it is *every network egress an
//! agent causes is visible at the moment it happens and afterwards in a
//! record*. So this asserts three things in order:
//!
//! 1. the indicator is quiet before anything is sent;
//! 2. the question is answered, which means something really left;
//! 3. a [`Departing`](alo_egress::Departing) came back describing what left,
//!    which is what a record is written from.
//!
//! The local road's run asserted the indicator stayed quiet. This asserts the
//! opposite, and the pair is the claim: **the indicator distinguishes the two
//! roads.** Either one alone would pass against an indicator that never says
//! anything, or against one that always does.
//!
//! # What it still does not prove
//!
//! **That no other traffic occurred.** A quiet indicator means nothing was
//! reported through our own indicator, which is not a measurement at the
//! network boundary — the owner's correction of 2026-10-11, and law 1 asks for
//! the measurement rather than the promise. Showing that a working day
//! produces zero inference egress needs a capture at the boundary and is its
//! own task.
//!
//! **That `alo-agentd` is wired.** This drives `alo_asking`'s provider door,
//! which is the road. The production caller for that road is `alo-agentd`,
//! and exercising it needs the daemon, a grant and a bounded turn.
//!
//! # Run it
//!
//! ```text
//! ALO_PROVIDER_KEY=sk-… cargo run -p alo-asking --example a_provider_really_answers
//! ```
//!
//! `ALO_PROVIDER` and `ALO_PROVIDER_MODEL` override the endpoint and the
//! model. **The key is read from the environment and never written down**: not
//! into a file, not into the output, and not into any record this writes.
//! `Secret` holds it and `alo-asking`'s own test
//! `nothing_here_keeps_the_key` is what holds that true in the library.

use std::process::ExitCode;
use std::time::SystemTime;

use alo_answering::Answering;
use alo_asking::{Asking, Hosted, Question};
use alo_capability::Grantee;
use alo_egress::Indicator;
use alo_models::{Provider, Region, Secret, SourcePolicy};

fn main() -> ExitCode {
    let Ok(key) = std::env::var("ALO_PROVIDER_KEY") else {
        eprintln!("no ALO_PROVIDER_KEY in the environment, so there is nothing to ask with");
        eprintln!("  this example never reads a key from a file, by design");
        return ExitCode::FAILURE;
    };
    let endpoint =
        std::env::var("ALO_PROVIDER").unwrap_or_else(|_| "https://api.openai.com".to_owned());
    let of = std::env::var("ALO_PROVIDER_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_owned());
    let asked = std::env::var("ALO_ASK")
        .unwrap_or_else(|_| "In one short sentence: what is an operating system?".to_owned());

    // **The region is declared, not detected.** `Region::Declared` is what a
    // person or an organisation says about where a provider answers from;
    // nothing here can verify it, and ADR 0003's rule is that the mechanism is
    // ours and the rule is theirs.
    let provider = match Provider::checked(
        "the owner's test provider",
        &endpoint,
        Region::Declared("not stated for a test key".to_owned()),
        None,
    ) {
        Ok(provider) => provider,
        Err(why) => {
            eprintln!("that is not a provider this machine can use: {why:?}");
            return ExitCode::FAILURE;
        }
    };
    let secret = match Secret::typed(&key) {
        Ok(secret) => secret,
        Err(why) => {
            eprintln!("that is not a key this machine can send: {why:?}");
            return ExitCode::FAILURE;
        }
    };
    let hosted = Hosted::provider(&provider, Some(&secret));

    // Where it may go, decided before anything is asked.
    let policy = SourcePolicy::Anywhere;
    let answering = match Answering::chosen(hosted.named_source(), &policy) {
        Ok(answering) => answering,
        Err(refusal) => {
            eprintln!("the rule on this machine refuses that provider: {refusal:?}");
            return ExitCode::FAILURE;
        }
    };
    let question = match Question::asked(&asked, &of) {
        Ok(question) => question,
        Err(why) => {
            eprintln!("that is not a question this machine can put: {why:?}");
            return ExitCode::FAILURE;
        }
    };

    // **Resolved by the caller, outside any boundary** (ADR 0020), so the
    // request itself resolves nothing. These are the only addresses it may
    // reach.
    let Some((host, port)) = hosted.where_it_would_connect() else {
        eprintln!("that provider names no address to connect to");
        return ExitCode::FAILURE;
    };
    let to: Vec<std::net::SocketAddr> =
        match std::net::ToSocketAddrs::to_socket_addrs(&(host.as_str(), port)) {
            Ok(found) => found.collect(),
            Err(why) => {
                eprintln!("{host}:{port} could not be resolved: {why}");
                return ExitCode::FAILURE;
            }
        };
    println!("asking {of} at {host}:{port}, resolved to {} address(es)", to.len());

    let mut indicator = Indicator::default();
    // **Quiet first**, so the assertion after the ask is about this ask.
    if !indicator.is_quiet() {
        eprintln!("REFUSED: the indicator was already showing something before anything was sent");
        return ExitCode::FAILURE;
    }
    println!("the indicator is quiet before anything leaves: true");

    let who = Grantee::named("@the-owner");
    let asking = Asking::by(&who, answering, &[], &policy);
    let asked_now = match asking.to_a_provider(&question, &hosted, &mut indicator, SystemTime::now(), &to) {
        Ok(asked_now) => asked_now,
        Err(not_asked) => {
            eprintln!("it did not answer: {not_asked:?}");
            return ExitCode::FAILURE;
        }
    };

    // **What left, which is what a record is written from.** Printed without
    // the key: `Departing` describes the egress, never the credential.
    let departing = asked_now.departing();
    println!("\nwhat left, for the record: {departing:?}");

    let showing = indicator.showing().len();
    println!("the indicator showed {showing} thing(s) while it was in flight");

    let answer = asked_now.ended(&mut indicator);
    println!("\n--- the answer ---\n{}\n------------------", answer.text());
    println!("it came from: {:?}, model {}", answer.source(), answer.model());
    println!("the indicator is quiet again afterwards: {}", indicator.is_quiet());

    if showing == 0 {
        eprintln!(
            "REFUSED: a question went to a provider and the indicator showed nothing - law 1 \
             asks for egress to be visible at the moment it happens"
        );
        return ExitCode::FAILURE;
    }
    if answer.text().trim().is_empty() {
        eprintln!("REFUSED: it answered with nothing, which is not an answer");
        return ExitCode::FAILURE;
    }
    println!("\nthe provider road works, and the egress was shown while it happened.");
    ExitCode::SUCCESS
}
