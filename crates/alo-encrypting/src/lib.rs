//! What full-disk encryption is on this machine, as
//! [ADR 0054](../../../docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md)
//! decided it.
//!
//! **The disk is sealed to this machine's own security chip and opened with a
//! PIN; on a machine whose chip cannot be used it is opened with a passphrase;
//! and either way the person writes down a recovery key and types it back
//! before the install finishes.** That sentence is the whole of this crate, held
//! as types rather than as prose, so that the part of it nobody can be trusted
//! to remember — *and types it back* — is the part a compiler refuses to skip.
//!
//! | | |
//! |---|---|
//! | [`TheChip`] | What this machine has, and whether a key may be sealed to it |
//! | [`WhatToAskFor`], [`HowItUnlocks`] | Which of the two roads this machine takes, and the secret it takes |
//! | [`Pin`], [`Passphrase`] | The one secret the person is asked for, and why the two have different lengths |
//! | [`RecoveryKey`] | The key the rented tool made, for the length of one screen |
//! | [`WrittenDown`] | That the person typed it back — the only proof there is, and it cannot be made any other way |
//! | [`THE_ROAD`], [`Step`] | The order enrolment happens in, which is what makes the proof unskippable |
//! | [`Enrolment`] | A machine whose disk is encrypted, and which cannot be described without the proof |
//! | [`TheDisk`], [`TheVolume`] | Which volume it is enrolled on, named the way it stays named |
//! | [`ASecretOnItsWay`] | Where a secret is put while a rented tool is given it, which is memory |
//! | [`TheSequence`], [`Run`], [`TheTool`] | The road as the exact runs it is, built and never taken here |
//! | [`TheDiskRefused`] | What the disk refused, read off what the rented tool answered |
//!
//! # The road is decided, and so is how it is shown to work
//!
//! ADR 0054 is **accepted**: the road above is settled. How an enrolment is
//! *shown* to work is
//! [ADR 0056](../../../docs/decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md),
//! **accepted, option C**, and it draws the line where the chip is. Everything
//! about the sequence that is about LUKS — six steps in order, the recovery key
//! made and kept before anything the person unlocks with exists, the installer's
//! first key opening nothing afterwards, the person's secret and the recovery
//! key each opening the volume, a near miss opening nothing — runs against a
//! real LUKS2 volume in the pinned base in
//! `tests/the_sequence_against_a_virtual_disk.rs`. The three facts that are
//! about a chip — that it releases the key when the PIN is typed, that an
//! update's new measurements do not stop it, and that a change to what register
//! 7 measures does — are shown on a certified machine and nowhere else, which is
//! task 9 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`.
//!
//! **Neither half ticks `docs/features.md`'s v0.5 encryption line on its own.**
//! *Enrolled at install* that has never been installed onto a machine with a
//! chip is not done, and a green suite here does not say otherwise.
//!
//! # This crate builds the runs and takes none of them
//!
//! [`TheSequence`] is a description of what is to be run: which rented tool,
//! which arguments, and which of the four named secrets has to be in its file
//! first. Nothing here starts a program, opens a file or reaches a network — it
//! depends on nothing, and
//! `tests/the_key_is_never_kept_on_the_disk_it_recovers.rs` reads the source and
//! the manifest to hold that. What runs the sequence is the installer, and what
//! runs it against a virtual disk is this crate's own integration test.
//!
//! # Where the recovery key is, and for how long
//!
//! It exists as a value in one process, between [`RecoveryKey::as_printed`] —
//! which reads the 72 bytes the rented tool wrote — and
//! [`RecoveryKey::written_back`], which **consumes** it. After the person has
//! confirmed they kept it, no code in any later line can be holding one, because
//! there is no such value any more. It is never written to the disk it recovers,
//! and this crate could not write it anywhere if it wanted to: it depends on
//! nothing, names no file and runs no program, and
//! `tests/the_key_is_never_kept_on_the_disk_it_recovers.rs` reads the source and
//! the manifest to hold both.
//!
//! On a managed machine the organisation holds a copy
//! ([ADR 0004](../../../docs/decisions/0004-the-organisations-machine.md)). That
//! is v1, and it is not built here; what it needs from this shape is that the
//! key is a value at exactly one known moment, which is what the paragraph above
//! is.
//!
//! # This crate says nothing to anybody
//!
//! It declares no words, so nothing in it reaches a person's screen — and it
//! cannot, because a vocabulary is a dependency and this crate has none. The
//! sentences a person meets on this road are `alo-enrolling`'s: one for every
//! refusal here, each with a translator's note, declared into the machine's one
//! vocabulary. The English on the refusals in this crate is for a service log
//! and a record's reason, the way `alo-drives`' is.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod chip;
mod enrolment;
mod handing_over;
mod passphrase;
mod pin;
mod recovery_key;
mod refusing;
mod road;
mod sequence;
mod unlocking;
mod volume;
mod written_down;

pub use chip::TheChip;
pub use enrolment::Enrolment;
pub use handing_over::{ASecretOnItsWay, ONLY_ITS_OWNER_MAY_READ_IT, THE_ONE_PLACE};
pub use passphrase::{A_PASSPHRASE_IS_AT_LEAST, NotAPassphrase, Passphrase};
pub use pin::{A_PIN_IS_AT_LEAST, NotAPin, Pin};
pub use recovery_key::{GROUPS, IN_A_GROUP, NotARecoveryKey, RecoveryKey, THE_ALPHABET};
pub use refusing::TheDiskRefused;
pub use road::{Step, THE_ROAD};
pub use sequence::{OnTheRoad, Run, TheSequence, TheTool};
pub use unlocking::{HowItUnlocks, NoChipToSealTo, THE_PCRS_IT_IS_SEALED_AGAINST, WhatToAskFor};
pub use volume::{
    BY_ID, IT_OPENS_AS, NotADisk, THE_LAST_PARTITION, TheDisk, TheVolume, WHERE_IT_OPENS,
};
pub use written_down::{Again, NotWhatWasShown, WrittenDown};
