//! **The model at a provider**, asked the way a turn's question is asked.
//!
//! `alo_turn` puts an agent's next request to a hosted model through
//! `Asking::to_a_provider_held_to`, holding the answer to
//! `the_envelope_naming_the_verbs` (`crates/alo-turn/src/asking.rs`). This is
//! the same call with the same schema, so the model is asked exactly what a
//! machine asks it. It leaves the machine, and is on this run's own indicator
//! while it does.

use alo_answering::Answering;
use alo_asking::{Asking, Hosted, Question};
use alo_capability::{Grantee, Verbs};
use alo_egress::Indicator;
use alo_models::{Provider, Secret, SourcePolicy};
use alo_protocol::ToAnAgent;
use alo_strings::Strings;
use serde_json::Value;

use crate::a_try::TheModel;

/// Who the run asks as. Not a person, and never on a machine's grants.
const ASKING_AS: &str = "@every-try";

/// A model at a provider, with the key it is asked with and the schema its
/// answers are held to.
pub struct AtAProvider {
    provider: Provider,
    key: Secret,
    model: String,
    held_to: Value,
    indicator: Indicator,
}

impl AtAProvider {
    /// `model` at `provider`, asked with `key`, its answers held to the shape
    /// a turn holds them to for `verbs`.
    #[must_use]
    pub fn asking(provider: Provider, key: Secret, model: &str, verbs: &Verbs) -> Self {
        Self {
            provider,
            key,
            model: model.to_owned(),
            held_to: alo_turn::the_shape_a_provider_is_held_to::the_envelope_naming_the_verbs(
                verbs,
            ),
            indicator: Indicator::default(),
        }
    }

    /// Whether every question this run put has come back: the indicator is
    /// quiet again.
    #[must_use]
    pub fn is_quiet(&self) -> bool {
        self.indicator.is_quiet()
    }
}

/// Where a provider's endpoint resolves to, for the boundary to be told.
fn resolved(where_to: Option<(String, u16)>) -> Vec<std::net::SocketAddr> {
    use std::net::ToSocketAddrs as _;
    where_to
        .and_then(|(host, port)| (host.as_str(), port).to_socket_addrs().ok())
        .map(Iterator::collect)
        .unwrap_or_default()
}

impl TheModel for AtAProvider {
    fn answering(&mut self, shown: &str, strings: &Strings) -> Result<ToAnAgent, String> {
        let question = Question::asked(shown, &self.model).map_err(|why| format!("{why:?}"))?;
        let hosted = Hosted::provider(&self.provider, Some(&self.key));
        let policy = SourcePolicy::Anywhere;
        let answering = Answering::chosen(hosted.named_source(), &policy)
            .map_err(|why| format!("the policy forbids this provider: {why:?}"))?;
        let agent = Grantee::named(ASKING_AS);
        let asking = Asking::by(&agent, answering, &[], &policy);
        let to = resolved(hosted.where_it_would_connect());
        let asked = asking
            .to_a_provider_held_to(
                &question,
                &hosted,
                &mut self.indicator,
                std::time::SystemTime::now(),
                &to,
                self.held_to.clone(),
            )
            .map_err(|why| format!("{why:?}"))?;
        let answer = asked.ended(&mut self.indicator);
        Ok(ToAnAgent::answered(
            answer.text(),
            &answer.came_from(strings),
            answer.model(),
        ))
    }
}
