//! What the Dock says it is about to do, before it does it.
//!
//! Two gestures end in something happening to a person's work — dragging an
//! icon onto the canvas makes a window, and dragging a file onto an icon opens
//! it somewhere. **Both are named before they happen**, and this file is that
//! naming.
//!
//! # Nothing happens on the way
//!
//! The owner's constraint, in their words: *hovering a file over an icon should
//! not silently open or transmit it.* Hovering produces an [`Offer`] — a
//! sentence and an outline — and nothing else. The act is the drop, and until
//! the drop there is a proposal that can be abandoned.
//!
//! **This matters beyond tidiness.** An application asked to open a file may
//! read it, and a machine sold on knowing what leaves it cannot have a surface
//! where passing a pointer over an icon begins that. So *the offer is not the
//! act* is a law-1 shape rather than a nicety, and [`Offer::taken`] is the only
//! place an offer becomes anything.
//!
//! # Escape leaves nothing behind
//!
//! Abandoning an offer restores exactly what was there: no window, no half-made
//! thing, nothing moved. [`Offer::abandoned`] answers nothing at all, which is
//! the point — there is no undo because nothing was done.
//!
//! # The same offer without dragging
//!
//! A person who cannot drag, or would rather not, reaches the same two acts from
//! a selected file and from the icon's menu. [`Offer::of`] does not ask how the
//! offer arose, for the same reason [`crate::previews`] does not ask how a list
//! was opened: a road that ends somewhere different is a consolation rather than
//! a road.

use crate::on_the_canvas::{Patch, TheView};
use crate::window::AppId;

/// What a drop would do, named before it is done.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatWouldHappen {
    /// A new window of this application, here.
    ///
    /// The patch is the outline a person sees before letting go, so what they
    /// are shown and what they get are one value rather than two that can
    /// disagree.
    AWindowWouldOpen {
        /// Whose window.
        app: AppId,
        /// Where it would be.
        at: Patch,
    },
    /// This file would be opened in this application.
    ///
    /// Named with both, because *Open in Blender* is the sentence and neither
    /// half of it is guessable from the other.
    AFileWouldOpenIn {
        /// Which application.
        app: AppId,
        /// What the person calls the file. Their own words, never translated.
        file: String,
    },
}

impl WhatWouldHappen {
    /// Which application this is about.
    #[must_use]
    pub const fn app(&self) -> &AppId {
        match self {
            Self::AWindowWouldOpen { app, .. } | Self::AFileWouldOpenIn { app, .. } => app,
        }
    }

    /// The outline to draw, where there is one.
    ///
    /// A new window has a place before it exists; a file opening in an
    /// application does not, because where that window goes is the
    /// application's to decide.
    #[must_use]
    pub const fn outline(&self) -> Option<Patch> {
        match self {
            Self::AWindowWouldOpen { at, .. } => Some(*at),
            Self::AFileWouldOpenIn { .. } => None,
        }
    }
}

/// A proposal: what would happen, held while a person decides.
///
/// **An offer is not an act.** Nothing in this type does anything; it is a
/// sentence and an outline, and the only way out of it is [`Offer::taken`] or
/// [`Offer::abandoned`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// What a drop would do.
    would: WhatWouldHappen,
}

impl Offer {
    /// Offer this.
    ///
    /// **Does not ask how the offer arose** — dragged, chosen from a menu, or
    /// invoked on a selected file, it is the same proposal with the same words.
    #[must_use]
    pub const fn of(would: WhatWouldHappen) -> Self {
        Self { would }
    }

    /// Offer a new window of this application at the place a person is pointing.
    ///
    /// **Where a dropped window goes is where it was dropped**, not a place the
    /// system prefers. The canvas is somewhere a person arranges things, and a
    /// window that landed somewhere tidier than the outline promised would be
    /// the system overruling them at the last moment.
    #[must_use]
    pub const fn a_window_at(app: AppId, at: Patch) -> Self {
        Self::of(WhatWouldHappen::AWindowWouldOpen { app, at })
    }

    /// Offer to open this file in this application.
    #[must_use]
    pub fn this_file_in(app: AppId, file: &str) -> Self {
        Self::of(WhatWouldHappen::AFileWouldOpenIn {
            app,
            file: file.to_owned(),
        })
    }

