//! **Which window a band hangs over**: an application's, or one of alo's own.
//!
//! The keyboard on a band and the window panel it opens were kept per
//! application window, keyed by its `WlSurface`, because only applications'
//! bands had Window actions. Since the owner's *"Yes add that"* of 2026-11-14
//! the terminal's and the task windows' bands have it too, and none of alo's
//! windows has a surface. So the one table, and every road that reads it, is
//! keyed by this: **one panel, one keyboard state, whichever window it is**.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::the_bands_over_alos_windows::AlosBand;

/// The window a band hangs over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ABand {
    /// An application's window.
    App(WlSurface),
    /// One of alo's own windows.
    Alos(AlosBand),
}

impl ABand {
    /// The application's window, if this is one.
    #[must_use]
    pub(crate) const fn the_application(&self) -> Option<&WlSurface> {
        match self {
            Self::App(surface) => Some(surface),
            Self::Alos(_) => None,
        }
    }

    /// alo's window, if this is one.
    #[must_use]
    pub(crate) const fn alos(&self) -> Option<AlosBand> {
        match self {
            Self::App(_) => None,
            Self::Alos(which) => Some(*which),
        }
    }
}
