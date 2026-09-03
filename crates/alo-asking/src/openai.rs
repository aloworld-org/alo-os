//! The only file that knows what an OpenAI-compatible API looks like.
//!
//! `alo_models::ollama` is the only file that knows Ollama exists (ADR 0006) and
//! `alo_models::trying` is the only one that knows how a provider lists what it
//! offers. This is the third of that kind and the one that matters most: it is
//! where a question actually goes.
//!
//! What it knows is one thing — the OpenAI-compatible convention:
//! `POST {endpoint}/v1/chat/completions`, the key as a bearer token, one message
//! in, one message out. That is what `alo-workplace`'s `AiConfig` has spoken
//! since 2025, what every provider somebody is likely to already pay for offers,
//! and what vLLM, llama.cpp's server and LM Studio serve on this machine. A
//! service that answers some other way is reported as one this machine could not
//! use, which is true, rather than as a bad key, which would not be.
//!
//! # Two callers, and the difference between them is not in this file
//!
//! [`put`] is `pub(crate)` and is reached from [`crate::hosted`], which is
//! reached only by a door holding an `alo_egress::Departing`, and from
//! [`crate::served`], which is reached only by a door carrying a question to an
//! address `alo_models::Provider::source` calls this machine. **Neither of those
//! guarantees is made here**, and that is deliberate: this file knows a
//! protocol, and which addresses may be spoken to under which conditions is law
//! 1's question rather than the wire's. A third caller would need its own answer
//! to it, and this file is not where one could be assumed.
//!
//! The one thing that *is* different per caller is how long to wait, which is a
//! parameter rather than a constant here: this machine waits longer for itself
//! than for anybody else, for ADR 0007's reason, and the callers say so.
//!
//! # Three decisions carry more weight than the request does
//!
//! **A redirect is refused, never followed.** The address that was decided about
//! is the address that gets reached; following a redirect would open a
//! connection to a host nobody decided about, carrying a credential *and*
//! somebody's question. `alo_models::trying` made the same call about a test,
//! and this is the same refusal where the stakes are the person's own work. It
//! matters twice over on the local door: a service on loopback that answered
//! with a redirect could otherwise carry a question off the machine with the
//! indicator quiet.
//!
//! **Nothing of the person's leaves except the question.** The body is the
//! question, the model and `stream: false` — no identifier, no machine name, no
//! previous turn, no "system prompt" this crate composed. What is in the request
//! is what a test reads off the socket, which is why the fixture yields the body
//! as well as the head.
//!
//! **The answer is read whole, and bounded.** Streaming would put a person in
//! front of text arriving before anything has decided whether it is usable, and
//! nothing in this repository has yet decided what a half-arrived answer is. So
//! `stream` is false and stays false until something says otherwise; an answer
//! longer than [`MOST_OF_AN_ANSWER`] is one this machine will not show, and it
//! is reported as unusable rather than truncated into something the model did
//! not say.

use std::time::Duration;

use alo_answering::WentWrong;
use alo_models::Secret;
use serde::{Deserialize, Serialize};

use crate::question::Question;

/// The most of an answer that is read.
///
/// No model writes a megabyte in one answer, so the bound costs nothing real —
/// and an answer past it is not truncated, because half of what a model said is
/// not what it said.
const MOST_OF_AN_ANSWER: u64 = 1_000_000;

/// One question, in the shape every OpenAI-compatible service speaks.
#[derive(Serialize)]
struct Sent<'a> {
    /// The model to answer it.
    model: &'a str,
    /// The question, as the one message in this conversation.
    messages: [Message<'a>; 1],
    /// Whole answers only — this module's third decision.
    stream: bool,
}

/// One message in that shape.
#[derive(Serialize)]
struct Message<'a> {
    /// Who is speaking. Always the person.
    role: &'a str,
    /// What they said.
    content: &'a str,
}

/// What a service answers with.
///
/// `choices` is `#[serde(default)]` so that a reply which is valid JSON and not
/// an answer is reported as unusable rather than as a parse failure: the two
/// reach a person as one sentence, and the shorter path to it is fewer branches.
#[derive(Deserialize)]
struct Spoke {
    /// The answers offered. One is asked for and the first is taken.
    #[serde(default)]
    choices: Vec<Choice>,
}

