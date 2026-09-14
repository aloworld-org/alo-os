//! The door an agent turn's question takes to the runtime on this machine,
//! holding the answer to the protocol's envelope —
//! [ADR 0032](../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md).
//!
//! [`crate::Asking::to_this_machine`] puts a question to the runtime and takes
//! whatever comes back, which is right for a question a person asks: it is
//! answered in prose. An agent turn's question must answer in the protocol, and
//! the pinned runtime can be asked to hold its answer to the protocol's version
//! and exactly one of the three doors an agent has — which took Qwen 2.5 7B from
//! 40% to 87.5% of requests alo OS would act on. This is that ask, as a door, so
//! that a turn can take it and the measurement asks the same way the product
//! does.
//!
//! # The same place, asked a different way
//!
//! Everything but the question's shape is [`crate::locally`]'s: the permission
//! must name this machine's runtime, nothing is asked of the runtime when it
//! does not, no policy is asked and nothing is shown, and a runtime failure
//! reaches a person as the same sentence. Only the pinned runtime — typed here
//! as `alo_models::Ollama`, never any `ModelRuntime` — can be asked this way
//! (ADR 0032, decision 4), and nothing a person asks is ever sent through it
//! (decision 3).
//!
//! # Where this ends
//!
//! `alo-turn` and `alo-agentd` decide what a turn asks, and they are lane A's.
//! This door is what they would call; wiring the turn through it is lane A's
//! task, and until it is done the catalogue's recommendation reads the free
//! grade (ADR 0032, decision 5).

use alo_models::Ollama;

use crate::answer::Answer;
use crate::asking::Asking;
use crate::question::Question;
use crate::refusing::NotAnswered;

impl Asking<'_> {
    /// **Put an agent turn's question to the runtime on this machine, holding
    /// the answer to the protocol's envelope.**
    ///
    /// Takes `self`, as every door does: one permission, one attempt.
    ///
    /// # Errors
    /// [`NotAnswered`], exactly as [`Asking::to_this_machine`] answers: a
    /// permission for somewhere else refuses before the runtime is asked, and a
    /// runtime that could not answer is the failure a person is told.
    pub fn to_this_machine_in_the_envelope(
        self,
        question: &Question,
        runtime: &Ollama,
    ) -> Result<Answer, NotAnswered> {
        let source = self.the_runtime_is_the_place()?;
        let said = runtime.answers_in_the_envelope(question.text(), question.of());
        self.answered_here(source, question, said)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::refusing::Miswired;
    use crate::testing::{mistral_source, serving};
    use alo_answering::Answering;
    use alo_capability::Grantee;
    use alo_models::{Catalogue, InferenceSource, SourcePolicy};

    const AN_ENVELOPE: &str = r#"{"message":{"role":"assistant","content":"{\"format\":1,\"asks\":{\"read\":{\"verb\":\"list_folder\",\"given\":[]}}}"}}"#;

    fn here() -> Answering {
        Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::ThisMachineOnly).unwrap()
    }

    fn the_body_of(request: &str) -> serde_json::Value {
        serde_json::from_str(
            request
                .split_once("\r\n\r\n")
                .map(|(_, body)| body)
                .unwrap(),
        )
        .unwrap()
    }

    /// **The request carries the envelope, and nothing about the call's inside**,
    /// read off the socket rather than off the code that built it.
    #[test]
    fn an_agent_turns_question_goes_to_the_runtime_held_to_the_envelope() {
        let (url, server) = serving(AN_ENVELOPE, 200);
        let runtime = Ollama::at(&url, Catalogue::built_in().unwrap());
        let question = Question::asked("the next request, please", "a-model").unwrap();
        let answer = Asking::by(
            &Grantee::named("@agent"),
            here(),
            &[],
            &SourcePolicy::ThisMachineOnly,
        )
        .to_this_machine_in_the_envelope(&question, &runtime)
        .unwrap();
        let request = server.join().unwrap();

        assert!(request.starts_with("POST /api/chat "), "{request}");
        let body = the_body_of(&request);
        let format = body.get("format").unwrap();
        assert_eq!(format, &alo_models::in_the_envelope::the_envelope());
        for into_the_call in [
            "\"verb\"",
            "\"given\"",
            "\"named\"",
            "\"is\"",
            "\"question\"",
        ] {
            assert!(
                !format.to_string().contains(into_the_call),
                "the envelope reaches into the call through {into_the_call}: {format}"
            );
        }
        assert_eq!(answer.source(), &InferenceSource::ThisMachine);
        assert!(answer.text().contains("\"asks\""), "{}", answer.text());
    }

    /// **A question a person asks is never given the envelope**, through the
    /// door a person's question takes — read off the socket.
    #[test]
    fn a_persons_question_through_the_runtime_door_carries_no_envelope() {
        let (url, server) = serving(r#"{"message":{"role":"assistant","content":"No."}}"#, 200);
        let runtime = Ollama::at(&url, Catalogue::built_in().unwrap());
        let question = Question::asked("may the tenant sublet?", "a-model").unwrap();
        Asking::by(
            &Grantee::named("@mail"),
            here(),
            &[],
            &SourcePolicy::ThisMachineOnly,
        )
        .to_this_machine(&question, &runtime)
        .unwrap();
        let body = the_body_of(&server.join().unwrap());
        assert!(
            body.get("format").is_none(),
            "a person's question was given a shape: {body}"
        );
    }

    /// **A permission for somewhere else asks the runtime nothing** on this door
    /// either — nothing listens on port 1, so a request would come back as a
    /// failure rather than as the refusal.
    #[test]
    fn a_permission_for_somewhere_else_asks_the_runtime_nothing() {
        let runtime = Ollama::at("http://127.0.0.1:1", Catalogue::built_in().unwrap());
        let question = Question::asked("the next request, please", "a-model").unwrap();
        let refused = Asking::by(
            &Grantee::named("@agent"),
            Answering::chosen(mistral_source(), &SourcePolicy::Anywhere).unwrap(),
            &[],
            &SourcePolicy::Anywhere,
        )
        .to_this_machine_in_the_envelope(&question, &runtime)
        .unwrap_err();
        assert!(
            matches!(refused, NotAnswered::Miswired(Miswired::NotOnThisMachine)),
            "{refused:?}"
        );
    }

    /// **The doors that reach a provider or a service somebody runs are
    /// untouched**: nothing in them names the envelope. Read off this crate's
    /// own source, so a later change that held a hosted answer to it is asked
    /// the question ADR 0032 decision 4 left open.
    #[test]
    fn the_hosted_and_served_doors_never_hold_an_answer_to_the_envelope() {
        let source = |file: &str| {
            std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("src")
                    .join(file),
            )
            .unwrap()
        };
        for file in ["hosted.rs", "served.rs", "openai.rs", "corridor.rs"] {
            let text = source(file);
            for envelope in ["answers_in_the_envelope", "the_envelope", "in_the_envelope"] {
                assert!(!text.contains(envelope), "{file} names `{envelope}`");
            }
        }
    }
}
