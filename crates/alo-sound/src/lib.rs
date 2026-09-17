//! **The outputs and inputs this machine has, and which one a person meant.**
//!
//! `ROADMAP.md` v0.5's *Sound: outputs and inputs, per-application volume, and
//! switching mid-call*. Task 2 of
//! `docs/autonomy/v0-5-devices-and-media-plan.md`.
//!
//! # What is rented, and what is ours
//!
//! The audio server is rented: PipeWire routes, WirePlumber decides policy, and
//! neither is patched (ADR 0011). **What is ours is which device a person
//! meant, remembered** — and that is the whole of this crate.
//!
//! A person who plugs a headset in during a call means *that one, now*. A person
//! who pinned their desk speakers means *those whenever they are here, whatever
//! else appears*. Neither is a fact the server holds: it knows which devices
//! exist and which are in use, and it has no view about what somebody wanted.
//!
//! # A device keeps its name across a replug
//!
//! [`Identity`] is what survives being unplugged and plugged in again. A device
//! that came back as a new thing would lose its volume, its mute and its pin
//! every time somebody moved a cable — and a pin that forgets is worse than no
//! pin, because a person stops checking.
//!
//! # A mute is a mute
//!
//! [`Mute`] is the source stopped, not the volume taken to zero. A microphone at
//! gain zero is a microphone that is still listening and still says so on the
//! indicator, and the difference matters exactly when somebody is relying on it:
//! `tests/a_mute_is_silence_not_a_low_volume.rs` reads what the stream carries.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod applying;
pub mod asking;
pub mod device;
pub mod heard;
pub mod keeping;
pub mod meant;
pub mod mute;
pub mod pinned;
pub mod refusing;
pub mod server;
pub mod testing;
pub mod words;

pub use applying::{Brought, bring_into_line};
pub use asking::Sound;
pub use device::{Devices, Identity, Kind, OneDevice, Volume};
pub use heard::Heard;
pub use meant::{WhatWasMeant, Why, what_was_meant};
pub use mute::Mute;
pub use pinned::Pinned;
pub use refusing::{NotDone, NotHeard};
pub use server::TheAudioServer;
pub use words::{declare_into, sound_words};