    /// What would happen.
    #[must_use]
    pub const fn would(&self) -> &WhatWouldHappen {
        &self.would
    }

    /// The outline to draw, where there is one.
    #[must_use]
    pub const fn outline(&self) -> Option<Patch> {
        self.would.outline()
    }

    /// Let go: the offer becomes what it said it would.
    ///
    /// **The only door out of an offer into an act**, so that a reader looking
    /// for *where does this actually happen* finds one answer.
    #[must_use]
    pub fn taken(self) -> WhatWouldHappen {
        self.would
    }

    /// Escape, or let go somewhere that is not a drop.
    ///
    /// Answers nothing, and that is the whole of it: nothing was made, so there
    /// is nothing to unmake and nothing to put back.
    ///
    /// It takes `self` by value so that the offer cannot be used afterwards —
    /// abandoning it is the end of it. Not `const`, because an offer about a
    /// file carries the file's name and letting one go runs a destructor.
    pub fn abandoned(self) {
        drop(self);
    }
}

/// Where a window dropped at a point would sit.
///
/// The size is the shell's business; this puts a patch of that size **centred on
/// where the person let go**, clamped to nothing and snapped to nothing.
///
/// **Not tidied.** A window that landed on a grid, or nudged clear of its
/// neighbours, would be the system rearranging the canvas — which the owner
/// refuses for restoring a window and which is no more welcome here.
#[must_use]
pub fn dropped_at(x: i64, y: i64, wide: u32, tall: u32) -> Option<Patch> {
    let corner = crate::on_the_canvas::Spot::at(
        x.saturating_sub(i64::from(wide) / 2),
        y.saturating_sub(i64::from(tall) / 2),
    );
    Patch::of(corner, wide, tall).ok()
}

