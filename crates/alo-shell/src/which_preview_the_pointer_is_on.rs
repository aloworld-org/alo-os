//! Which put-aside window a pointer is on, if any.
//!
//! **This makes an existing promise true rather than adding one**, which is the judgement
//! `CLAUDE.md` asks a change to state. `docs/features.md` has carried *minimising puts a
//! preview in the panel at the edge of the screen* since the owner moved it to `[v0.01]`,
//! and a preview a person cannot point at is not a preview. `alo_put_aside::peeking_at_a_preview`
//! has been complete since 2026-09-30 and `crate::panel_raster` has drawn the slots since
//! `#343`; nothing connected the two, so a person could put a window aside and then not
//! look at it. No line of `features.md` changes below.
//!
//! # One clause of task 5 was stale and one was not
//!
//! Task 5 read *blocked on the panel not being drawn or routed at all, 2026-09-30*. The
//! drawing half stopped being true in `#343` and nobody re-read the status, so the hit test
//! below looked like it was waiting on something rather than being the work.
//!
//! **The routing half is still true**, and `crate::peeking_at_a_put_aside_window` carries the
//! measurement rather than repeating the claim: nothing on a running machine reaches any of
//! this yet. So this file does not unblock the task. It builds the part that was genuinely
//! absent and is genuinely this lane's.
//!
//! # Told the geometry, never asking for it
//!
//! The [`PanelPicture`] is handed in. This file does not lay the panel out and does not ask
//! a display anything — `crate::panel_raster` owns the conversion from the design's figures
//! to a display's pixels, and doing it twice is how two answers that must agree stop
//! agreeing. The same seam `crate::putting_a_window_aside` takes for the patch and the Place.
//!
//! So there is **no figure from the design in this file**, which is how the standing rule is
//! best satisfied: there is nothing here to convert for another screen size.
//!
//! # Half-open, for the reason the pointer classifier gives
//!
//! A slot contains its top-left edge and not its bottom-right, so two slots that share a
//! border do not both contain the points along it. Without that, a pointer on the seam
//! between the third and fourth preview is on both, and *which preview* has two answers —
//! which the return type below cannot express and should not have to.

use alo_dock::window::WindowId;
use alo_put_aside::{Panel, Preview};
use smithay::utils::{Physical, Point, Rectangle};

use crate::panel_raster::PanelPicture;

/// Where a pointer is, as far as the panel is concerned.
///
/// **Three cases rather than an [`Option`], because the middle one is not nothing.** A
/// pointer inside the panel's reserved column but not on any preview is still the panel's —
/// it keeps a revealed panel open, and `alo_put_aside::the_region_the_panel_claims` says why:
/// the previews, the controls and the path between them are one continuous interaction
/// region. An `Option<WindowId>` would spell that the same way as *not on the panel at all*,
/// and a caller would then conceal the panel while the pointer was still inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnThePanel {
    /// On this window's preview.
    APreview(WindowId),
    /// Inside the panel's region, but not on a preview — the gaps, the clearance, the path.
    TheRegionButNoPreview,
    /// Not the panel's at all. The canvas, or another surface.
    Elsewhere,
}

// **No accessors, and that is deliberate.** A first draft carried `preview()` and
// `keeps_the_panel_open()`, and the compiler reported both as never used — which was correct:
// the reveal machine that will ask *does this keep the panel open* does not exist yet, and
// `crate::peeking_at_a_put_aside_window` matches on the variants because it cares about all
// three. Convenience methods written for a caller that has not arrived are the same shape as
// a module with no consumer, which `CLAUDE.md` names and this change is otherwise about. The
// distinction they encoded is in the variants, where it cannot rot; whoever needs an accessor
// adds it in the change that calls it.

/// The panel as one draw laid it out, **with the windows it was laid out for**.
///
/// # Why the window ids are kept and a count was not enough
///
/// `PanelPicture::slots` is one rectangle per put-aside window and nothing in it says *which*
/// window, so a slot's meaning comes from its position in a list. The question is whether that
/// list is still the same list.
///
/// A first version of this file compared the two **lengths**, and that check could not fail.
/// Two reasons, and the second is the one that loses a person's window:
///
/// - `zip` already stops at the shorter list, so a picture with more slots than the panel has
///   previews cannot name a window that is not there. The length check did nothing the
///   iteration was not already doing.
/// - and equal lengths do not mean the same windows. `Panel::bring_back` is
///   `previews.remove(at)`, which **shifts every later preview down one index**, and
///   `put_aside` appends. So a person who brings one window back and puts another aside
///   between a draw and a pointer move has a panel of the **same length** whose indices now
///   name different windows — and the old check passed that straight through to the wrong
///   preview.
///
/// That is the shape this repository spent 2026-10-01 removing: a wrong answer the screen
/// corroborates, because the wrong preview is still a real preview. **So the identities are
/// compared, not the count.** Found by mutating the length check to `if false` and watching
/// all eight tests still pass, which is the only reason it is not in `main`.
#[derive(Debug, Clone)]
pub(crate) struct ThePanelAsDrawn {
    /// The geometry that draw produced: the reserved column, and one slot per window.
    picture: PanelPicture,
    /// The windows the slots were laid out for, in slot order.
    windows: Vec<WindowId>,
}

