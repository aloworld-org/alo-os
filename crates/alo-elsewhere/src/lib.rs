//! The machines a person has added, and whether each one answered.
//!
//! A person with several computers, all their own, adds them here. This crate
//! holds what that list *is* — which machines, what the person calls them, and
//! how each one is reaching — and nothing about how they are reached, which is
//! `alo-nearby`'s.
//!
//! # The promise this crate exists to keep
//!
//! `docs/design/a-persons-other-machines.md` says a machine that cannot be
//! reached is **a state that is shown, never an omission**, and names the shape
//! that breaks it: *a dashboard that is only truthful when every machine is
//! reachable, and a summary that silently omits a machine it could not reach.*
//!
//! So it is not left to whoever draws the list to remember. [`TheMachines::each`]
//! yields every machine **with** how it is reaching, and there is no method that
//! yields the machines alone. A caller cannot produce the dishonest summary
//! without writing the filter themselves and meaning it.
//!
//! # Three states, not two
//!
//! [`Reaching`] separates *did not answer* from *not asked yet*, and
//! [`crate::words`] tells a translator in as many words that merging them is
//! wrong. A machine added a moment ago has not failed — nobody has asked it —
//! and a person shown a failure thirty seconds after adding a machine that is
//! fine goes looking for a fault that does not exist.
//!
//! # What is deliberately not here
//!
//! **Asking.** Nothing in this crate opens a socket. The state is written down
//! by whoever did the asking, through [`TheMachines::now_reaching`].
//!
//! **What a machine may do.** A grant is a separate thing a person gives, under
//! [ADR 0079](../../../docs/decisions/0079-a-person-may-hand-their-agent-a-whole-machine.md),
//! and being on this list permits nothing.

pub mod machine;
pub mod machines;
pub mod reaching;
pub mod refusing;
pub mod words;

pub use machine::{AMachine, AT_MOST, TheName};
pub use machines::TheMachines;
pub use reaching::Reaching;
pub use refusing::NotElsewhere;
pub use words::{declare_into, elsewhere_words};