/// Where a window opens when nobody pointed anywhere — near what is being
/// looked at.
///
/// This is [`crate::clicking::WhatAClickDoes::OpenAWindowNearTheView`]'s *near*:
/// the middle of the view, so a window a person did not place is where they are
/// already looking rather than somewhere they must go and find.
#[must_use]
pub fn near_the_view(view: TheView, wide: u32, tall: u32) -> Option<Patch> {
    let shown = view.shown();
    let middle_x = shown
        .corner()
        .x()
        .saturating_add(i64::from(shown.wide()) / 2);
    let middle_y = shown
        .corner()
        .y()
        .saturating_add(i64::from(shown.tall()) / 2);
    dropped_at(middle_x, middle_y, wide, tall)
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::on_the_canvas::Spot;

    fn app(name: &str) -> AppId {
        AppId::named(name).unwrap()
    }

    /// **The owner's example, named before the drop:** *Open in Blender*, with
    /// both halves, because neither is guessable from the other.
    #[test]
    fn a_file_over_an_icon_names_the_application_and_the_file() {
        let offer = Offer::this_file_in(app("Blender"), "Scene study.blend");
        match offer.would() {
            WhatWouldHappen::AFileWouldOpenIn { app: which, file } => {
                assert_eq!(which.name(), "Blender");
                assert_eq!(file, "Scene study.blend");
            }
            other => panic!("wrong offer: {other:?}"),
        }
        assert_eq!(
            offer.outline(),
            None,
            "where it opens is the app's to decide"
        );
    }

    /// **The outline shown is the window that is made.** One value, so what a
    /// person was promised and what they get cannot disagree.
    #[test]
    fn the_outline_shown_is_the_window_that_would_be_made() {
        let at = dropped_at(5_000, 3_000, 800, 600).unwrap();
        let offer = Offer::a_window_at(app("Docs"), at);

        assert_eq!(offer.outline(), Some(at));
        match offer.taken() {
            WhatWouldHappen::AWindowWouldOpen {
                app: which,
                at: made,
            } => {
                assert_eq!(which.name(), "Docs");
                assert_eq!(made, at, "the outline is the window");
            }
            other => panic!("wrong offer: {other:?}"),
        }
    }

    /// **A window lands where it was dropped**, centred on the point, and is not
    /// nudged onto a grid or clear of its neighbours: the canvas is the
    /// person's arrangement.
    #[test]
    fn a_dropped_window_is_centred_on_the_point_and_not_tidied() {
        let at = dropped_at(1_000, 800, 800, 600).unwrap();
        assert_eq!(at.corner(), Spot::at(600, 500));
        assert_eq!((at.wide(), at.tall()), (800, 600));

        // Somewhere awkward: no rounding, no snapping.
        let odd = dropped_at(1_003, 807, 801, 601).unwrap();
        assert_eq!(odd.corner(), Spot::at(603, 507));
    }

    /// A window can be dropped left of and above the plane's origin, because the
    /// canvas extends in every direction.
    #[test]
    fn a_window_can_be_dropped_anywhere_on_the_plane() {
        let at = dropped_at(-9_000, -4_000, 800, 600).unwrap();
        assert_eq!(at.corner(), Spot::at(-9_400, -4_300));
    }

    /// A window with no extent is not a window, so there is no offer of one.
    #[test]
    fn there_is_no_offer_of_a_window_with_no_extent() {
        assert_eq!(dropped_at(0, 0, 0, 600), None);
        assert_eq!(dropped_at(0, 0, 800, 0), None);
    }

    /// **Abandoning leaves nothing behind.** There is no undo because nothing
    /// was done — the offer is consumed and no window was ever made.
    #[test]
    fn abandoning_an_offer_makes_nothing() {
        let at = dropped_at(0, 0, 800, 600).unwrap();
        let offer = Offer::a_window_at(app("Docs"), at);
        offer.abandoned();
        // Nothing to assert about a result, which is the whole point: the only
        // other road out of an `Offer` is `taken`, and it was not travelled.
    }

    /// **The same offer however it arose.** Dragged, chosen from a menu, or
    /// invoked on a selected file — the sentence and the outline are identical,
    /// so the road that does not involve dragging is not a lesser one.
    #[test]
    fn an_offer_is_the_same_however_it_was_made() {
        let at = dropped_at(2_000, 2_000, 800, 600).unwrap();
        let dragged = Offer::a_window_at(app("Docs"), at);
        let from_a_menu = Offer::of(WhatWouldHappen::AWindowWouldOpen {
            app: app("Docs"),
            at,
        });
        assert_eq!(dragged, from_a_menu);

        let over_an_icon = Offer::this_file_in(app("Blender"), "Scene study.blend");
        let from_a_selection = Offer::of(WhatWouldHappen::AFileWouldOpenIn {
            app: app("Blender"),
            file: "Scene study.blend".to_owned(),
        });
        assert_eq!(over_an_icon, from_a_selection);
    }

    /// **A window nobody placed opens where the person is looking**, in the
    /// middle of the view rather than somewhere they must go and find.
    #[test]
    fn a_window_nobody_placed_opens_in_the_middle_of_the_view() {
        let view = TheView::showing(Patch::of(Spot::at(10_000, 5_000), 1_920, 1_080).unwrap());
        let at = near_the_view(view, 800, 600).unwrap();

        assert_eq!(at.corner(), Spot::at(10_560, 5_240));
        assert!(
            view.already_shows(at),
            "a window opened out of sight would be a click that did nothing"
        );
    }

    /// And it is in view however far the person has travelled, including left of
    /// and above the origin.
    #[test]
    fn a_window_nobody_placed_is_in_view_wherever_the_person_is() {
        for (x, y) in [(0_i64, 0_i64), (-40_000, -30_000), (90_000, 90_000)] {
            let view = TheView::showing(Patch::of(Spot::at(x, y), 1_920, 1_080).unwrap());
            let at = near_the_view(view, 800, 600).unwrap();
            assert!(view.already_shows(at), "at {x},{y}");
        }
    }

    /// Both kinds of offer name their application, which is what a sentence
    /// about them is filled from.
    #[test]
    fn every_offer_names_its_application() {
        let at = dropped_at(0, 0, 800, 600).unwrap();
        assert_eq!(
            Offer::a_window_at(app("Docs"), at).would().app().name(),
            "Docs"
        );
        assert_eq!(
            Offer::this_file_in(app("Blender"), "x.blend")
                .would()
                .app()
                .name(),
            "Blender"
        );
    }
}
