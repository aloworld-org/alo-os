//! The door an agent turn takes when what it asks a model for is **its next
//! request** — [ADR 0032](../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md).
//!
//! An agent drives the verbs by asking a model what to ask for next, and that
//! answer must be a line of the protocol. The pinned runtime can hold an answer
//! to a JSON schema, and holding it to the envelope — the protocol's version and
//! exactly one of `read`, `propose` and `ask`, with nothing said about what is
//! inside the door — took Qwen 2.5 7B from 40% to 88.75% of requests alo OS
//! would act on. `alo-asking` made that ask a door
//! (`Asking::to_this_machine_in_the_envelope`); this is the turn taking it.
//!
//! # One road, and one arm of it that differs
//!
//! [`Turning::asking_for_the_next_request`] is [`Turning::asking`] with
//! `Held::ToTheEnvelope`, through the same function. What is refused, shown,
//! bounded and written down is therefore the same by construction rather than
//! by two copies agreeing:
//!
//! | Where it goes | Asked in words | Asked for the next request |
//! |---|---|---|
//! | The pinned runtime ([`Answers::ThePinnedRuntime`]) | freely | **held to the envelope** |
//! | A runtime known only by its trait ([`Answers::Runtime`]) | freely | freely |
//! | A service somebody runs here ([`Answers::Service`]) | freely | freely |
//! | A provider ([`Answers::Provider`]) | freely, under a departure | freely, under a departure |
//! | A paired machine ([`Answers::PairedMachine`]) | freely, under a departure | freely, under a departure |
//!
//! **Only the pinned runtime** (decision 4): a provider and a paired machine
//! answer through other doors with other capabilities, and whether they are
//! held to a shape is a separate measurement nobody has made. A service
//! somebody else runs is not the runtime ADR 0032 measured either.
//!
//! **A question in words is never given the schema** (decision 3). *May the
//! tenant sublet?* is answered in prose, and [`Turning::asking`] cannot reach
//! the envelope: `Held::InWords` is the only value it passes.
//!
//! # The record does not know how a model was asked
//!
//! An answer from this machine is `alo_record::Entry::answered_here` down both
//! roads, and a departure is `alo_record::Entry::left` down both. How the model
//! was asked is not a fact about what happened on this machine that a person
//! would look for, and an entry that carried it would be a second shape for one
//! moment — so it is not kept, and a test holds the two entries equal.
//!
//! # What this lets through
//!
//! Nothing new. The answer comes back to the agent as a model's words, exactly
//! as an answer in words does, and becomes anything only when the agent sends
//! it as its next line — where `alo-protocol` reads it and `alo-capability`
//! validates it as it would any other. The schema removes a way for a model to
//! fail to be understood; it is not a gate and nothing relies on it as one.

use std::time::SystemTime;

use alo_answering::Answering;
use alo_asking::Answer;

use crate::answers::Answers;
use crate::places::Places;
use crate::turning::Turning;
use crate::unanswered::NoAnswer;

/// How the answer to a question is held.
///
/// `pub(crate)`: a caller says which door it wants by which method it calls,
/// so there is no value a caller could pass that picks the envelope for a
/// person's question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Held {
    /// Answered in words, however the model likes. Every question a person or
    /// an agent asks about the world.
    InWords,
    /// Held to the protocol's envelope where the place can be asked that way —
    /// the pinned runtime — and asked in words everywhere else.
    ToTheEnvelope,
}

