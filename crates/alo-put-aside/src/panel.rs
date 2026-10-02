//! The panel itself: which windows are put aside, in the order they were put there.
//!
//! # The order is the order they were put away, and nothing re-sorts it
//!
//! Most recently put aside is **first**, because the thing a person is most likely to
//! want back is the thing they just put down. Nothing else reorders the list — not use,
//! not application, not title. A panel that re-sorted itself would move a preview out
//! from under a pointer that was travelling towards it, which is the same class of
//! fault as an animation changing a hit target midway through a press.
//!
//! # Collapsing changes the panel and never the windows
//!
//! That is the owner's sixth behaviour rule, and it is held here by [`Chosen`] being
//! the only thing [`Panel::collapse`] and [`Panel::expand`] can reach: neither takes a
//! window, neither returns one, and the previews are not touched. The test for it
//! asserts the whole list is **unchanged**, rather than asserting that the windows look
//! right afterwards — a rule about what does *not* happen is only held by a test whose
//! subject is the absence.

use alo_canvas::{Place, Zoom};
use alo_dock::window::{Window, WindowId};

use crate::preview::Preview;
use crate::showing::{Chosen, HowItShows};
use crate::the_collapse_choice_per_place::TheCollapseChoice;
use crate::what_alo_is_doing::WhatAloIsDoing;
use crate::whether_it_is_private::Privacy;

/// Why a window could not be put aside, or brought back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotPutAside {
    /// It is already in the panel.
    ///
    /// **Refused rather than ignored.** Putting the same window aside twice would
    /// give a person two previews of one window, and choosing either would restore
    /// the same thing while the other stayed — a row that cannot be got rid of.
    #[error("that window is already put aside, so it is not put aside again")]
    ItIsAlreadyThere,

    /// It is not in the panel.
    ///
    /// **A separate case from an empty panel.** *Nothing is put aside* and *that
    /// particular window is not* are different answers, and a caller that could not
    /// tell them apart would report the wrong one to a person.
    #[error("that window is not in the panel, so there is nothing to bring back")]
    ItIsNotThere,
}

/// The windows a person put aside, most recent first.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Panel {
    /// The previews, most recently put aside first.
    previews: Vec<Preview>,
    /// What the person chose to see, **on each Place separately**.
    ///
    /// One field held one answer for every surface, which is the clause task 1 stayed open
    /// for: a person who collapses the rail on the Place they are writing on has said nothing
    /// about the Place where they keep their reading.
    /// `crate::the_collapse_choice_per_place` carries why absent means expanded.
    chosen: TheCollapseChoice,
}

impl Panel {
    /// A panel holding nothing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Put a window aside, remembering the zoom the person was at.
    ///
    /// The zoom is **handed in, never read**. This crate cannot ask a camera for one, and
    /// `tests/the_panel_never_reaches_the_camera.rs` is what holds that — the dependency
    /// on `alo-canvas` exists for [`Zoom`] and would otherwise put a `Camera` in reach.
    ///
    /// # Errors
    ///
    /// [`NotPutAside::ItIsAlreadyThere`] if it is in the panel already.
    pub fn put_aside(
        &mut self,
        window: &Window,
        zoom: Zoom,
        place: Place,
        privacy: Privacy,
    ) -> Result<(), NotPutAside> {
        if self.holds(window.id()) {
            return Err(NotPutAside::ItIsAlreadyThere);
        }
        self.previews
            .insert(0, Preview::of(window, zoom, place, privacy));
        Ok(())
    }

    /// Take a window back out, and say where it goes.
    ///
    /// The patch it returns is **the one saved when it was put aside**, which is what
    /// makes *restore it where it was* a sentence rather than a wish.
    ///
    /// # Errors
    ///
    /// [`NotPutAside::ItIsNotThere`] if the panel does not hold it.
    pub fn bring_back(&mut self, window: WindowId) -> Result<Preview, NotPutAside> {
        let at = self
            .previews
            .iter()
            .position(|preview| preview.window() == window)
            .ok_or(NotPutAside::ItIsNotThere)?;
        Ok(self.previews.remove(at))
    }

    /// Say what alo is doing in a window that is put aside.
    ///
    /// Returns whether the panel held it. **A report for a window nobody put aside is
    /// dropped, and saying so is the point** — a preview that appeared because a report
    /// arrived would be a row for a window that is not away, which is worse than losing the
    /// report.
    ///
    /// This is how a person watches work in a window they cannot see: the report is replaced
    /// as it changes, and passing [`WhatAloIsDoing::Nothing`] is how it ends.
    pub fn alo_is_now(&mut self, window: WindowId, what: WhatAloIsDoing) -> bool {
        match self
            .previews
            .iter_mut()
            .find(|preview| preview.window() == window)
        {
            Some(preview) => {
                preview.alo_is_now(what);
                true
            }
            None => false,
        }
    }

    /// The put-aside windows where alo has stopped and needs the person.
    ///
    /// **In the panel's own order**, most recently put aside first — not waiting ones first.
    /// A panel that reordered itself when a task needed attention would move a row under a
    /// pointer travelling towards it, which is the fault the reveal state machine exists to
    /// prevent arriving by way of sorting.
    pub fn waiting_on_the_person(&self) -> impl Iterator<Item = &Preview> {
        self.previews
            .iter()
            .filter(|preview| preview.alo().requires_you())
    }

    /// Whether this window is put aside.
    #[must_use]
    pub fn holds(&self, window: WindowId) -> bool {
        self.previews
            .iter()
            .any(|preview| preview.window() == window)
    }

    /// The previews, most recently put aside first.
    #[must_use]
    pub fn previews(&self) -> &[Preview] {
        &self.previews
    }

    /// How many windows are put aside.
    #[must_use]
    pub fn holding(&self) -> usize {
        self.previews.len()
    }

    /// What the person chose to see **on this Place**.
    ///
    /// Task 1's owed clause: the choice is keyed per Place, so this needs to know which one.
    /// A Place a person has never collapsed answers [`Chosen::Expanded`], which is what a
    /// fresh panel answers and for the same reason.
    #[must_use]
    pub fn chosen(&self, place: Place) -> Chosen {
        self.chosen.on(place)
    }

    /// Collapse the panel to a rail **on this Place**, leaving every other Place alone.
    ///
    /// **Touches no window.** See this file's header: the rule is held by what this
    /// method can reach, not by a check afterwards. Taking a Place does not weaken that —
    /// a Place is a number the canvas minted, not a position, and there is still no way
    /// from here to a window or a camera.
    pub fn collapse(&mut self, place: Place) {
        self.chosen.collapse(place);
    }

    /// Expand the panel to named previews **on this Place**.
    pub fn expand(&mut self, place: Place) {
        self.chosen.expand(place);
    }

    /// What this panel shows **on this Place**.
    #[must_use]
    pub fn showing(&self, place: Place) -> HowItShows {
        HowItShows::of(self.previews.len(), self.chosen.on(place))
    }

    /// How many Places the person has collapsed the panel on.
    ///
    /// Offered because *the choice persists per Place* is otherwise only observable one Place
    /// at a time, and a test that asked each Place separately could not tell *expanding
    /// forgets* from *expanding overwrites*. See
    /// `the_collapse_choice_per_place::TheCollapseChoice::expand`.
    #[must_use]
    pub fn how_many_places_are_collapsed(&self) -> usize {
        self.chosen.how_many_are_collapsed()
    }
}
