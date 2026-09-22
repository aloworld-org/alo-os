//! The machine the installer is walked in, and the two ways it is read.
//!
//! One place for everything that is not the walk itself: what the host has to
//! have ([`needs`]), the download a person would have ([`download`]), the discs
//! the guest is handed ([`medium`], [`guest`]), how the machine is started and
//! stopped ([`machine`]) — and the two ways anything is known about it: what
//! the guest printed on its serial line while it ran ([`console`]), and every
//! file of its disk read from the host with the machine off ([`reading`]). The
//! walk is `the_installer_walked_on_a_real_windows.rs`, and it says what is
//! done with all of this.
//!
//! # It is QEMU, and not Hyper-V
//!
//! The installer plan's task 10 names a Hyper-V generation-2 machine. The
//! account the tests run under on this development PC cannot manage Hyper-V
//! (the plan's task 2 report), so the machine is QEMU's `q35` with OVMF, which
//! is the same shape: UEFI, GPT, a TPM 2.0 and two disks on one AHCI
//! controller. What that costs is named where it is paid — `naming.rs`'s
//! Hyper-V SCSI and NVMe rows cannot be measured in it, and the walk says so.
//!
//! # Secure Boot is off in this machine, and only here
//!
//! ADR 0033 §4: the shipped installer **refuses to run with Secure Boot on**,
//! and this test runs the installer. So the firmware is the Secure Boot capable
//! build with nothing enrolled — Secure Boot off, and still answerable, which
//! matters because `deciding.rs` refuses *could not be found out* as firmly as
//! it refuses *on*. Task 9's machine is the one that enforces Secure Boot, on
//! the other half of the road.

pub mod console;
pub mod download;
pub mod firmware;
pub mod guest;
pub mod machine;
pub mod medium;
pub mod needs;
pub mod reading;
