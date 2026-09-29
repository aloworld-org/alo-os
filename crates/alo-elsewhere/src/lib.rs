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
//! **Asking.** Nothing here opens a socket, and the list of machines permits
//! nothing at all.
//!
//! # What a person grants is separate, and deliberately so
//!
//! [`Driving`] is whether the person's agent may drive one of their machines,
//! under [ADR 0079](../../../docs/decisions/0079-a-person-may-hand-their-agent-a-whole-machine.md).
//! It shares no state with [`TheMachines`]: **being on the person's list of
//! machines permits nothing**, and the two are never read from one place.
//!
//! That grant is the only one in this product that cannot enumerate what it
//! permits, because a pointer and a keyboard reach everything a machine can do.
//! [`WHAT_IT_CANNOT_ENUMERATE`] is the sentence the person reads when they give
//! it, and showing it is a term of that record rather than a choice about
//! presentation.

pub mod driving;
pub mod machine;
pub mod machines;
pub mod reaching;
pub mod refusing;
pub mod words;

pub use driving::{Driving, MayDrive, WHAT_IT_CANNOT_ENUMERATE};
pub use machine::{AMachine, AT_MOST, TheName};
pub use machines::TheMachines;
pub use reaching::Reaching;
pub use refusing::NotElsewhere;
pub use words::{declare_into, elsewhere_words};
