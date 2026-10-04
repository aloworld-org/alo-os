//! The panel of put-aside windows, laid out for one display and ready to paint.
//!
//! `alo-put-aside` decides what is in the panel and holds **no geometry at
//! all** — deliberately, so that the conversion from a design's figures to a
//! display's pixels happens once, in the crate that knows the display. This is
//! that crate. Every number below is derived here and none is stored there.
//!
//! # Nothing here is copied from the design; it is derived, and it agrees
//!
//! The standing rule is that a figure taken from a frame becomes a proportion
//! or a named rule, never a constant. The panel's frames were measured on a
//! 1440×960 design frame, and the striking thing is that **every figure in them
//! already falls out of `alo_dock::measures`**:
//!
//! | drawn | derivation | at the design's sizes |
//! |---|---|---|
//! | rail 64 wide | `ICON + 2 × MARGIN` | 48 + 16 = 64 |
//! | rail 232 tall, 3 put aside | `(n + 1) × (ICON + GAP) + MARGIN` | 224 + 8 = 232 |
//! | rail 288 tall, 4 put aside | the same | 280 + 8 = 288 |
//! | rail 344 tall, 5 put aside | the same | 336 + 8 = 344 |
//! | rail 400 tall, 6 put aside | the same | 392 + 8 = 400 |
//! | 24 from the screen's edge | `3 × MARGIN` | 24 |
//! | reserved column 112 wide | rail + `2 × 3 × MARGIN` | 64 + 48 = 112 |
//!
//! Four different rail heights, each exact, from one formula. That is a strong
//! enough agreement to treat as evidence that the design was drawn from these
//! measures rather than a coincidence fitted afterwards — and it means this file
//! follows the design *by deriving it*, which is what the rule asks for and what
//! a copied `64` would only appear to do.
//!
//! **One figure is not explained: a rail 296 tall appears eight times.** The
//! formula gives 288 for five and 344 for six. It is recorded here unexplained
//! rather than rounded into the nearest bucket, because a table that quietly
//! absorbs the one value that does not fit stops being a measurement.
//!
//! # Why the reserved column is not the rail
//!
//! The rail is what is drawn. The **reserved column** is what the panel owns
//! for the purposes of deciding which surface a pointer belongs to, and the
//! owner's ruling of 2026-09-30 makes it the full height of the display: *the
//! right panel owns that area, including the top-right corner; the top controls
//! and the bottom Dock stop before it.* So the column runs edge to edge
//! vertically while the rail occupies a part of it, and the two are separate
//! fields rather than one rectangle serving both jobs.
//!
//! That separation is the same one the owner drew for the Dock — a 32 glyph
//! inside a 44 target — and the same one `alo_dock::places` does not yet have,
//! where a place is *always* `ICON` across and the drawn thing and the pressed
//! thing are one number.
//!
//! # The edge is data
//!
//! Which edge the panel belongs to is passed in, never assumed, and nothing in
//! this file names one. `alo_put_aside::the_region_the_panel_claims::WhichEdge`
//! says why: one frame in the design mirrors the panel to the opposite edge,
//! the owner has ruled that a stray unless its screen is marked right-to-left,
//! and **the cost of taking the edge as data is nothing if it is a stray and
//! the difference between a rename and a rewrite if it is not.**
//!
//! # Furniture, not authority
//!
//! Nothing here grants, approves or revokes, and nothing here decides what the
//! panel holds. It is given a panel and a display and answers with rectangles.

use alo_dock::measures::{GAP, ICON, MARGIN};
use alo_put_aside::Panel;
use alo_put_aside::the_region_the_panel_claims::WhichEdge;
use smithay::utils::{Physical, Point, Rectangle, Size};

use crate::RenderError;
use crate::desktop_look::DesktopLook;
use crate::painted::Solid;

/// The largest display side, in pixels, the panel is laid out for. The same
/// ceiling `crate::dock_raster` holds, and for the same reason: beyond it the
/// arithmetic below stops being worth trusting.
const LARGEST_SIDE: i32 = 16_384;

