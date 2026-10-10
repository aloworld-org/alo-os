//! Who draws a frame's header, as alo answers it today — and the one place
//! that answer lives.
//!
//! # Why this is a file of its own and not a line at each caller
//!
//! Two things need it and they must not disagree: `crate::window_edge` decides
//! which controls to **draw**, and `crate::window_edge_reading` decides which
//! controls a reader is **told about**. A reader told about a control nobody
//! drew, or a control drawn that no reader can name, are the same bug read from
//! two ends. One function means they cannot drift apart, and it means the
//! exact answer arrives at both at once.
//!
//! # What it answers, and why that is honest rather than a guess
//!
//! `crate::surfaces`' `XdgDecorationHandler` answers `Mode::ServerSide` to
//! every client that asks, in all three of `new_decoration`, `request_mode`
//! and `unset_mode`. So for every window whose decoration alo has been asked
//! about, **the shell draws it**, and that is not an assumption — it is the
//! only answer this compositor gives.
//!
//! # The part that is not answered here, and whose it is
//!
//! A client that never binds `xdg_decoration` at all is never asked and never
//! told: most GTK applications draw their own headerbar without negotiating.
//! Telling those apart needs the toplevel's protocol state read per frame, and
//! that read belongs to the lane holding §5 and §6 of
//! `docs/design/the-external-window-edge.md` — the Mac, who said on 2026-10-09
//! that they *will read `Decorations` from protocol state only*.
//!
//! **So this function's body is what changes when that lands, and nothing
//! else.** Both callers keep asking the same question of the same file.

use crate::window_edge::Decorations;

/// Who draws this frame's header.
///
/// Today: the shell, for every frame — see this file's header for why that is
/// this compositor's only answer and which case is still to be told apart.
#[must_use]
pub const fn who_draws_a_frame() -> Decorations {
    Decorations::TheShellDraws
}