impl ThePanelAsDrawn {
    /// Capture a draw: its geometry, and which windows that geometry was for.
    pub(crate) fn of(panel: &Panel, picture: PanelPicture) -> Self {
        Self {
            windows: panel.previews().iter().map(Preview::window).collect(),
            picture,
        }
    }
}

/// Which put-aside window `at` is on, given the panel as it was laid out.
///
/// A pointer inside the reserved column whose panel has changed since the draw answers
/// [`OnThePanel::TheRegionButNoPreview`] rather than guessing: the pointer really is in the
/// panel's region, which is the part still known to be true, and no window is named.
pub(crate) fn which_preview_the_pointer_is_on(
    panel: &Panel,
    drawn: &ThePanelAsDrawn,
    at: Point<i32, Physical>,
) -> OnThePanel {
    if !holds(drawn.picture.reserved, at) {
        return OnThePanel::Elsewhere;
    }

    // **The same windows in the same order, or no window is named.** See the type's note: a
    // count cannot tell a reordered panel from an unchanged one.
    let now: Vec<WindowId> = panel.previews().iter().map(Preview::window).collect();
    if now != drawn.windows {
        return OnThePanel::TheRegionButNoPreview;
    }

    for (slot, id) in drawn.picture.slots.iter().zip(drawn.windows.iter()) {
        if holds(*slot, at) {
            return OnThePanel::APreview(*id);
        }
    }

    OnThePanel::TheRegionButNoPreview
}

/// Whether a rectangle contains a point, half-open, with no area containing nothing.
///
/// Its own function rather than `Rectangle::contains`, because smithay's is inclusive on both
/// edges and this needs the half-open rule the module header gives. A rectangle with no width
/// or no height contains nothing, checked rather than left to the comparisons — an empty
/// panel has a rail of no height, which is the true answer there and would otherwise make
/// every point in a zero-height rail a hit.
fn holds(rectangle: Rectangle<i32, Physical>, at: Point<i32, Physical>) -> bool {
    if rectangle.size.w <= 0 || rectangle.size.h <= 0 {
        return false;
    }
    let right = rectangle.loc.x.saturating_add(rectangle.size.w);
    let bottom = rectangle.loc.y.saturating_add(rectangle.size.h);
    at.x >= rectangle.loc.x && at.x < right && at.y >= rectangle.loc.y && at.y < bottom
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or absent index is the failure \
              being reported — these tests index slots and previews by position because the \
              correspondence between those two lists by position is the thing under test, and \
              a `get` returning None would report it as a pass"
)]
mod tests {
    use super::*;
    use crate::desktop_look::DesktopLook;
    use crate::desktop_testing::{an_appearance, noon_look};
    use crate::panel_raster::picture;
    use alo_canvas::Zoom;
    use alo_dock::{AppId, HowItSits, Patch, Spot, Window};
    use alo_put_aside::the_region_the_panel_claims::WhichEdge;
    use alo_put_aside::whether_it_is_private::Privacy;
    use alo_strings::Direction;

    /// The design's own frame, used as *a* display rather than *the* display.
    const AS_DRAWN: (i32, i32) = (1440, 960);

    fn a_look() -> DesktopLook {
        noon_look(&an_appearance(), Direction::LeftToRight)
    }

    fn a_window(number: u64) -> Window {
        Window::of(
            WindowId::numbered(number),
            Some(AppId::named("Docs").expect("a named application")),
            "A window",
            Patch::of(Spot::at(0, 0), 800, 600).expect("a window has extent"),
            HowItSits::OnTheCanvas,
        )
    }

    /// **Window ids that are not their slot index.** A panel holding windows 1, 2, 3 would let
    /// a hit test that returned the index pass every assertion below, so the ids start at 11.
    fn a_panel_holding(how_many: usize) -> Panel {
        let mut panel = Panel::new();
        for which in 0..how_many {
            panel
                .put_aside(
                    &a_window(u64::try_from(which).unwrap() + 11),
                    Zoom::LIFE_SIZE,
                    alo_canvas::Place::FIRST,
                    Privacy::Ordinary,
                )
                .unwrap();
        }
        panel
    }

