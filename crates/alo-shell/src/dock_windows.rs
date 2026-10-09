//! What the Dock is told is open, built from the windows actually mapped.
//!
//! # The link that was missing
//!
//! `alo-dock` has known how to answer *what is on the Dock* since it was
//! written: [`alo_dock::Holding::showing`] takes an [`alo_dock::Windows`] and
//! returns one `OnTheDock` per application, each carrying whether it is
//! pinned, how many windows it has and how many of those are put aside. That
//! crate is 30 files and holds the states, what a click does, the window
//! picker, overflow, hover, revealing, dragging and every accessible name.
//!
//! **Nothing ever built it a `Windows`.** `crates/alo-shell/src` named
//! `alo_dock::Holding` in two comments and no line of code, and
//! `desktop_raster` handed `dock_raster::picture` a literal `0` under a comment
//! saying exactly that. The bar drawn on this machine on 2026-10-08 was
//! `MARGIN + MARGIN` — two margins with nothing between them — which was the
//! honest width of a Dock holding nothing, because nothing had ever told it
//! there was anything.
//!
//! This file is that telling, and it is the whole of what it does.
//!
//! # The order, which is the part that could be quietly wrong
//!
//! [`alo_dock::Windows`] keeps **two** orders, and they are not the same fact:
//!
//! - **the order windows opened in**, which `Windows::each_as_opened` returns
//!   and which `Holding::showing` iterates. `windows.rs` says why: *where an
//!   icon sits is a fact a person learns once, so it follows when the
//!   application arrived and never what they have been using.* This is the
//!   Dock's order, and `Server::mapped_surfaces` is already in it.
//! - **the order they were last used**, which `Windows::most_recent_of` reads
//!   and which answers *take me back to the window I was in*.
//!
//! **This builder is right about the first and cannot be right about the
//! second.** `Windows::opened` inserts at the front of the recency order, so
//! replaying mapped surfaces in order leaves the newest window looking like the
//! one most recently used. That is not true; it is just the only thing the list
//! can conclude from being told about windows in creation order.
//!
//! **Nothing in this shell records when a person last used a window.** There is
//! no public accessor for focus or activation order on `Server`. So the honest
//! state is: the Dock's icon order is correct, and
//! [`alo_dock::Windows::most_recent_of`] applied to one of these answers *most
//! recently opened*. Until something tracks use, `Windows::used` is the call
//! that would make it true and there is nothing to call it with.
//!
//! It is written here rather than discovered later because a wrong answer to
//! *return me to what I was doing* is indistinguishable from a right one until
//! somebody has two windows of one application open.

use alo_dock::{HowItSits, Patch, Spot, Window, WindowId, Windows};

impl crate::Server {
    /// Every mapped window, as the Dock understands windows.
    ///
    /// One [`alo_dock::Window`] per mapped surface, in `mapped_surfaces`'
    /// order, each carrying the application that mapped it where it named one
    /// and what the window calls itself.
    ///
    /// **A window whose application is unknown is included, not skipped.** It
    /// is open, a person can see it, and leaving it out would make the Dock
    /// disagree with the screen. `alo_dock::Window` holds its application as an
    /// [`Option`] for exactly this, by the owner's direction that *a missing
    /// application identity must not prevent minimization* and that nobody may
    /// *substitute a fabricated app identity*. Such a window has no icon to
    /// appear under, which `Holding::showing` already knows.
    ///
    /// **Minimised windows are not here**, because `mapped_surfaces` excludes
    /// them: a window a person put aside is not a window on the canvas. The
    /// Dock learns about those through `alo-put-aside`, which is a different
    /// road and not this one.
    ///
    /// See this file's own header for what the resulting order does and does
    /// not mean.
    #[must_use]
    pub fn the_windows_the_dock_sees(&self) -> Windows {
        let mut windows = Windows::none();
        // **Where a window sits, for a Dock not being asked about geometry.**
        // `Patch` refuses a zero extent, so there is no placeholder even if one
        // were wanted; one unit at the origin is the smallest honest thing to
        // say. Nothing this feeds reads it — `Holding::showing` groups by
        // application and counts, and the picker reads names. **When the Dock
        // is asked where to travel to, this has to become the window's real
        // patch**, which is canvas geometry this builder cannot see.
        //
        // Taken once, and a refusal ends the list rather than being unwrapped:
        // `expect` outside a test is forbidden here, and a Dock drawn holding
        // nothing is a better failure than a session that panicked over a
        // placeholder.
        let Ok(somewhere) = Patch::of(Spot::at(0, 0), 1, 1) else {
            return windows;
        };
        for (number, surface) in self.mapped_surfaces().enumerate() {
            // This list's own index, not anything a client chose: a `WindowId`
            // only has to tell two windows apart, and a client cannot be
            // trusted to supply one that does.
            let Ok(number) = u64::try_from(number) else {
                break;
            };
            let called = self.the_name_of(surface);
            windows.opened(Window::of(
                WindowId::numbered(number),
                self.the_application_of(surface),
                // The application's own string where it gave one, and an empty
                // name otherwise. **Not a translated word**: `FrameName`
                // returns `AnApplication` for a window that named neither a
                // title nor a class precisely so that a caller decides, and a
                // Dock entry carrying the English phrase *an application* would
                // be a translated string in a field that holds client text.
                called.its_own_words().unwrap_or_default(),
                somewhere,
                HowItSits::OnTheCanvas,
            ));
        }
        windows
    }

    /// How many entries the Dock would show, for the bar's own width.
    ///
    /// `dock_raster::picture` is given a count rather than the entries, because
    /// all it decides is how wide the bar is — `Room::a_bar_holding` turns the
    /// count into pixels. This is that count, and it exists so that **every
    /// caller takes it from one place.** Two call sites holding two literal
    /// zeros is how `desktop_raster` and `screens_raster` came to agree with
    /// each other and with nothing else; two call sites each doing their own
    /// arithmetic would be the same fault wearing more code.
    ///
    /// `pinned` is what the person has pinned. **Nothing in this shell stores
    /// that yet** — `alo_dock::Holding` is the pinned list and no file reads or
    /// writes one — so every caller today passes `Holding::nothing()` and this
    /// counts the applications that have a window open. That is a true count
    /// and the Dock drawn from it is a Dock of what is running, which is what
    /// a machine with no saved pins should show anyway.
    #[must_use]
    pub fn how_many_the_dock_holds(&self, pinned: &alo_dock::Holding) -> usize {
        pinned.showing(&self.the_windows_the_dock_sees()).len()
    }
}

#[cfg(test)]
#[path = "dock_windows_tests.rs"]
mod tests;
