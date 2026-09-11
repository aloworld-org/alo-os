//! The grants a person can see, before there is anywhere to show them.
//!
//! ADR 0001 §3 makes visibility part of the grant model itself: grants are
//! *enumerated, deliberate, revocable, expiring* — and **visible where the
//! person can find them**, because a grant a person cannot find is a grant
//! they cannot revoke. `alo-capability` has held the other four properties
//! from the start, `alo-picking` gave a person the act that makes a grant,
//! `alo-remembering` keeps the list between one sign-in and the next, and
//! `alo-agentd` enforces it — and nothing had ever shaped the list for a
//! person to look at, so *see what is granted* was the one clause of the
//! promise with no surface behind it.
//!
//! This crate is that surface's model, and deliberately nothing more:
//!
//! - [`Listing`] — what a surface would show, derived from the machine's own
//!   [`alo_capability::Grants`] at one moment and from nothing else, with the
//!   *nothing granted* state a sentence rather than an empty list;
//! - [`Seen`] — one row: who may reach what, since when, and for how much
//!   longer, with no constructor a caller can reach;
//! - [`Revoked`] — what revoking a row comes back as: done and already
//!   stopped, or a stale row that changed nothing at all.
//!
//! ```
//! use alo_capability::{Ask, Grant, Grantee, Grants, Reach};
//! use alo_granted::{Listing, Revoked};
//! use std::path::PathBuf;
//! use std::time::{Duration, SystemTime};
//!
//! let noon = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
//! let mut grants = Grants::default();
//! grants.grant(Grant::checked(
//!     "@files",
//!     Reach::Folder(PathBuf::from("/home/anna/Invoices")),
//!     noon,
//!     Duration::from_secs(60 * 60),
//! )?);
//!
//! // The list is the machine's own grants, read at one moment.
//! let listing = Listing::of(&grants, noon);
//! let row = listing.rows().first().expect("one grant, one row");
//! assert_eq!(row.to(), "@files");
//!
//! // Revoking through the row is the machine's own revocation: the daemon's
//! // next question is refused, with nothing to wait for.
//! assert_eq!(row.revoke(&mut grants), Revoked::Now);
//! assert!(!grants.permits(
//!     &Grantee::named("@files"),
//!     &Ask::path("/home/anna/Invoices/march.pdf"),
//!     noon,
//! ));
//!
//! // And the empty machine is a sentence, not an empty list.
//! assert!(Listing::of(&grants, noon).is_nothing_granted());
//! # Ok::<(), alo_capability::GrantError>(())
//! ```
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`listing`] | The list, derived from the machine's own grants at one moment |
//! | [`seen`] | One row, and revoking through it |
//! | [`revoking`] | The two things a revocation comes back as |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! # There is one door, and it is the machine's own list
//!
//! The plan's acceptance for this task is a sentence about provenance — *the
//! list a surface would show is derived from the machine's own kept grants
//! and from nothing else — no constructor from text, so nothing can show a
//! grant the machine does not hold* — so provenance is what the types here
//! are shaped around rather than what their documentation promises.
//!
//! [`Listing::of`] takes `&alo_capability::Grants` — the value the daemon's
//! own `permits` answers from, whether it was granted this session or read
//! back off the disk through `Grants::remembered`, which is `alo-remembering`'s
//! one road in. [`Seen`] has no public field, no `From`, no deserialiser and
//! no constructor from a path or a name, so a compositor can only ever be
//! handed rows that came off the machine's own list. The compile-fail
//! examples on [`Seen`] and [`Listing`] are that argument as tests: a second
//! door added later stops being a design discussion and starts being a
//! failing build.
//!
//! # Revoking is the machine's own revocation, not a second mechanism
//!
//! [`Seen::revoke`] calls [`alo_capability::Grants::revoke`] with the handle
//! the row was derived with, and does nothing beside it. That is the same
//! revocation the daemon enforces — it takes effect on the next question
//! asked, because there is no cache in front of the grants and no decision
//! made ahead of time — so what a person does in their list and what the
//! daemon refuses a moment later cannot be two mechanisms that disagree.
//! Handles are never reused (`alo-capability`'s own rule), so a revocation
//! made from a stale list cannot land on a grant made since: it lands on
//! nothing, and [`Revoked::AlreadyGone`] says so in words.
//!
//! # Three things this crate is deliberately not
//!
//! **It is not a capability.** A person looking at their own grants, or
//! taking one away, is not an agent doing something — so there is no verb, no
//! proposal and no approval here, and nothing an agent could call to read
//! this list. Law 2's enumerated verbs are not where a person's own oversight
//! belongs.
//!
//! **It does not draw anything.** No panel, no rows of pixels, no button. The
//! drawing is the compositor's, in `crates/alo-shell`, which is the desktop
//! lane's; this is the value that lane wires to, with its refusals decided
//! before anybody can see them.
//!
//! **It does not make grants.** There is no method here that adds one, and
//! there could not be: making a grant is a person picking a folder
//! (ADR 0001 §3, `alo-picking`), and a surface that could both show and make
//! would be one bug away from showing what it made.
//!
//! # Nothing here says anything in English by itself
//!
//! Every sentence a person can be shown is declared in [`words`] and answered
//! through a `said` in the language they read: the empty list's sentence, the
//! row, and both answers to a revocation. The *what* inside a row is
//! `alo-capability`'s own clause, carried rather than re-worded, because a
//! machine with two accounts of what one grant covers would be a machine a
//! person cannot check.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod listing;
pub mod revoking;
pub mod seen;
pub mod words;

#[cfg(test)]
mod testing;

pub use listing::Listing;
pub use revoking::Revoked;
pub use seen::Seen;
pub use words::{Word, WordsError, declare_into, granted_words};
