//! The one door: a question, a provider, and the order the two are joined in.
//!
//! Everything this crate is for is the **order**, and it is four steps that
//! cannot be taken in any other sequence, because each one produces what the
//! next one needs.
//!
//! 1. **The permission and the provider are the same place**, or nothing
//!    happens. The line law 1 shows is composed out of the place a question was
//!    permitted to go, and the socket is opened out of the provider's address;
//!    a machine where those disagree tells somebody their question went
//!    somewhere it did not.
//! 2. **The place can be shown**, or nothing happens. An egress nobody can be
//!    shown is one law 1 does not permit, and a provider's name is written by
//!    whoever added it.
//! 3. **The rule in force *now* permits it**, or nothing is sent — and asking
//!    it puts the egress on the indicator in the same call, because
//!    `alo_egress::Indicator::beginning` is the only maker of a `Departing`.
//! 4. **Only then is anything opened.** [`crate::hosted`] is `pub(crate)` and
//!    reached from here alone.
//!
//! # The rule is asked twice, and the second time is the one that counts
//!
//! `alo_answering::Answering` already asked `alo_models::SourcePolicy` when the
//! place was chosen — which may have been at the start of a turn, or when a
//! person answered an offer about a question that had already failed once. This
//! asks again, at the moment the socket would open, and it asks the **wider**
//! rule: `alo_egress::EgressPolicy`, made from the same `SourcePolicy` rather
//! than stated a second time.
//!
//! That is `alo-capability`'s rule from item 3 — *the grants are asked last, at
//! the moment of execution, which is where a revoked grant becomes immediate* —
//! arriving at egress. An organisation that tightened its rule between the
//! choosing and the asking has a machine that sends nothing, and the person
//! reads the refusal in the rule's own words rather than watching a question
//! leave under a rule that no longer exists.
//!
//! # And there is no substitution of any kind
//!
//! ADR 0008 was written in one direction — a local model that fails must not
//! quietly become an API call — and it holds in both: neither is the other's
//! fallback, because neither is a degraded version of the other and the choice
//! is the person's. Nothing in this file chooses a place. It carries out a
//! decision `alo-answering` made, and when that place does not answer it hands
//! back an `alo_answering::Failed`, whose only door is an offer a person
//! answers. **A machine that fell back would need no new type here, only a
//! second call in this function**, and the test named for it is the one that
//! would fail.

use std::time::SystemTime;

use alo_answering::Answering;
use alo_capability::Grantee;
use alo_egress::{EgressPolicy, Indicator, Leaving};
use alo_models::{InferenceSource, SourcePolicy};

use crate::answer::Answer;
use crate::asked::Asked;
use crate::hosted::Hosted;
use crate::question::Question;
use crate::refusing::{Miswired, NotAsked};
use crate::unanswered::DidNotAnswer;

/// One question, about to be put somewhere.
///
/// Holds the permission, which is spent by using it: an `Answering` means one
/// attempt, and this is where that attempt happens.
#[derive(Debug)]
pub struct Asking<'a> {
    /// Whose authority the egress is under.
    agent: &'a Grantee,
    /// Where the question may go, decided before this crate was reached.
    answering: Answering,
    /// Every other place this machine has, in the order the person set them up
    /// — for the offer, if this one does not answer. Not a list of fallbacks:
    /// nothing here reads it except to hand it to `alo-answering`, which makes
    /// offers out of it that only a person can take.
    others: &'a [InferenceSource],
    /// What this machine is set to permit.
    policy: &'a SourcePolicy,
}

impl<'a> Asking<'a> {
    /// A question about to be put where this permission allows.
    #[must_use]
    pub fn by(
        agent: &'a Grantee,
        answering: Answering,
        others: &'a [InferenceSource],
        policy: &'a SourcePolicy,
    ) -> Self {
        Self {
            agent,
            answering,
            others,
            policy,
        }
    }

