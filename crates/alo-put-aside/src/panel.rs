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

use alo_canvas::Zoom;
use alo_dock::window::{Window, WindowId};

use crate::preview::Preview;
use crate::showing::{Chosen, HowItShows};
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
    /// What the person chose to see.
    chosen: Chosen,
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
        privacy: Privacy,
    ) -> Result<(), NotPutAside> {
        if self.holds(window.id()) {
            return Err(NotPutAside::ItIsAlreadyThere);
        }
        self.previews.insert(0, Preview::of(window, zoom, privacy));
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

    /// What the person chose to see.
    #[must_use]
    pub const fn chosen(&self) -> Chosen {
        self.chosen
    }

    /// Collapse the panel to a rail.
    ///
    /// **Touches no window.** See this file's header: the rule is held by what this
    /// method can reach, not by a check afterwards.
    pub const fn collapse(&mut self) {
        self.chosen = Chosen::Collapsed;
    }

    /// Expand the panel to named previews.
    pub const fn expand(&mut self) {
        self.chosen = Chosen::Expanded;
    }

    /// What this panel shows.
    #[must_use]
    pub const fn showing(&self) -> HowItShows {
        HowItShows::of(self.previews.len(), self.chosen)
    }
}
