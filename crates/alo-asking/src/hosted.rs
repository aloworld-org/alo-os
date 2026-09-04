//! A provider somewhere else, and what the person is told it is called.
//!
//! The wire itself is `openai.rs` — the convention this and
//! [`crate::served`] both speak, in one file, because two renderings of one
//! protocol is two things that can disagree about what left the machine. What is
//! here is the half that is about a *provider*: the name and region a person
//! wrote down, and how long this machine waits for somebody else's service.
//!
//! # Nothing here opens anything on its own
//!
//! `Hosted::ask` is `pub(crate)` and its one caller is
//! [`crate::Asking::to_a_provider`], which has already obtained an
//! `alo_egress::Departing` — so there is no public function reaching a provider
//! without law 1's indicator having shown it first. The guarantee is
//! `alo-egress`' and this crate does not get a second way round it.
//!
//! [`crate::served`] reaches the same wire without a `Departing`, which is the
//! one exception and is not an exception to law 1: it will not carry a question
//! to an address that is not this machine, so there is nothing for an indicator
//! to show. That file makes its own argument for it.

use std::time::Duration;

use alo_answering::WentWrong;
use alo_models::{InferenceSource, Provider, Secret};

use crate::openai;
use crate::question::Question;

/// How long this machine waits for an answer from somebody else.
///
/// Shorter than the five minutes [`crate::served`] waits for a service on this
/// machine, which is ADR 0007's asymmetry rather than an oversight: a model
/// thinking on this machine's CPU is the ordinary case and a provider is a
/// datacentre. Longer than the ten seconds `alo_models::trying` waits, because
/// that is somebody watching a dialogue and this is a model thinking. Short
/// enough that a provider which has stopped answering is *said* to have stopped
/// rather than left looking like a machine that has hung.
const WHILE_A_MODEL_THINKS: Duration = Duration::from_secs(120);

/// A provider a person added, and the key it was given.
///
/// Borrowed rather than owned, for the length of one question: neither the
/// provider nor the key is kept anywhere by this.
#[derive(Debug)]
pub struct Hosted<'a> {
    /// Where to reach it, and what it is called.
    provider: &'a Provider,
    /// The key, for a provider that needs one.
    key: Option<&'a Secret>,
}

impl<'a> Hosted<'a> {
    /// This provider, with this key — which is [`None`] for one that needs
    /// none.
    #[must_use]
    pub fn provider(provider: &'a Provider, key: Option<&'a Secret>) -> Self {
        Self { provider, key }
    }

    /// Where an answer from this provider would say it came from, out of what
    /// the person wrote down when they added it.
    ///
    /// **Not `alo_models::Provider::source`**, and the difference is worth the
    /// sentence. That answers *is this address on this machine*, which is a
    /// fact about the endpoint; this answers *what would the indicator say*,
    /// which is a fact about the name and the region a person stated. This one
    /// is what [`crate::Asking`] checks the permission against, because the
    /// line law 1 shows is composed out of the source and the connection is
    /// opened out of the endpoint — and a machine where those two disagree is a
    /// machine telling somebody their question went somewhere it did not.
    ///
    /// A provider on this machine is therefore named here as though it were
    /// somewhere else, and that mismatch is exactly what
    /// [`crate::Asking::to_a_provider`] refuses: `Provider::source` calls it
    /// this machine, the permission is for this machine, and the two do not
    /// meet. [`crate::Served`] is the door such a provider belongs to.
    #[must_use]
    pub fn named_source(&self) -> InferenceSource {
        InferenceSource::Hosted {
            provider: self.provider.name.clone(),
            region: self.provider.region.clone(),
        }
    }

    /// Put the question, and read what comes back.
    ///
    /// `pub(crate)`, and the only caller is [`crate::Asking::to_a_provider`],
    /// which holds an `alo_egress::Departing` by the time it gets here. That is
    /// the whole of why this is not public: a public method here would be a way
    /// to reach a provider without law 1 having shown it.
    ///
    /// # Errors
    /// [`WentWrong`], as `openai::put` answers it.
    pub(crate) fn ask(&self, question: &Question) -> Result<String, WentWrong> {
        openai::put(
            &self.provider.endpoint,
            self.key,
            question,
            WHILE_A_MODEL_THINKS,
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{mistral, serving};

    /// One answer, in the shape every OpenAI-compatible provider replies with.
    const AN_ANSWER: &str = r#"{"choices":[{"index":0,"message":{"role":"assistant","content":"The tenant may not sublet."}}]}"#;

    fn question() -> Question {
        Question::asked("may the tenant sublet?", "mistral-small-latest").unwrap()
    }

    /// What the indicator would say about this provider is made out of what the
    /// person wrote down, and it is what the permission is checked against.
    #[test]
    fn what_this_provider_would_be_called_is_what_the_person_named_it() {
        let provider = mistral("https://api.mistral.ai");
        assert_eq!(
            Hosted::provider(&provider, None).named_source(),
            crate::testing::mistral_source()
        );
    }

    /// The question reaches the address the person typed, over the convention
    /// `openai.rs` owns — which is asserted in full there, and here only
    /// as the join between this type and it.
    #[test]
    fn the_question_goes_to_the_address_the_person_typed() {
        let (url, server) = serving(AN_ANSWER, 200);
        let provider = mistral(&url);
        let key = alo_models::Secret::typed("sk-live-0123456789").unwrap();
        let answer = Hosted::provider(&provider, Some(&key)).ask(&question());
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
    }

    /// **This machine waits longer for itself than for anybody else** (ADR
    /// 0007). A constant either door could have shared would have made that a
    /// coincidence rather than a decision, so the two are asserted against each
    /// other.
    #[test]
    fn a_provider_is_waited_on_for_less_time_than_this_machine_is() {
        assert!(WHILE_A_MODEL_THINKS < crate::served::WHILE_THIS_MACHINE_THINKS);
    }
}