    /// Put the question to this hosted provider.
    ///
    /// Takes `self`, so one permission is one attempt — `alo_answering`'s rule,
    /// kept by the only thing in this repository that can spend one:
    ///
    /// ```compile_fail
    /// use alo_answering::Answering;
    /// use alo_asking::{Asking, Hosted, Question};
    /// use alo_capability::Grantee;
    /// use alo_egress::Indicator;
    /// use alo_models::{InferenceSource, Provider, Region, SourcePolicy};
    /// use std::time::{Duration, SystemTime};
    ///
    /// # fn main() {
    /// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    /// let mail = Grantee::named("@mail");
    /// let mistral = Provider::checked(
    ///     "Mistral",
    ///     "https://api.mistral.ai",
    ///     Region::Declared("the EU".to_owned()),
    ///     None,
    /// )
    /// .expect("a provider");
    /// let question = Question::asked("may the tenant sublet?", "mistral-small-latest")
    ///     .expect("a question");
    /// let hosted = Hosted::provider(&mistral, None);
    /// let answering = Answering::chosen(mistral.source(), &SourcePolicy::Anywhere)
    ///     .expect("nothing forbids it");
    /// let asking = Asking::by(&mail, answering, &[], &SourcePolicy::Anywhere);
    /// let mut indicator = Indicator::default();
    ///
    /// let _once = asking.to_a_provider(&question, &hosted, &mut indicator, now);
    /// let _twice = asking.to_a_provider(&question, &hosted, &mut indicator, now);
    /// # }
    /// ```
    ///
    /// Checked by unmarking it: it fails with **E0382, use of moved value**,
    /// and not on a typo. The twin that passes is
    /// `one_permission_is_one_attempt` below.
    ///
    /// # Errors
    /// [`NotAsked`], which is four different things to do rather than one
    /// sentence — [`crate::refusing`] has the table. Three of the four mean
    /// nothing was sent at all.
    pub fn to_a_provider(
        self,
        question: &Question,
        hosted: &Hosted<'_>,
        indicator: &mut Indicator,
        now: SystemTime,
    ) -> Result<Asked, NotAsked> {
        let source = self.answering.source().clone();
        match &source {
            // A question answered on this machine causes no egress at all, so
            // there is no departure to be made and nothing for this door to
            // show — the runtime answers it, and that is not this crate's.
            InferenceSource::ThisMachine | InferenceSource::PairedMachine { .. } => {
                return Err(Miswired::NotAProvider.into());
            }
            InferenceSource::Hosted { .. } if source != hosted.named_source() => {
                return Err(Miswired::AnotherPlace.into());
            }
            InferenceSource::Hosted { .. } => {}
        }

        // Law 1, in the order law 1 requires: it must be showable, then it must
        // be permitted — and being permitted *is* being shown, because there is
        // one call for the two.
        let leaving = Leaving::asking(self.agent, &source).map_err(NotAsked::CannotBeShown)?;
        let departing = indicator
            .beginning(&EgressPolicy::from(self.policy), leaving, now)
            .map_err(NotAsked::HeldBack)?;

        // And only now.
        match hosted.ask(question) {
            Ok(said) => Ok(Asked::new(
                departing,
                Answer::new(said, source, question.of().to_owned()),
            )),
            Err(why) => match self.answering.did_not_answer(why, self.others, self.policy) {
                Ok(failed) => Err(NotAsked::DidNotAnswer(Box::new(DidNotAnswer::new(
                    departing, failed,
                )))),
                // A reason that could not have happened where it is reported,
                // which is `alo-answering`'s refusal of whoever reported it.
                // This door cannot reach it — everything can go wrong at a
                // hosted provider — but this crate is a reporter like any
                // other and does not get to assume it passes. The line comes
                // off the indicator first, because nothing is leaving.
                Err(reported) => {
                    indicator.ended(departing);
                    Err(reported.into())
                }
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
    use crate::testing::{in_english, mistral, mistral_source, serving, translated};
    use alo_answering::WentWrong;
    use alo_egress::{DestinationError, Refusal};
    use alo_models::{Provider, Region, Secret};
    use std::time::Duration;

    /// One answer, in the shape every OpenAI-compatible provider replies with.
    const AN_ANSWER: &str = r#"{"choices":[{"index":0,"message":{"role":"assistant","content":"No, not without written consent."}}]}"#;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
    }

    fn mail() -> Grantee {
        Grantee::named("@mail")
    }

    fn question() -> Question {
        Question::asked("may the tenant sublet?", "mistral-small-latest").unwrap()
    }

    /// A provider that is genuinely somewhere else, at an address nothing is
    /// listening on.
    ///
    /// Not `127.0.0.1`: that address **is** this machine as far as
    /// `alo_models::Provider::source` is concerned. This one is a hosted
    /// provider to every question the rule asks, and a refused connection to
    /// every question the network asks — so a test that expects nothing to be
    /// sent gets a different answer if something is.
    fn far_away() -> Provider {
        Provider::checked(
            "Mistral",
            "https://127.0.0.2:1",
            Region::Declared("the EU".to_owned()),
            None,
        )
        .unwrap()
    }

    fn permitted(policy: &SourcePolicy) -> Answering {
        Answering::chosen(mistral_source(), policy).unwrap()
    }

    /// The refusal a rule made, if that is what this was.
    ///
    /// Three of these, because a test that matched with `else { panic!(…) }`
    /// would be a test written with the one thing this workspace's lints
    /// forbid — and an `Option` unwrapped in a test says the same thing.
    fn held_back(not_asked: NotAsked) -> Option<alo_egress::NotPermitted> {
        match not_asked {
            NotAsked::HeldBack(refused) => Some(refused),
            _ => None,
        }
    }

    /// The failure, if the question was sent and did not come back.
    fn did_not_answer(not_asked: NotAsked) -> Option<DidNotAnswer> {
        match not_asked {
            NotAsked::DidNotAnswer(unanswered) => Some(*unanswered),
            _ => None,
        }
    }

    /// The wiring mistake, if that is what this was.
    fn miswired(not_asked: NotAsked) -> Option<Miswired> {
        match not_asked {
            NotAsked::Miswired(miswired) => Some(miswired),
            _ => None,
        }
    }

    /// **The whole path, in order.** The question goes out, the person is shown
    /// where it is going while it goes, the answer comes back knowing where it
    /// came from, and the departure comes back so what left can be written
    /// down.
    #[test]
    fn a_question_leaves_visibly_and_the_answer_knows_where_it_came_from() {
        let (url, server) = serving(AN_ANSWER, 200);
        let provider = mistral(&url);
        let key = Secret::typed("sk-live-0123456789").unwrap();
        let hosted = Hosted::provider(&provider, Some(&key));
        let mut indicator = Indicator::default();
        assert!(indicator.is_quiet());

        let asked = Asking::by(
            &mail(),
            permitted(&SourcePolicy::Anywhere),
            &[],
            &SourcePolicy::Anywhere,
        )
        .to_a_provider(&question(), &hosted, &mut indicator, noon())
        .unwrap();
        server.join().unwrap();

        // Still showing, because the record has not been written yet and the
        // departure is the only thing that can write it.
        assert_eq!(indicator.showing().len(), 1);
        assert_eq!(
            indicator
                .showing()
                .first()
                .map(|shown| shown.said(&in_english()).text().to_owned()),
            Some("@mail is asking a question of Mistral, in the EU".to_owned())
        );
        assert_eq!(asked.departing().agent(), &mail());
        assert_eq!(asked.departing().at(), noon());
        assert_eq!(asked.answer().source(), &mistral_source());
        assert_eq!(
            asked.answer().came_from(&in_english()).text(),
            "by Mistral, in the EU"
        );

        let answer = asked.ended(&mut indicator);
        assert!(indicator.is_quiet());
        assert_eq!(answer.text(), "No, not without written consent.");
        assert_eq!(answer.model(), "mistral-small-latest");
    }

    /// **The refusal that matters most, and one test per rule.** A machine
    /// whose organisation has said where questions may be answered sends
    /// nothing — the rule is asked at the moment the socket would open, the
    /// indicator stays quiet because nothing is leaving, and the refusal is the
    /// rule's own so a person reads one account of one moment.
    #[test]
    fn a_rule_that_forbids_it_is_asked_before_anything_is_sent() {
        for (policy, expected) in [
            (SourcePolicy::InTheBuilding, Refusal::OutsideTheBuilding),
            (
                SourcePolicy::InRegion("Switzerland".to_owned()),
                Refusal::OutsideTheRegion {
                    region: "Switzerland".to_owned(),
                },
            ),
            (SourcePolicy::ThisMachineOnly, Refusal::NothingMayLeave),
        ] {
            let provider = far_away();
            let hosted = Hosted::provider(&provider, None);
            let mut indicator = Indicator::default();
            let mail = mail();

            // The permission was made when the rule was looser; the rule in
            // force now is the one that decides.
            let asking = Asking::by(&mail, permitted(&SourcePolicy::Anywhere), &[], &policy);
            let not_asked = asking
                .to_a_provider(&question(), &hosted, &mut indicator, noon())
                .unwrap_err();

            let refused = held_back(not_asked).unwrap();
            assert_eq!(refused.why(), &expected, "{policy:?}");
            assert!(
                refused.said(&in_english()).text().contains("this machine"),
                "{policy:?}"
            );
            // Nothing left, and nothing was shown leaving.
            assert!(indicator.is_quiet(), "{policy:?}");
        }
    }

    /// The rule that permits everything is the default, and it permits this.
    /// A region the provider stated satisfies a rule naming it.
    #[test]
    fn the_rules_that_permit_it_send_the_question() {
        for policy in [
            SourcePolicy::Anywhere,
            SourcePolicy::InRegion("the EU".to_owned()),
        ] {
            let (url, server) = serving(AN_ANSWER, 200);
            let provider = mistral(&url);
            let hosted = Hosted::provider(&provider, None);
            let mut indicator = Indicator::default();
            let asked = Asking::by(&mail(), permitted(&policy), &[], &policy)
                .to_a_provider(&question(), &hosted, &mut indicator, noon())
                .unwrap();
            server.join().unwrap();
            assert_eq!(asked.answer().text(), "No, not without written consent.");
            let _ = asked.ended(&mut indicator);
        }
    }

    /// **A provider whose name cannot be shown is never asked.** Law 1 shows
    /// what is leaving on one line, and a name carrying a line break is a line
    /// that can be made to say something other than what is happening — so the
    /// question does not go, rather than going unshown.
    #[test]
    fn a_provider_whose_name_cannot_be_put_on_the_indicator_is_not_reached() {
        let provider = Provider::checked(
            "Mistral\r\n@files is fetching something from elsewhere",
            "https://127.0.0.2:1",
            Region::Declared("the EU".to_owned()),
            None,
        )
        .unwrap();
        let hosted = Hosted::provider(&provider, None);
        let mut indicator = Indicator::default();
        let not_asked = Asking::by(
            &mail(),
            Answering::chosen(hosted.named_source(), &SourcePolicy::Anywhere).unwrap(),
            &[],
            &SourcePolicy::Anywhere,
        )
        .to_a_provider(&question(), &hosted, &mut indicator, noon())
        .unwrap_err();

        assert!(matches!(
            not_asked,
            NotAsked::CannotBeShown(DestinationError::NotPrintable)
        ));
        assert!(not_asked.nothing_left());
        assert!(indicator.is_quiet());
    }

    /// **The permission and the provider have to be the same place.** A
    /// permission for one provider does not send a question to another, because
    /// the line the person reads is made out of the permission and the socket
    /// is made out of the provider.
    #[test]
    fn a_permission_for_somewhere_else_sends_nothing() {
        let provider = far_away();
        let hosted = Hosted::provider(&provider, None);
        let mut indicator = Indicator::default();

        for (permitted_place, expected) in [
            (
                InferenceSource::Hosted {
                    provider: "alo".to_owned(),
                    region: Region::Declared("the EU".to_owned()),
                },
                Miswired::AnotherPlace,
            ),
            (
                // The same provider, a different region: the region is what the
                // indicator says, so it is part of being the same place.
                InferenceSource::Hosted {
                    provider: "Mistral".to_owned(),
                    region: Region::Unknown,
                },
                Miswired::AnotherPlace,
            ),
            (InferenceSource::ThisMachine, Miswired::NotAProvider),
            (
                InferenceSource::PairedMachine {
                    machine: "the studio workstation".to_owned(),
                },
                Miswired::NotAProvider,
            ),
        ] {
            let not_asked = Asking::by(
                &mail(),
                Answering::chosen(permitted_place.clone(), &SourcePolicy::Anywhere).unwrap(),
                &[],
                &SourcePolicy::Anywhere,
            )
            .to_a_provider(&question(), &hosted, &mut indicator, noon())
            .unwrap_err();

            assert_eq!(miswired(not_asked), Some(expected), "{permitted_place:?}");
            assert!(indicator.is_quiet(), "{permitted_place:?}");
        }
    }

    /// **A question that failed still left the machine**, so the departure
    /// comes back on this path too — a machine that wrote down only the
    /// questions that were answered would report a quieter day than it had.
    #[test]
    fn a_question_that_was_not_answered_still_left_and_says_so() {
        let (url, server) = serving(r#"{"message":"Unauthorized"}"#, 401);
        let provider = mistral(&url);
        let key = Secret::typed("sk-live-0123456789").unwrap();
        let hosted = Hosted::provider(&provider, Some(&key));
        let mut indicator = Indicator::default();

        let not_asked = Asking::by(
            &mail(),
            permitted(&SourcePolicy::Anywhere),
            &[],
            &SourcePolicy::Anywhere,
        )
        .to_a_provider(&question(), &hosted, &mut indicator, noon())
        .unwrap_err();
        server.join().unwrap();

        assert!(!not_asked.nothing_left());
        let unanswered = did_not_answer(not_asked).unwrap();
        assert_eq!(unanswered.failed().why(), WentWrong::KeyNotAccepted);
        assert_eq!(unanswered.failed().source(), &mistral_source());
        assert_eq!(unanswered.departing().at(), noon());
        assert_eq!(indicator.showing().len(), 1);

        let failed = unanswered.ended(&mut indicator);
        assert!(indicator.is_quiet());
        assert!(
            failed
                .said(&in_english())
                .text()
                .contains("the key for this provider was not accepted")
        );
    }

    /// **Never a silent fallback, as the test that would fail if it were
    /// written.** The machine has somewhere else it could ask and does not: one
    /// departure happened, the other place is an *offer* nobody has answered,
    /// and the person is told outright that nothing was sent anywhere.
    #[test]
    fn a_failure_asks_nowhere_else_and_the_person_is_told_so() {
        let (url, server) = serving(r#"{"error":"upstream capacity"}"#, 503);
        let provider = mistral(&url);
        let hosted = Hosted::provider(&provider, None);
        let mut indicator = Indicator::default();
        let others = [
            InferenceSource::ThisMachine,
            InferenceSource::Hosted {
                provider: "alo".to_owned(),
                region: Region::Declared("the EU".to_owned()),
            },
        ];

        let not_asked = Asking::by(
            &mail(),
            permitted(&SourcePolicy::Anywhere),
            &others,
            &SourcePolicy::Anywhere,
        )
        .to_a_provider(&question(), &hosted, &mut indicator, noon())
        .unwrap_err();
        server.join().unwrap();

        let unanswered = did_not_answer(not_asked).unwrap();
        // One departure, and it is the one that failed. Nothing was asked
        // anywhere else — there is no second line and no second answer.
        assert_eq!(indicator.showing().len(), 1);
        let failed = unanswered.ended(&mut indicator);
        assert!(indicator.is_quiet());

        assert_eq!(failed.why(), WentWrong::HavingTrouble(503));
        assert_eq!(failed.elsewhere().offers().len(), 2);
        assert_eq!(
            failed.nothing_was_sent(&in_english()).text(),
            "nothing was sent anywhere, and nothing will be unless you say so"
        );
    }

    /// **The line a person reads while their question leaves, in their own
    /// language** — and the place named in the middle of it is in that language
    /// too, so a German indicator is a German line rather than half of one.
    #[test]
    fn the_indicator_line_and_the_answers_provenance_are_one_language() {
        let (url, server) = serving(AN_ANSWER, 200);
        let provider = mistral(&url);
        let hosted = Hosted::provider(&provider, None);
        let strings = translated(&[]);
        let mut indicator = Indicator::default();

        let asked = Asking::by(
            &mail(),
            permitted(&SourcePolicy::Anywhere),
            &[],
            &SourcePolicy::Anywhere,
        )
        .to_a_provider(&question(), &hosted, &mut indicator, noon())
        .unwrap();
        server.join().unwrap();

        let line = indicator.showing().first().unwrap().said(&strings);
        assert!(line.is_translated(), "{line}");
        assert_eq!(line.text(), "@mail stellt Mistral, in the EU eine Frage");

        let answer = asked.ended(&mut indicator);
        let came_from = answer.came_from(&strings);
        assert!(came_from.is_translated(), "{came_from}");
        assert_eq!(came_from.text(), "von Mistral, in the EU");
    }

    /// The passing twin of the `compile_fail` above, so the pair cannot rot
    /// into a test of a typo: one permission is one attempt, and this is what
    /// spending it looks like.
    #[test]
    fn one_permission_is_one_attempt() {
        let provider = far_away();
        let hosted = Hosted::provider(&provider, None);
        let mut indicator = Indicator::default();
        let not_asked = Asking::by(
            &mail(),
            permitted(&SourcePolicy::Anywhere),
            &[],
            &SourcePolicy::Anywhere,
        )
        .to_a_provider(&question(), &hosted, &mut indicator, noon())
        .unwrap_err();
        // It was permitted, shown, attempted, and nothing was there.
        let unanswered = did_not_answer(not_asked).unwrap();
        assert_eq!(unanswered.failed().why(), WentWrong::NothingAnswered);
        let _ = unanswered.ended(&mut indicator);
        assert!(indicator.is_quiet());
    }
}
