//! The other door that does not leave: a question, and a service somebody runs
//! on this machine.
//!
//! vLLM, llama.cpp's server or LM Studio, listening on loopback. alo OS did not
//! install it, cannot list, fetch, load or remove models on it, and has no
//! business managing it — but it is **this machine** in the only sense law 1
//! cares about, so a question put to it goes nowhere and shows nothing.
//!
//! Item 18a answered what such a service *is* and deliberately did not build
//! it. This is the building of it, and the reason it was its own item is the
//! paragraph below.
//!
//! # The second road to a socket, in a crate designed to have one
//!
//! [`crate::Asking::to_a_provider`] reaches `openai.rs` holding an
//! `alo_egress::Departing`, which only `alo_egress::Indicator::beginning`
//! makes, so a question cannot leave without a person having been shown it
//! leaving. This door reaches the same wire holding **nothing of the kind**. A
//! [`Served`] pointed at `https://api.mistral.ai` would therefore be a way to
//! send somebody's question to a provider with the indicator quiet — which is
//! law 1 failing in the exact manner law 1 exists to prevent.
//!
//! So the address is policed here, at construction, and not at the moment of
//! asking: [`Served::at`] refuses a provider whose address is not this machine,
//! and there is no other constructor. **What may be reached without an
//! indicator is decided by whether a value exists**, which is
//! `alo_files::Touching` and `alo_egress::Departing`'s shape brought to the one
//! place in this crate that had no token of its own.
//!
//! # And the rule it polices is `alo-models`', not a second opinion
//!
//! [`Served::at`] asks `alo_models::Provider::source` whether this address is
//! this machine, which is the same question `alo_models::Provider::checked`
//! asks before permitting `http://` and the same one the indicator's silence
//! rests on. A check written here would be a second rule about loopback, and
//! two rules about loopback is one machine able to disagree with itself about
//! whether a question left. `alo_models::address` has the parsing and the three
//! addresses that used to fool it.
//!
//! # What is the same as the runtime door, and what is not
//!
//! Same: no `alo_egress::Indicator`, no `alo_egress::Departing`, no
//! `alo_models::SourcePolicy` asked, and an [`crate::Answer`] that says *on
//! this machine*. Same refusals, in [`crate::NotAnswered`]. Same
//! never-a-fallback: a service that does not answer hands back an
//! `alo_answering::Failed`, whose only way onward is an offer somebody
//! answered.
//!
//! Not the same: **a key can be sent here.** vLLM started with `--api-key`
//! refuses one that is wrong, and that is an ordinary thing which really
//! happens on somebody's own machine. `alo_answering::WentWrong::KeyNotAccepted`
//! stopped being impossible on `InferenceSource::ThisMachine` because of this
//! file, and `alo_answering::wrong` records why. The runtime alo OS ships is
//! still never given a key, and what keeps that true is
//! [`crate::locally`]'s mapping, which has no arm that can produce that reason.
//!
//! Not the same either: **this machine is not asked to identify itself.** An
//! answer from a service here says *on this machine* and not the name a person
//! gave it, because that is what item 18a decided `InferenceSource::ThisMachine`
//! means — where the answer came from, not who wrote the server.

use std::time::Duration;

use alo_models::{InferenceSource, Provider, Secret};

use crate::answer::Answer;
use crate::asking::Asking;
use crate::openai;
use crate::question::Question;
use crate::refusing::{Miswired, NotAnswered};

/// How long this machine waits for itself.
///
/// Five minutes, the same as `alo_models::ollama` waits for the runtime and
/// more than double what [`crate::hosted`] waits for a provider. ADR 0007 makes
/// the CPU the default, so a model on this machine thinking for minutes is the
/// ordinary case rather than a fault — and a person told *nothing answered*
/// about a machine that was working would go looking for a problem that is not
/// there.
///
/// `pub(crate)` so that `hosted.rs` can assert the asymmetry rather than leave
/// it to be noticed.
pub(crate) const WHILE_THIS_MACHINE_THINKS: Duration = Duration::from_secs(300);

/// An OpenAI-compatible service on this machine, and the key it was given.
///
/// **The only thing that may be reached without an `alo_egress::Departing`**,
/// and the only way to have one is [`Served::at`], which refuses every address
/// that is not this machine. Borrowed rather than owned, for the length of one
/// question, as `alo_models::Hosted` is: neither the provider nor the key is
/// kept anywhere by this.
#[derive(Debug)]
pub struct Served<'a> {
    /// Where to reach it. Its name and region are not used: an answer from here
    /// says *on this machine*.
    provider: &'a Provider,
    /// The key, for a service somebody started with one.
    key: Option<&'a Secret>,
}

