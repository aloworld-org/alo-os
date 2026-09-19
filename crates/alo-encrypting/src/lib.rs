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
//!
//! # The road is decided; how it is shown to work is not, and this crate waits
//!
//! ADR 0054 is **accepted**, so the road above is settled. What is not settled
//! is how an enrolment is shown to work: its acceptance asks for the sequence to
//! run with a software TPM in a virtual machine, and no machine in this fleet
//! can present a TPM device to anything — the Linux the gates run in is built
//! with `CONFIG_TCG_VTPM_PROXY` unset, and the host offers no nested
//! virtualisation, so every virtual machine on it is emulated. What a virtual
//! disk shows about encryption and what only a machine with a chip can is
//! therefore
//! [ADR 0056](../../../docs/decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md),
//! proposed, with the measurements in it.
//!
//! Until that is accepted nothing here enrols anything on any disk, virtual or
//! real: there is no command sequence in this crate, no program is named, and no
//! file is opened. `tests/the_enrolment_waits_on_its_decision.rs` holds that,
//! and **fails the day ADR 0056 stops saying *proposed***, which is the
//! instruction to turn [`THE_ROAD`] into the tested sequence against a virtual
//! disk.
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
//! It declares no words, so nothing in it reaches a person's screen. The
//! sentences a person meets while their disk is being encrypted are task 6's to
//! write and task 7's to put in the vocabulary with a translator's note. The
//! English on the refusals here is for a service log and a record's reason, the
//! way `alo-drives`' is.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod chip;
mod enrolment;
mod passphrase;
mod pin;
mod recovery_key;
mod road;
mod unlocking;
mod written_down;

pub use chip::TheChip;
pub use enrolment::Enrolment;
pub use passphrase::{A_PASSPHRASE_IS_AT_LEAST, NotAPassphrase, Passphrase};
pub use pin::{A_PIN_IS_AT_LEAST, NotAPin, Pin};
pub use recovery_key::{GROUPS, IN_A_GROUP, NotARecoveryKey, RecoveryKey, THE_ALPHABET};
pub use road::{Step, THE_ROAD};
pub use unlocking::{HowItUnlocks, NoChipToSealTo, THE_PCRS_IT_IS_SEALED_AGAINST, WhatToAskFor};
pub use written_down::{Again, NotWhatWasShown, WrittenDown};