    /// A draw captured with the windows it was laid out for, which is what the hit test takes.
    fn captured(panel: &Panel, picture: &PanelPicture) -> ThePanelAsDrawn {
        ThePanelAsDrawn::of(panel, picture.clone())
    }

    fn at(x: i32, y: i32) -> Point<i32, Physical> {
        Point::from((x, y))
    }

    /// The middle of a slot, so a test is not accidentally about an edge.
    fn middle_of(slot: Rectangle<i32, Physical>) -> Point<i32, Physical> {
        at(slot.loc.x + slot.size.w / 2, slot.loc.y + slot.size.h / 2)
    }

    /// **Each slot answers its own window, and the ids are not the indices.**
    ///
    /// The test task 5's hit-test clause exists for. It fails for an implementation that
    /// returns the slot's position, which is the mistake available here and the one a screen
    /// would corroborate: the wrong preview is still a real preview.
    #[test]
    fn every_slot_answers_the_window_that_slot_belongs_to() {
        let panel = a_panel_holding(4);
        let drawn = picture(&panel, a_look(), AS_DRAWN, WhichEdge::Right).unwrap();
        assert_eq!(drawn.slots.len(), 4, "the fixture must draw four slots");

        for (which, slot) in drawn.slots.iter().enumerate() {
            let expected = panel.previews()[which].window();
            assert_eq!(
                which_preview_the_pointer_is_on(
                    &panel,
                    &captured(&panel, &drawn),
                    middle_of(*slot)
                ),
                OnThePanel::APreview(expected),
                "slot {which} answered the wrong window"
            );
            assert_ne!(
                expected,
                WindowId::numbered(u64::try_from(which).unwrap()),
                "the fixture's ids must not equal their indices, or this test proves nothing"
            );
        }
    }

    /// **Inside the panel's region but on no preview is its own answer.**
    ///
    /// The clearance between the rail and the screen edge is the panel's — it is ground a
    /// pointer crosses on its way to a preview — and reporting `Elsewhere` there would
    /// conceal the panel under the pointer that was reaching for it.
    #[test]
    fn the_clearance_beside_the_rail_is_the_panels_but_names_no_window() {
        let panel = a_panel_holding(3);
        let drawn = picture(&panel, a_look(), AS_DRAWN, WhichEdge::Right).unwrap();

        // One pixel inside the reserved column's left edge: inside the region, left of the rail.
        let beside = at(drawn.reserved.loc.x, drawn.reserved.loc.y + 400);
        assert!(
            !drawn.slots.iter().any(|slot| holds(*slot, beside)),
            "the fixture point must not be on a slot, or this test is about something else"
        );

        let whose = which_preview_the_pointer_is_on(&panel, &captured(&panel, &drawn), beside);
        assert_eq!(
            whose,
            OnThePanel::TheRegionButNoPreview,
            "a pointer in the panel's own region is the panel's and names no window"
        );
    }

    /// **The canvas is nobody's preview**, and it does not keep the panel open.
    #[test]
    fn the_middle_of_the_canvas_is_elsewhere() {
        let panel = a_panel_holding(3);
        let drawn = picture(&panel, a_look(), AS_DRAWN, WhichEdge::Right).unwrap();

        assert_eq!(
            which_preview_the_pointer_is_on(&panel, &captured(&panel, &drawn), at(600, 400)),
            OnThePanel::Elsewhere
        );
    }

    /// **The seam between two previews belongs to one of them.**
    ///
    /// Half-open, so the boundary row is the lower slot's and not both. An inclusive test
    /// would make *which preview* have two answers on exactly the pixel a person crosses
    /// while moving between them.
    #[test]
    fn a_pointer_on_the_seam_between_two_slots_is_on_one_of_them() {
        let panel = a_panel_holding(4);
        let drawn = picture(&panel, a_look(), AS_DRAWN, WhichEdge::Right).unwrap();
        let first = drawn.slots[0];

        let bottom_edge = at(first.loc.x + 1, first.loc.y + first.size.h);
        assert!(
            !holds(first, bottom_edge),
            "the first slot must not contain its own bottom edge"
        );

        let last_row_of_it = at(first.loc.x + 1, first.loc.y + first.size.h - 1);
        assert_eq!(
            which_preview_the_pointer_is_on(&panel, &captured(&panel, &drawn), last_row_of_it),
            OnThePanel::APreview(panel.previews()[0].window()),
            "the row above the seam is the first preview's"
        );
    }

