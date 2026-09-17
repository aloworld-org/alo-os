//! **The battery, power profiles, and what a person is told about both.**
//!
//! `ROADMAP.md` v0.5's *Devices*, and task 5 of
//! `docs/autonomy/v0-5-devices-and-media-plan.md`.
//!
//! | | |
//! |---|---|
//! | [`Charge`], [`Charging`], [`Reading`] | what the kernel says about the battery at one moment |
//! | [`TheBattery`], [`ChargeLimit`] | reading it, and whether this machine stops charging early |
//! | [`HowLong`], [`how_long_is_left`] | how long is left — and the four rules for when this machine will not say |
//! | [`Telling`] | once at low, once at nearly gone, and never again |
//! | [`Profile`], [`TheProfiles`], [`Profiles`] | the rented daemon's closed set, and which this machine really has |
//! | [`WhatIsDrainingIt`], [`TheModel`] | *why is my battery gone*, answered — and the model named when it is the model |
//! | [`keeping`] | the profile a person chose (ADR 0038) |
//!
//! # Nothing here suspends anything
//!
//! Sleeping is `alo-sleeping`'s, and suspend is not a power profile. Nothing in
//! this crate turns a machine off, puts it to sleep, or decides that it should.
//!
//! # And no power-management daemon of our own
//!
//! The profiles are the rented daemon's three names, read from it rather than
//! declared here (ADR 0011). The battery is read straight from the kernel, which
//! is deliberate: a reading that went through a service is a reading that stops
//! when the service does, and *why is my battery gone* is a question this
//! machine should answer while things are going wrong.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod battery;
pub mod draining;
pub mod keeping;
pub mod profiles;
pub mod reading;
pub mod steady;
pub mod telling;
#[cfg(target_os = "linux")]
pub mod the_daemon;
pub mod words;

pub use battery::{ChargeLimit, NotRead, TheBattery};
pub use draining::{TheModel, WhatIsDrainingIt, what_is_draining_it};
pub use profiles::{NotAProfile, Profile, TheProfiles};
pub use reading::{Charge, Charging, NotAReading, Reading};
pub use steady::{HowLong, how_long_is_left};
pub use telling::Telling;
#[cfg(target_os = "linux")]
pub use the_daemon::{NotAsked, Profiles, ThePowerDaemon};
pub use words::{declare_into, power_words};
