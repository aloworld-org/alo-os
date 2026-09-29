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
//! # A machine on the canvas is a window, and not a new kind of one
//!
//! [`WhichMachines`] is a note saying which window is showing which machine, and
//! that is all it is. The window itself is an ordinary [`alo_dock::Window`]: it
//! sits on the canvas, is put aside, or fills the screen, and the Dock's rule
//! answers a click on it without being rewritten. [`an_app_for`] gives each
//! machine its own stable identifier, so *click an app to return to where you
//! last used it* reads as *click a machine to return to where you last were on
//! it*.
//!
//! # And it never stops saying it is elsewhere
//!
//! [`Marked`] is what a window carries so a person can tell it is not this
//! machine. It is **the machine's name — a word**, never a colour, following
//! `docs/design/who-is-acting.md` and ADR 0010: a signal carried by hue alone is
//! not a signal. It is shown in every state a window can be in, and there is no
//! argument a caller could pass to suppress it, because **filling the screen is
//! the one case it exists for** — every other cue is gone and what is left is a
//! screen showing a machine.
//!
//! A window whose machine the person has removed says
//! [`Marked::ElsewhereUnnamed`] and never [`Marked::Here`]. A remote window that
//! looks local is what this costs a password.
//!
//! # Work sent to another machine
//!
//! [`TheWork`] is what this machine has asked another of the person's machines
//! to do. What crosses is [`AGoal`] — **words, with nowhere to put a program, an
//! argument or a path** — because the working machine decides how, under its own
//! rules. A struct with somewhere to put an argument list is a command protocol
//! whatever its documentation says.
//!
//! The record lives on the machine that asked, so a result comes back here
//! because there is nowhere else it could be addressed. And a result arriving
//! after the person stopped something is refused rather than kept: keeping it
//! would undo the stop quietly, which is the one outcome pressing stop rules
//! out.
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

pub mod as_a_window;
pub mod changing;
pub mod driving;
pub mod machine;
pub mod machines;
pub mod reaching;
pub mod refusing;
pub mod saying_where;
pub mod sending;
pub mod words;

pub use as_a_window::{A_MACHINES_APP, WhichMachines, an_app_for};
pub use changing::{ARow, Change, TheRows, WhatTheAgentHas, carry_out};
pub use driving::{Driving, MayDrive, WHAT_IT_CANNOT_ENUMERATE};
pub use machine::{AMachine, AT_MOST, TheName};
pub use machines::TheMachines;
pub use reaching::Reaching;
pub use refusing::NotElsewhere;
pub use saying_where::Marked;
pub use sending::{AGoal, APieceOfWork, AT_MOST_A_GOAL, HowItIsGoing, TheWork, WorkId};
pub use words::{declare_into, elsewhere_words};
