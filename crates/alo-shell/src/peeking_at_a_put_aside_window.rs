//! A person pointing at a preview, and getting a look at the window.
//!
//! **This makes an existing promise true rather than adding one.** `docs/features.md` has
//! carried the put-aside panel at `[v0.01]` since the owner moved it there, and
//! `alo_put_aside::peeking_at_a_preview` has decided every part of what a peek means since
//! 2026-09-30 — what it may not change, that letting go leaves the window minimised, that a
//! peek at nothing is refused. **Nothing could reach it.** No line of `features.md` changes
//! below, and nothing here decides anything that crate already decided.
//!
//! # What was actually missing, which the plan recorded as something else
//!
//! `docs/autonomy/putting-a-window-aside.md` task 5 read *blocked on the panel not being
//! drawn or routed at all, 2026-09-30* and added *nothing consumes this crate*. Two of those
//! three clauses have stopped being true: `crate::panel_raster` has laid the panel's slots
//! out since `#343`, `crate::desktop_raster` calls it every frame, `alo-desktop` holds a real
//! `Panel` and hands it in, and `crate::putting_a_window_aside` has consumed the crate since
//! `#350`.
//!
//! **The third clause is still true, and it is bigger than the status said.** *Routed* was
//! never only a hit test. Measured on `main` at `ad229c9c`:
//!
//! - `Server::put_this_window_aside` has **six callers and all six are integration tests**;
//! - `alo-desktop`, which owns the `Panel`, has **no pointer or keyboard handling at all**;
//! - so the panel a person sees is drawn from a `Panel::new()` that nothing can add to.
//!
//! So this file does not unblock task 5. It builds the half that is this lane's — the
//! classification and the state change, with the geometry the draw already produces — and
//! leaves the task blocked on an input road in the crate that holds the panel. **Saying that
//! plainly is the point**: an earlier draft of this header claimed *what remained was one
//! road*, which was written before the six-test count was measured and would have been a
//! fresh record outliving the truth inside the change that corrected the last one.
//!
//! # Two steps, and the classification is the other file's
//!
//! `crate::which_preview_the_pointer_is_on` answers *which window*, and this answers *then
//! what*. They are apart because the first is geometry against a laid-out panel and the
//! second is a state change — `alo_put_aside::the_region_the_panel_claims` asks for exactly
//! that division in its own words: *coordinate classification belongs to the caller; the
//! machine consumes that classification.*
//!
//! # Peeking at nothing is a refusal, not an empty peek
//!
//! A pointer in the panel's region but on no preview does **not** begin a peek and does not
//! end one either. It is ground a person crosses on the way to a preview, so ending the peek
//! there would make a peek impossible to hold while reaching for it, and beginning one would
//! claim a window nobody pointed at.

use alo_dock::Peeking;
use alo_put_aside::Panel;
use alo_put_aside::panel::NotPutAside;
use alo_put_aside::peeking_at_a_preview::{peek_at, stop_peeking};
use smithay::utils::{Physical, Point};

use crate::which_preview_the_pointer_is_on::{
    OnThePanel, ThePanelAsDrawn, which_preview_the_pointer_is_on,
};

/// What a pointer moving over the panel did to the peek.
///
/// **Named cases rather than a bare [`Peeking`]**, because *began*, *ended* and *left alone*
/// are three things a caller does three different things about — a redraw, a redraw, and
/// nothing — and a caller handed only the new state would have to compare it against the old
/// one to tell which happened. That comparison is the kind that is written once correctly and
/// then written backwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThePeek {
    /// A peek began, at this window.
    Began(Peeking),
    /// The peek ended. The window **stays minimised** — the owner's sentence, and
    /// `stop_peeking` is what holds it.
    Ended(Peeking),
    /// Nothing changed: already peeking at this one, or the pointer is on ground that
    /// neither begins nor ends a peek.
    Unchanged(Peeking),
}

impl ThePeek {
    /// The peek as it now stands, whatever happened to it.
    #[must_use]
    pub const fn now(self) -> Peeking {
        match self {
            Self::Began(peeking) | Self::Ended(peeking) | Self::Unchanged(peeking) => peeking,
        }
    }

    /// Whether anything changed, so a caller knows whether to draw again.
    #[must_use]
    pub const fn changed(self) -> bool {
        matches!(self, Self::Began(_) | Self::Ended(_))
    }
}

