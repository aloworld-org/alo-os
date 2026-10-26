//! Asking the daemon what is waiting, and answering it.
//!
//! A trait of its own rather than a third half of [`crate::Door`]. `Door` is
//! `Knocking + RevokingPairings`, and every surface that holds one — and every
//! stand-in a test writes — would have to grow this too. A surface that
//! answers approvals needs this; Settings does not; so they are separate and a
//! caller asks for exactly what it uses.
//!
//! # It decides nothing, and words nothing
//!
//! **One approval is one execution**, and that is not kept here. It is
//! `alo_capability::Approved::redeem`'s, and a second answer to the same
//! number is refused there. A door that also guarded it would be a second
//! place to get it wrong while the daemon's stayed the one that counts.
//!
//! **A refusal arrives already worded**, in the language the person reads, and
//! [`crate::Answered::Refused`] carries it unchanged. Nothing here shortens it
//! or decides what it meant.
//!
//! # Why every answer is an `Option`
//!
//! For [`crate::Knocking`]'s reason: a daemon that is not running, a socket
//! that will not open and an answer that does not parse are the same fact to a
//! surface — **nobody said anything** — and a surface that had to tell them
//! apart would be deciding something it cannot see. [`None`] is that, and what
//! a surface does about it belongs to the surface.

use alo_protocol::Standing;

use crate::answered::Answered;

/// What a person's side can ask about changes that are waiting.
pub trait Answering: std::fmt::Debug {
    /// Everything proposed and not yet answered, in the order it was asked.
    ///
    /// **The list the turn holds**, each change carrying its own number, its
    /// own sentence in the language the person reads, how long is left, and
    /// which machine proposed it. A surface draws those and invents nothing.
    fn waiting(&self) -> Option<Vec<Standing>>;

    /// The person approved the change standing under this number.
    ///
    /// Worth exactly one execution, which the daemon keeps and this does not.
    fn approve(&self, number: u64) -> Option<Answered>;

    /// The person said no. Nothing is carried about why, because nothing was
    /// asked.
    fn decline(&self, number: u64) -> Option<Answered>;
}
