//! What an agent may reach, and under whose authority.
//!
//! This crate is
//! [ADR 0001](../../../docs/decisions/0001-the-capability-model.md) as working
//! code. It holds the part of `alo-agentd` that decides: grants, the verb
//! registry and approvals, so that the rules an agent is bound by can be read,
//! tested and argued with on their own, rather than being discovered inside a
//! running daemon.
//!
//! **What happened is remembered elsewhere**, in `alo-record`. Nothing in this
//! crate writes anything down, and the separation is deliberate twice over: a
//! crate that decides should not be reachable from one that observes, and this
//! one deliberately cannot read its own decisions back — [`Call`], [`Value`],
//! [`Proposal`] and [`Approvals`] serialise and do not deserialise, because a
//! call read back off a disk would be one nothing had validated. A record has
//! the opposite job, so it keeps types of its own.
//!
//! **Grants are the durable thing.** Verbs are what may be done; a grant is
//! what they may be done *to*. Nothing else in the system decides reach: if a
//! path, a file or an application is not covered by a grant a person made, no
//! verb can touch it, whatever the model has been persuaded to ask for.
//!
//! The four properties come from ADR 0001 §3 and are enforced by the types
//! rather than by convention:
//!
//! - **enumerated** — [`Grants`] is a list a person can read, never a rule they
//!   must reason about;
//! - **deliberate** — a [`Grant`] is constructed from a folder somebody picked;
//!   nothing here widens one, and asking about a path never adds it;
//! - **revocable** — [`Grants::revoke`] takes effect on the next question,
//!   because there is no cache in front of it;
//! - **expiring** — a [`Grant`] cannot be built without an end
//!   ([`Grant::checked`] takes a duration and refuses zero), so "for ever" is
//!   not a value this type can hold.
//!
//! There is no grant to `/`, and [`GrantError::TheWholeMachine`] is what
//! happens to code that tries.
//!
//! # Verbs
//!
//! A grant is what may be touched; a [`Verb`] is what may be done. [`Verbs`] is
//! the closed list of them — `docs/contracts/agent-verbs.md` says that a
//! capability not written down does not exist, and this is where that stops
//! being a sentence. Nothing here executes anything: a name and a set of
//! arguments become a [`Call`], which knows what it would touch, what a person
//! would be approving, and whether it waits for that approval at all.
//!
//! Law 2 — no verb runs an arbitrary command — is carried by two things
//! together. [`Takes`] is a closed list of what an argument can be, and none of
//! them is free text, so there is no shape in which something to run could
//! arrive. And [`Verb::checked`] refuses a declaration that announces an
//! interpreter, which catches the plausible mistake at the moment it is
//! written. What neither can do is stop a verb's *implementation* from passing
//! an argument to a shell, and that stays a rule on whoever writes one.
//!
//! Three questions are kept apart, because merging any two of them is how the
//! model would come undone:
//!
//! 1. **is the call well-formed** — [`Verbs::call`], which validates every
//!    argument at the boundary and generates the sentence;
//! 2. **is it permitted** — [`Call::permitted_by`], against the grants;
//! 3. **is it approved** — [`Approvals`], and only for a change.
//!
//! # Approvals
//!
//! ADR 0001 §5: a read answers inside the turn, a change waits for one
//! approval of the sentence describing it. The journey a change makes is four
//! types long, and each of them can only be reached from the one before:
//!
//! 1. [`Call`] — validated, with its sentence generated from those arguments;
//! 2. [`Proposal`] — the question put to a person, which lapses. A read is
//!    refused here, and so is a change the grants already do not permit;
//! 3. [`Approved`] — the answer, worth exactly one execution. It is not
//!    `Clone`, it holds the call rather than lending out its arguments, and
//!    redeeming it consumes it;
//! 4. [`Authorised`] — a call that may run now, which an executor takes by
//!    value. A read reaches it directly through [`Authorised::read`]; a change
//!    reaches it only through [`Approved::redeem`]. It carries all four answers
//!    ADR 0001 §7 asks of a record — what ran, under whose authority, from
//!    which approval, and against which grant ([`Authorised::against`]) —
//!    because this is the one moment all four are true at once.
//!
//! **The grants are asked last at the moment of execution**, which is what
//! makes a revoked grant take effect immediately: an approval given at noon and
//! redeemed after the folder was revoked is refused, because nothing decided
//! anything ahead of time.
//!
//! There is no "remember this", no duration on an approval and no allowing
//! something for an application. Durable permission is a grant.
//!
//! # Telling the time
//!
//! Every question that depends on time takes `now` as an argument. Nothing here
//! reads the clock. That keeps expiry testable without sleeping, and it keeps
//! the decision in the caller's hands: the daemon answering a verb and the
//! settings panel listing grants must agree on what "now" means, and the way to
//! guarantee that is to pass it.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod approval;
pub mod approvals;
pub mod arg;
pub mod authorised;
pub mod call;
pub mod grant;
pub mod grants;
pub mod path;
pub mod proposal;
pub mod reach;
pub mod sentence;
pub mod verb;
pub mod verbs;

#[cfg(test)]
mod test_calls;

pub use approval::Approved;
pub use approvals::{AnswerError, Approvals, ProposalId, Waiting};
pub use arg::{Arg, ArgError, Given, Takes, Value};
pub use authorised::{Authorised, NotAuthorised, Refused};
pub use call::{Call, CallError};
pub use grant::{Grant, GrantError, Grantee};
pub use grants::{GrantId, Grants, Held};
pub use proposal::{Proposal, ProposalError};
pub use reach::{Ask, Reach};
pub use sentence::{Part, Sentence, SentenceError};
pub use verb::{Effect, Requires, Verb, VerbError};
pub use verbs::{Verbs, VerbsError};
