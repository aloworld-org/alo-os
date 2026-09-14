//! The road a turn's question takes to a machine this person is paired with,
//! held up from the outside.
//!
//! The road itself is [`crate::Answers::PairedMachine`] and one arm of
//! `Turning::putting` in `crate::asking`, which shares everything with the
//! provider's arm — the boundary, the departure, the record. What is different
//! about going down the corridor is what can come back: an answer from the
//! model **that** machine's person chose, or that machine's own word for why it
//! will not answer. This file is the tests of both, kept beside `asking.rs`
//! rather than inside it, which was already the longest file in the crate.
//!
//! Every request here is read off a real socket, for `alo-asking`'s reason:
//! what is worth testing is what crossed the wire.

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::SocketAddr;

    use alo_answering::{Answering, RefusedThere, WentWrong};
    use alo_asking::{DownTheCorridor, Miswired, THE_PROOF_HEADER, THE_QUESTION_PATH};
    use alo_capability::Grants;
    use alo_context::Context;
    use alo_egress::{Destination, Indicator};
    use alo_files::OnThisMachine;
    use alo_models::{InferenceSource, SourcePolicy};
    use alo_nearby::{MayAskIts, Pairings};
    use alo_record::{Asking as AskingAbout, Happened, Only, Record};
    use alo_strings::{Strings, Word};

    use crate::answers::Answers;
    use crate::machine::Machine;
    use crate::places::Places;
    use crate::testing::{
        AN_ANSWER, NothingIsBounded, Stub, hour, in_english, noon, paired_with_the_studio, serving,
        the_studio, this_machine, translated,
    };
    use crate::turning::Turning;
    use crate::unanswered::NoAnswer;

    /// What the person here called the studio machine.
    const CALLED: &str = "the studio machine";

    /// The model a question down the corridor names — `alo_choosing::WHAT_THAT_MACHINE_CHOSE` —
    /// which the machine that answers sets aside for the one its own person chose.
    const THE_MODEL_NAMED: &str = "what-that-machine-chose";

    /// What an agent asks.
    const ASKED: &str = "how many invoices are unpaid?";

    /// The rule these tests run under: nothing leaves the building, which a
    /// paired machine does not.
    const IN_THE_BUILDING: SourcePolicy = SourcePolicy::InTheBuilding;

    /// Where a test server listens, as the address discovery would have
    /// measured.
    fn at(url: &str) -> SocketAddr {
        url.trim_start_matches("http://").parse().unwrap()
    }

    /// The corridor to the studio, as this machine's pairings permit it.
    fn the_corridor<'a>(pairings: &Pairings, to: SocketAddr) -> DownTheCorridor<'a> {
        DownTheCorridor::paired(
            pairings,
            &this_machine(),
            &the_studio(),
            CALLED,
            to,
            None,
            noon(),
        )
        .unwrap()
    }

    /// The person's setting, spent on one question: the studio machine.
    fn permitted_down_the_corridor() -> Answering {
        Answering::chosen(
            InferenceSource::PairedMachine {
                machine: CALLED.to_owned(),
            },
            &IN_THE_BUILDING,
        )
        .unwrap()
    }

    /// One turn by `@files` on a machine speaking `strings`, ending when the
    /// closure is done.
    fn on_a_machine<T>(
        strings: &Strings,
        record: &mut Record,
        indicator: &mut Indicator,
        doing: impl FnOnce(&mut Turning<'_, '_>) -> T,
    ) -> T {
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            strings,
            &OnThisMachine,
            &mut bounding,
            indicator,
            record,
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

    /// Everything after the request's head.
    fn the_body_of(request: &str) -> &str {
        request.split_once("\r\n\r\n").map_or("", |(_, body)| body)
    }

    /// How many entries say something left.
    fn departures(record: &Record) -> usize {
        record
            .answering(&AskingAbout::anything().only(Only::Egress))
            .count()
    }

    /// **A turn's question to the paired machine goes down the corridor under a
    /// departure the record keeps as left, naming the machine** — the request
    /// on the question path carrying its proof — **beside a local answer that
    /// leaves nothing at all.**
    #[test]
    fn a_turns_question_to_the_paired_machine_leaves_naming_it_and_a_local_answer_leaves_nothing() {
        let pairings = paired_with_the_studio(&[MayAskIts::Models]);
        let (url, server) = serving(AN_ANSWER, 200);
        let corridor = the_corridor(&pairings, at(&url));
        let strings = in_english();
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let answer = on_a_machine(&strings, &mut record, &mut indicator, |turning| {
            turning.asking(
                ASKED,
                THE_MODEL_NAMED,
                permitted_down_the_corridor(),
                &Answers::PairedMachine(corridor),
                &Places::under(&IN_THE_BUILDING),
                noon(),
            )
        })
        .unwrap();
        let request = server.join().unwrap();

        assert!(
            request.starts_with(&format!("POST {THE_QUESTION_PATH} ")),
            "{request}"
        );
        assert!(
            request
                .to_ascii_lowercase()
                .contains(&format!("{THE_PROOF_HEADER}:")),
            "the question went down the corridor without its proof: {request}"
        );
        assert!(the_body_of(&request).contains("unpaid"));
        assert_eq!(answer.text(), "No, not without written consent.");
        assert_eq!(
            answer.source(),
            &InferenceSource::PairedMachine {
                machine: CALLED.to_owned()
            }
        );
        // Shown while it left — `Entry::left` is made only from the departure
        // the indicator hands out — and off the indicator now it has ended.
        assert!(indicator.is_quiet());
        assert_eq!(departures(&record), 1);
        assert!(record.everything().any(|entry| matches!(
            entry.happened(),
            Happened::Left {
                destination: Destination::PairedMachine { machine },
                ..
            } if machine == CALLED
        )));

        // And beside it, the same turn's question answered on this machine
        // leaves nothing: no departure, nothing shown.
        let runtime = Stub::answering("Three.");
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let here = on_a_machine(&strings, &mut record, &mut indicator, |turning| {
            turning.asking(
                ASKED,
                "a-model",
                Answering::chosen(InferenceSource::ThisMachine, &IN_THE_BUILDING).unwrap(),
                &Answers::Runtime(&runtime),
                &Places::under(&IN_THE_BUILDING),
                noon(),
            )
        })
        .unwrap();
        assert_eq!(here.source(), &InferenceSource::ThisMachine);
        assert!(indicator.is_quiet());
        assert_eq!(departures(&record), 0);
    }

    /// Put one question down the corridor to a machine that answers with this
    /// status and body, on a machine reading German, and hand back what the
    /// agent is told and what the record kept.
    fn refused_there(
        status: u16,
        body: &'static str,
        translated_as: &[(Word, &str)],
    ) -> (NoAnswer, Strings, Record) {
        let pairings = paired_with_the_studio(&[MayAskIts::Models]);
        let (url, server) = serving(body, status);
        let corridor = the_corridor(&pairings, at(&url));
        let mut words = vec![(
            alo_models::words::ON_A_PAIRED_MACHINE,
            "auf {machine}, in Ihrem Netzwerk",
        )];
        words.extend_from_slice(translated_as);
        let strings = translated(&words);
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let refused = on_a_machine(&strings, &mut record, &mut indicator, |turning| {
            turning.asking(
                ASKED,
                THE_MODEL_NAMED,
                permitted_down_the_corridor(),
                &Answers::PairedMachine(corridor),
                &Places::under(&IN_THE_BUILDING),
                noon(),
            )
        })
        .unwrap_err();
        server.join().unwrap();
        assert!(indicator.is_quiet());
        (refused, strings, record)
    }

    /// What a refusal from the paired machine reached the agent as: the
    /// reason, and the sentence in this machine's language.
    fn what_the_agent_reads(refused: &NoAnswer, strings: &Strings) -> (WentWrong, String) {
        let NoAnswer::DidNotAnswer(failed) = refused else {
            unreachable!("a refusal from the paired machine came back as {refused:?}")
        };
        let said = refused.said(strings).unwrap();
        assert!(said.is_translated(), "{said}");
        (failed.why(), said.into_text())
    }

    /// **A pairing the other machine keeps without its models reaches the agent
    /// as that machine's own word**, rendered in the language this machine
    /// reads — and it is a departure, because the question went.
    #[test]
    fn the_paired_machine_saying_not_permitted_reaches_the_agent_in_this_machines_language() {
        let (refused, strings, record) = refused_there(
            403,
            "not-permitted\n",
            &[(
                alo_answering::words::NOT_PERMITTED_THERE,
                "{source} wurde nichts beantwortet — die Kopplung erlaubt es nicht",
            )],
        );
        let (why, said) = what_the_agent_reads(&refused, &strings);
        assert_eq!(why, WentWrong::RefusedThere(RefusedThere::NotPermitted));
        assert!(said.contains("die Kopplung erlaubt es nicht"), "{said}");
        assert!(said.contains(CALLED), "{said}");
        assert_eq!(departures(&record), 1);
    }

    /// **A machine whose person chose a provider says so in its own word**, and
    /// the agent here reads it in this machine's language.
    #[test]
    fn the_paired_machine_saying_it_answers_elsewhere_reaches_the_agent_in_this_machines_language()
    {
        let (refused, strings, record) = refused_there(
            503,
            "answers-elsewhere\n",
            &[(
                alo_answering::words::ANSWERS_ELSEWHERE_THERE,
                "{source} wurde nichts beantwortet — dort antwortet ein Anbieter",
            )],
        );
        let (why, said) = what_the_agent_reads(&refused, &strings);
        assert_eq!(why, WentWrong::RefusedThere(RefusedThere::AnswersElsewhere));
        assert!(said.contains("dort antwortet ein Anbieter"), "{said}");
        assert_eq!(departures(&record), 1);
    }

    /// **A machine where nothing is chosen says so in its own word**, and the
    /// agent here reads it in this machine's language.
    #[test]
    fn the_paired_machine_saying_nothing_is_chosen_there_reaches_the_agent_in_this_machines_language()
     {
        let (refused, strings, record) = refused_there(
            503,
            "not-answered-here\n",
            &[(
                alo_answering::words::NOTHING_CHOSEN_THERE,
                "{source} wurde nichts beantwortet — dort ist nichts gewählt",
            )],
        );
        let (why, said) = what_the_agent_reads(&refused, &strings);
        assert_eq!(
            why,
            WentWrong::RefusedThere(RefusedThere::NothingChosenThere)
        );
        assert!(said.contains("dort ist nichts gewählt"), "{said}");
        assert_eq!(departures(&record), 1);
    }

    /// **An agent's next request down the corridor is asked exactly as a
    /// question in words is** (ADR 0032, decision 4): the two bodies read off
    /// the socket are the same bytes, neither holds a schema, and both leave.
    #[test]
    fn an_agents_next_request_down_the_corridor_is_asked_exactly_as_a_question_in_words() {
        let pairings = paired_with_the_studio(&[MayAskIts::Models]);
        let strings = in_english();
        let mut bodies = Vec::new();
        for next_request in [false, true] {
            let (url, server) = serving(AN_ANSWER, 200);
            let corridor = the_corridor(&pairings, at(&url));
            let mut indicator = Indicator::default();
            let mut record = Record::default();
            on_a_machine(&strings, &mut record, &mut indicator, |turning| {
                let answers = Answers::PairedMachine(corridor);
                let places = Places::under(&IN_THE_BUILDING);
                if next_request {
                    turning.asking_for_the_next_request(
                        ASKED,
                        THE_MODEL_NAMED,
                        permitted_down_the_corridor(),
                        &answers,
                        &places,
                        noon(),
                    )
                } else {
                    turning.asking(
                        ASKED,
                        THE_MODEL_NAMED,
                        permitted_down_the_corridor(),
                        &answers,
                        &places,
                        noon(),
                    )
                }
            })
            .unwrap();
            assert_eq!(departures(&record), 1);
            bodies.push(the_body_of(&server.join().unwrap()).to_owned());
        }
        let [in_words, for_the_next] = bodies.as_slice() else {
            unreachable!("two roads, two requests")
        };
        assert_eq!(
            in_words, for_the_next,
            "the paired machine was asked differently"
        );
        assert!(!for_the_next.contains("format"), "{for_the_next}");
    }

    /// **A paired machine is never a fallback, in either direction.** A
    /// permission for the studio handed a model on this machine asks that
    /// model nothing; a permission for this machine, or for a provider, handed
    /// the corridor sends nothing down it. Nothing is shown and nothing kept.
    #[test]
    fn a_paired_machine_is_never_a_fallback_for_a_local_model_or_a_provider_nor_they_for_it() {
        let strings = in_english();

        // The studio chosen, something here offered instead.
        let runtime = Stub::answering("an answer nobody chose");
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let refused = on_a_machine(&strings, &mut record, &mut indicator, |turning| {
            turning.asking(
                ASKED,
                "a-model",
                permitted_down_the_corridor(),
                &Answers::Runtime(&runtime),
                &Places::under(&IN_THE_BUILDING),
                noon(),
            )
        })
        .unwrap_err();
        assert!(
            matches!(
                refused,
                NoAnswer::Miswired(Miswired::BelongsDownTheCorridor)
            ),
            "{refused:?}"
        );
        assert_eq!(runtime.times_asked(), 0);
        assert!(indicator.is_quiet());
        assert_eq!(record.len(), 0);

        // This machine, or a provider, chosen — and the corridor offered. The
        // address listens nowhere, so a request would come back as a failure
        // rather than as this refusal.
        let pairings = paired_with_the_studio(&[MayAskIts::Models]);
        let nowhere: SocketAddr = "127.0.0.1:1".parse().unwrap();
        for chosen in [
            InferenceSource::ThisMachine,
            crate::testing::mistral_source(),
        ] {
            let mut indicator = Indicator::default();
            let mut record = Record::default();
            let refused = on_a_machine(&strings, &mut record, &mut indicator, |turning| {
                turning.asking(
                    ASKED,
                    "a-model",
                    Answering::chosen(chosen.clone(), &SourcePolicy::Anywhere).unwrap(),
                    &Answers::PairedMachine(the_corridor(&pairings, nowhere)),
                    &Places::under(&SourcePolicy::Anywhere),
                    noon(),
                )
            })
            .unwrap_err();
            assert!(
                matches!(refused, NoAnswer::Miswired(Miswired::NotAPairedMachine)),
                "{chosen:?}: {refused:?}"
            );
            assert!(refused.nothing_left());
            assert!(indicator.is_quiet());
            assert_eq!(record.len(), 0);
        }
    }
}
