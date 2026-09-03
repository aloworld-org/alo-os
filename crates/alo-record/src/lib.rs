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
//! but *what did it try*. So all seven things that can happen are kept, and
//! three of them are ways of being stopped:
//!
//! | | |
//! |---|---|
//! | [`Happened::Ran`] | It ran, with ADR 0001 §7's four answers |
//! | [`Happened::Stopped`] | A properly formed call that was refused — and [`Stopped`] says where in the journey |
//! | [`Happened::TurnedAway`] | Something that never became a call at all |
//! | [`Happened::AnsweredHere`] | A question answered on this machine ([ADR 0008](../../../docs/decisions/0008-where-inference-happens.md)) |
//! | [`Happened::Left`] | Something left this machine (law 1) |
//! | [`Happened::HeldBack`] | Something the egress policy refused to let leave |
//! | [`Happened::LeftOnItsOwn`] | alo OS reached the network with nobody having asked (★ *no telemetry*) |
//!
//! # The seventh, and the one with nobody in it
//!
//! Six of the seven are an agent's, and answer *whose authority was this
//! under*. The seventh is the machine's own errand, and it has **no agent
//! field** — [`Entry::agent`] answers `None` for it, because nobody granted alo
//! OS permission to sign somebody in and a name in that position would be an
//! authority the record invented. [`happened`] has the reasoning, and it is the
//! answer `alo-egress` already gave one crate earlier.
//!
//! It is still a departure: [`Only::Egress`] finds it, so *what left this
//! machine today* answers with everything that left. [`Only::OnItsOwn`] is the
//! half of that nobody caused.
//!
//! It has the other half too, over the whole record rather than over the
//! departures. [`Only::ByAnAgent`] finds everything **with** a name in that
//! column, which is what somebody who declined the agent entirely
//! ([`alo_capability::Agent`], ADR 0009) puts to their own record: *is there
//! anything in here that an agent did?* Naming every agent that might have run
//! and trusting the list is no way to establish that none did.
//!
//! # What left, kept once
//!
//! Law 1 asks *what left this machine today* and law 1's answer has to be one
//! entry per departure. An answer from a provider is both a departure and where
//! an answer came from, so it is **one** entry — the departure — and the
//! destination says everything an inference source said. A question answered
//! here never left and is [`Happened::AnsweredHere`], which has nowhere to
//! name. [`happened`] has the reasoning.
//!
//! The guarantee that goes with it is [`departed`]'s: an egress entry can only
//! be made from an [`alo_egress::Departing`], and the indicator is the only
//! maker of one of those. **An egress the indicator never showed is not an
//! entry that can be written**, and neither is one it showed and the record
//! then contradicted. The same holds of an errand and an
//! [`alo_egress::Underway`], which is the whole of what makes the record of
//! ★ *no telemetry* worth more than the sentence.
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
//! reason. [`Entry::ran`], [`Entry::left`] and [`Entry::left_on_its_own`] are
//! the exceptions that prove it: each takes its moment from the thing it is
//! recording — the moment the grants were asked, and the moment the egress went
//! on the indicator — because that is the moment it was allowed to happen, and
//! not the one the writing happened at.
//!
//! # What is not kept
//!
//! **Not what was asked.** [`Happened::AnsweredHere`] records that a question
//! was answered and never the question, and [`Happened::Left`] records where
//! something went and never what went — a record that kept those would be a
//! transcript of everything a person ever said to their machine, which is the
//! thing this product exists not to be.
//!
//! **Not the arguments of a call that never validated.** They are whatever a
//! model was persuaded to send, and an entry that carried them would look like
//! every other entry while saying something nobody did.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod departed;
pub mod entry;
pub mod explain;
pub mod happened;
pub mod line;
pub mod record;
pub mod what;
pub mod written;

#[cfg(test)]
mod test_calls;
#[cfg(test)]
mod testing;

pub use entry::Entry;
pub use explain::{Asking, Only};
pub use happened::{Happened, Stopped};
pub use line::Line;
pub use record::Record;
pub use what::What;
pub use written::Written;
