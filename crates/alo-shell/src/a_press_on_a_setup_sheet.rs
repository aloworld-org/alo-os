//! What a press on a first-start sheet is on, read from the sheet that was
//! drawn.
//!
//! # Why not the sign-in screen's layout
//!
//! Until 2026-10-30 every press before anybody had signed in was hit-tested
//! against `crate::sign_in_layout` — the returning screen's centred entry and
//! its submit arrow — whatever was on the display. So on the overview, the
//! language sheet or the PIN sheet, a click on empty ground where the sign-in
//! arrow would have been arrived as Enter, and a click on a row a person could
//! see did nothing at all.
//!
//! # The controls are the ones drawn, in the pixels they were drawn in
//!
//! A sheet's layout depends on how its heading wraps, in a language this code
//! has never seen, and on the scale it was finally drawn at
//! (`crate::a_scale_held_to_what_draws` may step it down). Only the drawing
//! knows either. So each sheet hands back, beside its picture, the rectangles
//! of its rows and actions **in the picture's own pixels**
//! (`crate::setup_sheet_in_pixels`), and a press — which arrives in the
//! output's pixels — is asked against those with no conversion between. At
//! 200 per cent the rows are twice the size and so are their targets, because
//! they are the same rectangles.
//!
//! # A press anywhere else is nothing
//!
//! The stage line, the heading, the footer and the ground answer [`None`], and
//! [`SheetControl::Accessibility`] is recognised and does nothing, as it does
//! on the sign-in screen (`crate::a_press_on_the_sign_in_screen`): the surface
//! it opens does not exist yet.

use smithay::utils::{Physical, Rectangle};

use crate::setup_sheet_in_pixels::SetupSheetInPixels;

/// One control on a setup sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnASheet {
    /// A choice row, **by where it is in the whole list** rather than among
    /// the rows drawn: a short screen draws a window of a long list
    /// (`crate::a_window_onto_a_list`), and only the drawing knows which
    /// window, so the drawing names the row.
    Choice(usize),
    /// The first action: *Continue*, *Use this language*, *Save*.
    WayOn,
    /// The second action: *Back*.
    WayBack,
    /// *Accessibility*, at the bottom left.
    Accessibility,
}

/// A control, and where it was drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SheetControl {
    /// Where, in the picture's pixels.
    pub(crate) area: Rectangle<i32, Physical>,
    /// What it is.
    pub(crate) what: OnASheet,
}

/// **Every control a sheet drew**, from the rectangles it lettered into, each
/// row named by `row` from where it was drawn.
pub(crate) fn the_controls_of(
    at: &SetupSheetInPixels,
    row: impl Fn(usize) -> usize,
) -> Vec<SheetControl> {
    let rows = at
        .choices
        .iter()
        .enumerate()
        .map(|(which, area)| SheetControl {
            area: *area,
            what: OnASheet::Choice(row(which)),
        });
    let actions = at
        .actions
        .iter()
        .zip([OnASheet::WayOn, OnASheet::WayBack])
        .map(|(area, what)| SheetControl { area: *area, what });
    rows.chain(actions)
        .chain(std::iter::once(SheetControl {
            area: at.accessibility,
            what: OnASheet::Accessibility,
        }))
        .collect()
}

/// What the pixel at `x`, `y` is on, or [`None`] for the sheet's ground.
///
/// Half-open, and a rectangle of no area holds nothing: the rule every hit
/// test in this crate keeps, so two controls sharing an edge cannot both
/// answer for it.
#[must_use]
pub(crate) fn what_is_under(controls: &[SheetControl], (x, y): (i32, i32)) -> Option<OnASheet> {
    controls
        .iter()
        .find(|it| {
            let area = it.area;
            area.size.w > 0
                && area.size.h > 0
                && x >= area.loc.x
                && y >= area.loc.y
                && x < area.loc.x.saturating_add(area.size.w)
                && y < area.loc.y.saturating_add(area.size.h)
        })
        .map(|it| it.what)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A control of this size at this place.
    fn a_control(x: i32, y: i32, w: i32, h: i32, what: OnASheet) -> SheetControl {
        SheetControl {
            area: Rectangle::new((x, y).into(), (w, h).into()),
            what,
        }
    }

    /// **Inside a control is that control; its far edges are not.**
    #[test]
    fn a_pixel_inside_a_control_is_that_control_and_its_far_edge_is_not() {
        let controls = [
            a_control(10, 10, 100, 50, OnASheet::Choice(0)),
            a_control(10, 60, 100, 50, OnASheet::Choice(1)),
        ];
        assert_eq!(
            what_is_under(&controls, (10, 10)),
            Some(OnASheet::Choice(0))
        );
        assert_eq!(
            what_is_under(&controls, (109, 59)),
            Some(OnASheet::Choice(0))
        );
        // The shared edge belongs to the second, and only to it.
        assert_eq!(
            what_is_under(&controls, (50, 60)),
            Some(OnASheet::Choice(1))
        );
        assert_eq!(what_is_under(&controls, (110, 20)), None);
    }

    /// **The ground is nothing**, and so is a control of no area.
    #[test]
    fn the_ground_and_a_control_of_no_area_are_nothing() {
        let controls = [a_control(10, 10, 0, 50, OnASheet::WayOn)];
        assert_eq!(what_is_under(&controls, (10, 20)), None);
        assert_eq!(what_is_under(&[], (0, 0)), None);
    }
}
