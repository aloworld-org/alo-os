//! **The cameras this machine has, and the switch that turns the camera and the
//! microphone off for everyone.**
//!
//! `ROADMAP.md` v0.5's *Devices*, and task 4 of
//! `docs/autonomy/v0-5-devices-and-media-plan.md`.
//!
//! | | |
//! |---|---|
//! | [`CameraId`], [`OneCamera`], [`TheCameras`], [`OfItsOwn`] | what this machine can see with, and what each camera has of its own |
//! | [`Seen`], [`Cameras`], [`TheMediaServer`] | asking the machine, through the rented server's own record |
//! | [`Which`], [`Switch`], [`TheSwitches`], [`TurnedOff`] | the switch, and the refusal that names which one |
//! | [`letting_go`], [`turning_off`] | off held below the door: the machine lets go of the hardware |
//! | [`keeping`] | the two switches, kept (ADR 0038) |
//!
//! # A camera is never a device number
//!
//! [`CameraId`] refuses one. The number the kernel hands a camera is the order
//! the machine happened to find it in, and a grant made against it is a grant
//! that quietly moves to whatever is plugged in tomorrow — which is what
//! [ADR 0040](../../../docs/decisions/0040-what-an-applications-grant-is-over.md)
//! is about. The number is used in exactly one place, as a road to the hardware
//! when the machine is letting go of it, and it is never remembered.
//!
//! # The switch is not a grant
//!
//! A grant is about one application. The switch is about the machine, and it
//! wins: a person who turned the camera off has turned it off for the
//! application they granted it to last week and for the one they install
//! tomorrow. `tests/an_application_with_a_grant_gets_nothing.rs` holds that
//! against a real grant that really does permit the camera.
//!
//! # And what is not here
//!
//! No image processing of any kind — no blur, no *beautify*, no framing. The
//! plan says so and it is worth repeating where somebody might add one: a
//! machine that touches the picture is a machine that has looked at it.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod keeping;
pub mod letting_go;
pub mod reported;
pub mod seen;
pub mod switch;
pub mod the_media_server;
pub mod turning_off;
pub mod words;

pub use reported::{CameraId, NotACamera, OfItsOwn, OneCamera, TheCameras};
pub use seen::{NotSeen, Seen};
pub use switch::{Switch, TheSwitches, TurnedOff, Which};
pub use the_media_server::{Cameras, TheMediaServer};
pub use turning_off::{WhatWasDone, every_camera_given_back, every_camera_let_go};
pub use words::{camera_words, declare_into};