/// Whether the panel is on the screen at all.
///
/// # Not `alo_dock::Showing`, and the distinction is the reason this type exists
///
/// That enum means *shown* or *it has given way*, and giving way is the Dock's reason: a
/// window needs its room. The panel's reason is different — **nobody has reached for it** —
/// and `alo_dock::Revealing` is the machine that decides it, from the edge, the surface, the
/// keyboard, a drag and an open menu.
///
/// Reusing one enum for two reasons would be the two-vocabularies fault this repository has
/// already paid for once: the word would be right in both places and mean a different thing in
/// each, and the first person to write a rule about *hidden* would write it about both.
///
/// # The answer, not the machine
///
/// `Revealing` is not taken here. The caller asks `Revealing::is_revealed` and passes what it
/// answered, which is the seam this file takes for everything else — the zoom, the Place, the
/// look. A raster that read a state machine would be a drawing deciding whether it should
/// happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhetherRevealed {
    /// Somebody has reached for it, or is on it, or holds it open.
    Revealed,
    /// Nobody has. It keeps its column and draws no rail.
    Concealed,
}

/// The panel for one display, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PanelPicture {
    /// The display size it was laid out for.
    pub(crate) size: (i32, i32),
    /// The rail itself — what is drawn.
    pub(crate) rail: Rectangle<i32, Physical>,
    /// The column the panel owns for deciding which surface a pointer is in.
    ///
    /// The full height of the display, by the owner's ruling. Wider than the
    /// rail, and not the same thing as it.
    pub(crate) reserved: Rectangle<i32, Physical>,
    /// Where the expansion control sits: the rail's **first** slot.
    ///
    /// The design draws it above every window, so a rail holding three windows
    /// is four slots tall. `None` exactly when there is no rail.
    ///
    /// **Reserved, and not yet clickable.** What it opens is the expanded
    /// presentation, which does not exist yet, and a control that opened
    /// nothing would be worse than one that is not there.
    /// `crate::a_click_brings_a_window_back` claims presses on [`Self::slots`]
    /// and deliberately does not claim this one.
    pub(crate) control: Option<Rectangle<i32, Physical>>,
    /// One slot per put-aside window, top to bottom, inside the rail, **below
    /// the expansion control**.
    pub(crate) slots: Vec<Rectangle<i32, Physical>>,
    /// Flat shapes, in painting order.
    pub(crate) solids: Vec<Solid>,
}

