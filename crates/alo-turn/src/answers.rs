//! What can answer a question on this machine, and the door each one is behind.
//!
//! `alo-asking` has three doors and says outright that which one a question
//! takes is decided before that crate is reached — *by the place a person
//! chose*. This is that decision, as a value a turn can be handed: the three
//! things a question can be put to, each holding what it needs to be reached.
//!
//! | | Where the answer comes from | Does anything leave |
//! |---|---|---|
//! | [`Answers::Provider`] | A provider somebody added | Yes, and law 1 shows it |
//! | [`Answers::Runtime`] | The model runtime alo OS ships | No |
//! | [`Answers::Service`] | A service somebody runs here on loopback | No |
//!
//! # This is not a route, and it decides nothing
//!
//! A turn does not choose between these and there is no method here that could.
//! Which one a shell hands over is the setting the person made, and the
//! permission that travels beside it — `alo_answering::Answering` — is made
//! from that same setting. ADR 0008 is the reason it reads that way: a machine
//! that fell back from one of these to another would need no new type at all,
//! only a second arm in the match that spends this one.
//!
//! [`Answers::source`] is what a shell makes that permission out of, which is
//! how the two come to be the same place. When they are not, the door refuses
//! it as `alo_asking::Miswired` — the crate that owns the addresses owns that
//! check too, and a second one here would be this machine holding two opinions
//! about where a question is going.
//!
//! # A paired machine is not here, because there is no door for it
//!
//! `alo_models::InferenceSource::PairedMachine` is a place a question may be
//! answered and nothing in this repository reaches one. A variant for it would
//! be a stub wearing a capability, which law 3 forbids; what happens instead is
//! that the permission arrives naming somewhere none of the three doors goes,
//! and `alo_asking::Miswired::NoPathToAPairedMachine` says so.

use alo_asking::{Hosted, Served};
use alo_models::{InferenceSource, ModelRuntime};

/// One thing that can answer a question, and everything needed to reach it.
///
/// Borrowed for the length of one question, as `alo_asking::Hosted` and
/// `alo_asking::Served` are: neither a provider nor the key it was given is
/// kept anywhere by this.
#[derive(Debug)]
pub enum Answers<'a> {
    /// A hosted provider somebody added, with the key they gave it.
    ///
    /// **The one that leaves.** A question put here goes on the indicator
    /// before a socket opens, is refused if the rule in force does not permit
    /// it, and is written into the record as a departure whether or not it is
    /// answered.
    Provider(Hosted<'a>),
    /// The model runtime alo OS ships, on this machine.
    ///
    /// Nothing leaves, so there is no indicator, no rule to ask and no
    /// departure — which is `docs/features.md`'s *a working day with a local
    /// model produces zero inference egress*, carried by the absence of a type
    /// rather than by a counter that reads zero.
    Runtime(&'a dyn ModelRuntime),
    /// An OpenAI-compatible service somebody runs on this machine.
    ///
    /// vLLM, llama.cpp's server, LM Studio. `alo_asking::Served::at` has already
    /// refused every address that is not this machine, so holding one of these
    /// is the proof that nothing put to it leaves.
    Service(Served<'a>),
}

impl Answers<'_> {
    /// Where an answer from here says it came from.
    ///
    /// What a shell builds `alo_answering::Answering::chosen` out of, so that
    /// the permission and the place are one setting read twice rather than two
    /// settings that can disagree. It is read off the thing itself, in the way
    /// `alo_asking::Hosted::named_source` and `alo_asking::Served::source` are
    /// meant to be.
    #[must_use]
    pub fn source(&self) -> InferenceSource {
        match self {
            Self::Provider(hosted) => hosted.named_source(),
            Self::Runtime(_) => InferenceSource::ThisMachine,
            Self::Service(served) => served.source(),
        }
    }

    /// Whether putting a question here sends anything off this machine.
    ///
    /// True for the provider and false for the two on this machine. A caller
    /// asks it before a question rather than after: it is the difference
    /// between a turn that will put something on law 1's indicator and one that
    /// will not.
    #[must_use]
    pub fn leaves(&self) -> bool {
        self.source().causes_egress()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{Stub, a_provider, a_service, mistral_source};
    use alo_asking::Question;
    use alo_models::Secret;

    /// **The two on this machine say they are this machine**, and the one that
    /// is somewhere else names the provider and the region it declared — which
    /// is what a permission is made out of and what the indicator shows.
    #[test]
    fn where_an_answer_would_come_from_is_read_off_the_place_that_would_answer() {
        let runtime = Stub::answering("no");
        let provider = a_provider("https://api.mistral.ai");
        let key = Secret::typed("sk-live-0123456789").unwrap();
        assert_eq!(
            Answers::Provider(Hosted::provider(&provider, Some(&key))).source(),
            mistral_source()
        );

        assert_eq!(
            Answers::Runtime(&runtime).source(),
            InferenceSource::ThisMachine
        );

        let service = a_service("http://127.0.0.1:8000");
        assert_eq!(
            Answers::Service(Served::at(&service, None).unwrap()).source(),
            InferenceSource::ThisMachine
        );
    }

    /// **Exactly one of the three leaves**, and it is the one law 1 is about.
    /// A service on loopback is this machine however OpenAI-compatible it is.
    #[test]
    fn one_of_the_three_leaves_this_machine_and_two_do_not() {
        let runtime = Stub::answering("no");
        let provider = a_provider("https://api.mistral.ai");
        assert!(Answers::Provider(Hosted::provider(&provider, None)).leaves());

        assert!(!Answers::Runtime(&runtime).leaves());

        let service = a_service("http://127.0.0.1:8000");
        assert!(!Answers::Service(Served::at(&service, None).unwrap()).leaves());
    }

    /// A question is somebody's own words and the model is a name somebody
    /// wrote, so neither reaches this file: `Answers` says *where*, and the
    /// question says *what*. This is the test that keeps them apart.
    #[test]
    fn nothing_here_holds_the_question() {
        let runtime = Stub::answering("no");
        let question = Question::asked("may the tenant sublet?", "mistral-small-latest").unwrap();
        assert_eq!(question.of(), "mistral-small-latest");
        let printed = format!("{:?}", Answers::Runtime(&runtime));
        assert!(!printed.contains("sublet"), "{printed}");
    }
}
