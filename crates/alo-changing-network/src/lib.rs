//! Changing this machine's network — for an agent's proposal a person approved,
//! and for a person's own choice in Settings — through the privileged broker's
//! network verbs, and by no other road.
//!
//! ★ *System verbs through the privileged broker: printers, network, updates,
//! storage.* Joining a network, forgetting one, turning Wi-Fi on or off and
//! setting the proxy are changes to the whole machine, so ADR 0001 §2 puts them
//! behind the broker, with no free-form parameter: what crosses its door is a
//! verb and the digest of what the network manager reported, never a network's
//! name, a password or an address. ADR 0049 is the decision.
//!
//! **The proxy's own password travels the same way** (ADR 0060): a person on
//! their own machine hands it over as bytes, beside the proxy, and what crosses
//! the door is the digest of the two together — one act, one approval, and the
//! credential written before the proxy, so that a machine is never left with a
//! proxy it cannot sign in to. No agent verb reaches any of it.
//!
//! | | |
//! |---|---|
//! | [`verbs`], [`approved`] | `join_network`, `forget_network`, `switch_wireless`: changes an agent proposes and a person approves |
//! | [`would`], [`Answered`] | Whether a change cuts the connection this conversation is answered over |
//! | [`proposable`] | Whether a call may be put to a person at all: its sentence must be true |
//! | [`chosen`] | The one network an approved name is |
//! | [`Listed`] | This machine's networks as a person picks among them in Settings |
//! | [`carry_out_approved`], [`carry_out_by_hand`], [`set_proxy_by_hand`], [`TheBroker`] | The one road from an approval to the broker's door, and the proxy and its password over it (Unix only) |
//! | [`NotChanged`], [`changed_said`], [`proxy_set_said`] | What a person reads, either way |
//! | [`words`] | Every sentence, with a note for whoever translates it |
//!
//! # Who decides what
//!
//! The network manager decides what a network is and whether joining it worked.
//! A person decides whether — by approving a sentence, or by choosing in
//! Settings — and is the only one ever asked for a network's password
//! (`alo_networks::secret_agent`). The broker decides only that what arrived is
//! exactly one of its verbs under an approval, and writes that down before
//! carrying it out. This crate decides which verb to ask for, whether the
//! sentence about the conversation is true, and says what happened.
//!
//! # What it has not done
//!
//! **It is not yet reached from a turn.** `alo-turn`'s machine carries out the
//! file verbs and offers only those, and it is not this plan's to change; like
//! `print_document` and `install_application`, these verbs are declared, checked
//! before they are proposed, carried out from an approved authority, and tested
//! end to end against a real door. Handing a turn's call to [`proposable`] and
//! its redeemed approval to [`carry_out_approved`] is the daemon's wiring, owed
//! where the report for this task says.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod by_hand;
#[cfg(unix)]
mod carrying_out;
mod choosing;
mod cutting;
mod proposing;
mod refusing;
#[cfg(test)]
mod testing;
pub mod verbs;
mod wanted;
pub mod words;

pub use by_hand::Listed;
#[cfg(unix)]
pub use carrying_out::{TheBroker, carry_out_approved, carry_out_by_hand, set_proxy_by_hand};
pub use choosing::chosen;
pub use cutting::{Answered, would};
pub use proposing::proposable;
pub use refusing::{NotChanged, changed_said, proxy_set_said};
pub use verbs::{
    FORGET_NETWORK, JOIN_NETWORK, SWITCH_WIRELESS, approved, declare_into as declare_verbs_into,
    network_verbs,
};
pub use wanted::{Change, ThisConversation, Wanted};
pub use words::{EVERY_WORD, WordsError, changing_network_words, declare_into};
