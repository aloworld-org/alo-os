//! Who draws a window's header, and the one place that answer lives.
//!
//! # Why this is a file of its own and not a line at each caller
//!
//! Two things need it and they must not disagree. `crate::window_edge` decides
//! which controls to **draw**, and `crate::window_edge_reading` decides which
//! controls a reader is **told about**. A reader told about a control nobody
//! drew, and a control drawn that no reader can name, are the same bug seen from
//! two ends. With one function, they cannot drift apart.
//!
//! # What it reads, and what it never reads
//!
//! **The application's protocol state**, which is what the application told
//! the compositor:
//!
//! - `crate::surfaces` answers `Mode::ServerSide` to every client that binds
//!   `xdg_decoration`. A toplevel whose current, acknowledged state carries
//!   that mode **agreed that the shell draws its frame**. Qt and KDE
//!   applications do this.
//! - A toplevel with no decoration mode **never asked**, and draws its own
//!   header. GTK 3, GTK 4 and libadwaita never bind the protocol (measured, see
//!   `docs/quirks/an-application-that-draws-its-own-header-is-not-asked-which-buttons.md`).
//!
//! **And, for a window that draws its own header, which buttons it shows**,
//! from its `app_id` (which it declared itself) in a reviewed table of what
//! each application was measured to draw
//! (`crate::the_buttons_an_application_draws`).
//!
//! **Never its pixels and never its accessibility tree.** Reading either in the
//! background to find buttons would be watching an application, which
//! `CLAUDE.md` forbids: *context is offered, never watched*.

use smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::compositor::with_states;
use smithay::wayland::shell::xdg::XdgToplevelSurfaceData;

use crate::window_edge::Decorations;

/// Who draws a header, from what the application said.
///
/// `agreed_the_shell_draws` is whether its acknowledged state carries the
/// server-side mode. `application` is its `app_id`, where it gave one.
///
/// **The reviewed table is asked first.** An application in it was measured
/// drawing its own header, and an agreement it made on the protocol does not
/// change what is on the screen. With no entry, the protocol decides. A window
/// that agreed gets the shell's whole band. One that never asked is told
/// apart by its `app_id`, or is [`crate::ItsOwnButtons::NOT_KNOWN`].
#[must_use]
pub fn who_draws_the_header(agreed_the_shell_draws: bool, application: Option<&str>) -> Decorations {
    match application.and_then(crate::the_buttons_an_application_draws::measured) {
        Some(its_own) => Decorations::TheApplicationDraws(its_own),
        None if agreed_the_shell_draws => Decorations::TheShellDraws,
        None => Decorations::TheApplicationDraws(crate::ItsOwnButtons::NOT_KNOWN),
    }
}

/// Whether this toplevel's acknowledged state says the shell draws its frame.
///
/// Reads committed state and sends nothing. Safe to ask every frame.
#[must_use]
fn agreed_the_shell_draws(surface: &WlSurface) -> bool {
    with_states(surface, |states| {
        states
            .data_map
            .get::<XdgToplevelSurfaceData>()
            .and_then(|data| data.lock().ok())
            .is_some_and(|state| state.current.decoration_mode == Some(Mode::ServerSide))
    })
}

impl crate::Server {
    /// Who draws this window's header: [`who_draws_the_header`], with what
    /// this window told the compositor.
    #[must_use]
    pub fn who_draws_the_header_of(&self, surface: &WlSurface) -> Decorations {
        let application = self.the_application_of(surface);
        who_draws_the_header(
            agreed_the_shell_draws(surface),
            application.as_ref().map(alo_dock::AppId::name),
        )
    }
}

#[cfg(test)]
#[path = "window_edge_who_draws_tests.rs"]
mod tests;
