//! A person pointing at a preview, and getting a look at the window.
//!
//! **This makes an existing promise true rather than adding one.** `docs/features.md` has
//! carried the put-aside panel at `[v0.01]` since the owner moved it there, and
//! `alo_put_aside::peeking_at_a_preview` has decided every part of what a peek means since
//! 2026-09-30 — what it may not change, that letting go leaves the window minimised, that a
//! peek at nothing is refused. **Nothing could reach it.** No line of `features.md` changes
//! below, and nothing here decides anything that crate already decided.
//!
//! # The road exists now, and this paragraph has been rewritten twice
//!
//! Task 5 read *blocked on the panel not being drawn or routed at all, 2026-09-30*. The
//! drawing clause stopped being true in `#343`. **The routing clause stopped being true in the
//! change that added this paragraph**, and the two earlier drafts of it are worth keeping
//! because each was accurate when written and wrong within hours:
//!
//! - the first said *what remained was one road*, before the six-test count was measured;
//! - the second said this file *does not unblock task 5* and leaves it blocked on an input
//!   road, which was true for one commit — `#367` — and stopped being true here.
//!
//! What a pointer now travels: `libinput_routing` translates the event, `direct_seat` and
//! `direct_pointer` settle the position, `Desk::dispatch` asks
//! `Server::where_the_pointer_is_on_the_panel` for a classification, and
//! `TheDesktop::the_pointer_is_now` hands it to the crate that holds the `Panel`. **A person
//! moving the pointer over a preview now gets a peek**, on a running machine, for the first
//! time.
//!
//! What is still owed, so this header does not overclaim in the other direction: nothing yet
//! *puts* a window aside by gesture — `Server::put_this_window_aside` still has only its
//! integration tests — so a running machine has an empty panel to peek at until that road
//! exists too. Peeking is reachable; filling the panel is not.
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
    /// Where the pointer is now, as far as the panel is concerned.
    ///
    /// `None` when there is nothing to answer with: before the first draw there is no panel
    /// geometry, and before the seat has a pointer there is no position. **Two absences with
    /// one answer, and that is right here** — both mean *no classification exists*, and a
    /// caller does the same thing about either, which is nothing.
    ///
    /// Answers nothing rather than the wrong window when the panel has changed since the draw —
    /// the identity check in `which_preview_the_pointer_is_on` is what holds that, and its own note
    /// gives the reason.
    pub(crate) fn where_the_pointer_is_on_the_panel(&self, panel: &Panel) -> Option<OnThePanel> {
        let drawn = self.panel_as_drawn.as_ref()?;
        let at = self.where_the_pointer_is_in_pixels()?;
        // **The Panel-aware classification, because the frame has one.** `Desk::dispatch` asks
        // the desktop for its frame, which carries `put_aside`, so the identity check in
        // `which_preview_the_pointer_is_on` applies here rather than only to callers that
        // happen to hold a `Panel` — a reordered panel names no window instead of the one that
        // now occupies that slot.
        Some(which_preview_the_pointer_is_on(panel, drawn, at))
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
}

