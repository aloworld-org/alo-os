//! A question to a provider that nothing shows and nothing records.
//!
//! [`crate::Asking::to_a_provider`]'s sibling, for the one question in this
//! product that law 1 does not reach: **a question the person asked
//! themselves.**
//!
//! Law 1's body is *every network egress **an agent causes** is visible at the
//! moment it happens and afterwards in a record*, and
//! [ADR 0126](../../../docs/decisions/0126-a-persons-own-question-is-a-third-authority-on-the-indicator-and-in-the-record.md)
//! is the owner's reading of it: a person who chose their provider in Settings
//! is not shown a line for the thing they chose. So there is no
//! `alo_egress::Leaving` here, no `Departing`, and nothing to put on an
//! indicator or into a record.
//!
//! # What is **not** dropped, and it is the whole reason this is its own file
//!
//! **The organisation's rule.** That is
//! [ADR 0016](../../../docs/decisions/0016-the-organisation-bounds-and-the-person-chooses.md)
//! and law 5's own carve-out — *Only the law, or an organisation's policy on
//! machines it owns (ADR 0016), may truly forbid* — and it is a different thing
//! from law 1. A person on a machine set to keep questions in the building is
//! refused here, in the rule's own words, and nothing is sent.
//!
//! **And it is asked twice about two different subjects, which is not a
//! duplicate.** `alo_answering::Answering::chosen` asked the rule about the
//! provider's **stated source** before this crate was reached. This asks it
//! about the **resolved destination**. A provider that says it runs in the EU
//! and resolves to Virginia passes the first and fails the second, and dropping
//! either would be dropping a real check.
//!
//! # Why this is not a road around the indicator
//!
//! `alo_egress::Indicator::beginning` asks the policy and shows the egress in
//! one call **because they must not be two**, and that is untouched: anything
//! holding a `Leaving` still reaches the policy only through it. What this file
//! serves is the case with no `Leaving` at all — so there is nothing to show,
//! and therefore nothing a caller here could have forgotten to show.
//! `alo_egress::EgressPolicy::permits_going_to` is the door, and
//! `crates/alo-egress/src/policy_without_an_egress_tests.rs` is the test that
//! it is the same rule as the shown road's.

use std::net::SocketAddr;

use alo_answering::{Answering, Failed, NotWhatFailed};
use alo_egress::{Destination, DestinationError, EgressPolicy, Refusal};
use alo_models::{InferenceSource, SourcePolicy};

use crate::answer::Answer;
use crate::hosted::Hosted;
use crate::question::Question;
use crate::refusing::Miswired;

/// Why a question the person asked was not put, or was put and not answered.
///
/// [`crate::NotAsked`]'s sibling, and the difference between them is the one
/// this module is about: every variant there that carries a departure carries
/// none here, because there was none.
#[derive(Debug)]
pub enum NotPut {
    /// **Nothing was sent.** The provider's name could not be made into a
    /// destination, so the rule had nothing to be asked about.
    ///
    /// Kept even though nothing is shown: a name that cannot be made into a
    /// destination is a name `alo_egress::Destination` refused, and a question
    /// sent to a place this machine cannot name is a question nobody could
    /// answer *where it went* about afterwards.
    NotSomewhereNameable(DestinationError),
    /// **Nothing was sent.** The rule this machine is under refuses a question
    /// going there, in the rule's own words.
    HeldBack(Refusal),
    /// It was sent, and nothing came back — what the person may be told, and
    /// what they may be asked next.
    ///
    /// Boxed for [`crate::NotAsked::DidNotAnswer`]'s reason: it is much the
    /// largest value here, and an unboxed one would make every answering call
    /// return it.
    DidNotAnswer(Box<Failed>),
    /// **Nothing was sent.** The permission and the provider were not the same
    /// place, so nothing was decided about.
    Miswired(Miswired),
    /// **Nothing was sent.** A reason that could not have happened where it was
    /// reported, which is `alo-answering`'s refusal of whoever reported it.
    NotWhatFailed(NotWhatFailed),
}

impl From<Miswired> for NotPut {
    fn from(why: Miswired) -> Self {
        Self::Miswired(why)
    }
}

impl From<NotWhatFailed> for NotPut {
    fn from(why: NotWhatFailed) -> Self {
        Self::NotWhatFailed(why)
    }
}

/// Put the person's own question to the provider they chose.
///
/// `answering` is what the organisation's rule already permitted about the
/// provider's stated source; `policy` is that same rule, asked here about where
/// the question is actually going. `to` is the addresses the caller resolved
/// and registered, as [`crate::Asking::to_a_provider`] requires of its caller
/// and for the same reason.
///
/// # Errors
///
/// [`NotPut`], whose first, second, fourth and fifth variants all mean nothing
/// was sent.
pub fn a_question_nobody_is_showing(
    answering: Answering,
    others: &[InferenceSource],
    policy: &SourcePolicy,
    question: &Question,
    hosted: &Hosted<'_>,
    to: &[SocketAddr],
) -> Result<Answer, NotPut> {
    let source = answering.source().clone();
    the_place_is_this_provider(&source, hosted)?;

    // The organisation's rule, about where this is actually going. Not law 1
    // and not an indicator — see this file's header for why it is asked here as
    // well as of the stated source.
    let destination = Destination::of(&source).map_err(NotPut::NotSomewhereNameable)?;
    if let Some(refused) = EgressPolicy::from(policy).refusal_going_to(&destination) {
        return Err(NotPut::HeldBack(refused));
    }

    // And only now.
    match hosted.ask(question, to) {
        Ok(said) => Ok(Answer::new(said, source, question.of().to_owned())),
        Err(why) => Err(NotPut::DidNotAnswer(Box::new(
            answering.did_not_answer(why, others, policy)?,
        ))),
    }
}

/// That the place the rule permitted is the provider this is being put to.
///
/// [`crate::Asking::to_a_provider`]'s own first act, shared rather than written
/// twice: two readings of *is this actually a provider, and is it this one* is
/// how one of them one day sends a question to a place nothing decided about.
fn the_place_is_this_provider(
    source: &InferenceSource,
    hosted: &Hosted<'_>,
) -> Result<(), Miswired> {
    match source {
        // Either kind of local: this door cannot tell which of the two the
        // person meant, only that they did not mean a provider.
        InferenceSource::ThisMachine | InferenceSource::AServiceAtThisMachinesAddress => {
            Err(Miswired::NotAProvider)
        }
        // A machine on this network has a door of its own, and this is not it.
        InferenceSource::PairedMachine { .. } => Err(Miswired::BelongsDownTheCorridor),
        InferenceSource::Hosted { .. } if *source != hosted.named_source() => {
            Err(Miswired::AnotherPlace)
        }
        InferenceSource::Hosted { .. } => Ok(()),
    }
}
