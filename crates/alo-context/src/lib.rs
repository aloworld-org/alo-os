//! What reaches an agent at the moment it is invoked, and for that turn only.
//!
//! `docs/features.md` promises at v0.01: **context on invocation — focused
//! window, selection, open document, offered and never watched.** ADR 0001 §4
//! is the rule behind it, and `CLAUDE.md` makes it a standing one: *a
//! background reader is a bug in this product, not a feature request.* This
//! crate is the portable half of that promise — what a context is, what it may
//! grant, and how long it lasts.
//!
//! # The three things, and the one that grants
//!
//! A [`Context`] holds up to three parts and the moment they were read:
//! [`Focused`] (the window in front), [`Selection`] (the text somebody had
//! highlighted) and [`Document`] (the file they had open). **Only the document
//! grants anything**, and that is the decision the whole crate turns on.
//!
//! ADR 0001 §3 names two deliberate acts that make a grant: a folder chosen in
//! a picker, and the document offered at invocation. A window somebody happened
//! to be looking at is not one of them, and neither is text they had
//! highlighted. Reading it the other way round would give an agent the run of
//! whatever was in front of a person at the moment they pressed a key — which
//! is a capability model decided by where somebody's mouse was. [`Turn`] is
//! where the difference is enforced, and there is a test there that offers all
//! three at once and asserts the grant list holds exactly one thing.
//!
//! # Only for that turn, carried by the types
//!
//! [`Turn::beginning`] takes a [`Context`] **by value** and a `Context` is not
//! `Clone`, so one invocation is one turn and a second is not a program that
//! compiles. The grant it makes ends twice over: it **expires** at the turn's
//! end, so a daemon that forgets a turn still has an agent that can reach
//! nothing, and [`Turn::ending`] **revokes** it, so a turn that finishes early
//! does not leave a document reachable for the rest of its allotted time.
//!
//! And a grant a context made is a grant like any other — it goes into the
//! machine's own `Grants`, where a person can see it beside the folder they
//! picked on Monday and revoke it in one action. Authority kept in a list of
//! this crate's own would satisfy none of ADR 0001 §3's four words while still
//! deciding what an agent may touch.
//!
//! # Nothing here can be read back
//!
//! **This crate has no serde dependency at all**, and that is a guarantee
//! rather than an omission. A context that could be deserialised would be a
//! context that exists without an invocation having made one, and *offered,
//! never watched* would depend on nobody writing that constructor rather than
//! on there being nowhere to write it. It is `alo_capability::Call`'s rule one
//! step earlier: the deciding side of alo OS does not read things back.
//!
//! For the same reason nothing here reaches `alo-record`. Writing down what was
//! on somebody's screen at every invocation would build, entry by entry, the
//! watched-context log this ADR exists to forbid — so what the record keeps is
//! what the agent then **did**, against the grant it did it under, which is the
//! turn's grant like any other. `tests/from_an_invocation_to_a_change.rs` walks
//! that whole journey and asserts the record kept the sentence and the grant
//! and never the person's own text.
//!
//! # What a person can see
//!
//! [`Context::shown`] hands a shell one row per part, and one row saying
//! nothing was offered when there was nothing. A rule nobody can check is a
//! promise, and a person who cannot see what they are offering has no way to
//! tell a system that reads three things at invocation from one that watches
//! everything all day.
//!
//! # What this crate does not do
//!
//! **It reads nothing.** There is no compositor here and no accessibility tree:
//! what is in front of somebody, what they have selected and what they have
//! open are Wayland's and AT-SPI's to answer, on a Linux host, and the daemon
//! that asks them is the other half of this promise. What is here is everything
//! that can be decided and tested on any machine, so that when the reading half
//! is written it is an implementation of a settled model rather than a place
//! where the model gets decided by accident.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod context;
pub mod document;
pub mod focused;
pub mod refusing;
pub mod selection;
pub mod turn;
pub mod words;

#[cfg(test)]
mod testing;

pub use context::Context;
pub use document::Document;
pub use focused::Focused;
pub use refusing::NotOffered;
pub use selection::{MOST, Selection};
pub use turn::Turn;
pub use words::{Counted, EVERY_WORD, Word, WordsError, context_words, declare_into};
