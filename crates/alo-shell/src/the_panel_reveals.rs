//! Whether the panel is on the screen, and what a pointer does to that.
//!
//! Task 6 of `docs/autonomy/putting-a-window-aside.md`, the clause its status named: **the
//! reveal machine had no caller.** `alo_dock::Revealing` decides whether a surface is revealed
//! and had **zero callers** in this crate — measured — so nothing watched the panel's edge and
//! the panel was simply always drawn.
//!
//! # No new figure, and that is the whole of why this file is short
//!
//! The design says activation strips exist — the owner's ruling of 2026-09-30 says *activation
//! strips and pointer paths must follow those bounds too* — and **nowhere gives one a width**.
//! So a width here would be invented, which the standing rule forbids: a figure becomes a
//! proportion or a named rule, never a constant.
//!
//! It does not need one. `crate::panel_raster` already produces two rectangles, and the
//! difference between them *is* the strip:
//!
//! - the **rail** is what is drawn — the panel itself;
//! - the **reserved column** is what the panel owns, running flush to the screen edge.
//!
//! So the ground inside the reserved column and outside the rail is exactly what a person
//! crosses on the way to the panel, which `docs/design/the-regions-a-pointer-can-be-in.md`
//! describes as the reason the region runs past the surface at all: *if the region stopped
//! where the surface stops, the strip between the surface and the edge would belong to nothing,
//! and a pointer coming from the edge would cross dead ground on its way to the thing it is
//! reaching for.*
//!
//! **A concealed panel is therefore all strip**, and that is right rather than a special case:
//! an empty or collapsed panel draws a rail of no height, so the whole reserved column asks.
//! The edge is what a person reaches for when there is nothing yet to reach.
//!
//! # One opinion about the panel's edge
//!
//! `alo_put_aside::the_region_the_panel_claims` asks for *three places rather than a
//! coordinate, because then two files would have opinions about the same edge*. This file is
//! the only thing in this crate that turns a position into `alo_dock::ThePointer`, and it asks
//! the panel's own geometry and nothing else — so it has no opinion about the Dock's edge or
//! the top controls', and cannot disagree with whatever eventually arbitrates between them.
//!
//! **When that arbiter exists it goes in front of this, not inside it.** The handover on
//! `handover/dev-pc/the-pointer-classifier` holds one, and it cannot land yet: it arbitrates
//! between the panel, the Dock and the top controls, and the top controls do not exist in this
//! crate and are promised in `docs/features.md` at no tier. When they do, this file's caller
//! asks the arbiter first and passes `Elsewhere` when the answer is not the panel's. One line,
//! and nothing here changes.

use alo_dock::revealing::ThePointer;
use smithay::utils::{Physical, Point, Rectangle};

use crate::which_preview_the_pointer_is_on::ThePanelAsDrawn;

/// What this pointer is to the panel's reveal machine.
///
/// **Asked of the draw, because the draw is what a person is looking at.** The rail and the
/// reserved column both come from `crate::panel_raster`, which is the one place that converts
/// the design's measures into this display's pixels — so this answers about the panel as it
/// actually appears rather than about a second calculation that would have to agree.
///
/// The order is the correctness: **the rail is asked before the strip**, because they overlap
/// by construction — the rail is drawn inside the column — and *on the surface* is the more
/// specific answer. The other order would report a pointer resting on a preview as merely
/// asking, and the machine would then let the panel conceal underneath it.
pub(crate) fn what_the_pointer_is(drawn: &ThePanelAsDrawn, at: Point<i32, Physical>) -> ThePointer {
    if holds(drawn.rail(), at) {
        return ThePointer::OnTheSurface;
    }
    if holds(drawn.reserved(), at) {
        return ThePointer::AtTheEdge;
    }
    ThePointer::Elsewhere
}