    /// **An empty panel still owns its column and names no window.**
    ///
    /// `panel_raster` gives an empty panel a rail of no height, which is the true answer
    /// there. A zero-height rectangle must contain nothing, or every point in it is a hit on
    /// a preview that does not exist.
    #[test]
    fn an_empty_panel_reserves_its_column_and_has_no_previews_to_point_at() {
        let panel = a_panel_holding(0);
        let drawn = picture(&panel, a_look(), AS_DRAWN, WhichEdge::Right).unwrap();
        assert!(drawn.slots.is_empty());
        assert_eq!(drawn.rail.size.h, 0, "an empty panel draws no rail");

        let in_the_column = at(drawn.reserved.loc.x + 10, 400);
        assert_eq!(
            which_preview_the_pointer_is_on(&panel, &captured(&panel, &drawn), in_the_column),
            OnThePanel::TheRegionButNoPreview,
            "an empty panel still owns its column"
        );
    }

    /// **A layout drawn for more windows than the panel now holds names none.**
    ///
    /// Worth keeping even though `zip` alone would also stop short here, because what it
    /// asserts is the behaviour rather than the mechanism: the region is still known, the
    /// window is not.
    #[test]
    fn a_layout_drawn_for_more_windows_than_are_there_refuses_to_name_one() {
        let four = a_panel_holding(4);
        let drawn_from_four = captured(
            &four,
            &picture(&four, a_look(), AS_DRAWN, WhichEdge::Right).unwrap(),
        );
        let now_holding_two = a_panel_holding(2);

        let on_the_third = middle_of(
            picture(&four, a_look(), AS_DRAWN, WhichEdge::Right)
                .unwrap()
                .slots[2],
        );
        assert_eq!(
            which_preview_the_pointer_is_on(&now_holding_two, &drawn_from_four, on_the_third),
            OnThePanel::TheRegionButNoPreview,
            "a stale layout named a window from a list it was not drawn from"
        );
    }

    /// **The same number of windows, in a different order, names none — and this is the test
    /// the length check could not pass.**
    ///
    /// `Panel::bring_back` removes from the middle, which shifts every later preview down one
    /// index, and `put_aside` appends. So bringing one back and putting another aside leaves
    /// the panel the **same length** with different windows at the same indices. A check on
    /// counts sees nothing wrong and hands back the window that now happens to occupy that
    /// slot — a wrong answer the screen corroborates, because the wrong preview is a real one.
    ///
    /// Mutating the identity comparison to `if false` must fail this test. The length check it
    /// replaced passed with the mutation applied, which is how the hole was found.
    #[test]
    fn the_same_count_in_a_different_order_names_no_window() {
        let mut panel = a_panel_holding(3);
        let drawn = captured(
            &panel,
            &picture(&panel, a_look(), AS_DRAWN, WhichEdge::Right).unwrap(),
        );
        let second_slot = middle_of(
            picture(&panel, a_look(), AS_DRAWN, WhichEdge::Right)
                .unwrap()
                .slots[1],
        );
        let was_second = panel.previews()[1].window();

        // Take the first one back, then put a different window aside. Three again.
        panel.bring_back(WindowId::numbered(11)).unwrap();
        panel
            .put_aside(
                &a_window(99),
                Zoom::LIFE_SIZE,
                alo_canvas::Place::FIRST,
                Privacy::Ordinary,
            )
            .unwrap();
        assert_eq!(
            panel.previews().len(),
            3,
            "the counts must match, or this proves nothing"
        );
        assert_ne!(
            panel.previews()[1].window(),
            was_second,
            "the window at that index must have changed, or this proves nothing"
        );

        assert_eq!(
            which_preview_the_pointer_is_on(&panel, &drawn, second_slot),
            OnThePanel::TheRegionButNoPreview,
            "a reordered panel of the same length named the window now in that slot"
        );
    }

    /// **A panel on the left is the same arithmetic**, because nothing here reads an edge.
    #[test]
    fn a_panel_mirrored_to_the_left_needs_no_change_here() {
        let panel = a_panel_holding(3);
        let drawn = picture(&panel, a_look(), AS_DRAWN, WhichEdge::Left).unwrap();

        assert_eq!(
            which_preview_the_pointer_is_on(
                &panel,
                &captured(&panel, &drawn),
                middle_of(drawn.slots[1])
            ),
            OnThePanel::APreview(panel.previews()[1].window())
        );
        assert_eq!(
            which_preview_the_pointer_is_on(&panel, &captured(&panel, &drawn), at(700, 400)),
            OnThePanel::Elsewhere
        );
    }

    /// **A display the design was not drawn on**, since this file holds no figure.
    #[test]
    fn a_screen_that_is_not_the_frame_the_design_was_drawn_on() {
        let panel = a_panel_holding(5);
        let drawn = picture(&panel, a_look(), (1366, 768), WhichEdge::Right).unwrap();

        assert_eq!(drawn.slots.len(), 5);
        assert_eq!(
            which_preview_the_pointer_is_on(
                &panel,
                &captured(&panel, &drawn),
                middle_of(drawn.slots[4])
            ),
            OnThePanel::APreview(panel.previews()[4].window())
        );
    }
}
