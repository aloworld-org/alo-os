//! The dock on one display: the band along the edge `alo-dock` names, and the
//! status area at its far end.
//!
//! # Whose decisions these are
//!
//! Which edge, how thick, how far it runs and which end is the far one are all
//! `alo_dock::Dock::layout_on`'s answers for this display's own size, the
//! person's text size and the way they read. This file turns those answers into
//! pixels and decides none of them. Asked once per display, so a laptop and the
//! screen beside it each get the dock laid out for their own size — and a dock
//! that gives its names way on the small one keeps them on the large one.
//!
//! # The status area
//!
//! A segment at the far end of the band, set off by a rule in ink. The egress
//! indicator's lines grow from its corner (`crate::egress_status_place`) out of
//! the same layout, so the status area and the indicator cannot disagree about
//! where the far end is.
//!
//! # Furniture, not authority
//!
//! Nothing here grants, approves or revokes, and nothing here measures: the
//! dock is a band of the person's colours with the accent along its inside
//! edge. `tests/desktop_source.rs` reads these files to hold that.

use alo_dock::{Along, Dock, Edge, End, Layout, Screen};
use smithay::utils::{Physical, Rectangle};

use crate::RenderError;
use crate::desktop_look::DesktopLook;
use crate::painted::Solid;

/// The largest display side, in pixels, the dock is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// The dock for one display, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DockPicture {
    /// The display size it was laid out for.
    pub(crate) size: (i32, i32),
    /// `alo-dock`'s layout for this display.
    pub(crate) layout: Layout,
    /// The whole band.
    pub(crate) band: Rectangle<i32, Physical>,
    /// The status area, at the band's far end.
    pub(crate) status_area: Rectangle<i32, Physical>,
    /// The accent along the band's inside edge.
    pub(crate) accent: Rectangle<i32, Physical>,
    /// Flat shapes, in painting order.
    pub(crate) solids: Vec<Solid>,
}