/// Lay `panel` out on a display of `size` and rasterise it.
///
/// An empty panel has a rail of no height and no slots. **That is the true
/// answer rather than a placeholder** — a person who has put nothing aside has
/// no rail, and the reserved column still exists because the panel still owns
/// its edge.
///
/// # Errors
/// [`RenderError::DesktopScene`] for a display larger than this lays out for,
/// or one too narrow to hold the reserved column;
/// [`RenderError::AccentRefused`] for a look whose accent is not one a person
/// can choose.
pub(crate) fn picture(
    panel: &Panel,
    look: DesktopLook,
    size: (i32, i32),
    edge: WhichEdge,
    revealed: WhetherRevealed,
) -> Result<PanelPicture, RenderError> {
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE || width <= 0 || height <= 0 {
        return Err(RenderError::DesktopScene);
    }
    let palette = look.palette().map_err(|_| RenderError::AccentRefused)?;
    let measure = look.measure();

    let icon = px(measure, ICON)?;
    let margin = px(measure, MARGIN)?;
    let gap = px(measure, GAP)?;
    let clearance = margin.saturating_mul(3);

    let rail_width = icon.saturating_add(margin.saturating_mul(2));
    let reserved_width = rail_width.saturating_add(clearance.saturating_mul(2));
    if reserved_width >= width {
        return Err(RenderError::DesktopScene);
    }

    // Which side the column sits on is the only place the edge is read, and it
    // is read as a position rather than as a name.
    let reserved_x = match edge {
        WhichEdge::Right => width.saturating_sub(reserved_width),
        WhichEdge::Left => 0,
    };
    let reserved = Rectangle::new(
        Point::from((reserved_x, 0)),
        Size::from((reserved_width, height)),
    );

    let holding = i32::try_from(panel.holding()).map_err(|_| RenderError::DesktopScene)?;
    // **A concealed panel draws no rail and keeps its column.** The same picture an empty
    // panel gives, and for a reason worth stating: in both states there is nothing to be *on*,
    // and the column still asks, because the panel owns its edge whether or not it is showing.
    // `crate::the_panel_reveals` relies on exactly this — a rail of no height contains nothing,
    // so every point in the column is `AtTheEdge` rather than `OnTheSurface`.
    let rail_height = if holding == 0 || revealed == WhetherRevealed::Concealed {
        0
    } else {
        // **One slot per window, plus one for the expansion control**, which the
        // design puts above them all. The owner's ruling of 2026-10-04:
        // *rail height = 64 + 56 × minimized-window count*, where 64 is
        // `MARGIN + ICON + GAP` — the control's slot and the rail's padding.
        holding
            .saturating_add(1)
            .saturating_mul(icon.saturating_add(gap))
            .saturating_add(margin)
    };

    // The rail keeps the clearance from the edge it belongs to, and sits a
    // tenth of the display's height down from the top — which is where the
    // design puts it, expressed as a share of the screen rather than as the
    // 96 pixels that share happens to be on the frame it was drawn on.
    let rail_x = match edge {
        WhichEdge::Right => width.saturating_sub(clearance).saturating_sub(rail_width),
        WhichEdge::Left => clearance,
    };
    let rail_y = height / 10;
    let rail_height = rail_height.min((height - rail_y).max(0));
    let rail = Rectangle::new(
        Point::from((rail_x, rail_y)),
        Size::from((rail_width, rail_height)),
    );

    let mut slots = Vec::new();
    let mut solids = Vec::new();
    let mut control = None;
    if rail_height > 0 {
        solids.push(Solid {
            area: rail,
            colour: palette.dock,
        });
        // The expansion control holds the first slot, at the rail's own padding.
        let its_own = Rectangle::new(
            Point::from((rail_x.saturating_add(margin), rail_y.saturating_add(margin))),
            Size::from((icon, icon)),
        );
        control = Some(its_own);
        solids.push(Solid {
            area: its_own,
            colour: palette.accent,
        });
        for which in 0..holding {
            // `which + 1`, because the control is the slot above them all.
            let top = rail_y.saturating_add(margin).saturating_add(
                which
                    .saturating_add(1)
                    .saturating_mul(icon.saturating_add(gap)),
            );
            if top.saturating_add(icon) > rail_y.saturating_add(rail_height) {
                break;
            }
            let slot = Rectangle::new(
                Point::from((rail_x.saturating_add(margin), top)),
                Size::from((icon, icon)),
            );
            slots.push(slot);
            solids.push(Solid {
                area: slot,
                colour: palette.accent,
            });
        }
    }

    Ok(PanelPicture {
        size,
        rail,
        reserved,
        control,
        slots,
        solids,
    })
}

