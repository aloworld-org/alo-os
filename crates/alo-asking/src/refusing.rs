//! Why a question was not asked, or why what came back is not an answer.
//!
//! # There is no one sentence here, and the absence is the design
//!
//! Every other refusal in this workspace has a `said` on it. This one does not,
//! because the four things that can come back are four different things to do,
//! and a single *it did not work* would be the sentence that let a caller treat
//! them as one:
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
//! callers matching on four variants and one of them getting it wrong.
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

use alo_answering::NotWhatFailed;
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

/// The permission and the provider were not the same place.
///
/// **Keeps its English and its `Display`**, for the reason given at the top of
/// this file. Its reader is whoever wired a question to a provider, and every
/// one of these says what to do about it rather than naming a state.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum Miswired {
    /// The permission names a provider, and it is not this one.
    #[error(
        "this question was permitted to go to a different provider, so nothing was sent — the \
         permission and the provider have to be the same place, or the line a person reads while \
         their question leaves would name somewhere it did not go"
    )]
    AnotherPlace,
    /// The permission names this machine, or one on this network.
    #[error(
        "this question was permitted to be answered on this machine or on a paired one, which is \
         not a hosted provider — ask the runtime instead, and do not send it here"
    )]
    NotAProvider,
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
                .contains("ask the runtime instead")
        );
        assert_eq!(
            Miswired::from(NotWhatFailed::NoKeyThere).to_string(),
            NotWhatFailed::NoKeyThere.to_string()
        );
    }
}
