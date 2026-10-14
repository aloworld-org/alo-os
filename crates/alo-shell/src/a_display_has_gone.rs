//! One display has gone, and the session carries on without it.
//!
//! `Server::retire_output` is the **session's** retirement: it withdraws every
//! display's global, because a session ending takes all of them. A monitor
//! being unplugged from a desk with two is not that, and calling it would
//! withdraw the globals of the screens the person is still looking at.
//!
//! So this withdraws exactly one. `more-than-one-display-plan.md` task 13.

use crate::Server;

impl Server {
    /// Withdraw one display, leaving every other display alone.
    ///
    /// Returns whether this session had ever presented it: a display that
    /// arrived and left between two frames was never drawn to, and there is
    /// nothing to withdraw rather than something wrong.
    ///
    /// **The entry is kept and its contents withdrawn**, which is
    /// `Presentation::withdraw`'s own decision and not a shortcut taken here:
    /// a client may have queued a bind before receiving `global_remove`, and
    /// destroying the binding data immediately would disconnect a client that
    /// did nothing wrong. Withdrawing clears the metadata, and
    /// `Server::the_displays_as_reported` is written to skip a presentation
    /// with no metadata — so this display leaves the reported set, the
    /// arrangement is asked again, and no map surgery is needed to make that
    /// happen.
    ///
    /// **What this deliberately does not touch** is the two pieces of state
    /// that describe *a* display rather than each one: the popups' output size
    /// and the window-mode output. On a desk with two monitors one of them
    /// leaving does not make those meaningless, and clearing them here would
    /// take the remaining display's extent away from every popup on it.
    /// `retire_output` clears them because by then there is no display left.
    pub(crate) fn this_display_has_gone(&mut self, named: &str) -> bool {
        let handle = self.display_handle();
        let Some(presented) = self.presentations.get_mut(named) else {
            return false;
        };
        presented.withdraw(&handle);
        true
    }
}
