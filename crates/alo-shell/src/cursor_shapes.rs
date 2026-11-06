//! **An application asks for a cursor by name, and alo draws it**:
//! `wp_cursor_shape_v1`.
//!
//! Without this protocol every application drew its own cursor images — a text
//! cursor from its toolkit's theme, a hand from another — so the pointer
//! changed look at every window's edge. With it, an application that asks for
//! *text* or *pointer* or *ew-resize* gets alo's own drawing of that name
//! (`alo_cursors`), the same one alo's own windows show. `docs/features.md`,
//! *The alo Cursor*: *an application asks for a cursor by name … and alo draws
//! its own.*
//!
//! # Smithay checks the request; this file only takes the answer
//!
//! `smithay::wayland::cursor_shape` is the protocol. Before it hands a shape
//! over through `SeatHandler::cursor_image` it has already refused, by doing
//! nothing:
//!
//! - **a client without the pointer's focus**, or with a stale enter serial —
//!   the same `allow_setting_cursor` check a cursor *surface* goes through, so
//!   one application cannot change the pointer over another's window;
//! - **a shape number it does not know**, which is ignored rather than drawn as
//!   something else;
//! - **a device whose seat or pointer is gone**, which becomes inert.
//!
//! Each of those is tested from a real client in `tests/cursor_shapes/`.
//!
//! # A surface is still the application's business
//!
//! An application that sets its own cursor surface still gets its surface: it
//! is the latest request, and `crate::cursor` draws it. A shape request is a
//! request too, so whichever came last is what shows — and so **a shape always
//! wins over a stale surface** the application set earlier.

use crate::surfaces::Surfaces;
use smithay::{delegate_cursor_shape, wayland::tablet_manager::TabletSeatHandler};

/// A graphics tablet's pen may ask for a shape too. alo has no tablet support
/// to draw a pen's cursor with, so the request is received and changes nothing;
/// Smithay requires the handler for the protocol at all.
impl TabletSeatHandler for Surfaces {}

delegate_cursor_shape!(Surfaces);
