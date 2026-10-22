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
}

// **There is no per-application getter here, and there was until `alo-reached`
// refused it.**
//
// `what_this_application_draws(&AppId) -> Option<&APicture>` looked like the
// natural companion to the setter above and nothing ever called it. The road a
// picture really takes does not pass through this type: `crate::server` holds
// the whole `TheArtwork` behind an `Arc`, `crate::direct_desktop` clones that
// handle into the frame, and `crate::dock_raster` asks
// `TheArtwork::of(app)` itself at the moment it inks a cell. A getter on
// `Server` would be a second spelling of `TheArtwork::of` reachable only by
// borrowing the server, which the raster does not have and must not need.
//
// So it was written for a caller that the design does not want, which is the
// second of the three answers `alo-reached` offers and the honest one here.
// Written down rather than deleted silently, because the next person to add a
// setter to this file will feel the same pull.
