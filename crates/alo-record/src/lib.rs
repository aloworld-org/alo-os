//! What an agent did, and what it was stopped from doing.
//!
//! This crate is
//! [ADR 0001](../../../docs/decisions/0001-the-capability-model.md) §7 as
//! working code. Every execution **and every refusal** is kept with what ran,
//! under whose authority, from which approval and against which grant — and
//! "explain what it did" is a query put to [`Record`], not a log somebody
//! greps.
//!
//! **The refusals are the point.** A record keeping only successes cannot
//! answer what a security review actually asks, which is not *what did it do*
//! but *what did it try*. So all four things that can happen to an attempt are
//! kept, and three of them are ways of being stopped:
//!
//! | | |
//! |---|---|
//! | [`Happened::Ran`] | It ran, with ADR 0001 §7's four answers |
//! | [`Happened::Stopped`] | A properly formed call that was refused — and [`Stopped`] says where in the journey |
//! | [`Happened::TurnedAway`] | Something that never became a call at all |
//! | [`Happened::Answered`] | Where a question was answered ([ADR 0008](../../../docs/decisions/0008-where-inference-happens.md)) |
//!
//! # Why this is not part of `alo-capability`
//!
//! Because the record must be able to read itself back, and the capability
//! model must not. [`alo_capability::Call`], [`alo_capability::Value`] and
//! [`alo_capability::Proposal`] serialise and deliberately do not deserialise:
//! one read back off a disk would be a call nothing had validated, and a type
//! whose whole meaning is *this was checked* would become a promise instead of
//! a fact. A record has the opposite job — it exists to be read back — so it
//! keeps its own types, [`What`] and [`Written`], and there is no way from
//! either of them to anything that runs. **A record is evidence, not an
//! instruction.**
//!
//! The second reason is what a reader has to hold in their head. `alo-capability`
//! decides; this crate remembers. Keeping them apart means the crate that
//! decides depends on nothing but `serde` and `thiserror`, which is what keeps
//! it small enough to be audited by somebody who did not write it — and it
//! means nothing on the deciding path can reach the record, so a future grant
//! list cannot quietly start writing entries about itself.
//!
//! # Telling the time
//!
//! Nothing here reads the clock, as in `alo-capability` and for the same
//! reason. [`Entry::ran`] is the exception that proves it: it takes its moment
//! from the [`alo_capability::Authorised`] it is recording, because the moment
//! that matters is the one the grants were asked at, not the one the writing
//! happened at.
//!
//! # What is not kept
//!
//! **Not what was asked.** [`Happened::Answered`] records *where* a question
//! was answered and never the question — a record that kept those would be a
//! transcript of everything a person ever said to their machine, which is the
//! thing this product exists not to be.
//!
//! **Not the arguments of a call that never validated.** They are whatever a
//! model was persuaded to send, and an entry that carried them would look like
//! every other entry while saying something nobody did.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod entry;
pub mod explain;
pub mod happened;
pub mod line;
pub mod record;
pub mod what;
pub mod written;

#[cfg(test)]
mod test_calls;

pub use entry::Entry;
pub use explain::{Asking, Only};
pub use happened::{Happened, Stopped};
pub use line::Line;
pub use record::Record;
pub use what::What;
pub use written::Written;