/// One of `alo-dock`'s logical measures in this display's pixels.
///
/// **This is the whole of how the standing rule is kept here.** A logical
/// measure is what the design is drawn in; what a display shows is that measure
/// through the person's text size and this screen's scale. Nothing below this
/// line is a pixel count somebody wrote down.
fn px(measure: crate::desktop_look::Measure, logical: u32) -> Result<i32, RenderError> {
    let logical = i32::try_from(logical).map_err(|_| RenderError::DesktopScene)?;
    Ok(measure.px(logical))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::desktop_testing::{an_appearance, noon_look};
    use alo_canvas::Zoom;
    use alo_dock::{AppId, HowItSits, Patch, Spot, Window, WindowId};
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

    fn a_panel_holding(how_many: usize) -> Panel {
        let mut panel = Panel::new();
        for which in 0..how_many {
            panel
                .put_aside(
                    &a_window(u64::try_from(which).unwrap() + 1),
                    Zoom::LIFE_SIZE,
                    alo_canvas::Place::FIRST,
                    Privacy::Ordinary,
                )
                .unwrap();
        }
        panel
    }

    fn laid_out(how_many: usize, size: (i32, i32)) -> PanelPicture {
        picture(
            &a_panel_holding(how_many),
            a_look(),
            size,
            WhichEdge::Right,
            WhetherRevealed::Revealed,
        )
        .unwrap()
    }

    /// **The rail's width is the design's 64, and it was derived.** `ICON`
    /// plus a margin each side, not the number read off a frame.
    #[test]
    fn the_rails_width_is_derived_and_agrees_with_the_design() {
        let picture = laid_out(4, AS_DRAWN);
        assert_eq!(picture.rail.size.w, 64);
    }

    /// **Four rail heights, one formula, each exact** — and the formula counts
    /// the expansion control.
    ///
    /// *This test asserted `3 → 176` until 2026-10-04, and each of its four
    /// numbers was genuinely in the design file.* What it had wrong was what they
    /// are heights **of**: the design puts an `Expand minimized windows` control
    /// in the rail's first slot, so a 232-tall rail holds **three** windows and
    /// the control, not four windows. The numbers were copied from the design, so
    /// of course they matched; nobody asked what they counted.
    ///
    /// The owner ruled on 2026-10-04: *rail height = 64 + 56 × minimized-window
    /// count*, which is `(n + 1) × (ICON + GAP) + MARGIN` — and the canonical
    /// `Fixed icon rail` in the design file is 64 x 288 with a 48 x 48 control at
    /// (8, 8) and four windows at y 64, 120, 176, 232.
    #[test]
    fn every_rail_height_the_design_draws_falls_out_of_the_measures() {
        for (put_aside, drawn) in [(3, 232), (4, 288), (5, 344), (6, 400)] {
            let picture = laid_out(put_aside, AS_DRAWN);
            assert_eq!(
                picture.rail.size.h, drawn,
                "{put_aside} put aside should be {drawn} tall"
            );
        }
    }

    /// **The reserved column is the design's 112 and is not the rail.** It is
    /// wider, and it runs the whole height of the display because the owner
    /// ruled that the panel owns the corner.
    #[test]
    fn the_reserved_column_is_wider_than_the_rail_and_runs_the_whole_height() {
        let picture = laid_out(4, AS_DRAWN);
        assert_eq!(picture.reserved.size.w, 112);
        assert_eq!(picture.reserved.size.h, 960);
        assert_eq!(picture.reserved.loc.y, 0);
        assert!(picture.reserved.size.w > picture.rail.size.w);
    }

    /// **The rail sits inside the column it reserves**, with the clearance the
    /// design keeps from the screen's edge.
    #[test]
    fn the_rail_sits_inside_the_reserved_column() {
        let picture = laid_out(4, AS_DRAWN);
        assert_eq!(picture.reserved.loc.x, 1328);
        assert_eq!(picture.rail.loc.x, 1352);
        assert_eq!(
            picture.rail.loc.x + picture.rail.size.w,
            1416,
            "the rail keeps its clearance from the screen's edge"
        );
        assert!(picture.rail.loc.x >= picture.reserved.loc.x);
        assert!(
            picture.rail.loc.x + picture.rail.size.w
                <= picture.reserved.loc.x + picture.reserved.size.w
        );
    }

    /// **A person who has put nothing aside has no rail**, and still owns the
    /// edge. The true answer rather than a placeholder.
    #[test]
    fn an_empty_panel_draws_no_rail_and_still_reserves_its_column() {
        let picture = laid_out(0, AS_DRAWN);
        assert_eq!(picture.rail.size.h, 0);
        assert!(picture.slots.is_empty());
        assert!(picture.solids.is_empty());
        assert_eq!(picture.reserved.size.w, 112);
        assert_eq!(picture.reserved.size.h, 960);
    }

    /// **One slot per window, in order, each an icon square — and each big
    /// enough to press.**
    ///
    /// The owner asked on 2026-09-30 for the target floors to be held against
    /// the Dock, the status area and this panel. A preview is a thing somebody
    /// clicks to bring a window back, so it is a control, and a control is
    /// built to `ENHANCED_TARGET` rather than merely clearing
    /// `SMALLEST_TARGET`. Asserted against the rectangles this file actually
    /// produces rather than against `ICON`, because a constant compared with
    /// itself is folded away before it can fail.
    /// **The expansion control holds the rail's first slot**, at the coordinates
    /// the design draws it at rather than at coordinates that merely agree.
    ///
    /// `Minimized panel / 07 · Collapsed rail` draws `Fixed icon rail` at
    /// (1352, 96), 64 x 288, with `Expand minimized windows / 48px target` at
    /// (8, 8) inside it and four windows at y 64, 120, 176 and 232. On a
    /// 1440 x 960 display that is a control at (1360, 104) and a first window at
    /// (1360, 160) — which is what this asserts, absolutely, because a relative
    /// check would pass on a rail in the wrong place.
    #[test]
    fn the_expansion_control_holds_the_first_slot_where_the_design_draws_it() {
        let picture = laid_out(4, AS_DRAWN);
        let control = picture.control.expect("a rail with windows has a control");

        assert_eq!((control.loc.x, control.loc.y), (1360, 104));
        assert_eq!((control.size.w, control.size.h), (48, 48));

        let first = picture.slots.first().expect("four windows give four slots");
        assert_eq!((first.loc.x, first.loc.y), (1360, 160));

        // One pitch between them, and the same pitch between the windows.
        assert_eq!(first.loc.y - control.loc.y, 56);
        assert_eq!(
            (control.loc.y - picture.rail.loc.y),
            8,
            "the rail's own padding"
        );
        assert_eq!(picture.rail.size.h, 288, "64 + 56 x 4");
    }

    /// **No rail, no control** — `None` rather than a rectangle of no size.
    ///
    /// A zero-sized rectangle at the rail's corner would hit-test as a point and
    /// read as a control that is there; `None` cannot be clicked by accident.
    #[test]
    fn a_panel_with_no_rail_has_no_expansion_control() {
        assert!(laid_out(0, AS_DRAWN).control.is_none());
    }

    #[test]
    fn there_is_one_slot_for_each_window_put_aside() {
        let picture = laid_out(4, AS_DRAWN);
        assert_eq!(picture.slots.len(), 4);
        for slot in &picture.slots {
            assert_eq!(slot.size.w, 48);
            assert_eq!(slot.size.h, 48);
            let floor = i32::try_from(alo_appearance::targets::ENHANCED_TARGET).unwrap();
            assert!(
                slot.size.w >= floor && slot.size.h >= floor,
                "a preview is {}x{} and a control is built to {floor}",
                slot.size.w,
                slot.size.h
            );
        }
        for pair in picture.slots.windows(2) {
            let (above, below) = (pair.first().unwrap(), pair.last().unwrap());
            assert!(
                below.loc.y > above.loc.y,
                "the slots are not in order down the rail"
            );
        }
    }

    /// **A display that is not the one the design was drawn on.** Nothing here
    /// is a constant, so a small panel and a wide desk screen both lay out —
    /// and the reserved column stays the same width, because it is derived
    /// from the measures rather than from the screen's width.
    #[test]
    fn it_lays_out_on_a_display_the_design_was_not_drawn_on() {
        let small = laid_out(4, (1366, 768));
        assert_eq!(
            small.reserved.size.h, 768,
            "the column runs the full height"
        );
        assert_eq!(small.reserved.loc.x, 1366 - 112);
        assert_eq!(small.rail.size.w, 64);

        let wide = laid_out(4, (3840, 2160));
        assert_eq!(wide.reserved.size.h, 2160);
        assert_eq!(wide.reserved.loc.x, 3840 - 112);
    }

    /// **Mirrored to the other edge, with nothing in this file naming one.**
    #[test]
    fn a_panel_on_the_left_is_the_same_layout_mirrored() {
        let right = laid_out(4, AS_DRAWN);
        let left = picture(
            &a_panel_holding(4),
            a_look(),
            AS_DRAWN,
            WhichEdge::Left,
            WhetherRevealed::Revealed,
        )
        .unwrap();

        assert_eq!(left.reserved.loc.x, 0);
        assert_eq!(left.reserved.size, right.reserved.size);
        assert_eq!(left.rail.size, right.rail.size);
        assert_eq!(left.rail.loc.x, 24, "the same clearance from its own edge");
        assert_eq!(left.rail.loc.y, right.rail.loc.y);
    }

    /// **A display too narrow for the column is refused, not squeezed.**
    #[test]
    fn a_display_too_narrow_for_the_column_is_refused() {
        let panel = a_panel_holding(4);
        assert!(
            picture(
                &panel,
                a_look(),
                (80, 600),
                WhichEdge::Right,
                WhetherRevealed::Revealed
            )
            .is_err()
        );
        assert!(
            picture(
                &panel,
                a_look(),
                (0, 600),
                WhichEdge::Right,
                WhetherRevealed::Revealed
            )
            .is_err()
        );
        assert!(
            picture(
                &panel,
                a_look(),
                (1440, 0),
                WhichEdge::Right,
                WhetherRevealed::Revealed
            )
            .is_err()
        );
    }

    /// **A rail longer than the display is cut at the display**, and the slots
    /// stop with it rather than being drawn off the bottom. Somebody who puts
    /// forty windows aside on a small screen gets a full rail, not a crash and
    /// not a row of invisible slots.
    #[test]
    fn a_rail_longer_than_the_screen_stops_at_the_screen() {
        let picture = laid_out(40, (1366, 768));
        let bottom = picture.rail.loc.y + picture.rail.size.h;
        assert!(bottom <= 768, "the rail ran off the bottom of the display");
        for slot in &picture.slots {
            assert!(
                slot.loc.y + slot.size.h <= bottom,
                "a slot was drawn past the end of the rail"
            );
        }
        assert!(
            picture.slots.len() < 40,
            "forty slots cannot fit and should not all have been made"
        );
    }

    /// **No figure from the design reaches the code.** Held by reading the
    /// source above the tests: every number here is a name from
    /// `alo_dock::measures` or arithmetic on one, and the agreements with the
    /// design are asserted in the tests rather than written into the layout.
    #[test]
    fn no_figure_from_the_design_reaches_the_code() {
        let source = include_str!("panel_raster.rs");
        let above = source
            .split_once("#[cfg(test)]")
            .map_or(source, |(above, _)| above);
        let code: String = above
            .lines()
            .filter(|line| {
                let line = line.trim_start();
                !line.starts_with("//!") && !line.starts_with("///") && !line.starts_with("//")
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            code.contains("pub(crate) fn picture"),
            "the filter took the code away with the prose"
        );
        for forbidden in ["64", "112", "176", "232", "288", "344", "1440", "960", "96"] {
            assert!(
                !code.contains(forbidden),
                "a figure from the design reached the code: {forbidden}"
            );
        }
    }
    /// **A concealed panel draws no rail and keeps its column**, which is the whole of what
    /// the reveal machine buys a person: a panel that is not in the way until it is reached
    /// for.
    ///
    /// Asserted against a panel that **does** hold windows, because a concealed panel and an
    /// empty one give the same picture and only a full one can tell them apart. A test on an
    /// empty panel would pass whether or not this file consulted the reveal state at all.
    #[test]
    fn a_concealed_panel_holding_windows_still_draws_no_rail() {
        let panel = a_panel_holding(4);
        let shown = picture(
            &panel,
            a_look(),
            AS_DRAWN,
            WhichEdge::Right,
            WhetherRevealed::Revealed,
        )
        .unwrap();
        let hidden = picture(
            &panel,
            a_look(),
            AS_DRAWN,
            WhichEdge::Right,
            WhetherRevealed::Concealed,
        )
        .unwrap();

        assert!(
            shown.rail.size.h > 0,
            "a revealed panel with four windows draws a rail"
        );
        assert_eq!(
            hidden.rail.size.h, 0,
            "a concealed panel drew its rail, so the reveal machine buys nothing"
        );
        assert!(
            hidden.slots.is_empty(),
            "a concealed panel laid out previews nobody can see"
        );
        assert_eq!(
            hidden.reserved, shown.reserved,
            "the panel gave up its column when it concealed — it owns its edge either way"
        );
    }

    /// **And nothing is painted for a concealed panel**, not merely laid out small.
    ///
    /// `solids` is what reaches the screen. A panel that produced its rail as a shape and
    /// relied on a later stage to skip it would be a concealed panel that is one forgotten
    /// branch away from being visible.
    #[test]
    fn a_concealed_panel_paints_nothing_at_all() {
        let panel = a_panel_holding(4);
        let hidden = picture(
            &panel,
            a_look(),
            AS_DRAWN,
            WhichEdge::Right,
            WhetherRevealed::Concealed,
        )
        .unwrap();

        assert!(
            hidden.solids.is_empty(),
            "a concealed panel handed shapes to the painter"
        );
    }
}
