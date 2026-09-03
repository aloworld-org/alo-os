//! Why a question was not asked, or why what came back is not an answer.
//!
//! # Three doors, two lists, and the shorter one is the whole of law 1
//!
//! [`NotAsked`] is what comes back from [`crate::Asking::to_a_provider`] and
//! [`NotAnswered`] from the two doors that answer on this machine —
//! [`crate::Asking::to_this_machine`] and
//! [`crate::Asking::to_a_service_on_this_machine`]. **The lists divide on law 1
//! rather than on how many doors there are**: one list for the door where
//! something leaves, one for the doors where nothing does. The second is the
//! first with law 1's two refusals taken out of it — no
//! [`CannotBeShown`](NotAsked::CannotBeShown), no
//! [`HeldBack`](NotAsked::HeldBack) — because a question answered on this
//! machine has nowhere to be shown going and no rule that can hold it back.
//!
//! **The zero-egress claim is those two missing variants.** `docs/features.md`
//! promises that a working day with a local model produces zero inference
//! egress; on this path there is no `alo_egress::Departing` to be made, nothing
//! to put on the indicator, and therefore nothing that could be refused for
//! leaving. A type with a variant about being held back would be a type that
//! believed something might leave.
//!
//! # There is no one sentence here, and the absence is the design
//!
//! Every other refusal in this workspace has a `said` on it. These do not,
//! because the things that can come back are different things to do, and a
//! single *it did not work* would be the sentence that let a caller treat them
//! as one:
//!
//! | | What happened | What the caller does |
//! |---|---|---|
//! | [`NotAsked::CannotBeShown`] | Nothing left. Law 1 could not show it | Says so with `alo_egress::DestinationError::said` |
//! | [`NotAsked::HeldBack`] | Nothing left. The rule refused it | Says so with `alo_egress::NotPermitted::said`, and records it with `alo_record::Entry::held_back` |
//! | [`NotAsked::DidNotAnswer`] | It left, and nothing came back | Shows `alo_answering::Failed`, records the departure, and asks nobody anything |
//! | [`NotAsked::Miswired`] | Nothing left. The attempt was put together wrongly | Is fixed by whoever wrote it |
//!
//! The first two and the last are cases where **nothing was sent at all**, and
//! [`NotAsked::nothing_left`] is that question asked once rather than by three
//! callers matching on four variants and one of them getting it wrong. On
//! [`NotAnswered`] the same question has one answer for every variant, so it is
//! not a method there: nothing on that path ever left.
//!
//! | | What happened | What the caller does |
//! |---|---|---|
//! | [`NotAnswered::DidNotAnswer`] | It was asked here, and nothing came back | Shows `alo_answering::Failed`, and asks nobody anything |
//! | [`NotAnswered::Miswired`] | Nothing was asked. The attempt was put together wrongly | Is fixed by whoever wrote it |
//!
//! # And one of them keeps its English
//!
//! [`Miswired`] is read by whoever is wiring this crate to a provider, in the
//! way `alo_capability::VerbError` refuses a declaration and
//! `alo_answering::NotWhatFailed` refuses a report. Nobody using the machine can
//! cause one and nobody using the machine is shown one, so it keeps its
//! `Display` — which is `CLAUDE.md`'s rule about hardcoded English being a bug,
//! read the way that rule is written: user-facing strings are externalized, and
//! this is not one.

use alo_answering::{Failed, NotWhatFailed};
use alo_egress::{DestinationError, NotPermitted};

use crate::unanswered::DidNotAnswer;

/// Why there is no answer.
///
/// Not `PartialEq`: two of the four carry things that are not comparable and
/// should not be — an `alo_egress::Departing` is an authority rather than a
/// value, and one failure is not another.
#[derive(Debug)]
pub enum NotAsked {
    /// **Nothing was sent.** The provider's name could not be put on the
    /// indicator, and law 1 does not permit an egress nobody can be shown.
    ///
    /// A provider is named by whoever added it, and `alo_models::Provider`
    /// asks only that the name is not empty — so a name carrying a line break
    /// or an escape code reaches here, where it is refused rather than drawn
    /// onto the one surface a person is expected to trust.
    CannotBeShown(DestinationError),
    /// **Nothing was sent.** The rule this machine is under refused the egress,
    /// in the rule's own words.
    HeldBack(NotPermitted),
    /// It was sent, and nothing came back. What the person may be told and
    /// what they may be asked next, with the departure that has still to be
    /// ended and written down.
    ///
    /// **Boxed**, and it is the only thing in this file that is not about the
    /// design: a failure carries a departure and every offer this machine could
    /// make, which is the largest value here by some way, and an unboxed one
    /// would make every call to [`crate::Asking::to_a_provider`] — the ones
    /// that answer included — return it. `Box::as_ref` and `*` reach through
    /// it, so a caller matching on this reads the same as one matching on the
    /// others.
    DidNotAnswer(Box<DidNotAnswer>),
    /// **Nothing was sent.** The permission and the provider were not the same
    /// place, so nothing was decided about and nothing was shown.
    Miswired(Miswired),
}

