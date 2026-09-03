//! One turn: what an invocation offered, what an agent asked for, what a
//! person approved, what the machine did, and what the record says about it.
//!
//! Thirteen crates in this workspace each decide one thing correctly and none
//! of them is joined to the next. This is the joining: the order the steps
//! happen in, and the guarantee that none of them can be skipped. It reaches
//! [`alo_context`], [`alo_capability`], [`alo_files`], [`alo_record`] and
//! [`alo_keeping`], and nothing reaches it.
//!
//! # The order, and why each step is where it is
//!
//! 1. **The invocation makes the turn.** [`alo_context::Turn`] takes what was
//!    offered — a window, a selection, a document — and grants the document and
//!    nothing else, for the length of the turn. A turn cannot begin on a
//!    machine where the person declined an agent (ADR 0009), because
//!    [`Turning::beginning`] needs the machine's grants and such a machine has
//!    none to lend.
//! 2. **A name and some values become a call, or they do not become one.**
//!    [`Turning::reading`] and [`Turning::proposing`] take the verb's name and
//!    what was given for each argument, and put both through
//!    [`alo_capability::Verbs::call`]. There is no door here that takes a call
//!    somebody else made, so the closed list is asked every time. Law 2 is that
//!    sentence: what an agent can ask for is what the registry holds, and there
//!    is no shape in which anything else could arrive.
//! 3. **A read answers inside the turn; a change waits for one approval.** ADR
//!    0001 §5, and it is two doors rather than a flag — a change offered to
//!    [`Turning::reading`] is refused by [`alo_capability::Authorised::read`],
//!    and a read offered to [`Turning::proposing`] is refused by
//!    [`alo_capability::Proposal::checked`].
//! 4. **The grants are asked at the moment of execution.** Not once, at the
//!    beginning: [`alo_capability::Approved::redeem`] asks them again when the
//!    approval is spent, [`alo_files::Touching`] asks about where each path
//!    really leads, and [`alo_files::Did`] asks about anything the call would
//!    create. A grant revoked between the approval and the execution stops it.
//! 5. **What happened is written down before anybody is told about it.** That
//!    is this crate's own rule and the next section is about it.
//! 6. **The turn ends and the grant goes with it**, whether or not anything was
//!    done under it.
//!
//! # Nothing is handed back that has not been written down
//!
//! `CLAUDE.md`'s gate asks that *every execution and every refusal leaves a
//! record*, and until this crate that was a sentence somebody had to remember.
//! Here it is the shape of the code: a [`Turning`] cannot be made without
//! somewhere to keep its record ([`Kept`], held by the [`Machine`] it runs on),
//! and every door writes its entry **before** it answers. There is no path
//! through this crate that reaches a caller without an entry having been
//! written first.
//!
//! What cannot be closed here is the window a change leaves open: a file has
//! moved on the disk before there is anything to write down about it, so a
//! record that cannot be written after that is a thing that happened with no
//! evidence of it. The answer is not to pretend otherwise. **A turn that could
//! not write something down does nothing else** — every door afterwards answers
//! [`NotDone::TurnClosed`], and the daemon that holds the turn has one thing
//! that happened and one reason its machine is no longer keeping evidence.
//! `docs/quirks.md` records the window and what closes it.
//!
//! # This crate says almost nothing
//!
//! One string, and [`crate::words`] is the argument for why. Every refusal a
//! turn can hand back was already worded by whoever made it — the call that did
//! not form, the change that was never put to anybody, the grants at the last
//! moment, the disk — and a second rendering of any of them would be a machine
//! able to describe one moment two ways. What is left is the one thing none of
//! them knows: that this turn has stopped.
//!
//! # What a turn does not do
//!
//! **It does not decide anything an agent should ask for.** A model's answer
//! becoming a verb and some arguments is the *agent's* work, and an agent is a
//! client of `alo-agentd` rather than a part of it — item 21's protocol takes
//! enumerated verbs with typed arguments, and this crate is what is behind that
//! protocol. Whatever composes a call composes it out of a name and values, and
//! walks in through step 2 above like everything else.
//!
//! **It puts no question to a model.** `alo-asking` has all three doors for
//! that and needs an indicator, a policy and a departure that reaches the
//! record by a road of its own; joining it to a turn is the next item and is
//! deliberately not here.
//!
//! **It reaches no window.** `alo-applications` decides what an agent may do to
//! an application and stops at its `Reaching`, because what actually moves a
//! window is Wayland and D-Bus on a Linux host. A [`Machine`] offers the verbs
//! it can carry out and no others, so an agent on this machine cannot ask for
//! one that would stop there.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod carrying;
pub mod kept;
pub mod machine;
pub mod refusing;
pub mod turning;
pub mod words;

#[cfg(test)]
mod testing;

pub use kept::Kept;
pub use machine::Machine;
pub use refusing::NotDone;
pub use turning::Turning;
pub use words::{EVERY_WORD, Word, WordsError, declare_into, turn_words};
