//! **The model at a provider**, asked the way a turn's question is asked.
//!
//! `alo_turn` puts an agent's next request to a hosted model through
//! `Asking::to_a_provider_held_to`, holding the answer to
//! `the_envelope_naming_the_verbs` (`crates/alo-turn/src/asking.rs`). This is
//! the same call with the same schema, so the model is asked exactly what a
//! machine asks it. It leaves the machine, and is on this run's own indicator
//! while it does.

use alo_answering::Answering;
use alo_asking::{Asking, Hosted, NotAsked, Question};
use alo_capability::{Grantee, Verbs};
use alo_egress::Indicator;
use alo_models::{Provider, Secret, SourcePolicy};
use alo_protocol::ToAnAgent;
use alo_strings::Strings;
use serde_json::Value;
use std::io::Write as _;
use std::path::PathBuf;

use crate::a_try::TheModel;

/// Who the run asks as. Not a person, and never on a machine's grants.
const ASKING_AS: &str = "@every-try";

/// A model at a provider, with the key it is asked with and the schema its
/// answers are held to.
pub struct AtAProvider {
    /// Where the model is.
    provider: Provider,
    /// The key it is asked with, never shown.
    key: Secret,
    /// The model, as the provider names it.
    model: String,
    /// The shape its answers are held to, a turn's own.
    held_to: Value,
    /// This run's indicator, which every question is on while it is away.
    indicator: Indicator,
    /// How many times the provider was asked.
    calls: usize,
    /// Where every question's whole text is written as it is put, when a run
    /// asked for that ([`AtAProvider::writing_what_it_is_shown_to`]).
    shown_to: Option<PathBuf>,
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
            calls: 0,
            shown_to: None,
        }
    }

    /// The same, appending the whole text of every question it puts to the
    /// file at `to`, each headed by its number: **what the model was told**,
    /// the evidence a wrong choice is read against (rule 6 of ADR 0156). The
    /// text holds no key, which travels in a header and never in a question.
    #[must_use]
    pub fn writing_what_it_is_shown_to(mut self, to: PathBuf) -> Self {
        self.shown_to = Some(to);
        self
    }

    /// Write `shown`, the question about to be put, where the run asked.
    ///
    /// # Errors
    /// Why it could not be written: a run that asked for the text and cannot
    /// have it stops, rather than measuring without its evidence.
    fn writing_down(&self, shown: &str) -> Result<(), String> {
        let Some(to) = &self.shown_to else {
            return Ok(());
        };
        let failed =
            |why: std::io::Error| format!("what the model is shown was not written: {why}");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(to)
            .map_err(failed)?;
        writeln!(
            file,
            "=== question {} ===\n{shown}",
            self.calls.saturating_add(1)
        )
        .map_err(failed)
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

/// How many times one question is put before the provider is said not to
/// answer: a provider that took too long once is not a model that chose.
const ATTEMPTS: usize = 2;

impl AtAProvider {
    /// One attempt at one question.
    ///
    /// **What went wrong is said by its kind alone**, never by what the
    /// provider sent back: a refused key's reply quotes part of the key, and
    /// GitHub masks only the whole of it.
    fn once(&mut self, question: &Question, strings: &Strings) -> Result<ToAnAgent, String> {
        let hosted = Hosted::provider(&self.provider, Some(&self.key));
        let policy = SourcePolicy::Anywhere;
        let answering = Answering::chosen(hosted.named_source(), &policy)
            .map_err(|_| "the policy forbids this provider".to_owned())?;
        let agent = Grantee::named(ASKING_AS);
        let asking = Asking::by(&agent, answering, &[], &policy);
        let to = resolved(hosted.where_it_would_connect());
        self.calls += 1;
        let asked = asking.to_a_provider_held_to(
            question,
            &hosted,
            &mut self.indicator,
            std::time::SystemTime::now(),
            &to,
            self.held_to.clone(),
        );
        match asked {
            Ok(asked) => {
                let answer = asked.ended(&mut self.indicator);
                Ok(ToAnAgent::answered(
                    answer.text(),
                    &answer.came_from(strings),
                    answer.model(),
                ))
            }
            // **The departure is ended whether or not an answer came**, so the
            // indicator is quiet again, as a turn leaves it. Measured on the
            // first real run (CI run 43682925362): a provider that took too
            // long left one open, and the run ended before its summary.
            Err(NotAsked::DidNotAnswer(unanswered)) => {
                let failed = unanswered.ended(&mut self.indicator);
                Err(format!("the provider did not answer: {:?}", failed.why()))
            }
            Err(NotAsked::CannotBeShown(_)) => {
                Err("the question could not be shown on the indicator".to_owned())
            }
            Err(NotAsked::HeldBack(_)) => Err("the question was held back".to_owned()),
            Err(NotAsked::Miswired(_)) => Err("the provider road is miswired".to_owned()),
        }
    }

    /// How many times the provider was asked, second attempts included.
    #[must_use]
    pub const fn calls(&self) -> usize {
        self.calls
    }
}

impl TheModel for AtAProvider {
    fn answering(&mut self, shown: &str, strings: &Strings) -> Result<ToAnAgent, String> {
        let question =
            Question::asked(shown, &self.model).map_err(|_| "nothing could be asked".to_owned())?;
        self.writing_down(shown)?;
        let mut last = String::new();
        for _ in 0..ATTEMPTS {
            match self.once(&question, strings) {
                Ok(answered) => return Ok(answered),
                Err(why) => last = why,
            }
        }
        Err(last)
    }
}
