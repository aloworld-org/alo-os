//! **The window in front, if alo may reach it for this application** — the one
//! question typing (ADR 0153) and taking a picture (ADR 0147 §1) ask before
//! they touch anything, answered in one place so the two cannot come to
//! disagree about whose window alo may act on.
//!
//! In order, and nothing else:
//!
//! 1. the window that holds the keyboard **now** — the keyboard is never moved
//!    to answer this;
//! 2. it must be a window of the named application
//!    (`crate::typing_into_the_window_in_front::a_window_of`);
//! 3. whoever answers for alo's copies and the person's setting must let its
//!    **process**, by the kernel's account, be reached
//!    (`crate::typing_into_the_window_in_front::MayTypeInto`).

use alo_dock::AppId;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{Client, Resource as _};

use crate::Server;
use crate::typing_into_the_window_in_front::{MayTypeInto, a_window_of};

/// Why the window in front is not one alo may reach for this application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NotReached {
    /// No window has the keyboard, or the one that has it is not that
    /// application's.
    NotInFront,
    /// It is that application's, and its process is not one alo may reach at
    /// the person's setting — or the kernel could not say which process it is.
    NotAlosCopy,
}

impl Server {
    /// The window that holds the keyboard now, and its client, if it is a
    /// window of `identifier` whose process `may` lets alo reach.
    pub(crate) fn the_window_in_front_alo_may_reach(
        &mut self,
        identifier: &str,
        may: &mut dyn MayTypeInto,
    ) -> Result<(WlSurface, Client), NotReached> {
        let root = self
            .surfaces
            .keyboard_root()
            .ok_or(NotReached::NotInFront)?;
        let app = AppId::named(identifier).map_err(|_| NotReached::NotInFront)?;
        let id = crate::window_number::the_window_id_of(&root);
        let theirs = self
            .the_windows_the_dock_sees()
            .each_as_opened()
            .any(|window| window.id() == id && a_window_of(window.app(), &app));
        if !theirs {
            return Err(NotReached::NotInFront);
        }
        let client = root.client().ok_or(NotReached::NotInFront)?;
        // **The kernel's account of the process**, recorded when it connected
        // — the same road the door asks of its own callers.
        let credentials = client
            .get_credentials(&self.display_handle())
            .map_err(|_| NotReached::NotAlosCopy)?;
        let pid = u32::try_from(credentials.pid).map_err(|_| NotReached::NotAlosCopy)?;
        if !may.may_type_into(identifier, pid) {
            return Err(NotReached::NotAlosCopy);
        }
        Ok((root, client))
    }
}