/// Lay `dock` out on a display of `size` and rasterise it.
///
/// # Errors
/// [`RenderError::DesktopScene`] for a display `alo-dock` refuses to lay a dock
/// out on, or one larger than any this dock is laid out for;
/// [`RenderError::AccentRefused`] for a look whose accent is not one a person
/// can choose.
pub(crate) fn picture(
    dock: &Dock,
    look: DesktopLook,
    size: (i32, i32),
) -> Result<DockPicture, RenderError> {
    let (width, height) = size;
    if width > LARGEST_SIDE || height > LARGEST_SIDE {
        return Err(RenderError::DesktopScene);
    }
    let screen = Screen::of(
        u32::try_from(width).map_err(|_| RenderError::DesktopScene)?,
        u32::try_from(height).map_err(|_| RenderError::DesktopScene)?,
    )
    .map_err(|_| RenderError::DesktopScene)?;
    let palette = look.palette().map_err(|_| RenderError::AccentRefused)?;
    let measure = look.measure();
    let layout = dock.layout_on(screen, look.scale(), look.reading());
    let thickness = i32::try_from(layout.thickness().as_pixels())
        .map_err(|_| RenderError::DesktopScene)?
        .clamp(1, width.min(height));
    let rule = measure.px(2).min(thickness);

    let band = match layout.edge() {
        Edge::Bottom => Rectangle::new((0, height - thickness).into(), (width, thickness).into()),
        Edge::Top => Rectangle::new((0, 0).into(), (width, thickness).into()),
        Edge::Left => Rectangle::new((0, 0).into(), (thickness, height).into()),
        Edge::Right => Rectangle::new((width - thickness, 0).into(), (thickness, height).into()),
    };
    let accent = match layout.edge() {
        Edge::Bottom => Rectangle::new(band.loc, (width, rule).into()),
        Edge::Top => Rectangle::new((0, thickness - rule).into(), (width, rule).into()),
        Edge::Left => Rectangle::new((thickness - rule, 0).into(), (rule, height).into()),
        Edge::Right => Rectangle::new(band.loc, (rule, height).into()),
    };
    let length = match layout.along() {
        Along::Across => width,
        Along::Down => height,
    };
    let segment = (2 * thickness).min(length);
    let (status_area, divider) = match (layout.along(), layout.status().at()) {
        (Along::Across, End::Left) => (
            Rectangle::new(band.loc, (segment, thickness).into()),
            Rectangle::new(
                (segment - rule, band.loc.y).into(),
                (rule, thickness).into(),
            ),
        ),
        (Along::Across, _) => (
            Rectangle::new(
                (width - segment, band.loc.y).into(),
                (segment, thickness).into(),
            ),
            Rectangle::new(
                (width - segment, band.loc.y).into(),
                (rule, thickness).into(),
            ),
        ),
        (Along::Down, End::Top) => (
            Rectangle::new(band.loc, (thickness, segment).into()),
            Rectangle::new(
                (band.loc.x, segment - rule).into(),
                (thickness, rule).into(),
            ),
        ),
        (Along::Down, _) => (
            Rectangle::new(
                (band.loc.x, height - segment).into(),
                (thickness, segment).into(),
            ),
            Rectangle::new(
                (band.loc.x, height - segment).into(),
                (thickness, rule).into(),
            ),
        ),
    };

    let solids = vec![
        Solid {
            area: band,
            colour: palette.dock,
        },
        Solid {
            area: divider,
            colour: palette.ink,
        },
        Solid {
            area: accent,
            colour: palette.accent,
        },
    ];
    Ok(DockPicture {
        size,
        layout,
        band,
        status_area,
        accent,
        solids,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::desktop_testing::{an_appearance, noon_look};
    use alo_appearance::{Accent, TextScale};
    use alo_strings::Direction;

    /// A dock on `edge`.
    fn on(edge: Edge) -> Dock {
        let mut dock = Dock::shipped();
        dock.set_edge(edge);
        dock
    }

    /// Whether `inner` lies wholly inside `outer`.
    fn inside(inner: Rectangle<i32, Physical>, outer: Rectangle<i32, Physical>) -> bool {
        outer.intersection(inner) == Some(inner)
    }

    /// **The dock is drawn on the edge `alo-dock` names**, as thick as it says
    /// and spanning the edge, on every edge — with the status area inside the
    /// band at the end `alo-dock` calls far, in both reading directions.
    #[test]
    fn a_dock_is_drawn_on_the_edge_alo_dock_names_with_the_status_area_at_its_far_end() {
        let size = (1920, 1080);
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            let look = noon_look(&an_appearance(), reading);
            for edge in Edge::ALL {
                let dock = on(edge);
                let drawn = picture(&dock, look, size).unwrap();
                let layout = dock.layout_on(Screen::of(1920, 1080).unwrap(), look.scale(), reading);
                assert_eq!(drawn.layout, layout);
                let thick = i32::try_from(layout.thickness().as_pixels()).unwrap();
                let expected = match edge {
                    Edge::Bottom => Rectangle::new((0, 1080 - thick).into(), (1920, thick).into()),
                    Edge::Top => Rectangle::new((0, 0).into(), (1920, thick).into()),
                    Edge::Left => Rectangle::new((0, 0).into(), (thick, 1080).into()),
                    Edge::Right => Rectangle::new((1920 - thick, 0).into(), (thick, 1080).into()),
                };
                assert_eq!(drawn.band, expected, "{edge:?} {reading:?}");
                assert!(inside(drawn.status_area, drawn.band), "{edge:?}");
                assert!(inside(drawn.accent, drawn.band), "{edge:?}");

                let area = drawn.status_area;
                match layout.status().at() {
                    End::Right => assert_eq!(area.loc.x + area.size.w, 1920),
                    End::Left => assert_eq!(area.loc.x, 0),
                    End::Bottom => assert_eq!(area.loc.y + area.size.h, 1080),
                    End::Top => assert_eq!(area.loc.y, 0),
                }
                if edge.along() == Along::Across {
                    let far = if reading == Direction::LeftToRight {
                        End::Right
                    } else {
                        End::Left
                    };
                    assert_eq!(layout.status().at(), far);
                } else {
                    assert_eq!(layout.status().at(), End::Bottom);
                }
            }
        }
    }

    /// **Per display.** The same dock on a laptop and on a large screen beside
    /// it is laid out for each display's own size: at a large text size its
    /// names give way on the laptop and are kept on the desk, and the two bands
    /// are as thick as `alo-dock` says for each.
    #[test]
    fn each_display_gets_the_dock_laid_out_for_its_own_size() {
        let mut appearance = an_appearance();
        appearance.set_text(TextScale::percent(300).unwrap());
        let look = noon_look(&appearance, Direction::LeftToRight);
        let dock = on(Edge::Left);

        let laptop = picture(&dock, look, (1366, 768)).unwrap();
        let desk = picture(&dock, look, (3840, 2160)).unwrap();
        let portrait = picture(&dock, look, (1080, 1920)).unwrap();
        assert!(!laptop.layout.labels().are_shown());
        assert!(desk.layout.labels().are_shown());
        assert_ne!(laptop.band.size.w, desk.band.size.w);
        for drawn in [&laptop, &desk, &portrait] {
            assert_eq!(
                i64::from(drawn.band.size.w),
                i64::from(drawn.layout.thickness().as_pixels())
            );
            assert_eq!(drawn.band.size.h, drawn.size.1);
        }
    }

    /// **The accent is `alo-appearance`'s**: the rule along the band's inside
    /// edge is exactly the accent the person chose, for the scheme on screen,
    /// and the band is never terracotta.
    #[test]
    fn the_dock_carries_the_persons_accent() {
        for accent in Accent::ALL {
            let mut appearance = an_appearance();
            appearance.set_accent(accent);
            let look = noon_look(&appearance, Direction::LeftToRight);
            let drawn = picture(&Dock::shipped(), look, (1920, 1080)).unwrap();
            let rule = drawn
                .solids
                .iter()
                .find(|solid| solid.area == drawn.accent)
                .unwrap();
            assert_eq!(
                rule.colour,
                crate::desktop_look::rgb(accent.on(look.scheme()))
            );
        }
    }

    /// **A display `alo-dock` refuses is refused**, and so is one larger than any
    /// this dock is laid out for, rather than a dock drawn over somebody's whole
    /// screen.
    #[test]
    fn a_display_too_small_or_too_large_for_a_dock_is_refused() {
        let look = noon_look(&an_appearance(), Direction::LeftToRight);
        for size in [(0, 0), (100, 100), (-5, 800), (20_000, 1080)] {
            assert!(
                matches!(
                    picture(&Dock::shipped(), look, size),
                    Err(RenderError::DesktopScene)
                ),
                "{size:?}"
            );
        }
    }
}
