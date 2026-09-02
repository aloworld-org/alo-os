//! What an agent may do to applications, and the machine it is checked
//! against.
//!
//! `docs/features.md` promises four application verbs at v0.01 — **open,
//! focus, arrange, close**. This crate is the portable half of them: the
//! declarations, the list of what this machine actually has, and the last
//! questions that have to be answered before anything reaches a window. It is
//! `alo-files`' shape one column along in the same table, and deliberately so.
//!
//! Three of the four are here. `arrange` is not, and [`verbs`] says why in the
//! place somebody looking for it will read: its *where* argument is a choice,
//! and a chosen option reaches the approval sentence as the identifier a model
//! picked it by — untranslated English in the one string the capability model
//! is built around. That is queue item 11a and a change to
//! `alo_capability::Takes`, not something this crate can word around.
//!
//! # The two questions this crate exists to ask
//!
//! [`alo_capability`] decides reach: it matches an application identifier
//! exactly and knows nothing else about this machine. That leaves one question
//! it cannot ask — **is there such an application here?** — and [`Reaching`] is
//! where it gets asked, after the grants have answered and never before.
//!
//! The order is the security property, and it is the file half's order for a
//! different reason. There, asking the disk first would tell an agent whether a
//! file it may not touch exists. Here, answering *that is not installed* about
//! an application nobody granted would tell an agent what somebody has on their
//! machine — which is a fingerprint of who they are and what they do for a
//! living. Asked in this order, an ungranted application refuses identically
//! whether it is installed or not.
//!
//! **There is no verb that reads the list**, either, and that is the same
//! decision made once more. What is running and what is in front of a person
//! reach an agent as *context*, offered at the moment of invocation and for
//! that turn only. A `list_applications` would be the background reader
//! `CLAUDE.md` calls a bug in this product.
//!
//! # What is approved, and what is merely shown
//!
//! An application has two names, and only one of them is ever approved. See
//! [`application`]: the identifier goes into the grant and into the sentence,
//! because the name an application gives itself is written by whoever packaged
//! it and two of them can claim the same one.
//!
//! # What a person reads
//!
//! Nothing here composes an English sentence for a screen. Every string this
//! crate can say is declared in [`words`] and answered through `alo-strings` in
//! the language the person in front of the machine reads, and the one refusal
//! that travels into the record — *nothing installed on this machine is that* —
//! is worded where it is made, so what somebody was told is what is written
//! down.
//!
//! # What this crate does not do
//!
//! **It opens, focuses and closes nothing.** That is the acting half: Wayland,
//! D-Bus and the portal backend (ADR 0005), which need a Linux host and are not
//! this crate's. What is here is everything that can be decided and tested on
//! any machine, so that when the acting half is written it is an implementation
//! of a settled model rather than a place where the model gets decided by
//! accident.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod application;
pub mod installed;
pub mod reaching;
pub mod refusing;
pub mod verbs;
pub mod words;

#[cfg(test)]
mod testing;

pub use application::Application;
pub use installed::Installed;
pub use reaching::Reaching;
pub use refusing::{NotAnApplication, NotInstalled};
pub use verbs::{Declaring, application_verbs, declare_into};
pub use words::{EVERY_WORD, Word, WordsError, application_words};
