//! What is holding the receiving machine between verbs from the network.
//!
//! `alo_turn::Arriving` borrows the machine mutably and there is one machine,
//! so while a remote turn is under way there is nothing else that can reach
//! it — and between turns there is no turn. That is `alo-agentd`'s
//! `Holding` for the person's door, and this is the same shape for the
//! network's: the machine, or the turn holding it, and never neither. A
//! type rather than two fields because two `Option`s of which exactly one is
//! ever `Some` is a state nothing can produce and every reader has to rule
//! out.
//!
//! **One remote turn at a time.** A verb from a third machine while a turn is
//! open for another is refused at the door as `another-turn-is-open`, and
//! that is a consequence of the borrow rather than a policy: two remote turns
//! on one machine would be two turns writing into one record with no order
//! between them, which is the thing `alo_turn::Machine` is built to refuse.

use alo_turn::{Arriving, Machine};

/// The machine, or the remote turn that is holding it.
#[derive(Debug)]
pub enum Holding<'a, 'm> {
    /// Nobody is, so the machine itself is here, ready for the next verb.
    Nobody(&'a mut Machine<'m>),
    /// A remote turn is under way and has the machine.
    ///
    /// Boxed: a turn is a few hundred bytes and a borrow is eight, and the
    /// enum is moved in and out of the doorway at every verb.
    ATurn(Box<Arriving<'a, 'm>>),
}

impl<'a, 'm> Holding<'a, 'm> {
    /// The turn, when there is one.
    pub fn turn(&mut self) -> Option<&mut Arriving<'a, 'm>> {
        match self {
            Self::ATurn(arriving) => Some(arriving.as_mut()),
            Self::Nobody(_) => None,
        }
    }
}