impl Turning<'_, '_> {
    /// **Ask a model for this agent's next request.**
    ///
    /// Everything [`Turning::asking`] says is true here, with one difference:
    /// when the place the person chose is the pinned runtime, the runtime is
    /// asked to hold its answer to the protocol's envelope (ADR 0032). Every
    /// other place is asked exactly as a question in words is.
    ///
    /// `asked` is what the agent composed for the model — its instructions,
    /// the verbs and what the person said — and, like any question, nothing of
    /// it is kept anywhere.
    ///
    /// # Errors
    /// [`NoAnswer`], exactly the seven [`Turning::asking`] answers, with the
    /// same records left behind.
    pub fn asking_for_the_next_request(
        &mut self,
        asked: &str,
        of_model: &str,
        answering: Answering,
        answers: &Answers<'_>,
        places: &Places<'_>,
        now: SystemTime,
    ) -> Result<Answer, NoAnswer> {
        self.putting(
            asked,
            of_model,
            answering,
            answers,
            places,
            now,
            Held::ToTheEnvelope,
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
    use crate::kept::Kept;
    use crate::machine::Machine;
    use crate::testing::{
        AN_ANSWER, NothingIsBounded, a_provider, a_service, files, hour, in_english,
        mistral_source, noon, permitting, serving,
    };
    use alo_asking::{Hosted, Miswired, Served};
    use alo_capability::Grants;
    use alo_context::Context;
    use alo_egress::Indicator;
    use alo_files::OnThisMachine;
    use alo_keeping::NotKept;
    use alo_models::in_the_envelope::the_envelope;
    use alo_models::{Catalogue, InferenceSource, Ollama, SourcePolicy};
    use alo_record::{Asking as AskingAbout, Entry, Only, Record};

    /// What an agent composes for a model when it wants its next request.
    const WHAT_NEXT: &str = "The person said: rename scan001.pdf to march.pdf. Your next request?";

    /// A question about the world, asked in words.
    const SUBLET: &str = "may the tenant sublet?";

    /// The pinned runtime's reply, holding one line of the protocol.
    const AN_ENVELOPE: &str = r#"{"message":{"role":"assistant","content":"{\"format\":1,\"asks\":{\"propose\":{\"verb\":\"rename_file\",\"given\":[]}}}"}}"#;

    /// The pinned runtime's reply in words.
    const IN_WORDS: &str = r#"{"message":{"role":"assistant","content":"No."}}"#;

    /// The pinned runtime, at an address a test serves on.
    fn the_pinned_runtime_at(url: &str) -> Ollama {
        Ollama::at(url, Catalogue::built_in().unwrap())
    }

    /// Everything after the request's head: what was actually sent.
    fn the_body_of(request: &str) -> &str {
        request.split_once("\r\n\r\n").map_or("", |(_, body)| body)
    }

    /// One turn by `@files` on a machine with this record, ending when the
    /// closure is done — `asking.rs`' fixture, for `asking.rs`' reason.
    fn on_a_machine<T>(
        kept: &mut dyn crate::Shortening,
        indicator: &mut Indicator,
        doing: impl FnOnce(&mut Turning<'_, '_>) -> T,
    ) -> T {
        let strings = in_english();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            indicator,
            kept,
        )
        .unwrap();
        let mut grants = Grants::default();
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@files",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        doing(&mut turning)
    }

    /// Every entry the record holds, in order.
    fn every_entry(record: &Record) -> Vec<Entry> {
        record
            .answering(&AskingAbout::anything())
            .cloned()
            .collect()
    }

    /// **The pinned runtime is asked for the next request held to the
    /// envelope, and to nothing about the call's inside** — read off the
    /// socket, not off the code that built the request. And what is left
    /// behind is an answer on this machine: nothing shown, one entry.
    #[test]
    fn an_agents_next_request_is_asked_of_the_pinned_runtime_held_to_the_envelope() {
        let (url, server) = serving(AN_ENVELOPE, 200);
        let runtime = the_pinned_runtime_at(&url);
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let answer = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking_for_the_next_request(
                WHAT_NEXT,
                "qwen2.5-7b-instruct",
                permitting(InferenceSource::ThisMachine),
                &Answers::ThePinnedRuntime(&runtime),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
        })
        .unwrap();
        let request = server.join().unwrap();

        assert!(request.starts_with("POST /api/chat "), "{request}");
        let body: serde_json::Value = serde_json::from_str(the_body_of(&request)).unwrap();
        assert_eq!(
            body.get("format"),
            Some(&the_envelope()),
            "the request was not held to the envelope: {body}"
        );
        let envelope = the_envelope().to_string();
        // The schema is the envelope and only the envelope: nothing in it
        // reaches inside a door (ADR 0032, decision 2).
        for into_the_call in [
            "\"verb\"",
            "\"given\"",
            "\"named\"",
            "\"is\"",
            "\"question\"",
        ] {
            assert!(
                !envelope.contains(into_the_call),
                "the envelope reaches into the call through {into_the_call}: {envelope}"
            );
        }
        assert!(answer.text().contains("\"propose\""), "{}", answer.text());
        assert_eq!(answer.source(), &InferenceSource::ThisMachine);
        assert!(indicator.is_quiet());
        assert_eq!(
            record
                .answering(&AskingAbout::anything().only(Only::Egress))
                .count(),
            0
        );
        assert_eq!(record.len(), 1);
    }

