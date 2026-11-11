//! **alo OS's pixels and text, drawn in memory with no compositor.**
//!
//! The shell's drawing, moved out of it so that alo's own applications draw
//! exactly as the desktop does (`docs/autonomy/the-alo-ui-kit-plan.md`, task
//! 2, and ADR 0002's *the UI runtime built for the shell*). Nothing here knows
//! what a window, a surface or a compositor is: it turns measures into pixels,
//! and a caller puts those pixels where it likes.
//!
//! **No Wayland, no Smithay, no D-Bus**, held by
//! `tests/it_never_reaches_a_compositor.rs` for every crate this one pulls in.
//!
//! **Moved in steps, each one leaving the desktop's pictures unchanged**:
//! `alo-shell`'s `the_desktops_pictures_pinned.rs` holds every surface it
//! draws to the hashes taken before anything moved. So far this holds the
//! display's scale, where things go, and the shapes and rasters; text, the
//! bundled fonts and the canvas follow.

pub mod area;
pub mod display_scale;
pub mod painted;
pub mod painted_flat;