impl NotAsked {
    /// Whether this machine sent anything at all.
    ///
    /// True for three of the four. The question is worth answering here rather
    /// than at each caller because it is the one a person asks first — *did my
    /// question go anywhere?* — and `alo_answering::Failed::nothing_was_sent`
    /// is the sentence for the fourth, where something did.
    #[must_use]
    pub fn nothing_left(&self) -> bool {
        match self {
            Self::CannotBeShown(_) | Self::HeldBack(_) | Self::Miswired(_) => true,
            Self::DidNotAnswer(_) => false,
        }
    }
}

impl From<Miswired> for NotAsked {
    fn from(miswired: Miswired) -> Self {
        Self::Miswired(miswired)
    }
}

impl From<NotWhatFailed> for NotAsked {
    fn from(reported: NotWhatFailed) -> Self {
        Self::Miswired(Miswired::NotWhatFailed(reported))
    }
}

/// Why there is no answer from a model on this machine.
///
/// Two things rather than [`NotAsked`]'s four, and the two that are missing are
/// law 1's: nothing here can fail to be shown and nothing here can be held back,
/// because nothing here goes anywhere. There is no `nothing_left` either —
/// **every variant means nothing left**, so a method answering the same thing
/// each time would be a question worth asking only if it might one day answer
/// otherwise.
///
/// Not `PartialEq`, as [`NotAsked`] is not: one failure is not another.
#[derive(Debug)]
pub enum NotAnswered {
    /// **Nothing was asked.** The permission named somewhere that is not a
    /// model on this machine.
    Miswired(Miswired),
    /// It was asked, here, and nothing came back. What the person may be told
    /// and what they may be asked next — with no departure beside it, because
    /// nothing departed.
    ///
    /// **Boxed for [`NotAsked::DidNotAnswer`]'s reason**: a failure carries
    /// every offer this machine could make, which would otherwise be the size
    /// of every answer this door returns.
    DidNotAnswer(Box<Failed>),
}

impl From<Miswired> for NotAnswered {
    fn from(miswired: Miswired) -> Self {
        Self::Miswired(miswired)
    }
}

impl From<NotWhatFailed> for NotAnswered {
    fn from(reported: NotWhatFailed) -> Self {
        Self::Miswired(Miswired::NotWhatFailed(reported))
    }
}

/// The permission and the place a question was put were not the same place.
///
/// **Keeps its English and its `Display`**, for the reason given at the top of
/// this file. Its reader is whoever wired a question to somewhere, and every one
/// of these says what to do about it rather than naming a state.
///
/// # One list for three doors, and none of it says *ask somewhere else instead*
///
/// Three of these are a permission arriving at the wrong door, and each names
/// **the door the permission's own place is behind** — never a different place.
/// The distinction is ADR 0008's: sending a question to the door the person
/// already chose is routing, and sending it to another one because this one
/// refused is the substitution that ADR forbids in both directions. A refusal
/// that said *ask the runtime instead* about a paired machine would be exactly
/// that mistake worded as advice, and until item 18a there was no local path for
/// it to be advice towards.
///
/// [`ReachesOffThisMachine`](Miswired::ReachesOffThisMachine) is the odd one and
/// is not about a permission at all: it refuses an *address*, before any
/// question exists to be routed. It still keeps the rule — the door it names is
/// the one an address anywhere else belongs to, which is the same road law 1
/// would have made that question take anyway.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum Miswired {
    /// The permission names a provider, and it is not this one.
    #[error(
        "this question was permitted to go to a different provider, so nothing was sent — the \
         permission and the provider have to be the same place, or the line a person reads while \
         their question leaves would name somewhere it did not go"
    )]
    AnotherPlace,
    /// The permission names this machine, and a provider was asked.
    #[error(
        "this question was permitted to be answered on this machine, which is not a hosted \
         provider — put it to the runtime with `to_this_machine`, or to a service running here \
         with `to_a_service_on_this_machine`, where nothing leaves and there is nothing to show"
    )]
    NotAProvider,
    /// The permission names a hosted provider, and something on this machine
    /// was asked.
    ///
    /// One variant for both local doors, because what it refuses is the same
    /// thing at each: answering a question here that a person chose a provider
    /// for. It was called `NotTheRuntime` until item 18b, when the runtime
    /// stopped being the only thing on this machine that can answer.
    #[error(
        "this question was permitted to go to a hosted provider, so it is not this machine's to \
         answer — send it there with `to_a_provider`, where law 1 shows it leaving; answering it \
         here instead, on the runtime or on a service somebody runs here, would be a different \
         model wearing the same face"
    )]
    NotOnThisMachine,
    /// The address given for a service on this machine is not on this machine.
    ///
    /// **The one refusal here that is not about a permission at all**, and the
    /// reason `crate::Served` exists as a type. A question carried to this
    /// address would leave the machine with law 1 having shown nothing, because
    /// the door that carries it is the one with no indicator in it.
    #[error(
        "this address is not on this machine, so a question put to it would leave with nothing on \
         the indicator — add it as a provider and ask it with `to_a_provider`, which is where an \
         address anywhere else belongs"
    )]
    ReachesOffThisMachine,
    /// The permission names a machine on this network, and neither door goes
    /// there.
    #[error(
        "this question was permitted to be answered on a machine on this network, and there is no \
         path to one in this repository yet — neither a provider nor the runtime on this machine \
         is that place, and neither is a substitute for it"
    )]
    NoPathToAPairedMachine,
    /// A failure was reported where it could not have happened.
    #[error(transparent)]
    NotWhatFailed(#[from] NotWhatFailed),
}