impl crate::Server {
    /// Where the pointer is now, and what that does to the peek.
    ///
    /// **A method on [`crate::Server`] for the same reason
    /// [`crate::Server::put_this_window_aside`] is one**, and with the same standing: the
    /// shell offers the road and the input layer has not called it yet. That method has six
    /// callers and all six are integration tests, because `alo-desktop` — which holds the
    /// `Panel` — has no pointer handling at all. This one is in the same position, which is
    /// stated here rather than left for a reader to infer from a test count.
    ///
    /// **The geometry is the one the last draw produced**, read from the server rather than
    /// taken as an argument, because a public method cannot name `PanelPicture` and because
    /// laying the panel out a second time here is how two answers that must agree stop
    /// agreeing. `Server::the_panel_was_drawn` is what puts it there — named rather than
    /// linked, because it is `pub(crate)` and a public page may not point inside the crate —
    /// and `crate::canvas_fixed_controls` is the precedent in this same struct: set at draw
    /// time for a question asked at pointer time.
    ///
    /// Answers `Ok(None)` before the first draw. **Not an error and not a peek**: a pointer
    /// moving before anything has been painted is an ordinary moment, and inventing a
    /// geometry for it would be answering about a panel nobody has laid out.
    ///
    /// # Errors
    ///
    /// [`NotPutAside::ItIsNotThere`] when the pointer is on a slot whose window the panel no
    /// longer holds.
    pub fn the_pointer_is_now_over_the_panel(
        &self,
        panel: &Panel,
        peeking: Peeking,
        at: Point<i32, Physical>,
    ) -> Result<Option<ThePeek>, NotPutAside> {
        let Some(drawn) = self.panel_as_drawn.as_ref() else {
            return Ok(None);
        };
        the_pointer_is_now(panel, drawn, peeking, at).map(Some)
    }

    /// Where the panel ended up, handed over by the draw that laid it out.
    ///
    /// **The draw is the only place that knows.** The panel's slots are computed in
    /// `crate::panel_raster` and nowhere else, and without this they exist only for the
    /// duration of a frame — which is why the hit test above had no geometry to ask about.
    /// The same reasoning, and the same shape, as `Server::the_dock_was_drawn`.
    pub(crate) fn the_panel_was_drawn(&mut self, drawn: ThePanelAsDrawn) {
        self.panel_as_drawn = Some(drawn);
    }

    /// Escape, or letting go: the peek ends wherever the pointer is.
    pub fn the_person_stopped_peeking(&self, peeking: Peeking) -> ThePeek {
        the_person_let_go(peeking)
    }
}

/// Where the pointer is now, and what that does to the peek.
///
/// `drawn` is the panel as the last draw laid it out, with the windows it was laid out for;
/// the peek as it stands. Both are handed in — this file asks a display nothing, for the
/// reason `crate::putting_a_window_aside` gives about the patch and the Place.
///
/// # Errors
///
/// [`NotPutAside::ItIsNotThere`] when the pointer is on a slot whose window the panel no
/// longer holds. **Nothing changes on a refusal**, including the peek: a refusal that also
/// reported a new peek would hand a caller something to act on about a window that is not
/// there.
fn the_pointer_is_now(
    panel: &Panel,
    drawn: &ThePanelAsDrawn,
    peeking: Peeking,
    at: Point<i32, Physical>,
) -> Result<ThePeek, NotPutAside> {
    match which_preview_the_pointer_is_on(panel, drawn, at) {
        OnThePanel::APreview(id) => {
            if peeking.at() == Some(id) {
                // Already looking at this one. Beginning again would be a redraw a person
                // asked for by not moving.
                return Ok(ThePeek::Unchanged(peeking));
            }
            Ok(ThePeek::Began(peek_at(panel, id)?))
        }
        // **Neither begins nor ends one.** See this module's header: the gaps and the
        // clearance are ground a person crosses while reaching for a preview.
        OnThePanel::TheRegionButNoPreview => Ok(ThePeek::Unchanged(peeking)),
        OnThePanel::Elsewhere => {
            if peeking.is_peeking() {
                Ok(ThePeek::Ended(stop_peeking(peeking)))
            } else {
                Ok(ThePeek::Unchanged(peeking))
            }
        }
    }
}

/// Escape, or letting go: the peek ends wherever the pointer is.
///
/// Its own function because it is a different cause with the same effect, and because the
/// owner named both roads — *releasing or Escape removes it* — so a keyboard road that went
/// through the pointer one would be the keyboard being a consolation for a gesture. That is
/// the thing `alo-dock`'s own header refuses.
fn the_person_let_go(peeking: Peeking) -> ThePeek {
    if peeking.is_peeking() {
        ThePeek::Ended(stop_peeking(peeking))
    } else {
        ThePeek::Unchanged(peeking)
    }
}
