//! That a model change waits for the next request, and that leaving the
//! machine is said before it is done.

use alo_models::{InferenceSource, Region};

use super::{BeforeItChanges, WhichModel};

/// A provider, as a person would name it.
fn a_provider() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "alo".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// **Only a provider is a provider**: a hosted service answers yes, and this
/// machine, a service at its own address and a paired machine answer no.
#[test]
fn only_a_hosted_service_is_a_provider() {
    assert!(WhichModel::answering_from(a_provider()).a_provider_answers());
    for not in [
        InferenceSource::ThisMachine,
        InferenceSource::AServiceAtThisMachinesAddress,
        InferenceSource::PairedMachine {
            machine: "workstation-2".to_owned(),
        },
    ] {
        assert!(
            !WhichModel::answering_from(not.clone()).a_provider_answers(),
            "{not:?}"
        );
    }
}

/// **A change does not touch the request in flight.**
///
/// The rule, and the reason: a model swapped under a running action gives a
/// person an answer whose halves came from two places — and where the second is
/// a provider, half their conversation left the machine because they changed a
/// setting.
#[test]
fn a_change_waits_and_does_not_touch_what_is_answering() {
    let mut model = WhichModel::answering_from(InferenceSource::ThisMachine);

    model.will_answer_next(a_provider());

    assert_eq!(
        model.answering(),
        &InferenceSource::ThisMachine,
        "the model answering now changed, so an action in flight has swapped places midway"
    );
    assert!(model.a_change_is_waiting());
    assert_eq!(model.next(), Some(&a_provider()));
}

/// **And it lands when the next request begins**, which is the one place the
/// answering model moves.
#[test]
fn the_next_request_takes_the_waiting_change() {
    let mut model = WhichModel::answering_from(InferenceSource::ThisMachine);
    model.will_answer_next(a_provider());

    model.the_next_request_begins();

    assert_eq!(model.answering(), &a_provider());
    assert!(
        !model.a_change_is_waiting(),
        "the change was taken and is still waiting, so it would land twice"
    );
}

/// **Changing back to what is already answering leaves nothing waiting.**
///
/// A person who changes their mind and changes back is not left with a pending
/// change to where they already are — which would show them *next: this
/// machine* while this machine is answering.
#[test]
fn changing_back_clears_the_waiting_change() {
    let mut model = WhichModel::answering_from(InferenceSource::ThisMachine);
    model.will_answer_next(a_provider());

    model.will_answer_next(InferenceSource::ThisMachine);

    assert!(
        !model.a_change_is_waiting(),
        "changing back to the answering model left a change waiting"
    );
    assert_eq!(model.next(), None);
}

/// **Beginning a request with nothing waiting changes nothing.**
#[test]
fn a_request_with_nothing_waiting_keeps_its_model() {
    let mut model = WhichModel::answering_from(InferenceSource::ThisMachine);

    model.the_next_request_begins();

    assert_eq!(model.answering(), &InferenceSource::ThisMachine);
}

/// **A change to a provider says that it would leave the machine, before it is
/// made.**
///
/// Asked and answered in two steps on purpose: one call that both decided and
/// applied would be a change a person could make without being told what it
/// does.
#[test]
fn changing_to_a_provider_says_what_it_costs_first() {
    let asked = WhichModel::before_changing_to(&a_provider());

    assert_eq!(
        asked,
        BeforeItChanges::ItWouldLeaveThisMachine { to: a_provider() },
        "changing to a provider did not say that the conversation would leave the machine"
    );
}

/// **A paired machine leaves this machine too**, which is the exception that
/// would otherwise erode law 1.
///
/// *It only went down the corridor* is still leaving, and
/// `InferenceSource::causes_egress` already refuses to make that exception.
/// This test is here so the task window cannot make it separately.
#[test]
fn a_paired_machine_is_also_leaving_this_machine() {
    let paired = InferenceSource::PairedMachine {
        machine: "the studio iMac".to_owned(),
    };

    assert_eq!(
        WhichModel::before_changing_to(&paired),
        BeforeItChanges::ItWouldLeaveThisMachine { to: paired },
        "a paired machine was treated as staying on this machine"
    );
}

/// **The two local sources say nothing leaves**, and both of them, because they
/// are bounded identically and described differently.
#[test]
fn the_local_sources_cost_nothing_to_change_to() {
    for local in [
        InferenceSource::ThisMachine,
        InferenceSource::AServiceAtThisMachinesAddress,
    ] {
        assert_eq!(
            WhichModel::before_changing_to(&local),
            BeforeItChanges::NothingLeaves,
            "{local:?} was said to leave the machine"
        );
    }
}
