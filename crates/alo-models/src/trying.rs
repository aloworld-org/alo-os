//! Reaching a provider to find out whether it works, before it is saved.
//!
//! `docs/features.md` promises this at v0.5: *test a provider before saving it,
//! so a mistyped key is found now rather than in the middle of a question.* A
//! key typed with a character missing is otherwise discovered days later, in
//! the middle of somebody's work, as a question that failed for no stated
//! reason — and the person who would have to guess is the one who typed it.
//!
//! **This is the only file that knows what a provider's API looks like**, in
//! the same way [`ollama`](crate::ollama) is the only file that knows Ollama
//! exists. What it knows is one thing: the OpenAI-compatible convention —
//! `GET {endpoint}/v1/models`, with the key as a bearer token — which is what
//! `alo-workplace`'s `AiConfig` has spoken since 2025 and what every provider
//! somebody is likely to already pay for offers. A provider that answers some
//! other way is reported as one this system cannot use
//! ([`NotTried::NotUnderstood`]), which is true, rather than as a bad key,
//! which would not be.
//!
//! Three decisions carry more weight than the request does.
//!
//! **The policy is asked first, and there is no way to skip it.** [`Trying`]
//! has one method and it takes a [`SourcePolicy`], so an organisation that has
//! said questions stay in the building does not have a machine that reaches out
//! to a provider anyway to see whether the key works. A refused test sends
//! nothing at all — not a connection, not a DNS lookup — and the refusal is the
//! policy's own words rather than a second explanation.
//!
//! **A redirect is refused, never followed.** The address the policy answered
//! about is the address that gets reached. Following a redirect would open a
//! connection to a host nobody decided about, which is precisely the shape law
//! 1 exists to prevent — and it would do so while carrying a credential.
//!
//! **Nothing of the person's leaves.** The test is a `GET` with no body: no
//! question, no document, no sample prompt to "check the model answers". What
//! leaves this machine is that somebody with this key asked this provider what
//! it offers, and a test that sent more than that would be a feature nobody
//! asked for on the one path built for people who are careful about egress.
//!
//! **What this is not.** A person pressing *Test* in Settings is not an agent
//! doing something, so there is no verb, no grant and no proposal here — the
//! same reasoning as `alo-shortcuts` and `alo-appearance`. It is also why this
//! egress is not on the indicator: the indicator answers *what is my machine
//! sending that I did not ask for*, and the answer to a button somebody has
//! just pressed, on the screen they pressed it, is that they asked for it. Any
//! egress an **agent** causes — a question put to this provider once it is
//! saved — goes through `alo-egress` and is shown, as it does today.

use std::time::Duration;

use serde::Deserialize;

use crate::provider::Provider;
use crate::secret::Secret;
use crate::source::SourcePolicy;
use crate::tried::{NotTried, Tried};

/// How long to wait while somebody watches a dialogue. Long enough for a slow
/// provider on a slow connection; short enough that a settings panel which has
/// stopped answering says so instead of appearing to have frozen.
const WHILE_SOMEBODY_WAITS: Duration = Duration::from_secs(10);

/// The most of an answer that is read. A model list is a few kilobytes; a
/// megabyte of it is either a mistake or somebody's idea of one, and either
/// way it is not going to be shown.
const MOST_OF_AN_ANSWER: u64 = 1_000_000;

/// A provider's model list, in the OpenAI-compatible shape.
///
/// `data` is deliberately **not** `#[serde(default)]`, unlike the runtime's
/// wire shapes: an address that answers without one is not a provider whose
/// answers this system could use later, and saying so now is the whole point.
#[derive(Deserialize)]
struct Listed {
    /// One entry per model offered.
    data: Vec<Offered>,
}

/// One model, as a provider names it.
#[derive(Deserialize)]
struct Offered {
    /// The name to ask for. Absent on a malformed entry, which is left out
    /// rather than failing the whole list.
    #[serde(default)]
    id: String,
}

/// A provider about to be reached, with the key as it was typed.
///
/// Borrowed rather than owned, for the length of one call: neither the provider
/// nor the key is kept anywhere by this.
#[derive(Debug)]
pub struct Trying<'a> {
    /// What is being tested.
    provider: &'a Provider,
    /// The key, when the provider needs one.
    key: Option<&'a Secret>,
}

