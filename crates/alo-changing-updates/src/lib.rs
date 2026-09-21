//! What a person reads when an update is applied, or this machine goes back to
//! the version it ran before one, through the privileged broker's two update
//! verbs.
//!
//! ★ *System verbs through the privileged broker: printers, network, updates,
//! storage.* Changing the system every person on a machine boots into is the
//! largest change there is, so ADR 0001 §2 puts it behind the broker, and
//! [ADR 0053](../../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)
//! puts the carrying out behind a unit the broker starts — so that the broker
//! itself still holds no capability. Task 8 of
//! `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` built that road. This
//! crate is what that road did not have: **a person in front of it.**
//!
//! | | |
//! |---|---|
//! | [`Change`] | Apply it, or go back — and the broker verb each is |
//! | [`NotChanged`], [`changed_said`] | What a person reads, either way |
//! | [`words`] | The four sentences that are this crate's, with a note for whoever translates each |
//!
//! # Four sentences, because the other eleven already existed
//!
//! Everything a person reads *about an update* was written where updates are
//! decided — `alo_keeping_up::words`: *an update is ready*, *the update will
//! apply the next time you restart*, *it could not be prepared*, *this machine
//! changed after the update was found*. This crate adds none of them and says
//! them instead, so that a person reads one line for one fact however it
//! reached them.
//!
//! What was missing is what only the **door** can answer, and it is three
//! things: an approval it would not take, a machine that has stopped writing
//! changes down, and nothing there to make them. They are facts about the road
//! rather than about the update, which is why `alo-keeping-up` had no reason to
//! have them and why the printers and the network each say their own.
//!
//! # What it deliberately does not do
//!
//! **No second road to the door**, and **no instruction of its own.** What the
//! base may be told is `alo_keeping_up::Staging` and `alo_keeping_up::Returning`
//! and nowhere else (ADR 0053); a second place that assembled those arguments
//! would be a second answer to what applying an update means. Nothing here
//! opens a socket, names a build, or holds a digest.
//!
//! **It is not yet reached from a turn.** Like the printers' and the network's,
//! these verbs are carried out from an approved authority and tested end to end
//! against a real door; handing a turn's redeemed approval to that road is the
//! daemon's wiring, owed where task 8's report says.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod refusing;
mod wanted;
pub mod words;

pub use refusing::{NotChanged, changed_said};
pub use wanted::Change;
pub use words::{EVERY_WORD, WordsError, changing_updates_words, declare_into};