#[cfg(test)]
mod tests {
    use super::*;
    use alo_egress::DestinationError;

    /// **The question a person asks first**, answered once rather than by every
    /// caller matching four variants.
    #[test]
    fn three_of_the_four_mean_nothing_left_this_machine() {
        assert!(NotAsked::CannotBeShown(DestinationError::NotPrintable).nothing_left());
        assert!(NotAsked::Miswired(Miswired::AnotherPlace).nothing_left());
        assert!(NotAsked::Miswired(Miswired::NotAProvider).nothing_left());
        // The fourth is asserted in `asking.rs`, where there is a departure to
        // make one with — which is the point: it is the only one that has one.
    }

    /// A refusal read by whoever wrote the wiring says what to do about it, in
    /// the way `alo_capability::VerbError` does about a declaration.
    #[test]
    fn the_wiring_refusals_tell_whoever_wrote_them_what_to_do() {
        assert!(
            Miswired::AnotherPlace
                .to_string()
                .contains("the same place")
        );
        assert!(
            Miswired::NotAProvider
                .to_string()
                .contains("`to_this_machine`")
        );
        assert!(
            Miswired::NotOnThisMachine
                .to_string()
                .contains("`to_a_provider`")
        );
        assert!(
            Miswired::NoPathToAPairedMachine
                .to_string()
                .contains("no path to one")
        );
        assert!(
            Miswired::ReachesOffThisMachine
                .to_string()
                .contains("nothing on the indicator")
        );
        assert_eq!(
            Miswired::from(NotWhatFailed::NoKeyThere).to_string(),
            NotWhatFailed::NoKeyThere.to_string()
        );
    }

    /// **No refusal here sends a question to the other door**, which is ADR
    /// 0008's *never a silent fallback* met where it would first be written as
    /// helpfulness. Each one names the door the permission's own place is
    /// behind; the one that has no door says so and offers neither.
    #[test]
    fn no_refusal_offers_the_place_the_person_did_not_choose() {
        // The permission is for this machine, so the two doors on this machine
        // are where it goes — and a provider is not offered as one of them.
        let local = Miswired::NotAProvider.to_string();
        assert!(local.contains("`to_this_machine`"), "{local}");
        assert!(local.contains("`to_a_service_on_this_machine`"), "{local}");
        assert!(!local.contains("`to_a_provider`"), "{local}");

        // The permission is for a provider, so that is where it goes — and the
        // sentence says outright why answering it here instead would be wrong.
        let hosted = Miswired::NotOnThisMachine.to_string();
        assert!(hosted.contains("`to_a_provider`"), "{hosted}");
        assert!(!hosted.contains("`to_this_machine`"), "{hosted}");
        assert!(hosted.contains("same face"), "{hosted}");

        // And the place with no door offers none of the other three.
        let paired = Miswired::NoPathToAPairedMachine.to_string();
        assert!(!paired.contains("`to_a_provider`"), "{paired}");
        assert!(!paired.contains("`to_this_machine`"), "{paired}");
        assert!(paired.contains("neither is a substitute"), "{paired}");
    }

    /// **An address refused for reaching off this machine is sent to the door
    /// where law 1 would show it**, which is the one road that address was ever
    /// going to be allowed to take. It is advice towards the rule rather than
    /// around it.
    #[test]
    fn an_address_somewhere_else_is_sent_to_the_door_that_shows_it_leaving() {
        let elsewhere = Miswired::ReachesOffThisMachine.to_string();
        assert!(elsewhere.contains("`to_a_provider`"), "{elsewhere}");
        assert!(!elsewhere.contains("`to_this_machine`"), "{elsewhere}");
        assert!(
            !elsewhere.contains("`to_a_service_on_this_machine`"),
            "{elsewhere}"
        );
    }
}
