//! Whether a proposed change to the network may be put to a person at all.
//!
//! A proposal is a sentence a person approves, and a network verb's sentence
//! says what the change does to this conversation's connection. What the agent
//! put there is whatever the agent said. So before a call becomes a proposal,
//! [`proposable`] asks the network manager what is true and refuses a call
//! whose sentence would say anything else — the person is never shown a
//! sentence that is wrong about it, in either direction. It also refuses a call
//! naming a network there is not exactly one of, so an agent hears at once what
//! a person would otherwise hear only after approving.
//!
//! Nothing here changes anything, and nothing here needs privilege: the network
//! manager answers these questions for the person's own session.

use alo_capability::Call;
use alo_networks::Networks;

use crate::choosing::chosen;
use crate::cutting::{Answered, would};
use crate::refusing::NotChanged;
use crate::verbs::wanted;

/// Whether this call may be proposed to the person, for a conversation
/// answered so.
///
/// # Errors
/// [`NotChanged::SaysTheWrongThingAboutThisConversation`] when the call says
/// the conversation keeps its connection and it would lose it, or the other way
/// round; and every refusal [`crate::chosen`] makes, and
/// [`NotChanged::NetworkNotAnswering`] when there is nobody to ask.
pub fn proposable(
    call: &Call,
    networks: &impl Networks,
    answered: Answered,
) -> Result<(), NotChanged> {
    let wanted = wanted(call)?;
    let now = networks
        .now()
        .map_err(|_| NotChanged::NetworkNotAnswering)?;
    chosen(wanted.change(), &now)?;
    if would(wanted.change(), &now, answered) == wanted.says() {
        Ok(())
    } else {
        Err(NotChanged::SaysTheWrongThingAboutThisConversation)
    }
}