impl<'a> Trying<'a> {
    /// This provider, with this key — which is [`None`] for a provider that
    /// needs none, such as a runtime on this machine.
    #[must_use]
    pub fn provider(provider: &'a Provider, key: Option<&'a Secret>) -> Self {
        Self { provider, key }
    }

    /// Ask this machine's policy, and then — only then — ask the provider.
    ///
    /// # Errors
    /// [`NotTried`], saying what to do. [`NotTried::Forbidden`] is the one that
    /// happens before anything is sent.
    pub fn under(&self, policy: &SourcePolicy) -> Result<Tried, NotTried> {
        // The policy first, and nothing on the wire until it has answered. A
        // machine set to keep questions in the building must not reach a
        // provider to find out whether its key works: the reaching is the thing
        // the policy forbids, and the key working would not make it permitted.
        if let Some(refusal) = policy.refusal(&self.provider.source()) {
            return Err(NotTried::Forbidden(refusal));
        }

        let request = ureq::get(models_url(&self.provider.endpoint))
            .config()
            .timeout_global(Some(WHILE_SOMEBODY_WAITS))
            // Refused rather than followed — see this module's second decision.
            .max_redirects(0)
            // Every answer comes back to be read here. A status this file has
            // an opinion about must not be turned into a transport error by the
            // client, because "that key was not accepted" and "nothing answered"
            // are different things to tell somebody.
            .http_status_as_error(false)
            .build();
        let request = match self.key {
            Some(key) => request.header("authorization", key.bearer()),
            None => request,
        };

        let mut response = request.call().map_err(|_| NotTried::Unreachable)?;
        match response.status().as_u16() {
            200 => {}
            300..=399 => return Err(NotTried::Redirected),
            401 | 403 => {
                return Err(if self.key.is_some() {
                    NotTried::KeyNotAccepted
                } else {
                    NotTried::NeedsAKey
                });
            }
            404 | 405 => return Err(NotTried::NotUnderstood),
            other => return Err(NotTried::NotWell(other)),
        }

        let body = response
            .body_mut()
            .with_config()
            .limit(MOST_OF_AN_ANSWER)
            .read_to_string()
            .map_err(|_| NotTried::NotUnderstood)?;
        let listed: Listed = serde_json::from_str(&body).map_err(|_| NotTried::NotUnderstood)?;
        Ok(Tried::of(listed.data.into_iter().map(|m| m.id)))
    }
}

