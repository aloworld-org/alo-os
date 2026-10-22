//! What each application draws on the Dock, held for the session.
//!
//! [ADR 0101](../../../docs/decisions/0101-one-dock-component-family-and-the-renderer-that-can-draw-it.md),
//! the owner's ruling: *real applications retain their actual application
//! icons … first letters are a missing-icon fallback, not the finished Dock.*
//! Until this existed the fallback was every machine's answer for every
//! application, because nothing carried artwork this far.
//!
//! # Handed in, never looked for
//!
//! Which file holds an application's icon is a **reading of the machine**, and
//! `crate::what_the_dock_is_filled_with` states the rule this crate lives
//! under: *the shell does not scan, and that is a rule rather than a layering
//! taste.* So artwork arrives the way the pins and the terminal do — worked out
//! by `alo-desktop` before the display is taken, and handed to
//! [`crate::stand_the_desktop_up`] as finished pixels.
//!
//! There is no path here, no file, no icon theme and no decoder. A test hands
//! over four bytes.
//!
//! # Once, and kept
//!
//! Reading it walks every icon directory for every installed application, so it
//! is done once at stand-up — exactly as the pins are worked out once, and for
//! the same reason: asking per frame would walk the disk per frame.
//!
//! **What this does not do is notice a change.** An application installed while
//! a person is signed in draws its first letter until they sign in again. That
//! is a real limit of this version and it is written here rather than left for
//! somebody to find: it needs something watching the icon directories, which is
//! separate work and is not pretended at.

use crate::server::Server;

impl Server {
    /// Tell this session what each application draws.
    ///
    /// **Replaces rather than merges**, because this is one reading of one
    /// machine and a second reading is a newer answer about the same question —
    /// merging would keep artwork for an application that has since been
    /// removed, under a name the Dock may now give to something else.
    pub fn its_applications_draw(&mut self, artwork: alo_dock::artwork::TheArtwork) {
        self.what_each_application_draws = std::sync::Arc::new(artwork);
    }

    /// What this application draws, or [`None`] where it draws its first
    /// letter.
    #[must_use]
    pub fn what_this_application_draws(
        &self,
        application: &alo_dock::AppId,
    ) -> Option<&alo_dock::artwork::APicture> {
        self.what_each_application_draws.of(application)
    }
}
