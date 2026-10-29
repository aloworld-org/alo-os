//! A question the person asked themselves, put to a service at this machine's
//! own address.
//!
//! [`crate::a_question_nobody_is_showing`] is the person's road to a hosted
//! provider; this is the same road to the other place a person can name in
//! Settings — **an OpenAI-compatible service they run on this machine**, at a
//! loopback address. `alo_choosing` accepts one (*a service on this machine is
//! accepted on any scheme, which is ADR 0021's own case*), and until 2026-10-29
//! no road in this product would put a question to it: the provider door
//! refuses this machine's address outright (`Miswired::NotAProvider`), and the
//! daemon called nothing else, so a person who configured one was told *something
//! in alo OS is set up wrongly* on every question.
//!
//! # What is the same as the agent's door to a service here, and what is not
//!
//! Same: nothing leaves this machine's address, so there is no indicator, no
//! departure and no destination to ask a rule about; and the answer says where
//! it came from in ADR 0021's words — *a service at this machine's address* —
//! never *on this machine*, because alo OS did not start the service and cannot
//! see what it does next. The address is policed by [`crate::Served::at`],
//! which refuses anything that is not this machine.
//!
//! Not the same: there is no agent, so no [`crate::Asking`] is built. That door
//! takes a `Grantee` because a turn is somebody's; a person's own question is
//! nobody's turn (ADR 0126).

use std::net::SocketAddr;

use alo_answering::Answering;
use alo_models::{InferenceSource, SourcePolicy};

use crate::a_question_nobody_is_showing::NotPut;
use crate::answer::Answer;
use crate::question::Question;
use crate::refusing::Miswired;
use crate::served::Served;

/// Put the person's own question to the service on this machine they chose.
///
/// `answering` is what the rule permitted about the service's source, which is
/// [`Served::source`]; `others` and `policy` are what a failure may offer
/// instead, exactly as on the provider road. `to` is the addresses the caller
/// resolved, as [`crate::Asking::to_a_service_on_this_machine`] takes them.
///
/// # Errors
///
/// [`NotPut::Miswired`] when the permission is for somewhere that is not this
/// machine — nothing is sent; [`NotPut::DidNotAnswer`] when the service was
/// asked and did not answer; [`NotPut::NotWhatFailed`] for a failure that could
/// not have happened here.
pub fn a_persons_question_to_a_service_here(
    answering: Answering,
    others: &[InferenceSource],
    policy: &SourcePolicy,
    question: &Question,
    served: &Served<'_>,
    to: &[SocketAddr],
) -> Result<Answer, NotPut> {
    let source = answering.source().clone();
    match &source {
        InferenceSource::ThisMachine | InferenceSource::AServiceAtThisMachinesAddress => {}
        // The person chose a provider; answering them from here would be a
        // different model wearing the same face.
        InferenceSource::Hosted { .. } => return Err(Miswired::NotOnThisMachine.into()),
        InferenceSource::PairedMachine { .. } => {
            return Err(Miswired::BelongsDownTheCorridor.into());
        }
    }
    match served.ask(question, to) {
        Ok(said) => Ok(Answer::new(said, source, question.of().to_owned())),
        Err(why) => Err(NotPut::DidNotAnswer(Box::new(
            answering.did_not_answer(why, others, policy)?,
        ))),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_models::{Provider, Region};

    use super::*;

    /// A service at this machine's address, nobody listening behind it.
    fn here() -> Provider {
        Provider::checked("Mine", "http://127.0.0.1:9", Region::Unknown, None).unwrap()
    }

    /// **A permission for a provider is not put to a service here**, and
    /// nothing is sent: answering from this machine a question the person sent
    /// elsewhere is the half of ADR 0008 that runs the other way.
    #[test]
    fn a_permission_for_a_provider_is_refused_here() {
        let provider = here();
        let served = Served::at(&provider, None).unwrap();
        let hosted = InferenceSource::Hosted {
            provider: "Mistral".to_owned(),
            region: Region::Declared("the EU".to_owned()),
        };
        let permission = Answering::chosen(hosted, &SourcePolicy::Anywhere).unwrap();
        let question = Question::asked("may the tenant sublet?", "a-model").unwrap();
        let refused = a_persons_question_to_a_service_here(
            permission,
            &[],
            &SourcePolicy::Anywhere,
            &question,
            &served,
            &[],
        );
        assert!(
            matches!(refused, Err(NotPut::Miswired(Miswired::NotOnThisMachine))),
            "{refused:?}"
        );
    }

    /// **A service that does not answer is a failure the person is told
    /// about**, said as this machine's address — not a miswiring.
    #[test]
    fn a_service_that_does_not_answer_is_a_failure_and_not_a_miswiring() {
        let provider = here();
        let served = Served::at(&provider, None).unwrap();
        let permission =
            Answering::chosen(served.source(), &SourcePolicy::ThisMachineOnly).unwrap();
        let question = Question::asked("may the tenant sublet?", "a-model").unwrap();
        let to: Vec<SocketAddr> = vec!["127.0.0.1:9".parse().unwrap()];
        let failed = a_persons_question_to_a_service_here(
            permission,
            &[],
            &SourcePolicy::ThisMachineOnly,
            &question,
            &served,
            &to,
        );
        assert!(matches!(failed, Err(NotPut::DidNotAnswer(_))), "{failed:?}");
    }
}
