//! A verb crosses between two paired machines, and is proven at the door.
//!
//! [ADR 0003]'s sharpest line — *pairing lets A ask; it never lets A act* —
//! and [ADR 0031]'s — *the pairing is the key* — met on a wire. An agent on
//! one machine asks for a verb on another; the verb reaches the machine it
//! names at the address discovery measured, carrying its name, its typed
//! arguments and a proof over exactly those bytes; and the receiving machine
//! judges the proof before it asks anything else of itself, then walks the
//! verb through the one door a remote verb has — `alo_turn::Arriving` — on
//! its own grants, for its own person, into its own record.
//!
//! # The two ends
//!
//! **The asking end** is [`Crossing`]: made only from a pairing this machine
//! holds and a machine discovery found, it puts a read or a change to the
//! other machine and hands back a [`Crossed`] holding the answer and the
//! [`alo_egress::Departing`] the indicator showed while it went. It hands the
//! departure back rather than writing it down, exactly as `alo-asking` does:
//! the record is the turn's, and the turn on the asking machine writes it.
//!
//! **The receiving end** is [`Receiving`] and [`Doorway`]: a connection is
//! accepted and read, and what arrived is judged in one order — is there a
//! proof, does it hold ([`alo_nearby::Proven::checked`], through
//! [`alo_nearby::Origin::proven`]), is a turn open here for that machine or
//! may one begin, does the body read as a verb, and then the door. Nothing on
//! this machine is consulted before the proof holds, and nothing is written
//! about a message whose proof did not.
//!
//! # And every answer leaves under the indicator
//!
//! An answer to a remote read is this machine's data leaving it, and law 1 is
//! not suspended for the length of a corridor. Every reply to a proven verb —
//! the answer, the number a change waits under, or the door's refusal — is
//! written to the socket only from a [`Replying`] made with a `Departing`
//! that `alo_turn::Arriving::departing` handed over, after the egress rule in
//! force said yes; the departure is then written down, stamped with where the
//! verb came from, and the line comes off. A rule that says nothing leaves
//! holds the answer back, written down as such, and the connection closes
//! with nothing on it.
//!
//! What is *not* under a departure is the word written back to a message that
//! proved nothing: *not paired*, *not from the machine it names*, *already
//! used*. It carries nothing of this machine — no agent on this machine
//! caused it, no grant was asked, and nothing was written — and it is there so
//! the person who asked can be told, in their language, what to do.
//!
//! # What the wire is
//!
//! HTTP on the port presence advertises, beside the pairing wire's two paths:
//! `POST` [`THE_READ_PATH`] or [`THE_CHANGE_PATH`], the proof in
//! [`alo_asking::THE_PROOF_HEADER`], and a body that is exactly a
//! [`Carried`] — the verb and its arguments — and nothing else. A field this
//! crate has no place for is refused rather than read around. The reply is an
//! [`Answered`] with status 200, or one word from [`AtTheDoor`]'s closed list
//! with a status that says so.
//!
//! # What is not here
//!
//! **No new verb, and nothing widening the list.** `alo-capability` decides
//! what a grant is and this crate never learns; the wire decides *whose* grant
//! is asked and how the asker is proven. **No daemon.** [`Receiving`] takes a
//! listener somebody bound, as `alo_nearby::Receiving` does; which daemon
//! binds the advertised port and holds a [`Doorway`] is the change that lets
//! a machine be asked. **No question.** A question to a paired machine's
//! models goes down `alo-asking`'s corridor, under the arm of the pairing
//! that permits it; a verb has no such arm, because what it may do is the
//! receiving person's grants and nothing on the pairing.
//!
//! [ADR 0003]: https://github.com/aloworld-org/alo-os/blob/main/docs/decisions/0003-the-network-is-not-authority.md
//! [ADR 0031]: https://github.com/aloworld-org/alo-os/blob/main/docs/decisions/0031-the-pairing-is-the-key.md

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod answered;
mod carried;
mod crossing;
mod dialling;
mod door;
mod doorway;
mod holding;
mod naming;
mod receiving;
mod refusing;
mod replying;
#[cfg(test)]
mod testing;
pub mod words;

pub use answered::{AT_MOST_AN_ANSWER, Answered};
pub use carried::{AT_MOST_A_VERB, Carried};
pub use crossing::{Crossed, Crossing};
pub use door::AtTheDoor;
pub use doorway::{AT_MOST_A_TURN, Door, Doorway, Judged, NotADoorway};
pub use holding::Holding;
pub use naming::Naming;
pub use receiving::{Arrived, Heard, Receiving, THE_CHANGE_PATH, THE_READ_PATH};
pub use refusing::{Left, NotCrossed, WentBack};
pub use replying::Replying;
pub use words::{EVERY_WORD, corridor_words, declare_into};
