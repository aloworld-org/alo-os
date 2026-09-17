//! **Pairing a device, what a paired device is for, and forgetting it.**
//!
//! `ROADMAP.md` v0.5's *Devices*, and task 3 of
//! `docs/autonomy/v0-5-devices-and-media-plan.md`. `docs/features.md` says why
//! it matters more than it sounds: *an operating system with a brilliant agent
//! and no working Bluetooth is not a product.*
//!
//! BlueZ is rented and unmodified (ADR 0011). What is here is the part that is
//! alo OS's: **nothing pairs unless a person chose it and said yes to what it
//! asked**, a paired device's kind decides what it is for, forgetting is one act
//! that takes the keys with it, and the radio switch means what it says.
//!
//! | | |
//! |---|---|
//! | [`Address`], [`DeviceName`], [`Kind`], [`Found`], [`TheDevices`] | what the service reported, and nothing it did not |
//! | [`Chosen`] | a device a person picked out of what was found — the only road to a pairing |
//! | [`Asked`], [`Answer`] | what the device requires, shown in full, and what the person said |
//! | [`Asking`], [`ThePersonsOwnSurface`] | where a person is asked, and the four questions a device can put |
//! | `pairing_agent` | the object the service asks, served in the person's session (Linux only) |
//! | [`Radio`] | on, or off for every device |
//! | [`Becomes`] | what a paired device is for — and where it is held, which for sound is `alo-sound` |
//! | [`WhatAPairingGrants`] | nothing, and the file saying why that is worth a type |
//! | [`BluetoothService`] | asking the service, and the four things a person can do |
//! | `bluez` | the client that speaks the service's own interface on the system bus (Linux only) |
//!
//! # A headset is not a machine
//!
//! alo OS uses the word *pairing* for two things and only one of them is
//! authority. `alo-nearby`'s pairing is two machines, each person agreeing on
//! their own; this crate's is a headset. Pairing a headset grants nothing, and
//! [`granting`] is a whole file about that because the day the two appear on one
//! list is the day the word stops meaning anything.
//!
//! # Nothing is paired with automatically
//!
//! Including devices that ask. A device that asks to pair is a device in a room
//! this machine cannot see, and *convenient* is not a reason to answer it.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod asking_the_person;
pub mod becoming;
#[cfg(target_os = "linux")]
pub mod bluez;
#[cfg(target_os = "linux")]
pub mod bus;
pub mod choosing;
pub mod confirming;
pub mod granting;
#[cfg(target_os = "linux")]
pub mod pairing_agent;
pub mod radio;
pub mod reported;
pub mod service;
pub mod testing;
pub mod words;

pub use asking_the_person::{Asking, Refused, ThePersonsOwnSurface};
pub use becoming::Becomes;
pub use choosing::{Chosen, NotChosen};
pub use confirming::{Answer, Asked};
pub use granting::WhatAPairingGrants;
pub use radio::Radio;
pub use reported::{Address, DeviceName, Found, Kind, NotADevice, TheDevices};
pub use service::{BluetoothService, Devices, NotAnswering, NotDone, look_for_devices, pair_with};
pub use words::{bluetooth_words, declare_into};