/// One answer in that reply.
#[derive(Deserialize)]
struct Choice {
    /// What the model wrote.
    message: Wrote,
}

/// The message a model wrote.
#[derive(Deserialize)]
struct Wrote {
    /// Its text.
    #[serde(default)]
    content: String,
}

/// Put the question to this address, and read what comes back.
///
/// `waiting` is how long this machine waits for an answer, which is the caller's
/// to say.
///
/// # Errors
/// [`WentWrong`], which is `alo-answering`'s closed list rather than one of this
/// crate's own — what went wrong where a question was put is a thing a person
/// reads in one voice, whether the place was a provider, a paired machine or
/// this one.
pub(crate) fn put(
    endpoint: &str,
    key: Option<&Secret>,
    question: &Question,
    waiting: Duration,
) -> Result<String, WentWrong> {
    let request = ureq::post(answers_url(endpoint))
        .config()
        .timeout_global(Some(waiting))
        // Refused rather than followed — this module's first decision.
        .max_redirects(0)
        // Every answer comes back to be read here. A status this file has an
        // opinion about must not be turned into a transport error by the
        // client, because "that key was not accepted" and "nothing answered"
        // are different things to tell somebody.
        .http_status_as_error(false)
        .build();
    // The key is handed the request rather than the other way round: it cannot
    // be read out of `alo-models`, and this crate never holds it as text it
    // could put anywhere else.
    let request = match key {
        Some(key) => key.carried_by(request),
        None => request,
    };

    let sent = Sent {
        model: question.of(),
        messages: [Message {
            role: "user",
            content: question.text(),
        }],
        stream: false,
    };
    let mut response = request.send_json(&sent).map_err(what_went_wrong)?;
    match response.status().as_u16() {
        200 => {}
        300..=399 => return Err(WentWrong::SentSomewhereElse),
        401 | 403 => return Err(WentWrong::KeyNotAccepted),
        // The convention answers 404 for a model it does not offer, and 405 for
        // an address that is not the API at all. Both reach a person as "what
        // was to answer this was not there", which is what both of them mean to
        // somebody who did not type either.
        404 | 405 => return Err(WentWrong::NoModelThere),
        // A refusal of the request itself. It is not the person's to fix and it
        // is not the service having a bad day, so it is neither sentence:
        // something answered, and not with an answer.
        400 | 422 => return Err(WentWrong::NothingUsable),
        other => return Err(WentWrong::HavingTrouble(other)),
    }

    let body = response
        .body_mut()
        .with_config()
        .limit(MOST_OF_AN_ANSWER)
        .read_to_string()
        .map_err(|_| WentWrong::NothingUsable)?;
    let spoke: Spoke = serde_json::from_str(&body).map_err(|_| WentWrong::NothingUsable)?;
    let said = spoke
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .unwrap_or_default();
    if said.trim().is_empty() {
        // A reply with no answer in it is not an empty answer to show somebody:
        // they asked something and nothing came back.
        return Err(WentWrong::NothingUsable);
    }
    Ok(said)
}

/// What a failure on the wire is, said as the thing a person is told.
///
/// A function of its own so that the one outcome no test can produce without
/// waiting the whole timeout for it — a service that accepts the connection and
/// then says nothing — is still checked rather than assumed.
fn what_went_wrong(error: ureq::Error) -> WentWrong {
    match error {
        ureq::Error::Timeout(_) => WentWrong::TookTooLong,
        // A service that answers a `POST` with 307 or 308 is telling this
        // machine to send the question somewhere else. ureq refuses to carry a
        // body across a redirect, and so does alo OS.
        ureq::Error::RedirectFailed => WentWrong::SentSomewhereElse,
        _ => WentWrong::NothingAnswered,
    }
}