impl<'a> Served<'a> {
    /// This service, with this key — which is [`None`] for one that needs none,
    /// as most on somebody's own machine do.
    ///
    /// # Errors
    /// [`Miswired::ReachesOffThisMachine`] when the address is not this
    /// machine. Nothing is opened and nothing is sent: this is the refusal the
    /// whole file exists for, and it is made before there is anything to refuse
    /// it *with*.
    pub fn at(provider: &'a Provider, key: Option<&'a Secret>) -> Result<Self, Miswired> {
        // `alo-models`' rule, asked rather than repeated. A provider whose
        // address is this machine reports as this machine, and one that only
        // looks like it does not.
        if provider.source() != InferenceSource::ThisMachine {
            return Err(Miswired::ReachesOffThisMachine);
        }
        Ok(Self { provider, key })
    }

    /// Where an answer from this service says it came from, which is always
    /// this machine.
    ///
    /// A method rather than a constant so that a caller building the
    /// `alo_answering::Answering` for this door reads it off the thing it is
    /// about, in the way [`crate::Hosted::named_source`] is read off a
    /// provider.
    #[must_use]
    pub fn source(&self) -> InferenceSource {
        InferenceSource::ThisMachine
    }

    /// Put the question, and read what comes back.
    ///
    /// Private, and the only caller is
    /// [`Asking::to_a_service_on_this_machine`]: a public one here would be a
    /// way to reach the wire without the permission having been checked.
    fn ask(&self, question: &Question) -> Result<String, alo_answering::WentWrong> {
        openai::put(
            &self.provider.endpoint,
            self.key,
            question,
            WHILE_THIS_MACHINE_THINKS,
        )
    }
}