/// Whether a rectangle contains a point, half-open, with no area containing nothing.
///
/// The same rule `crate::which_preview_the_pointer_is_on` applies, and for the same reason: two
/// rectangles that share an edge must not both contain the points along it. A rail of no height
/// contains nothing, which is what makes a concealed panel all strip rather than a panel whose
/// every point is on its surface.
fn holds(rectangle: Rectangle<i32, Physical>, at: Point<i32, Physical>) -> bool {
    if rectangle.size.w <= 0 || rectangle.size.h <= 0 {
        return false;
    }
    let right = rectangle.loc.x.saturating_add(rectangle.size.w);
    let bottom = rectangle.loc.y.saturating_add(rectangle.size.h);
    at.x >= rectangle.loc.x && at.x < right && at.y >= rectangle.loc.y && at.y < bottom
}

impl crate::Server {
    /// What the pointer is to the panel's reveal machine, if there is an answer.
    ///
    /// `None` before the first draw or before the seat has a pointer — two absences with one
    /// answer, because a caller does the same thing about either, which is nothing.
    ///
    /// **The same position the peek road reads**, from the same stored draw, so the two cannot
    /// disagree about where the pointer is. They answer different questions about it:
    /// `crate::peeking_at_a_put_aside_window` asks *which preview*, this asks *is the panel's
    /// edge or surface being touched*.
    pub(crate) fn what_the_pointer_is_to_the_panel(&self) -> Option<ThePointer> {
        let drawn = self.panel_as_drawn.as_ref()?;
        let at = self.surfaces.pointer.as_ref()?.location;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a pointer inside a pixel belongs to that pixel, the half-open rule the \
                      rectangles use — the same conversion the peek road makes"
        )]
        let at = Point::from((at.x as i32, at.y as i32));
        Some(what_the_pointer_is(drawn, at))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported — \
              the words `crate::panel_raster`'s own test module uses"
)]
mod tests {
    use super::*;
    use crate::desktop_look::DesktopLook;
    use crate::desktop_testing::{an_appearance, noon_look};
    use crate::panel_raster::{WhetherRevealed, picture};
    use alo_canvas::{Place, Zoom};
    use alo_dock::window::{AppId, HowItSits, Window, WindowId};
    use alo_dock::{Patch, Spot};
    use alo_put_aside::Panel;
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
            Some(AppId::named("Docs").expect("a fixture names its application")),
            "A window",
            Patch::of(Spot::at(0, 0), 800, 600).expect("a fixture gives its patch an extent"),
            HowItSits::OnTheCanvas,
        )
    }

    fn a_panel_holding(how_many: u64) -> Panel {
        let mut panel = Panel::new();
        for which in 0..how_many {
            panel
                .put_aside(
                    &a_window(which + 11),
                    Zoom::LIFE_SIZE,
                    Place::FIRST,
                    Privacy::Ordinary,
                )
                .unwrap();
        }
        panel
    }

    fn drawn(how_many: u64, size: (i32, i32)) -> ThePanelAsDrawn {
        let panel = a_panel_holding(how_many);
        let picture = picture(
            &panel,
            a_look(),
            size,
            WhichEdge::Right,
            WhetherRevealed::Revealed,
        )
        .unwrap();
        ThePanelAsDrawn::of(&panel, picture)
    }

    fn at(x: i32, y: i32) -> Point<i32, Physical> {
        Point::from((x, y))
    }

    /// **The rail is the surface**, so a pointer on a preview is on the panel rather than
    /// asking for it.
    #[test]
    fn a_pointer_on_the_rail_is_on_the_surface() {
        let panel = drawn(4, AS_DRAWN);
        let rail = panel.rail();
        let middle = at(rail.loc.x + rail.size.w / 2, rail.loc.y + rail.size.h / 2);

        assert_eq!(
            what_the_pointer_is(&panel, middle),
            ThePointer::OnTheSurface
        );
    }

    /// **The ground between the rail and the screen edge is the strip**, which is the clause
    /// this file exists for and the reason no width had to be invented.
    #[test]
    fn the_ground_between_the_rail_and_the_edge_asks_for_the_panel() {
        let panel = drawn(4, AS_DRAWN);
        let rail = panel.rail();
        let reserved = panel.reserved();

        // One pixel inside the reserved column's left edge, which is left of the rail.
        let approaching = at(reserved.loc.x, rail.loc.y + 10);
        assert!(
            !holds(rail, approaching),
            "the fixture point must be off the rail, or this test is about something else"
        );
        assert_eq!(
            what_the_pointer_is(&panel, approaching),
            ThePointer::AtTheEdge
        );
    }

    /// **The canvas is neither.**
    #[test]
    fn the_middle_of_the_canvas_reveals_nothing() {
        let panel = drawn(4, AS_DRAWN);
        assert_eq!(
            what_the_pointer_is(&panel, at(600, 400)),
            ThePointer::Elsewhere
        );
    }

    /// **A concealed panel is all strip, and that is the point.**
    ///
    /// An empty panel draws a rail of no height, so there is nothing to be *on* — and the whole
    /// reserved column asks, because the edge is what a person reaches for when there is
    /// nothing yet to reach. A rail of no height containing points would make every one of them
    /// `OnTheSurface` for a surface that is not drawn.
    #[test]
    fn an_empty_panel_is_all_strip_rather_than_all_surface() {
        let panel = drawn(0, AS_DRAWN);
        assert_eq!(panel.rail().size.h, 0, "an empty panel draws no rail");

        let reserved = panel.reserved();
        let in_the_column = at(reserved.loc.x + 10, 400);
        assert_eq!(
            what_the_pointer_is(&panel, in_the_column),
            ThePointer::AtTheEdge,
            "a concealed panel reported its own absent rail as a surface"
        );
    }

    /// **The rail is asked before the strip**, and the order is the correctness.
    ///
    /// They overlap by construction: the rail is drawn inside the reserved column. Asking the
    /// strip first would report a pointer resting on a preview as merely asking, and
    /// `Revealing` would then let the panel conceal underneath it.
    #[test]
    fn a_point_in_both_is_the_surface_and_not_the_strip() {
        let panel = drawn(4, AS_DRAWN);
        let rail = panel.rail();
        let middle = at(rail.loc.x + rail.size.w / 2, rail.loc.y + rail.size.h / 2);

        assert!(
            holds(panel.reserved(), middle),
            "the rail must be inside the reserved column, or this test proves nothing"
        );
        assert_eq!(
            what_the_pointer_is(&panel, middle),
            ThePointer::OnTheSurface
        );
    }

    /// **A display the design was not drawn on**, since this file holds no figure.
    #[test]
    fn a_screen_that_is_not_the_frame_the_design_was_drawn_on() {
        let panel = drawn(5, (1366, 768));
        let rail = panel.rail();
        let middle = at(rail.loc.x + rail.size.w / 2, rail.loc.y + rail.size.h / 2);

        assert_eq!(
            what_the_pointer_is(&panel, middle),
            ThePointer::OnTheSurface
        );
        assert_eq!(
            what_the_pointer_is(&panel, at(400, 400)),
            ThePointer::Elsewhere
        );
    }

    /// **The machine it feeds does what this file is for**, asserted end to end: at the edge
    /// reveals, on the surface holds, leaving both conceals.
    #[test]
    fn the_machine_reveals_at_the_edge_and_conceals_on_leaving() {
        use alo_dock::revealing::Revealing;
        let panel = drawn(4, AS_DRAWN);
        let rail = panel.rail();
        let reserved = panel.reserved();

        let mut machine = Revealing::covered();
        assert!(!machine.is_revealed(), "a fresh panel is not showing");

        machine = machine.the_pointer_is(what_the_pointer_is(
            &panel,
            at(reserved.loc.x, rail.loc.y + 10),
        ));
        assert!(machine.is_revealed(), "the edge did not reveal the panel");

        machine = machine.the_pointer_is(what_the_pointer_is(
            &panel,
            at(rail.loc.x + rail.size.w / 2, rail.loc.y + rail.size.h / 2),
        ));
        assert!(machine.is_revealed(), "moving onto the panel concealed it");

        machine = machine.the_pointer_is(what_the_pointer_is(&panel, at(600, 400)));
        assert!(
            !machine.is_revealed(),
            "leaving the panel did not conceal it"
        );
    }
}