/// Where a provider lists what it offers.
///
/// Providers are documented both ways — Mistral's address ends `/v1`, a local
/// runtime's does not — and appending a second `/v1` to the first would fail
/// with a 404 that a person would read as *my address is wrong* when it is
/// right. `docs/quirks.md` records it.
fn models_url(endpoint: &str) -> String {
    let base = endpoint.trim_end_matches('/');
    if base.ends_with("/v1") {
        format!("{base}/models")
    } else {
        format!("{base}/v1/models")
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::source::Region;
    use crate::testing::{serving, serving_with};

    /// Two models, in the shape every OpenAI-compatible provider answers with.
    const A_MODEL_LIST: &str = r#"{"object":"list","data":[{"id":"mistral-small-latest","object":"model"},{"id":"mistral-large-latest","object":"model"}]}"#;

    fn hosted(endpoint: &str) -> Provider {
        Provider::checked(
            "Mistral",
            endpoint,
            Region::Declared("the EU".to_owned()),
            None,
        )
        .unwrap()
    }

    fn key() -> Secret {
        Secret::typed("sk-live-0123456789").unwrap()
    }

    /// The happy path, and three promises in one test: the key travels as a
    /// bearer token to the address the person typed, the models come back as
    /// the provider spells them, and **nothing of the person's leaves** — the
    /// request is a `GET` and carries no body at all.
    #[test]
    fn a_working_provider_answers_with_what_it_offers_and_is_sent_nothing_of_ours() {
        let (url, server) = serving(A_MODEL_LIST, 200);
        let provider = hosted(&url);
        let key = key();
        let tried = Trying::provider(&provider, Some(&key)).under(&SourcePolicy::Anywhere);
        let request = server.join().unwrap();

        let tried = tried.unwrap();
        assert_eq!(
            tried.models(),
            ["mistral-small-latest", "mistral-large-latest"]
        );
        assert!(request.starts_with("GET /v1/models "), "{request}");
        assert!(
            request
                .to_lowercase()
                .contains("authorization: bearer sk-live-0123456789\r\n"),
            "{request}"
        );
        // No body, and nothing announcing one: a test sends what it needs to
        // ask "what do you offer", and not a syllable of anybody's work.
        assert!(
            !request.to_lowercase().contains("content-length"),
            "{request}"
        );
        assert!(request.ends_with("\r\n\r\n"), "{request}");
    }

    /// The whole reason the feature exists: the mistyped key is found while the
    /// person is still looking at the field they typed it into.
    #[test]
    fn a_key_the_provider_does_not_accept_is_found_now_and_said_plainly() {
        let (url, server) = serving(r#"{"message":"Unauthorized","request_id":"abc"}"#, 401);
        let provider = hosted(&url);
        let key = key();
        let error = Trying::provider(&provider, Some(&key))
            .under(&SourcePolicy::Anywhere)
            .unwrap_err();
        server.join().unwrap();

        assert_eq!(error, NotTried::KeyNotAccepted);
        // Neither the key nor the provider's own words travel into ours.
        assert!(!error.to_string().contains("sk-live"), "{error}");
        assert!(!error.to_string().contains("request_id"), "{error}");
    }

    /// The same status means something different when no key was given, and
    /// telling somebody their key was rejected when they never typed one is how
    /// they spend an hour checking a key that does not exist.
    #[test]
    fn a_provider_that_wants_a_key_and_was_given_none_says_which_of_the_two_it_is() {
        let (url, server) = serving(r#"{"message":"Unauthorized"}"#, 401);
        let provider = hosted(&url);
        let error = Trying::provider(&provider, None)
            .under(&SourcePolicy::Anywhere)
            .unwrap_err();
        server.join().unwrap();
        assert_eq!(error, NotTried::NeedsAKey);
    }

    /// **The refusal that matters most.** A machine an organisation has set to
    /// keep questions in the building does not reach out to a provider to find
    /// out whether a key works. Nothing is sent — the address needs no name
    /// lookup and nothing is listening on it, so a leak would come back
    /// `Unreachable` and this test would say so rather than hanging or, worse,
    /// quietly succeeding.
    #[test]
    fn a_provider_the_policy_forbids_is_never_reached_at_all() {
        // Not `127.0.0.1`: that address *is* this machine as far as
        // `Provider::source` is concerned, and a policy has nothing to forbid
        // about a runtime running here. This one is a hosted provider to every
        // question the policy asks, and a refused connection to every question
        // the network asks.
        let provider = hosted("https://127.0.0.2:1");
        let key = key();
        for policy in [
            SourcePolicy::InTheBuilding,
            SourcePolicy::ThisMachineOnly,
            SourcePolicy::InRegion("Switzerland".to_owned()),
        ] {
            let error = Trying::provider(&provider, Some(&key))
                .under(&policy)
                .unwrap_err();
            assert!(
                matches!(&error, NotTried::Forbidden(said) if said == &policy
                    .refusal(&provider.source())
                    .unwrap_or_default()),
                "{error:?} under {policy:?}"
            );
        }
    }

    /// A runtime on this machine is a provider like any other, and testing one
    /// is permitted under the strictest policy there is — because nothing
    /// leaves the machine to do it.
    #[test]
    fn a_provider_on_this_machine_can_be_tested_even_when_nothing_may_leave() {
        let (url, server) = serving(A_MODEL_LIST, 200);
        // The stub listens on loopback, which is what makes this a local
        // provider rather than a hosted one.
        let provider = Provider::checked("Local", &url, Region::Unknown, None).unwrap();
        let tried = Trying::provider(&provider, None).under(&SourcePolicy::ThisMachineOnly);
        server.join().unwrap();
        assert!(tried.unwrap().is_all());
    }

    /// The address the policy answered about is the address that gets reached.
    /// The redirect points at a port nothing is listening on: following it
    /// would come back `Unreachable`, so the two outcomes cannot be confused.
    #[test]
    fn a_redirect_is_refused_rather_than_followed_to_an_address_nobody_agreed_to() {
        let (url, server) = serving_with("{}", 302, "Location: http://127.0.0.1:1/v1/models\r\n");
        let provider = hosted(&url);
        let key = key();
        let error = Trying::provider(&provider, Some(&key))
            .under(&SourcePolicy::Anywhere)
            .unwrap_err();
        server.join().unwrap();
        assert_eq!(error, NotTried::Redirected);
    }

    /// Something answered, and it was not a provider. Saying "bad key" here
    /// would send somebody to check a key that is fine.
    #[test]
    fn an_address_that_is_not_a_provider_is_not_reported_as_a_bad_key() {
        for (body, status) in [
            (r#"{"hello":"world"}"#, 200),
            ("<!doctype html><title>Welcome</title>", 200),
            (r#"{"error":"not found"}"#, 404),
        ] {
            let (url, server) = serving(body, status);
            let provider = hosted(&url);
            let key = key();
            let error = Trying::provider(&provider, Some(&key))
                .under(&SourcePolicy::Anywhere)
                .unwrap_err();
            server.join().unwrap();
            assert_eq!(error, NotTried::NotUnderstood, "{body}");
        }
    }

    /// A provider having a bad day is not a person having typed something
    /// wrong, and the difference is worth saying out loud.
    #[test]
    fn a_provider_in_trouble_says_so_without_repeating_its_own_words() {
        let (url, server) = serving(r#"{"error":"upstream capacity exceeded"}"#, 503);
        let provider = hosted(&url);
        let key = key();
        let error = Trying::provider(&provider, Some(&key))
            .under(&SourcePolicy::Anywhere)
            .unwrap_err();
        server.join().unwrap();
        assert_eq!(error, NotTried::NotWell(503));
        assert!(!error.to_string().contains("capacity"), "{error}");
    }

    #[test]
    fn a_provider_that_is_not_answering_says_what_to_check() {
        let provider = hosted("http://127.0.0.1:1");
        let key = key();
        let error = Trying::provider(&provider, Some(&key))
            .under(&SourcePolicy::Anywhere)
            .unwrap_err();
        assert_eq!(error, NotTried::Unreachable);
        assert!(error.to_string().contains("check the address"), "{error}");
    }

    /// Providers document their address both ways. A second `/v1` would come
    /// back 404 and be read as a wrong address when the address was right.
    #[test]
    fn an_address_that_already_ends_at_the_api_is_not_given_a_second_v1() {
        assert_eq!(
            models_url("https://api.mistral.ai/v1"),
            "https://api.mistral.ai/v1/models"
        );
        assert_eq!(
            models_url("https://api.mistral.ai/v1/"),
            "https://api.mistral.ai/v1/models"
        );
        assert_eq!(
            models_url("http://127.0.0.1:11434"),
            "http://127.0.0.1:11434/v1/models"
        );
    }

    /// A pasted key arrives with the line around it, and that line must not
    /// reach the wire — a header carrying a newline is a header that could
    /// carry a second header.
    #[test]
    fn the_line_around_a_pasted_key_never_reaches_the_wire() {
        let (url, server) = serving(A_MODEL_LIST, 200);
        let provider = hosted(&url);
        let key = Secret::typed("  sk-live-0123456789\n").unwrap();
        let tried = Trying::provider(&provider, Some(&key)).under(&SourcePolicy::Anywhere);
        let request = server.join().unwrap();
        assert!(tried.is_ok(), "{tried:?}");
        assert!(
            request
                .to_lowercase()
                .contains("authorization: bearer sk-live-0123456789\r\n"),
            "{request}"
        );
    }

    /// A provider needing no key is not sent an empty one: an `Authorization`
    /// header nobody asked for is a header some services refuse outright.
    #[test]
    fn a_provider_given_no_key_is_sent_no_authorisation_at_all() {
        let (url, server) = serving(A_MODEL_LIST, 200);
        let provider = hosted(&url);
        let tried = Trying::provider(&provider, None).under(&SourcePolicy::Anywhere);
        let request = server.join().unwrap();
        assert!(tried.is_ok(), "{tried:?}");
        assert!(
            !request.to_lowercase().contains("authorization"),
            "{request}"
        );
    }

    /// An entry with no name is left out, and the rest of a good list still
    /// arrives: one malformed model should not cost somebody the other twenty.
    #[test]
    fn one_malformed_entry_does_not_cost_the_rest_of_the_list() {
        let (url, server) = serving(
            r#"{"data":[{"object":"model"},{"id":"mistral-small-latest"}]}"#,
            200,
        );
        let provider = hosted(&url);
        let tried = Trying::provider(&provider, None)
            .under(&SourcePolicy::Anywhere)
            .unwrap();
        server.join().unwrap();
        assert_eq!(tried.models(), ["mistral-small-latest"]);
        assert_eq!(tried.unshowable(), 1);
    }
}
