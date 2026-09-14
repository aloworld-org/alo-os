//! The door a measurement takes to a service on this machine that can hold an
//! answer to a grammar.
//!
//! [`crate::served`] is the door to a service somebody runs on their own
//! machine, and [`crate::in_the_envelope`] is the door an agent turn takes to
//! the pinned runtime, holding the answer to the protocol's envelope and no
//! further — [ADR 0032](../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md),
//! because the pinned runtime orders a schema's keys alphabetically and the
//! protocol's argument is `named` then `is`.
//!
//! This is the third: the same service, the same address and the same
//! permission as [`crate::served`], with every token of the answer held to a
//! grammar the caller wrote. `llama.cpp`'s server takes one, which is what
//! [ADR 0035](../../../docs/decisions/0035-the-wrapper-or-the-engine.md) puts on
//! trial and what task 17 measures.
//!
//! # Why it is its own door rather than a parameter
//!
//! What may be reached, and under what permission, is decided per door in this
//! crate — so a door that can constrain an answer is a door a reader can find
//! and a caller has to choose. Nothing in the product asks through it today:
//! the measurement does, and if ADR 0035 is accepted, the runtime behind
//! `alo_models::ModelRuntime` moves and this door is where the turn's question
//! would go.
//!
//! # What it does not do
//!
//! It writes no grammar and knows nothing about verbs. The grammar arrives as
//! text from whoever asked, because the shape of a call is `alo-protocol`'s and
//! the verbs are `alo-capability`'s, and this crate depends on neither.

use std::net::SocketAddr;

use alo_models::InferenceSource;

use crate::answer::Answer;
use crate::asking::Asking;
use crate::question::Question;
use crate::refusing::{Miswired, NotAnswered};
use crate::served::Served;

impl Asking<'_> {
    /// **Put an agent turn's question to a service on this machine, holding
    /// every token of the answer to `grammar`.**
    ///
    /// The permission is spent exactly as [`Asking::to_a_service_on_this_machine`]
    /// spends it, and the same two sources reach it. Nothing leaves this
    /// machine, so no policy is asked and no departure is made — `served`
    /// having refused every address that is not this machine's is what makes
    /// that true rather than assumed.
    ///
    /// # Errors
    /// [`NotAnswered`], as the door beside it answers: a service that does not
    /// know what a grammar is refuses the request, and that reaches a person as
    /// *something answered, and not with an answer* rather than as a model
    /// failing.
    pub fn to_a_service_on_this_machine_held_to(
        self,
        question: &Question,
        served: &Served<'_>,
        to: &[SocketAddr],
        grammar: &str,
    ) -> Result<Answer, NotAnswered> {
        let source = self.answering.source().clone();
        match &source {
            InferenceSource::ThisMachine | InferenceSource::AServiceAtThisMachinesAddress => {}
            InferenceSource::Hosted { .. } => return Err(Miswired::NotOnThisMachine.into()),
            InferenceSource::PairedMachine { .. } => {
                return Err(Miswired::BelongsDownTheCorridor.into());
            }
        }
        match served.ask_held_to(question, to, grammar) {
            Ok(said) => Ok(Answer::new(said, source, question.of().to_owned())),
            Err(why) => match self.answering.did_not_answer(why, self.others, self.policy) {
                Ok(failed) => Err(NotAnswered::DidNotAnswer(Box::new(failed))),
                Err(reported) => Err(reported.into()),
            },
        }
    }
}
