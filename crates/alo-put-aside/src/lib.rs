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
//! # Why this crate cannot see a camera
//!
//! `alo-canvas` states the architecture: a plane that moves under a viewport that does
//! not, and **nothing in the viewport layer may read the camera to correct itself. If
//! it has to, it is in the wrong layer.** The panel is a viewport surface.
//!
//! So the rule is held by **the dependency not existing** rather than by a test. That
//! crate's own warning says why: a dock that subtracted a pan to stay still *would pass
//! a test that only checked where the dock ended up*. A test asserting the panel is
//! still at some coordinate after a pan passes for a panel that is in the wrong layer
//! and compensating correctly — so what is held here is that **the type cannot ask**.
//! `Cargo.toml` does not list `alo-canvas`, and nothing in this crate takes a view.
//!
//! # Nothing here draws, and nothing here is a handle
//!
//! A [`alo_dock::window::WindowId`] is a number the compositor already uses. This crate
//! holds it, compares it, and never asks what is inside — so the whole panel can be
//! tested without a display, which is what makes seven of the owner's ten behaviour
//! rules answerable in a crate at all.

#![cfg_attr(not(test), forbid(unsafe_code))]

pub mod panel;
pub mod preview;
pub mod showing;

pub use panel::{NotPutAside, Panel};
pub use preview::Preview;
pub use showing::{Chosen, HowItShows};