/// Where a service answers questions.
///
/// Addresses are documented both ways — Mistral's ends `/v1`, a local runtime's
/// does not — and appending a second `/v1` to the first would come back 404,
/// which a person would read as *my address is wrong* when it is right.
/// `alo_models::trying` makes the same allowance for the same reason and
/// `docs/quirks.md` records it.
fn answers_url(endpoint: &str) -> String {
    let base = endpoint.trim_end_matches('/');
    if base.ends_with("/v1") {
        format!("{base}/chat/completions")
    } else {
        format!("{base}/v1/chat/completions")
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{serving, serving_with};

    /// One answer, in the shape every OpenAI-compatible service replies with.
    const AN_ANSWER: &str = r#"{"id":"cmpl-1","object":"chat.completion","model":"mistral-small-latest","choices":[{"index":0,"message":{"role":"assistant","content":"The tenant may not sublet."},"finish_reason":"stop"}]}"#;

    /// How long these tests wait. Nothing here reaches the bound.
    const A_MOMENT: Duration = Duration::from_secs(20);

    fn question() -> Question {
        Question::asked("may the tenant sublet?", "mistral-small-latest").unwrap()
    }

    /// The happy path, and three promises in one test: the question goes to the
    /// address the person typed, the answer comes back as the model wrote it,
    /// and **nothing of the person's leaves except the question** — the body is
    /// exactly the model, the message and `stream: false`.
    #[test]
    fn a_question_goes_out_whole_and_alone_and_the_answer_comes_back_as_it_was_written() {
        let (url, server) = serving(AN_ANSWER, 200);
        let key = Secret::typed("sk-live-0123456789").unwrap();
        let answer = put(&url, Some(&key), &question(), A_MOMENT);
        let request = server.join().unwrap();

        assert_eq!(answer.unwrap(), "The tenant may not sublet.");
        assert!(
            request.starts_with("POST /v1/chat/completions "),
            "{request}"
        );
        assert!(
            request
                .to_lowercase()
                .contains("authorization: bearer sk-live-0123456789\r\n"),
            "{request}"
        );
        // The whole of what was sent, and nothing beside it: the assertion is
        // on the parsed body rather than on the text, so a field added here
        // later fails this test rather than passing a substring check.
        let body = request.split("\r\n\r\n").nth(1).unwrap_or_default();
        let sent: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(
            sent,
            serde_json::json!({
                "model": "mistral-small-latest",
                "messages": [{"role": "user", "content": "may the tenant sublet?"}],
                "stream": false,
            }),
            "{request}"
        );
    }

    /// A service given no key is sent no authorisation at all: a header nobody
    /// asked for is one some services refuse outright.
    #[test]
    fn a_service_given_no_key_is_sent_no_authorisation_at_all() {
        let (url, server) = serving(AN_ANSWER, 200);
        let answer = put(&url, None, &question(), A_MOMENT);
        let request = server.join().unwrap();
        assert!(answer.is_ok(), "{answer:?}");
        assert!(
            !request.to_lowercase().contains("authorization"),
            "{request}"
        );
    }

    /// A key that was not accepted is the one failure a person can act on, and
    /// neither the key nor the service's own words travel into what they read.
    #[test]
    fn a_key_the_service_refuses_is_said_without_quoting_it_or_them() {
        let (url, server) = serving(r#"{"message":"Unauthorized","request_id":"abc"}"#, 401);
        let key = Secret::typed("sk-live-0123456789").unwrap();
        let went_wrong = put(&url, Some(&key), &question(), A_MOMENT).unwrap_err();
        server.join().unwrap();
        assert_eq!(went_wrong, WentWrong::KeyNotAccepted);

        let said = went_wrong.word().says();
        assert!(!said.contains("sk-live"), "{said}");
        assert!(!said.contains("request_id"), "{said}");
    }

    /// **The address that was decided about is the address that gets reached.**
    /// The redirect points at a port nothing is listening on: following it would
    /// come back as nothing answering, so the two outcomes cannot be confused.
    #[test]
    fn a_redirect_is_refused_rather_than_followed_with_a_question_in_hand() {
        let (url, server) = serving_with(
            "{}",
            302,
            "Location: http://127.0.0.1:1/v1/chat/completions\r\n",
        );
        let key = Secret::typed("sk-live-0123456789").unwrap();
        let went_wrong = put(&url, Some(&key), &question(), A_MOMENT).unwrap_err();
        server.join().unwrap();
        assert_eq!(went_wrong, WentWrong::SentSomewhereElse);
    }

    /// The statuses that mean something in particular, and the one that means
    /// the service is having a bad day.
    #[test]
    fn each_status_reaches_a_person_as_the_thing_it_actually_means() {
        for (body, status, expected) in [
            (r#"{"error":"no such model"}"#, 404, WentWrong::NoModelThere),
            (r#"{"error":"not allowed"}"#, 405, WentWrong::NoModelThere),
            (r#"{"error":"bad request"}"#, 400, WentWrong::NothingUsable),
            (
                r#"{"error":"unprocessable"}"#,
                422,
                WentWrong::NothingUsable,
            ),
            (
                r#"{"error":"slow down"}"#,
                429,
                WentWrong::HavingTrouble(429),
            ),
            (
                r#"{"error":"upstream capacity exceeded"}"#,
                503,
                WentWrong::HavingTrouble(503),
            ),
        ] {
            let (url, server) = serving(body, status);
            let went_wrong = put(&url, None, &question(), A_MOMENT).unwrap_err();
            server.join().unwrap();
            assert_eq!(went_wrong, expected, "{status}");
        }
    }

    /// Something answered, and it was not an answer. A model that replied with
    /// nothing is the same thing to the person who asked: they asked, and
    /// nothing came back.
    #[test]
    fn an_address_that_answers_without_answering_is_reported_as_nothing_usable() {
        for body in [
            "<!doctype html><title>Welcome</title>",
            r#"{"hello":"world"}"#,
            r#"{"choices":[]}"#,
            r#"{"choices":[{"index":0,"message":{"role":"assistant","content":"   "}}]}"#,
        ] {
            let (url, server) = serving(body, 200);
            let went_wrong = put(&url, None, &question(), A_MOMENT).unwrap_err();
            server.join().unwrap();
            assert_eq!(went_wrong, WentWrong::NothingUsable, "{body}");
        }
    }

    /// Nothing listening at all, which is the ordinary way an address turns out
    /// to be wrong.
    #[test]
    fn nothing_listening_is_nothing_answered() {
        assert_eq!(
            put("http://127.0.0.1:1", None, &question(), A_MOMENT).unwrap_err(),
            WentWrong::NothingAnswered
        );
    }

    /// **The outcome no stub can produce in a test worth running.** A service
    /// that accepts a connection and then says nothing for two minutes is a
    /// different sentence from one that was never there, so the mapping is
    /// checked directly rather than by waiting for it.
    #[test]
    fn a_service_that_goes_quiet_is_told_apart_from_one_that_was_never_there() {
        assert_eq!(
            what_went_wrong(ureq::Error::Timeout(ureq::Timeout::Global)),
            WentWrong::TookTooLong
        );
        assert_eq!(
            what_went_wrong(ureq::Error::RedirectFailed),
            WentWrong::SentSomewhereElse
        );
        assert_eq!(
            what_went_wrong(ureq::Error::HostNotFound),
            WentWrong::NothingAnswered
        );
    }

    /// Addresses are documented both ways. A second `/v1` would come back 404
    /// and be read as a wrong address when the address was right.
    #[test]
    fn an_address_that_already_ends_at_the_api_is_not_given_a_second_v1() {
        assert_eq!(
            answers_url("https://api.mistral.ai/v1"),
            "https://api.mistral.ai/v1/chat/completions"
        );
        assert_eq!(
            answers_url("https://api.mistral.ai/v1/"),
            "https://api.mistral.ai/v1/chat/completions"
        );
        assert_eq!(
            answers_url("https://api.example.com"),
            "https://api.example.com/v1/chat/completions"
        );
        assert_eq!(
            answers_url("http://127.0.0.1:8000"),
            "http://127.0.0.1:8000/v1/chat/completions"
        );
    }
}