impl Asking<'_> {
    /// Put the question to an OpenAI-compatible service on this machine.
    ///
    /// Takes `self`, as both other doors do and for the same reason: an
    /// `alo_answering::Answering` means one attempt, and this is where that
    /// attempt happens.
    ///
    /// ```compile_fail
    /// use alo_answering::Answering;
    /// use alo_asking::{Asking, Question, Served};
    /// use alo_capability::Grantee;
    /// use alo_models::{InferenceSource, Provider, Region, SourcePolicy};
    ///
    /// # fn main() {
    /// let mail = Grantee::named("@mail");
    /// let service = Provider::checked("vLLM", "http://127.0.0.1:8000", Region::Unknown, None)
    ///     .expect("a service on this machine");
    /// let served = Served::at(&service, None).expect("it is on this machine");
    /// let question = Question::asked("may the tenant sublet?", "a-model").expect("a question");
    /// let answering = Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere)
    ///     .expect("nothing forbids answering here");
    /// let asking = Asking::by(&mail, answering, &[], &SourcePolicy::Anywhere);
    ///
    /// let _once = asking.to_a_service_on_this_machine(&question, &served);
    /// let _twice = asking.to_a_service_on_this_machine(&question, &served);
    /// # }
    /// ```
    ///
    /// Checked by unmarking it, as every `compile_fail` in this workspace is:
    /// it fails with **E0382, use of moved value**, and not on a typo. The twin
    /// that passes is `one_permission_is_one_attempt` below.
    ///
    /// # Errors
    /// [`NotAnswered`], the same two things [`Asking::to_this_machine`] answers
    /// with: law 1's refusals do not exist on this path either, because nothing
    /// on it goes anywhere.
    pub fn to_a_service_on_this_machine(
        self,
        question: &Question,
        served: &Served<'_>,
    ) -> Result<Answer, NotAnswered> {
        let source = self.answering.source().clone();
        match &source {
            InferenceSource::ThisMachine => {}
            // The person chose a provider. Answering them from something on
            // this machine would give them a different answer wearing the same
            // face, which is the half of ADR 0008 that runs the other way.
            InferenceSource::Hosted { .. } => return Err(Miswired::NotOnThisMachine.into()),
            InferenceSource::PairedMachine { .. } => {
                return Err(Miswired::NoPathToAPairedMachine.into());
            }
        }

        // No policy is asked, no indicator is shown and no departure is made.
        // There is nothing here for any of the three to be about, and the
        // reason that is true rather than assumed is that `served` exists.
        match served.ask(question) {
            Ok(said) => Ok(Answer::new(said, source, question.of().to_owned())),
            Err(why) => match self.answering.did_not_answer(why, self.others, self.policy) {
                Ok(failed) => Err(NotAnswered::DidNotAnswer(Box::new(failed))),
                // A reason that could not have happened where it is reported.
                // Since this door can send a key, the one refusal
                // `alo-answering` makes is not reachable from here either — but
                // a reporter does not get to assume it passes its own reader's
                // check.
                Err(reported) => Err(reported.into()),
            },
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, mistral_source, serving, serving_with, translated};
    use alo_answering::{Answering, WentWrong};
    use alo_capability::Grantee;
    use alo_egress::Indicator;
    use alo_models::{Region, SourcePolicy};

    /// One answer, in the shape every OpenAI-compatible service replies with.
    const AN_ANSWER: &str = r#"{"choices":[{"index":0,"message":{"role":"assistant","content":"No, not without written consent."}}]}"#;

    fn mail() -> Grantee {
        Grantee::named("@mail")
    }

    fn question() -> Question {
        Question::asked("may the tenant sublet?", "a-local-model").unwrap()
    }

    fn here() -> Answering {
        Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere).unwrap()
    }

    /// A service somebody runs, at this address.
    fn service(endpoint: &str) -> Provider {
        Provider::checked("vLLM", endpoint, Region::Unknown, None).unwrap()
    }

    /// The failure, if it was asked and did not answer.
    fn did_not_answer(not_answered: NotAnswered) -> Option<alo_answering::Failed> {
        match not_answered {
            NotAnswered::DidNotAnswer(failed) => Some(*failed),
            NotAnswered::Miswired(_) => None,
        }
    }

    /// The wiring mistake, if that is what this was.
    fn miswired(not_answered: NotAnswered) -> Option<Miswired> {
        match not_answered {
            NotAnswered::Miswired(miswired) => Some(miswired),
            NotAnswered::DidNotAnswer(_) => None,
        }
    }

    /// **The whole path, in order.** The question goes to the service, the
    /// answer comes back knowing it came from this machine, and there is no
    /// indicator anywhere in the call.
    #[test]
    fn a_question_is_answered_here_and_the_answer_knows_it_never_left() {
        let (url, server) = serving(AN_ANSWER, 200);
        let provider = service(&url);
        let served = Served::at(&provider, None).unwrap();
        let answer = Asking::by(&mail(), here(), &[], &SourcePolicy::Anywhere)
            .to_a_service_on_this_machine(&question(), &served)
            .unwrap();
        let request = server.join().unwrap();

        assert_eq!(answer.text(), "No, not without written consent.");
        assert_eq!(answer.source(), &InferenceSource::ThisMachine);
        assert_eq!(answer.model(), "a-local-model");
        assert_eq!(answer.came_from(&in_english()).text(), "on this machine");
        assert!(
            request.starts_with("POST /v1/chat/completions "),
            "{request}"
        );
    }

    /// **The refusal this whole file exists for.** An address that is not this
    /// machine cannot become a [`Served`], so there is no value with which to
    /// reach the wire without law 1 having shown anything — including the three
    /// addresses that read as loopback and are not.
    #[test]
    fn a_service_that_is_not_on_this_machine_cannot_be_made_at_all() {
        for endpoint in [
            "https://api.mistral.ai",
            "https://api.mistral.ai/v1",
            "https://localhost.attacker.example",
            "https://127.0.0.1.attacker.example/v1",
            "https://127.0.0.1@attacker.example/",
        ] {
            let provider = service(endpoint);
            assert_eq!(
                Served::at(&provider, None).unwrap_err(),
                Miswired::ReachesOffThisMachine,
                "{endpoint}"
            );
        }
    }

    /// And the addresses that really are this machine are, however they are
    /// written — so the refusal above is a refusal rather than a door nobody
    /// can open.
    #[test]
    fn the_addresses_that_really_are_this_machine_open_the_door() {
        for endpoint in [
            "http://127.0.0.1:8000",
            "http://127.0.0.1:8000/v1",
            "http://localhost:1234",
            "http://[::1]:8080",
            "https://localhost:8443",
        ] {
            let provider = service(endpoint);
            let served = Served::at(&provider, None).unwrap();
            assert_eq!(served.source(), InferenceSource::ThisMachine, "{endpoint}");
        }
    }

    /// **Zero inference egress, as far as a type can carry it.** An indicator
    /// held beside a whole day of questions answered by a service on this
    /// machine is quiet at the end of it, because this door has no parameter to
    /// be given one and no way to make a departure.
    #[test]
    fn a_working_day_on_this_machine_puts_nothing_on_the_indicator() {
        let indicator = Indicator::default();
        let mail = mail();
        for _ in 0..4 {
            let (url, server) = serving(AN_ANSWER, 200);
            let provider = service(&url);
            let served = Served::at(&provider, None).unwrap();
            let asking = Asking::by(&mail, here(), &[], &SourcePolicy::ThisMachineOnly);
            assert!(
                asking
                    .to_a_service_on_this_machine(&question(), &served)
                    .is_ok()
            );
            server.join().unwrap();
        }
        assert!(indicator.is_quiet());
        assert_eq!(indicator.showing().len(), 0);
        // And law 1 refuses to make a departure for this place at all, which is
        // the guarantee underneath the absence of a parameter.
        assert!(alo_egress::Leaving::asking(&mail, &InferenceSource::ThisMachine).is_err());
    }

    /// **No rule can stop a machine answering its own question**, so this door
    /// asks none — walked over every policy, as the runtime door is, so a rule
    /// added later that did forbid it fails here rather than being permitted by
    /// a door that never asks.
    #[test]
    fn no_rule_can_stop_this_machine_answering_its_own_question() {
        let mail = mail();
        for policy in [
            SourcePolicy::Anywhere,
            SourcePolicy::InTheBuilding,
            SourcePolicy::InRegion("Switzerland".to_owned()),
            SourcePolicy::ThisMachineOnly,
        ] {
            assert!(policy.permits(&InferenceSource::ThisMachine), "{policy:?}");
            let (url, server) = serving(AN_ANSWER, 200);
            let provider = service(&url);
            let served = Served::at(&provider, None).unwrap();
            let answer = Asking::by(
                &mail,
                Answering::chosen(InferenceSource::ThisMachine, &policy).unwrap(),
                &[],
                &policy,
            )
            .to_a_service_on_this_machine(&question(), &served)
            .unwrap();
            server.join().unwrap();
            assert_eq!(
                answer.text(),
                "No, not without written consent.",
                "{policy:?}"
            );
        }
    }

    /// **A permission for somewhere else opens no connection**, and the refusal
    /// names the door that place is behind rather than offering this one as a
    /// substitute for it.
    #[test]
    fn a_permission_for_somewhere_else_reaches_no_service_at_all() {
        for (permitted_place, expected) in [
            (mistral_source(), Miswired::NotOnThisMachine),
            (
                InferenceSource::Hosted {
                    provider: "alo".to_owned(),
                    region: Region::Unknown,
                },
                Miswired::NotOnThisMachine,
            ),
            (
                InferenceSource::PairedMachine {
                    machine: "the studio workstation".to_owned(),
                },
                Miswired::NoPathToAPairedMachine,
            ),
        ] {
            // Nothing is listening on this address, so a question that reached
            // it would come back as a failure rather than as a refusal — which
            // is how this test tells "nothing was sent" from "it was sent".
            let provider = service("http://127.0.0.1:1");
            let served = Served::at(&provider, None).unwrap();
            let not_answered = Asking::by(
                &mail(),
                Answering::chosen(permitted_place.clone(), &SourcePolicy::Anywhere).unwrap(),
                &[],
                &SourcePolicy::Anywhere,
            )
            .to_a_service_on_this_machine(&question(), &served)
            .unwrap_err();

            assert_eq!(
                miswired(not_answered),
                Some(expected),
                "{permitted_place:?}"
            );
        }
    }

    /// **A service on this machine can refuse a key, and the person is told
    /// that is what happened.** vLLM started with `--api-key` does exactly
    /// this, and reporting it as anything else would send somebody to look at a
    /// service that is working.
    #[test]
    fn a_key_this_service_refuses_is_reported_as_a_refused_key() {
        let (url, server) = serving(r#"{"error":"Unauthorized"}"#, 401);
        let provider = service(&url);
        let key = Secret::typed("not-the-key-vllm-was-started-with").unwrap();
        let served = Served::at(&provider, Some(&key)).unwrap();
        let not_answered = Asking::by(&mail(), here(), &[], &SourcePolicy::Anywhere)
            .to_a_service_on_this_machine(&question(), &served)
            .unwrap_err();
        server.join().unwrap();

        let failed = did_not_answer(not_answered).unwrap();
        assert_eq!(failed.why(), WentWrong::KeyNotAccepted);
        assert_eq!(failed.source(), &InferenceSource::ThisMachine);
        assert!(
            failed
                .said(&in_english())
                .text()
                .contains("the key for this provider was not accepted"),
            "{}",
            failed.said(&in_english())
        );
    }

    /// **A service on loopback that answers with a redirect does not get to
    /// carry the question off the machine.** It is the one way this door could
    /// have caused an egress with the indicator quiet, and it is refused rather
    /// than followed — the person is told their machine stopped it.
    #[test]
    fn a_local_service_cannot_redirect_a_question_off_this_machine() {
        let (url, server) = serving_with(
            "{}",
            302,
            "Location: https://api.mistral.ai/v1/chat/completions\r\n",
        );
        let provider = service(&url);
        let served = Served::at(&provider, None).unwrap();
        let indicator = Indicator::default();
        let not_answered = Asking::by(&mail(), here(), &[], &SourcePolicy::Anywhere)
            .to_a_service_on_this_machine(&question(), &served)
            .unwrap_err();
        server.join().unwrap();

        let failed = did_not_answer(not_answered).unwrap();
        assert_eq!(failed.why(), WentWrong::SentSomewhereElse);
        assert!(indicator.is_quiet());
    }

    /// **Never a silent fallback, on the door where it would be cheapest to
    /// write.** The machine has a provider it could ask and does not: the other
    /// place is an *offer* nobody has answered, and the person is told outright
    /// that nothing was sent anywhere.
    #[test]
    fn a_local_failure_asks_no_provider_and_the_person_is_told_so() {
        let provider = service("http://127.0.0.1:1");
        let served = Served::at(&provider, None).unwrap();
        let others = [mistral_source()];
        let not_answered = Asking::by(&mail(), here(), &others, &SourcePolicy::Anywhere)
            .to_a_service_on_this_machine(&question(), &served)
            .unwrap_err();

        let failed = did_not_answer(not_answered).unwrap();
        assert_eq!(failed.why(), WentWrong::NothingAnswered);
        assert_eq!(failed.source(), &InferenceSource::ThisMachine);
        assert_eq!(failed.elsewhere().offers().len(), 1);
        assert_eq!(
            failed.nothing_was_sent(&in_english()).text(),
            "nothing was sent anywhere, and nothing will be unless you say so"
        );
    }

    /// **The failure a person reads is in their own language**, and *on this
    /// machine* inside it is in that language too rather than an English clause
    /// in the middle of a German line.
    #[test]
    fn what_went_wrong_here_is_read_in_the_readers_own_language() {
        let strings = translated(&[
            (alo_models::words::ON_THIS_MACHINE, "auf diesem Rechner"),
            (
                alo_answering::words::NOTHING_ANSWERED,
                "{source} hat nicht geantwortet",
            ),
        ]);
        let provider = service("http://127.0.0.1:1");
        let served = Served::at(&provider, None).unwrap();
        let not_answered = Asking::by(&mail(), here(), &[], &SourcePolicy::Anywhere)
            .to_a_service_on_this_machine(&question(), &served)
            .unwrap_err();
        let said = did_not_answer(not_answered).unwrap().said(&strings);
        assert!(said.is_translated(), "{said}");
        assert_eq!(said.text(), "auf diesem Rechner hat nicht geantwortet");
    }

    /// The passing twin of the `compile_fail` on this door: one permission is
    /// one attempt, and this is what spending it looks like.
    #[test]
    fn one_permission_is_one_attempt() {
        let provider = service("http://127.0.0.1:1");
        let served = Served::at(&provider, None).unwrap();
        let not_answered = Asking::by(&mail(), here(), &[], &SourcePolicy::Anywhere)
            .to_a_service_on_this_machine(&question(), &served)
            .unwrap_err();
        assert_eq!(
            did_not_answer(not_answered).map(|failed| failed.why()),
            Some(WentWrong::NothingAnswered)
        );
    }

    /// **This machine waits five minutes for itself**, which is the runtime's
    /// number rather than a second opinion about the same ADR.
    #[test]
    fn this_machine_is_waited_on_as_long_as_the_runtime_is() {
        assert_eq!(WHILE_THIS_MACHINE_THINKS, Duration::from_secs(300));
    }
}
