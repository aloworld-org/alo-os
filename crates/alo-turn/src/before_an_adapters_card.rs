//! **Before an adapter's card: may alo work in the person's own window of its
//! application at all?**
//!
//! An adapter's verb is one message to the application over the person's
//! session bus (`alo_adapters::Driving`), and the application that answers is
//! the one the person has open: alo's own copy of an application is a separate
//! process this machine starts (`alo_desktop::alos_copies`), and nothing sends
//! an adapter's message there. So an adapter's verb is **always** work in the
//! person's own window, and the person decides whether alo may do that
//! (`alo_capability::AloInApplications`, Settings → What alo can use).
//!
//! It is asked **here, before the card**, with the one function the desktop
//! answers `work-in-their-window` with (`alo_acting::alo_may_work_in_their_window`),
//! so a person is never asked to approve something the desktop would then
//! refuse, and the two cannot come to disagree. The desktop is still asked when
//! the verb runs (`crate::acting_through_an_adapter`), because the setting may
//! have changed in between and because its answer is what puts the line up
//! that alo's border reads.
//!
//! The setting is read at each ask, from the person's own file, as the desktop
//! reads it (`Machine::where_alo_works_now`). A machine that was not told how
//! to read it answers the setting a file that will not read is taken to say,
//! `AloInApplications::WHEN_UNREADABLE`: alo's own copy only.

use alo_adapters::Adapters;
use alo_capability::{AloInApplications, Call, Refused};
use alo_strings::Filling;

use crate::machine::Machine;
use crate::words;

/// What the question answers for one verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeforeTheCard {
    /// Not an adapter's verb: this question is not this verb's.
    NotAnAdaptersVerb,
    /// An adapter's verb, and alo may work in the person's window of its
    /// application.
    MayGo,
    /// An adapter's verb, and at this setting alo may not work in the person's
    /// own window of this application, by its identifier.
    NotHere(&'static str),
}

/// Ask the question of one verb, by its name on the machine's list.
///
/// `pub` so the desktop's side can be held to it: `alo-shell`'s tests run this
/// against the desktop's own answer to `work-in-their-window`, for every
/// setting and every shipped adapter.
#[must_use]
pub fn before_an_adapters_card(
    verb: &str,
    adapters: &Adapters,
    chosen: AloInApplications,
) -> BeforeTheCard {
    match adapters.carrying_out(verb) {
        None => BeforeTheCard::NotAnAdaptersVerb,
        Some((adapter, _)) => {
            if alo_acting::alo_may_work_in_their_window(chosen, adapter.application) {
                BeforeTheCard::MayGo
            } else {
                BeforeTheCard::NotHere(adapter.application)
            }
        }
    }
}

/// The question, asked of a call on its way to a card.
///
/// # Errors
/// [`Refused`], carrying the call, in the person's words: the sentence every
/// other road says for an application with no copy of its own, or
/// [`words::AN_ADAPTER_REACHES_YOUR_WINDOW`] for one that has a copy.
pub(crate) fn may_an_adapters_call_go(call: &Call, machine: &Machine<'_>) -> Result<(), Refused> {
    let BeforeTheCard::NotHere(application) = before_an_adapters_card(
        call.verb(),
        machine.the_adapters(),
        machine.where_alo_works_now(),
    ) else {
        return Ok(());
    };
    let word = if alo_acting::has_a_copy_of_its_own(application) {
        words::AN_ADAPTER_REACHES_YOUR_WINDOW
    } else {
        alo_capability::words::NO_COPY_OF_ITS_OWN
    };
    let said = machine.strings().say(
        &word.key(),
        &Filling::of("application", machine.what_a_person_calls(application)),
    );
    Err(Refused::worded_elsewhere(call.clone(), said))
}

#[cfg(test)]
#[path = "before_an_adapters_card_tests.rs"]
mod tests;
