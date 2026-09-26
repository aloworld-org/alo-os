//! Which desktop each window is on, and which desktop a person is looking at.
//!
//! **`alo-desktops` decides both; this writes them down.** A window joins the
//! desktop that was current when it opened, which is that crate's `put_on`; a
//! window that closes leaves every desktop, which is its `close`; and which
//! desktop a switch reaches is its `switch`. Nothing here chooses a desktop for
//! a window, chooses which desktop a switch goes to, or decides that a window
//! belongs on more than one — the three surfaces that are on every desktop are
//! there because `alo_desktops::OnADisplay::of` put them there when the display
//! arrived, not because this file adds them each time.
//!
//! # A window on another desktop is hidden, and that is not minimised
//!
//! `crate::surfaces` has a separate reason for each, for the reason written
//! beside it: a person who minimised a window expects to find it minimised, and
//! a person who switched desktop expects to find everything as they left it.
//!
//! # Membership follows what is buffered, not what is drawn
//!
//! The windows a display holds are the ones with a buffer, including those a
//! person minimised and those on another desktop. Asking what is *drawn* would
//! mean that switching desktop closed every window on the one being left —
//! which is what happened before this file existed, because the division was
//! told the drawn windows were all of them.

use alo_desktops::DisplayId;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

impl crate::Server {
    /// The windows that are open now, told to the desktops and the divisions.
    ///
    /// Run once a frame. A window that has appeared joins the current desktop
    /// of every display; a window that has gone leaves them, loses its share
    /// and its number is never given to anything else.
    pub(crate) fn the_windows_are_now_these(&mut self) {
        let open: Vec<WlSurface> = self.surfaces.buffered().cloned().collect();
        let displays: Vec<DisplayId> = self.desk.displays().collect();
        for surface in &open {
            let number = self.desk.number_of(surface);
            for display in &displays {
                self.desk.put_on_the_current_desktop(*display, number);
            }
        }
        self.desk
            .windows_are_now(open.iter().collect::<Vec<_>>().into_iter());
        self.show_the_current_desktops();
    }

    /// Hide every window whose desktop is not the one being looked at.
    ///
    /// Asked of `alo-desktops` for each display rather than remembered here, so
    /// there is no second answer to go stale. A window on no display's desktop
    /// list — one that has just appeared and has not been put on yet, or one on
    /// a machine with no display at all — is **shown**: a window nobody has
    /// placed is not a window somebody hid.
    pub(crate) fn show_the_current_desktops(&mut self) {
        let displays: Vec<DisplayId> = self.desk.displays().collect();
        if displays.is_empty() {
            return;
        }
        let here: Vec<u64> = displays
            .iter()
            .filter_map(|display| self.desk.windows_on_the_current_desktop(*display))
            .flatten()
            .collect();
        let buffered: Vec<WlSurface> = self.surfaces.buffered().cloned().collect();
        for surface in buffered {
            let Some(number) = self.desk.number_given_to(&surface) else {
                continue;
            };
            self.surfaces
                .set_elsewhere(&surface, !here.contains(&number));
        }
    }
}
