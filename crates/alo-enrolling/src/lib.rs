//! What a person is told while their disk is being encrypted.
//!
//! `alo-encrypting` is what full-disk encryption **is** on this machine: the
//! road [ADR 0054](../../../docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md)
//! decided, the secrets it asks for, the recovery key, and the exact runs of the
//! two rented tools that carry it out. This crate is the other half of the same
//! task: **one sentence for every refusal on that road**, and the few the road
//! itself has to ask for, each with the note a translator works from.
//!
//! | | |
//! |---|---|
//! | [`words`] | Every sentence, its English, and its translator's note |
//! | [`said`] | Which sentence each refusal is, as an exhaustive match |
//!
//! # Why this is a crate of its own
//!
//! Because `alo-encrypting` depends on nothing, and the absence is the argument:
//! a crate that holds a recovery key for the length of one screen must not be
//! able to serialise it, log it or hand it to anything, and a vocabulary is a
//! dependency that brings a serialiser with it. So the refusals live there, with
//! the English `Display` a service log reads, and the sentences a **person**
//! reads live here. The dependency is one way round and stays that way: nothing
//! in this crate is reachable from a value holding a secret.
//!
//! # What is not here
//!
//! The screens. What a person reads is this crate's; where they read it, in what
//! order and beside which button is the installer plan's, and this crate exists
//! so that the installer plan is handed sentences rather than left to write
//! English of its own. The walk from a new printer to a recovered disk, and the
//! table of exactly what a person meets in order, is task 7 of
//! `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod said;
pub mod words;

pub use said::{
    about_a_passphrase, about_a_pin, about_a_recovery_key, about_the_chip, about_the_disk,
    about_what_the_disk_refused, about_what_was_typed_back, in_the_persons_language,
};
pub use words::{EVERY_WORD, Word, WordsError, declare_into, enrolling_words};
