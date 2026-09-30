//! The panel at the edge that holds the windows a person put away.
//!
//! A person's windows live on a canvas, where they put them. The Dock at the bottom
//! answers *which applications are open*. This answers a different question — **what
//! did I put aside** — and it holds **individual windows**: three minimised Browser
//! windows are three named previews, not one Browser icon.
//!
//! `docs/design/the-windows-put-aside.md` is the design and
//! `docs/autonomy/putting-a-window-aside.md` is the order it gets built in. This crate
//! is task 1 of that plan: the panel's own state and how it shows.
//!
//! # Why this crate never reaches a camera
//!
//! `alo-canvas` states the architecture: a plane that moves under a viewport that does
//! not, and **nothing in the viewport layer may read the camera to correct itself. If
//! it has to, it is in the wrong layer.** The panel is a viewport surface.
//!
//! The rule is held by `tests/the_panel_never_reaches_the_camera.rs`, which reads this
//! crate's own source. It is a test rather than an absent dependency, and that was a
//! correction rather than a preference: this file claimed for a while that **not listing
//! `alo-canvas` was what held the rule**, and proposed moving `Zoom` out of that crate's
//! `camera` module so the dependency could be taken safely. **Rust's privacy boundary is
//! the crate, not the module** — `pub mod camera` makes `alo_canvas::camera::Camera`
//! nameable from any crate that depends on `alo-canvas`, whatever module `Zoom` sits in.
//! A boundary believed in and not held is worse than none, because it is the one nobody
//! keeps checking.
//!
//! What a test must not do here is check a position. That crate's own warning says why: a
//! dock that subtracted a pan to stay still *would pass a test that only checked where
//! the dock ended up*. So the check is on the source rather than on the arithmetic —
//! **whether anything here can ask**, not whether the answer came out looking right.
//!
//! # Nothing here draws, and nothing here is a handle
//!
//! A [`alo_dock::window::WindowId`] is a number the compositor already uses. This crate
//! holds it, compares it, and never asks what is inside — so the whole panel can be
//! tested without a display, which is what makes seven of the owner's ten behaviour
//! rules answerable in a crate at all.

#![cfg_attr(not(test), forbid(unsafe_code))]

pub mod panel;
pub mod peeking_at_a_preview;
pub mod preview;
pub mod proposing;
pub mod putting_aside;
pub mod restoring;
pub mod restoring_into_a_taken_place;
pub mod showing;
pub mod shown;
pub mod where_it_goes_back;

pub use panel::{NotPutAside, Panel};
pub use preview::Preview;
pub use proposing::Proposal;
pub use restoring::Travel;
pub use restoring_into_a_taken_place::{Placed, Restored};
pub use showing::{Chosen, HowItShows};
pub use shown::Shown;
pub use where_it_goes_back::WhereItGoesBack;
