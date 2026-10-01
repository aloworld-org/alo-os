//! A window filling the screen, and the furniture giving way to it.
//!
//! **This makes an existing promise true rather than adding one.**
//! `docs/features.md` carries *full screen* at `[v0.01]` since the owner put it
//! there on 2026-09-30, and before that this compositor could not put a window
//! into full screen at all: `Mode` had three cases and none filled the screen,
//! `XdgShellHandler` had no `fullscreen_request`, and a client that asked was
//! answered with **silence rather than a refusal**. Nothing promised to a
//! person is new below.
//!
//! # What distinguishes it from maximised here, which is not the area
//!
//! Both are the whole submitted output — that is what this compositor hands its
//! window modes, so there is no smaller *work area* for maximise to stop at.
//! Two things differ.
//!
//! **The client is told `Fullscreen`**, which is what makes an application hide
//! its own chrome. Told `Maximized` it keeps it, and a person gets a title bar
//! inside a window that is supposed to be nothing but content.
//!
//! **And the shell stops drawing its own furniture over it.** That is what
//! `docs/design/the-alo-dock.md` means by *true full screen covers the Dock*,
//! and without it this would be a flag a client reads while a person sees no
//! difference — which is the shape of every *done* thing this repository spent
//! 2026-09-30 correcting.
//!
//! # Asked and told apart
//!
//! A client asking is [`crate::Server`]'s XDG path; a person asking is
//! [`Server::set_window_full_screen`]. They reach the same mode, as maximise's
//! two roads do — but a client's request is *intent* and may be declined by
//! policy, while a person's is a decision. Keeping them separate is what lets
//! the first be refused without the second being.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::Serial;
use smithay::wayland::shell::xdg::ToplevelSurface;

use crate::window_mode::Mode;
use crate::{Server, WindowModeError, surfaces::Surfaces};

/// A refused full-screen request; refusal changes no protocol state.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WindowFullScreenError {
    /// Only this display's live mapped toplevel roots are eligible.
    #[error("full screen target is not a mapped toplevel in this display")]
    Unmapped,
    /// No successfully submitted output with supported dimensions is available.
    #[error("full screen requires a submitted output within supported dimensions")]
    OutputUnavailable,
    /// Pointer movement, resize or a popup grab currently owns an operation.
    #[error("another interactive window operation is outstanding")]
    Busy,
    /// Normal geometry or current restore limits cannot be represented safely.
    #[error(transparent)]
    Geometry(#[from] crate::ResizeGeometryError),
}

/// Keep the exhaustive refusal contract, as maximise does.
pub(crate) fn full_screen_refusal(error: WindowModeError) -> WindowFullScreenError {
    match error {
        WindowModeError::Unmapped => WindowFullScreenError::Unmapped,
        WindowModeError::OutputUnavailable => WindowFullScreenError::OutputUnavailable,
        WindowModeError::Busy => WindowFullScreenError::Busy,
        WindowModeError::Geometry(error) => WindowFullScreenError::Geometry(error),
    }
}

impl Server {
    /// Put this window into full screen, or take it out, from a trusted
    /// control.
    ///
    /// Leaving full screen returns the window to its normal mode and its saved
    /// geometry, the same road maximise takes out — **not** to maximised,
    /// however the window arrived. A window a person put into full screen from
    /// a normal size and got back maximised has been changed by a journey it
    /// did not ask for.
    ///
    /// *The mode itself is this crate's own and is not named here: rustdoc
    /// refuses a public item linking a private one, and it is right to — a
    /// reader of the public API cannot follow that link.*
    ///
    /// Returns `None` for an unchanged request.
    ///
    /// # Errors
    /// Every refusal [`WindowFullScreenError`] names.
    pub fn set_window_full_screen(
        &mut self,
        surface: &WlSurface,
        filling: bool,
    ) -> Result<Option<Serial>, WindowFullScreenError> {
        self.surfaces.set_window_full_screen(surface, filling)
    }

    /// Whether any mapped window on this display is filling the screen.
    ///
    /// **This is what the desktop asks before drawing its furniture.** The
    /// Dock and the panel give way to a window filling the screen and are
    /// reached by their edges instead, which is the whole of what separates
    /// this mode from maximised as far as a person can see.
    #[must_use]
    pub fn a_window_is_filling_the_screen(&self) -> bool {
        self.surfaces.a_window_is_filling_the_screen()
    }
}

impl Surfaces {
    /// Answer client intent without bypassing mapping, output or operation
    /// policy — the shape `client_window_maximize` takes, and for its reasons.
    pub(crate) fn client_window_full_screen(&mut self, role: ToplevelSurface, value: bool) {
        // Before the first empty commit there is no initial configure boundary.
        if !role.is_initial_configure_sent() {
            return;
        }
        if !matches!(
            self.set_window_full_screen(role.wl_surface(), value),
            Ok(Some(_))
        ) {
            // XDG requires a configure response even when policy declines or
            // the mode is unchanged.
            role.send_configure();
        }
    }

    /// Translate full-screen intent into the shared layout transaction.
    pub(crate) fn set_window_full_screen(
        &mut self,
        surface: &WlSurface,
        filling: bool,
    ) -> Result<Option<Serial>, WindowFullScreenError> {
        self.set_window_mode(
            surface,
            if filling {
                Mode::FillingTheScreen
            } else {
                Mode::Normal
            },
        )
        .map_err(full_screen_refusal)
    }

    /// Whether any window this display holds is filling the screen.
    ///
    /// **Asked of the requested mode rather than of a committed state**, which
    /// is the same reading `requested_window_mode` gives and for its reason: a
    /// toggle must supersede an uncommitted response, or a person pressing
    /// twice quickly gets the first answer twice.
    pub(crate) fn a_window_is_filling_the_screen(&self) -> bool {
        self.window_modes
            .iter()
            .any(|window| window.mode == Mode::FillingTheScreen)
    }
}
