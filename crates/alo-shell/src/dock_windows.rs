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
    /// Every window the Dock knows of, as `(its id, its surface, how it sits)`.
    ///
    /// **One iteration, so that two questions cannot answer differently.**
    /// [`Self::the_windows_the_dock_sees`] turns these into `alo_dock::Window`s
    /// and [`Self::the_window_numbered`] turns an id back into a surface, and
    /// both are wrong together or right together. Written out twice they would
    /// be two places deciding what a `WindowId` means — and a lookup that
    /// disagreed with the builder would focus **somebody else's window**, which
    /// is the worst failure this seam has available to it.
    ///
    /// Mapped first, then minimised. `mapped_surfaces` is the visible renderer
    /// and input list and excludes minimised roots, so the two are disjoint.
    ///
    /// A surface past `u64::MAX` ends the iteration rather than wrapping. It
    /// cannot happen and saying what would is cheaper than arguing that it
    /// cannot.
    fn the_windows_and_what_they_are(
        &self,
    ) -> impl Iterator<
        Item = (
            WindowId,
            &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
            HowItSits,
        ),
    > {
        let on_the_canvas = self
            .mapped_surfaces()
            .map(|surface| (surface, HowItSits::OnTheCanvas));
        let put_aside = self
            .minimized_surfaces()
            .map(|surface| (surface, HowItSits::PutAside));
        on_the_canvas
            .chain(put_aside)
            .enumerate()
            .map_while(|(number, (surface, sits))| {
                u64::try_from(number)
                    .ok()
                    .map(|number| (WindowId::numbered(number), surface, sits))
            })
    }

    /// The surface this `WindowId` names, if the Dock still knows of it.
    ///
    /// **`None` is an ordinary answer**, not a failure: a window can close
    /// between the frame that drew the Dock and the press that acts on it, and
    /// ids are this list's own indices, so a window closing renumbers the rest.
    /// Acting on a stale id is exactly the case this returns `None` for rather
    /// than quietly acting on whichever window inherited the number — and the
    /// id is re-looked-up at the moment of the press for that reason, instead
    /// of a surface being remembered from the draw.
    #[must_use]
    pub fn the_window_numbered(
        &self,
        id: WindowId,
    ) -> Option<&smithay::reexports::wayland_server::protocol::wl_surface::WlSurface> {
        self.the_windows_and_what_they_are()
            .find(|(numbered, _, _)| *numbered == id)
            .map(|(_, surface, _)| surface)
    }

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
    /// **Minimised windows are here, marked as minimised.** `mapped_surfaces`
    /// excludes them — it is the visible renderer and input list — so
    /// `minimized_surfaces` is read after it and its windows carry
    /// [`HowItSits::PutAside`].
    ///
    /// This sentence used to say the opposite: *minimised windows are not here,
    /// because `mapped_surfaces` excludes them … the Dock learns about those
    /// through `alo-put-aside`, which is a different road and not this one.*
    /// That was a true description of the code and it made three answers wrong,
    /// in a way that looked right:
    ///
    /// - **`how_many_put_aside` was always zero**, on every entry, on every
    ///   machine. `alo_dock::announcing` reads it for what a screen reader is
    ///   told, so a person with three windows put away was told about none of
    ///   them.
    /// - **`is_open` was false for an application whose windows were all
    ///   minimised**, so it drew no running indicator. The owner's ruling of
    ///   2026-10-11 is explicit that *open, including minimised windows* gets
    ///   one.
    /// - **The membership rule could not fire.** An unpinned application with
    ///   only minimised windows was already absent from the Dock — but by
    ///   *exclusion*, not by decision, and `OnTheDock::belongs_on_the_dock`
    ///   never saw it. The right answer for the wrong reason, which stops being
    ///   the right answer the moment the rule changes.
    ///
    /// **The order is unaffected for every application that has an icon.** The
    /// minimised windows are appended, so an application whose windows are all
    /// minimised would sort last — and that application is not on the Dock
    /// unless it is pinned, and a pinned one sits in the pinned section before
    /// any of this. An application with one window open and one put aside takes
    /// its place from the open one, which is where it already was.
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
        for (number, surface, sits) in self.the_windows_and_what_they_are() {
            // This list's own index, not anything a client chose: a `WindowId`
            // only has to tell two windows apart, and a client cannot be
            // trusted to supply one that does.
            //
            // **And it is stable only within one frame**, which was already
            // true and matters more now that there are two lists: minimising a
            // window moves it from the first to the second and renumbers
            // everything after it. Nothing today carries an id between frames —
            // `Holding::showing` groups and counts — so this is correct and
            // cheap. **The window picker will need an id that outlives a
            // frame**, because *choose this window* is a sentence about one
            // window said now and acted on later, and that is task 3 of
            // `docs/autonomy/the-dock-more-apps-and-the-minimised-panel.md`
            // rather than something to invent here.
            let called = self.the_name_of(surface);
            windows.opened(Window::of(
                number,
                self.the_application_of(surface),
                // The application's own string where it gave one, and an empty
                // name otherwise. **Not a translated word**: `FrameName`
                // returns `AnApplication` for a window that named neither a
                // title nor a class precisely so that a caller decides, and a
                // Dock entry carrying the English phrase *an application* would
                // be a translated string in a field that holds client text.
                called.its_own_words().unwrap_or_default(),
                somewhere,
                sits,
            ));
        }
        windows
    }

    /// The entries the Dock shows, in the order it shows them.
    ///
    /// **It answered only *how many* until 2026-10-10**, because the bar's
    /// width was all a count could give and nothing drew an icon. It gives the
    /// list now: an application with no artwork shows its first letter, by the
    /// owner's ruling of that day, so the Dock needs the applications
    /// themselves. The count is `.len()` and `dock_raster` takes it that way.
    ///
    /// It exists so that **every caller takes this from one place.** Two call
    /// sites holding two literal zeros is how `desktop_raster` and
    /// `screens_raster` came to agree with each other and with nothing else;
    /// two call sites each doing their own arithmetic would be the same fault
    /// wearing more code.
    ///
    /// `pinned` is what the person has pinned. **Nothing in this shell stores
    /// that yet** — `alo_dock::Holding` is the pinned list and no file reads or
    /// writes one — so every caller today passes `Holding::nothing()` and this
    /// counts the applications that have a window open. That is a true count
    /// and the Dock drawn from it is a Dock of what is running, which is what
    /// a machine with no saved pins should show anyway.
    #[must_use]
    pub fn what_the_dock_holds(&self, pinned: &alo_dock::Holding) -> Vec<alo_dock::OnTheDock> {
        pinned.showing(&self.the_windows_the_dock_sees())
    }

    /// The entries *More apps* shows, in the order it shows them.
    ///
    /// **A longer list than [`Self::what_the_dock_holds`], and not derivable
    /// from it.** An unpinned application whose windows are all minimised is in
    /// this one and not that one — the owner's ruling of 2026-10-11, ADR 0103 —
    /// so the applications that differ are precisely the ones the Dock's list
    /// does not contain.
    ///
    /// `alo_dock::Holding::in_the_overview` is that list, and
    /// [`Self::what_the_dock_holds`] is it with the membership rule applied, so
    /// the two are one walk and cannot come to disagree.
    #[must_use]
    pub fn what_more_apps_holds(&self, pinned: &alo_dock::Holding) -> Vec<alo_dock::OnTheDock> {
        pinned.in_the_overview(&self.the_windows_the_dock_sees())
    }

    /// Who is on **this person's** Dock: [`Self::what_the_dock_holds`] asked
    /// with the pins this session worked out.
    ///
    /// **Every production caller uses this one**, and the reason is a fault
    /// that was measured rather than feared. While the two methods above were
    /// the only way in, three call sites across two files each supplied
    /// `Holding::nothing()` — and when the pins became real, **854 of 854 tests
    /// passed with the draw still reading the constant.** A seam a caller has
    /// to remember to connect is a seam that will be found disconnected.
    ///
    /// The two methods above stay, because comparing a Dock with and without a
    /// pin is a real thing to want: `examples/a_real_application` does exactly
    /// that, and it is how *pinned keeps its icon* is told from *nothing was
    /// put aside at all*.
    #[must_use]
    pub fn what_this_persons_dock_holds(&self) -> Vec<alo_dock::OnTheDock> {
        self.what_the_dock_holds(self.what_the_dock_is_pinned_to())
    }

    /// What **this person's** *More apps* lists, for
    /// [`Self::what_this_persons_dock_holds`]' reason exactly.
    ///
    /// The keyboard reads this and so does the draw, which is what keeps the
    /// arrow keys moving through the grid a person is looking at. They read
    /// two different lists for one commit's length and the comment saying so
    /// was written before the bug could happen.
    #[must_use]
    pub fn what_this_persons_more_apps_holds(&self) -> Vec<alo_dock::OnTheDock> {
        let shown = self.what_more_apps_holds(self.what_the_dock_is_pinned_to());
        let typed = &self.what_was_typed_into_more_apps;
        if typed.is_nothing() {
            return shown;
        }
        // **Filtered here, where every reader of the grid already is.** The
        // draw, the hit test and the arrow keys all come through this one
        // method; filtering in the drawing instead would leave the keyboard
        // walking the unfiltered list, which is the fault this lane found and
        // fixed on 2026-10-12 — 854 of 854 tests passed with the two
        // disagreeing.
        let names: Vec<&str> = shown.iter().map(|on| self.what_it_is_called(on)).collect();
        let left = alo_dock::which_are_left(names, typed);
        left.into_iter()
            .filter_map(|which| shown.get(which).cloned())
            .collect()
    }

    /// What this person has typed into the overview's search field.
    #[must_use]
    pub fn what_was_typed_into_more_apps(&self) -> &alo_dock::WhatTheyTyped {
        &self.what_was_typed_into_more_apps
    }

    /// Tell this session what its Settings entry is called.
    ///
    /// Said once at sign-in: the label does not change for the life of a
    /// session, and the `Server` has no vocabulary to say it with. See the
    /// field's own note for why a native entry cannot be searched without it.
    pub fn its_settings_entry_is_called(&mut self, called: String) {
        self.the_settings_entry_is_called = called;
    }

    /// The name this row is drawn and searched by.
    ///
    /// **One answer for both**, which is the whole of a search field being
    /// trustworthy: a row that matched on something a person cannot see is a
    /// row they cannot explain. An application is its own identifier; the
    /// Settings entry is the translated label this session was told at
    /// sign-in.
    pub(crate) fn what_it_is_called<'a>(&'a self, on: &'a alo_dock::OnTheDock) -> &'a str {
        on.app()
            .map_or(self.the_settings_entry_is_called.as_str(), |app| app.name())
    }
}

#[cfg(test)]
#[path = "dock_windows_tests.rs"]
mod tests;
