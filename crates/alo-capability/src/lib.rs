//! What an agent may reach, and under whose authority.
//!
//! This crate is
//! [ADR 0001](../../../docs/decisions/0001-the-capability-model.md) as working
//! code. It holds the part of `alo-agentd` that is pure logic — grants today,
//! and the verb registry, approvals and the record beside it — so that the
//! rules an agent is bound by can be read, tested and argued with on their own,
//! rather than being discovered inside a running daemon.
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
//! # Telling the time
//!
//! Every question that depends on time takes `now` as an argument. Nothing here
//! reads the clock. That keeps expiry testable without sleeping, and it keeps
//! the decision in the caller's hands: the daemon answering a verb and the
//! settings panel listing grants must agree on what "now" means, and the way to
//! guarantee that is to pass it.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod grant;
pub mod grants;
pub mod path;
pub mod reach;

pub use grant::{Grant, GrantError, Grantee};
pub use grants::{GrantId, Grants, Held};
pub use reach::{Ask, Reach};