/// What a classification does to a peek: the decision, with no geometry in it.
///
/// **Its own function because it is the behaviour, and because a binary cannot test itself.**
/// `alo-desktop` holds the `Panel` and the peek, so it is the only place that can act on a
/// classification — and it is a binary with no room for a suite. The decision therefore lives
/// here, where the tests are, and that crate supplies its two values and keeps the answer.
///
/// That is the same division the classification itself follows: this crate laid the panel out
/// and consumes its own classification, the other crate owns the state, and
/// `alo_put_aside::peeking_at_a_preview` owns the rule. Three jobs, one each.
///
/// # Errors
///
/// `NotPutAside::ItIsNotThere` is **not** returned: it is turned into *nothing changed*, for
/// the reason given at the match arm. Any other refusal `peek_at` grows later is returned, so
/// a new case cannot be swallowed by this one's generosity — which is why the `Result` stays
/// rather than the signature being simplified to the only outcome it has today.
pub fn what_a_classification_does_to_a_peek(
    on: OnThePanel,
    peeking: Peeking,
    panel: &Panel,
) -> Result<ThePeek, NotPutAside> {
    match on {
        OnThePanel::APreview(id) => {
            if peeking.at() == Some(id) {
                // Already looking at this one. Beginning again would be a redraw a person
                // asked for by not moving.
                return Ok(ThePeek::Unchanged(peeking));
            }
            // **A refusal is *nothing changed*, not an error travelling upwards.** The
            // classification names what was *drawn* at that point, so a person pointing at a
            // preview whose window has since been brought back is pointing at a picture of
            // something gone. They have not made a mistake and there is nothing to tell them;
            // the peek stays exactly as it was. `peek_at` is still the thing that decides it,
            // which is the point — the rule is not re-implemented here, only its refusal is
            // given a meaning at this level.
            match peek_at(panel, id) {
                Ok(begun) => Ok(ThePeek::Began(begun)),
                Err(NotPutAside::ItIsNotThere) => Ok(ThePeek::Unchanged(peeking)),
                Err(why) => Err(why),
            }
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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_canvas::{Place, Zoom};
    use alo_dock::window::WindowId;
    use alo_dock::{AppId, HowItSits, Patch, Spot, Window};
    use alo_put_aside::whether_it_is_private::Privacy;

    /// Window ids that are not slot indices, so an off-by-one cannot pass.
    fn a_window(number: u64) -> Window {
        Window::of(
            WindowId::numbered(number),
            Some(AppId::named("Docs").expect("a named application")),
            "A window",
            Patch::of(Spot::at(0, 0), 800, 600).expect("a window has extent"),
            HowItSits::OnTheCanvas,
        )
    }

    fn a_panel_holding(ids: &[u64]) -> Panel {
        let mut panel = Panel::new();
        for id in ids {
            panel
                .put_aside(
                    &a_window(*id),
                    Zoom::LIFE_SIZE,
                    Place::FIRST,
                    Privacy::Ordinary,
                )
                .unwrap();
        }
        panel
    }

    /// **Pointing at a preview begins a peek at that window.**
    #[test]
    fn pointing_at_a_preview_begins_a_peek_at_that_window() {
        let panel = a_panel_holding(&[11, 12]);
        let what = what_a_classification_does_to_a_peek(
            OnThePanel::APreview(WindowId::numbered(12)),
            Peeking::at_nothing(),
            &panel,
        )
        .unwrap();

        assert!(what.changed(), "a peek beginning is a change");
        assert_eq!(what.now().at(), Some(WindowId::numbered(12)));
        assert!(matches!(what, ThePeek::Began(_)));
    }

    /// **Pointing at the same preview again changes nothing.**
    ///
    /// A person who has not moved has not asked for anything, and reporting a change would
    /// make the caller redraw once per input batch for as long as a hand is still.
    #[test]
    fn pointing_at_the_same_preview_again_changes_nothing() {
        let panel = a_panel_holding(&[11, 12]);
        let already = peek_at(&panel, WindowId::numbered(12)).unwrap();

        let what = what_a_classification_does_to_a_peek(
            OnThePanel::APreview(WindowId::numbered(12)),
            already,
            &panel,
        )
        .unwrap();

        assert!(!what.changed(), "nothing moved, so nothing changed");
        assert_eq!(what.now().at(), Some(WindowId::numbered(12)));
    }

    /// **The gaps between previews do not end a peek.**
    ///
    /// The clause this file exists for: that ground is what a person crosses while reaching
    /// for a preview, so ending the peek there would make one impossible to hold.
    #[test]
    fn the_panels_own_region_without_a_preview_leaves_the_peek_alone() {
        let panel = a_panel_holding(&[11, 12]);
        let already = peek_at(&panel, WindowId::numbered(11)).unwrap();

        let what = what_a_classification_does_to_a_peek(
            OnThePanel::TheRegionButNoPreview,
            already,
            &panel,
        )
        .unwrap();

        assert!(!what.changed());
        assert_eq!(
            what.now().at(),
            Some(WindowId::numbered(11)),
            "crossing a gap ended a peek that should have been held"
        );
    }

    /// **Leaving the panel ends the peek, and the window stays put aside.**
    #[test]
    fn leaving_the_panel_ends_the_peek_and_the_window_stays_put_aside() {
        let panel = a_panel_holding(&[11]);
        let already = peek_at(&panel, WindowId::numbered(11)).unwrap();

        let what =
            what_a_classification_does_to_a_peek(OnThePanel::Elsewhere, already, &panel).unwrap();

        assert!(what.changed());
        assert!(matches!(what, ThePeek::Ended(_)));
        assert!(!what.now().is_peeking());
        assert_eq!(
            panel.holding(),
            1,
            "the owner's sentence: letting go leaves the window minimised"
        );
    }

    /// **Leaving the panel when nothing was being peeked at is not a change.**
    ///
    /// Most pointer motion on a machine is this case — the pointer is somewhere on the canvas
    /// and no peek is open — so reporting a change here would report one constantly.
    #[test]
    fn leaving_the_panel_with_no_peek_open_is_not_a_change() {
        let panel = a_panel_holding(&[11]);
        let what = what_a_classification_does_to_a_peek(
            OnThePanel::Elsewhere,
            Peeking::at_nothing(),
            &panel,
        )
        .unwrap();

        assert!(!what.changed());
        assert!(!what.now().is_peeking());
    }

    /// **Pointing at a preview of a window the panel no longer holds changes nothing.**
    ///
    /// The classification names what was *drawn*, so this is a person pointing at a picture of
    /// a window that has since been brought back. `peek_at` refuses, and that refusal means
    /// *nothing changed* here rather than travelling upwards as an error — they have not made
    /// a mistake and there is nothing to tell them.
    #[test]
    fn pointing_at_a_window_the_panel_no_longer_holds_changes_nothing() {
        let panel = a_panel_holding(&[11]);

        let what = what_a_classification_does_to_a_peek(
            OnThePanel::APreview(WindowId::numbered(99)),
            Peeking::at_nothing(),
            &panel,
        )
        .expect("a stale preview is not an error a person should see");

        assert!(!what.changed());
        assert!(!what.now().is_peeking());
    }

    /// **A peek moves straight from one preview to another.**
    ///
    /// A person sliding down the rail crosses previews without leaving the panel, and the peek
    /// has to follow rather than needing the pointer to leave and come back.
    #[test]
    fn sliding_from_one_preview_to_another_moves_the_peek() {
        let panel = a_panel_holding(&[11, 12, 13]);
        let on_the_first = peek_at(&panel, WindowId::numbered(11)).unwrap();

        let what = what_a_classification_does_to_a_peek(
            OnThePanel::APreview(WindowId::numbered(13)),
            on_the_first,
            &panel,
        )
        .unwrap();

        assert!(what.changed());
        assert_eq!(what.now().at(), Some(WindowId::numbered(13)));
    }
}