    /// **A question in words, put to the same pinned runtime in the same kind
    /// of turn, is never given the schema** (ADR 0032, decision 3).
    #[test]
    fn a_question_in_words_to_the_pinned_runtime_is_never_given_the_envelope() {
        let (url, server) = serving(IN_WORDS, 200);
        let runtime = the_pinned_runtime_at(&url);
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let answer = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking(
                SUBLET,
                "qwen2.5-7b-instruct",
                permitting(InferenceSource::ThisMachine),
                &Answers::ThePinnedRuntime(&runtime),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
        })
        .unwrap();
        let request = server.join().unwrap();

        let body = the_body_of(&request);
        assert!(body.contains("sublet"), "{body}");
        assert!(
            !body.contains("\"format\""),
            "a question in words was given a shape: {body}"
        );
        assert_eq!(answer.text(), "No.");
    }

    /// **A hosted provider is asked for the next request exactly as it is
    /// asked a question in words** (ADR 0032, decision 4): the two requests
    /// that reach it are the same bytes, and both leave under a departure.
    #[test]
    fn a_provider_is_asked_for_the_next_request_exactly_as_before() {
        let mut bodies = Vec::new();
        let mut records = Vec::new();
        for next_request in [false, true] {
            let (url, server) = serving(AN_ANSWER, 200);
            let provider = a_provider(&url);
            let mut indicator = Indicator::default();
            let mut record = Record::default();
            let answer = on_a_machine(&mut record, &mut indicator, |turning| {
                let answers = Answers::Provider(Hosted::provider(&provider, None));
                let places = Places::under(&SourcePolicy::Anywhere);
                let permission = permitting(mistral_source());
                if next_request {
                    turning.asking_for_the_next_request(
                        WHAT_NEXT,
                        "mistral-small-latest",
                        permission,
                        &answers,
                        &places,
                        noon(),
                    )
                } else {
                    turning.asking(
                        WHAT_NEXT,
                        "mistral-small-latest",
                        permission,
                        &answers,
                        &places,
                        noon(),
                    )
                }
            })
            .unwrap();
            assert_eq!(answer.source(), &mistral_source());
            assert!(indicator.is_quiet());
            bodies.push(the_body_of(&server.join().unwrap()).to_owned());
            records.push(record);
        }

        let [in_words, for_the_next] = bodies.as_slice() else {
            unreachable!("two roads, two requests")
        };
        assert_eq!(in_words, for_the_next, "the provider was asked differently");
        for body in &bodies {
            assert!(!body.contains("format"), "{body}");
        }
        for record in &records {
            assert_eq!(
                record
                    .answering(&AskingAbout::anything().only(Only::Egress))
                    .count(),
                1
            );
        }
    }

    /// A service somebody runs on this machine is not the runtime ADR 0032
    /// measured, so it too is asked for the next request as it always was.
    #[test]
    fn a_service_on_this_machine_is_asked_for_the_next_request_exactly_as_before() {
        let mut bodies = Vec::new();
        for next_request in [false, true] {
            let (url, server) = serving(AN_ANSWER, 200);
            let service = a_service(&url);
            let mut indicator = Indicator::default();
            let mut record = Record::default();
            on_a_machine(&mut record, &mut indicator, |turning| {
                let answers = Answers::Service(Served::at(&service, None).unwrap());
                let places = Places::under(&SourcePolicy::Anywhere);
                let permission = permitting(InferenceSource::AServiceAtThisMachinesAddress);
                if next_request {
                    turning.asking_for_the_next_request(
                        WHAT_NEXT,
                        "a-model",
                        permission,
                        &answers,
                        &places,
                        noon(),
                    )
                } else {
                    turning.asking(WHAT_NEXT, "a-model", permission, &answers, &places, noon())
                }
            })
            .unwrap();
            bodies.push(the_body_of(&server.join().unwrap()).to_owned());
        }
        let [in_words, for_the_next] = bodies.as_slice() else {
            unreachable!("two roads, two requests")
        };
        assert_eq!(in_words, for_the_next, "the service was asked differently");
        assert!(!for_the_next.contains("format"), "{for_the_next}");
    }

    /// **A permission for a paired machine asks the pinned runtime nothing, for
    /// the next request exactly as for a question in words**: the envelope does
    /// not turn the runtime here into a substitute for the machine the person
    /// chose. The paired machine's own road is `crate::down_the_corridor`'s
    /// tests. Nothing is asked of anything —
    /// the runtime listens nowhere, so a request would come back as a failure
    /// rather than as this refusal — and nothing is written down.
    #[test]
    fn a_paired_machine_is_refused_for_the_next_request_exactly_as_before() {
        let runtime = the_pinned_runtime_at("http://127.0.0.1:1");
        let the_studio = InferenceSource::PairedMachine {
            machine: "the studio machine".to_owned(),
        };
        for next_request in [false, true] {
            let mut indicator = Indicator::default();
            let mut record = Record::default();
            let refused = on_a_machine(&mut record, &mut indicator, |turning| {
                let answers = Answers::ThePinnedRuntime(&runtime);
                let places = Places::under(&SourcePolicy::InTheBuilding);
                let permission =
                    Answering::chosen(the_studio.clone(), &SourcePolicy::InTheBuilding).unwrap();
                if next_request {
                    turning.asking_for_the_next_request(
                        WHAT_NEXT,
                        "m",
                        permission,
                        &answers,
                        &places,
                        noon(),
                    )
                } else {
                    turning.asking(WHAT_NEXT, "m", permission, &answers, &places, noon())
                }
            })
            .unwrap_err();
            assert!(
                matches!(
                    refused,
                    NoAnswer::Miswired(Miswired::BelongsDownTheCorridor)
                ),
                "{refused:?}"
            );
            assert_eq!(record.len(), 0);
            assert!(indicator.is_quiet());
        }
    }

    /// **The record entry is unchanged in shape**: an agent's next request and
    /// a question in words, both answered by the pinned runtime, leave the same
    /// entry. How the model was asked is not a thing the record keeps.
    #[test]
    fn how_the_model_was_asked_is_not_a_thing_the_record_keeps() {
        let mut entries = Vec::new();
        for (next_request, reply) in [(false, IN_WORDS), (true, AN_ENVELOPE)] {
            let (url, server) = serving(reply, 200);
            let runtime = the_pinned_runtime_at(&url);
            let mut indicator = Indicator::default();
            let mut record = Record::default();
            on_a_machine(&mut record, &mut indicator, |turning| {
                let answers = Answers::ThePinnedRuntime(&runtime);
                let places = Places::under(&SourcePolicy::Anywhere);
                let permission = permitting(InferenceSource::ThisMachine);
                if next_request {
                    turning.asking_for_the_next_request(
                        WHAT_NEXT,
                        "a-model",
                        permission,
                        &answers,
                        &places,
                        noon(),
                    )
                } else {
                    turning.asking(WHAT_NEXT, "a-model", permission, &answers, &places, noon())
                }
            })
            .unwrap();
            server.join().unwrap();
            entries.push(every_entry(&record));
        }
        let [in_words, for_the_next] = entries.as_slice() else {
            unreachable!("two roads, two records")
        };
        assert_eq!(in_words, for_the_next);
        assert_eq!(for_the_next, &vec![Entry::answered_here(&files(), noon())]);
    }

    /// **A permission for a provider asks the pinned runtime nothing**, on
    /// this door as on the other: it is refused as not this machine before a
    /// socket opens, and nothing is shown or kept.
    #[test]
    fn a_permission_for_a_provider_asks_the_pinned_runtime_nothing() {
        let runtime = the_pinned_runtime_at("http://127.0.0.1:1");
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let refused = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking_for_the_next_request(
                WHAT_NEXT,
                "a-model",
                permitting(mistral_source()),
                &Answers::ThePinnedRuntime(&runtime),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
        })
        .unwrap_err();
        assert!(
            matches!(refused, NoAnswer::Miswired(Miswired::NotOnThisMachine)),
            "{refused:?}"
        );
        assert!(refused.nothing_left());
        assert_eq!(record.len(), 0);
        assert!(indicator.is_quiet());
    }

    /// **A pinned runtime that does not answer leaves nothing**, as it does for
    /// a question in words — and nothing else is asked in its place.
    #[test]
    fn a_pinned_runtime_that_does_not_answer_the_next_request_leaves_nothing() {
        let runtime = the_pinned_runtime_at("http://127.0.0.1:1");
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let refused = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking_for_the_next_request(
                WHAT_NEXT,
                "a-model",
                permitting(InferenceSource::ThisMachine),
                &Answers::ThePinnedRuntime(&runtime),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
        })
        .unwrap_err();
        assert!(matches!(refused, NoAnswer::DidNotAnswer(_)), "{refused:?}");
        assert!(refused.nothing_left());
        assert_eq!(record.len(), 0);
        assert!(indicator.is_quiet());
    }

    /// Nothing composed is nothing asked, on this door as on the other.
    #[test]
    fn nothing_is_asked_for_a_next_request_that_says_nothing() {
        let runtime = the_pinned_runtime_at("http://127.0.0.1:1");
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let refused = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking_for_the_next_request(
                "  ",
                "a-model",
                permitting(InferenceSource::ThisMachine),
                &Answers::ThePinnedRuntime(&runtime),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
        })
        .unwrap_err();
        assert!(matches!(refused, NoAnswer::NotAQuestion(_)), "{refused:?}");
        assert_eq!(record.len(), 0);
    }

    /// A record that cannot be written to, as a full disk looks from a turn.
    struct ANoSpaceLeftDisk;

    impl Kept for ANoSpaceLeftDisk {
        fn keep(&mut self, _entry: Entry) -> Result<(), NotKept> {
            Err(NotKept::NotAddedTo {
                path: "/var/lib/alo/record.jsonl".to_owned(),
                why: "no space left on device".to_owned(),
            })
        }
    }

    impl crate::Shortening for ANoSpaceLeftDisk {
        fn shorten(
            &mut self,
            _keeping: alo_keeping::Keeping,
            _now: SystemTime,
        ) -> Result<crate::Shortened, NotKept> {
            Ok(crate::Shortened::NotOnADisk)
        }
    }

    /// **An answer that could not be written down closes the turn**, and the
    /// next request after it is refused without the runtime being asked.
    #[test]
    fn a_next_request_whose_answer_could_not_be_written_down_closes_the_turn() {
        let (url, server) = serving(AN_ENVELOPE, 200);
        let runtime = the_pinned_runtime_at(&url);
        let mut indicator = Indicator::default();
        let mut disk = ANoSpaceLeftDisk;
        let (first, second) = on_a_machine(&mut disk, &mut indicator, |turning| {
            let answers = Answers::ThePinnedRuntime(&runtime);
            let places = Places::under(&SourcePolicy::Anywhere);
            let first = turning.asking_for_the_next_request(
                WHAT_NEXT,
                "a-model",
                permitting(InferenceSource::ThisMachine),
                &answers,
                &places,
                noon(),
            );
            // The one server has already answered; a second request would find
            // nothing listening and come back as a failure, not as this.
            let second = turning.asking_for_the_next_request(
                WHAT_NEXT,
                "a-model",
                permitting(InferenceSource::ThisMachine),
                &answers,
                &places,
                noon(),
            );
            (first.unwrap_err(), second.unwrap_err())
        });
        server.join().unwrap();
        assert!(
            matches!(
                first,
                NoAnswer::NotRecorded {
                    after_it_left: false,
                    ..
                }
            ),
            "{first:?}"
        );
        assert!(matches!(second, NoAnswer::TurnClosed), "{second:?}");
        assert!(indicator.is_quiet());
    }
}
