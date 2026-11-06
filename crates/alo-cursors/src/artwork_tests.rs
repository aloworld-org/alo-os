//! What the table of shapes has to be, rather than what it looks like.

use super::*;

/// **Every name the protocol defines is listed once**, and the only names drawn
/// as the plain arrow are the arrow itself and the four said to be.
#[test]
fn every_name_has_an_answer_of_its_own() {
    let mut names: Vec<&str> = EVERY_SHAPE.iter().map(CursorIcon::name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), EVERY_SHAPE.len(), "a name is listed twice");

    let as_the_arrow: Vec<CursorIcon> = EVERY_SHAPE
        .into_iter()
        .filter(|&icon| drawing_for(icon) == Drawing::Arrow)
        .collect();
    assert_eq!(
        as_the_arrow,
        [
            CursorIcon::Default,
            CursorIcon::ContextMenu,
            CursorIcon::Help,
            CursorIcon::Alias,
            CursorIcon::DndAsk,
        ],
        "a name fell through to the arrow without being decided"
    );
    for icon in as_the_arrow {
        assert_eq!(
            has_its_own_drawing(icon),
            icon == CursorIcon::Default,
            "{} says the wrong thing about its own picture",
            icon.name()
        );
    }
}

/// **What the brief named is drawn as itself**: default, text, link, grab and
/// grabbing, move, every resize direction, not-allowed, wait and progress,
/// crosshair and both zooms.
#[test]
fn what_a_desktop_needs_has_a_picture_that_says_so() {
    for (icon, drawing) in [
        (CursorIcon::Text, Drawing::Text),
        (CursorIcon::Pointer, Drawing::Pointer),
        (CursorIcon::Grab, Drawing::Grab),
        (CursorIcon::Grabbing, Drawing::Grabbing),
        (CursorIcon::Move, Drawing::Move),
        (CursorIcon::NotAllowed, Drawing::NotAllowed),
        (CursorIcon::Wait, Drawing::Wait),
        (CursorIcon::Progress, Drawing::Progress),
        (CursorIcon::Crosshair, Drawing::Crosshair),
        (CursorIcon::ZoomIn, Drawing::ZoomIn),
        (CursorIcon::ZoomOut, Drawing::ZoomOut),
    ] {
        assert_eq!(drawing_for(icon), drawing, "{}", icon.name());
    }
}

/// **Opposite directions share one double arrow, and the four axes are four.**
///
/// Asserted pair by pair rather than by counting, because four drawings paired
/// wrongly would pass a count and point a person the wrong way.
#[test]
fn the_resize_names_point_the_way_they_say() {
    for (names, drawing) in [
        (
            [CursorIcon::EResize, CursorIcon::WResize, CursorIcon::EwResize, CursorIcon::ColResize],
            Drawing::Across,
        ),
        (
            [CursorIcon::NResize, CursorIcon::SResize, CursorIcon::NsResize, CursorIcon::RowResize],
            Drawing::Down,
        ),
    ] {
        for icon in names {
            assert_eq!(drawing_for(icon), drawing, "{}", icon.name());
        }
    }
    for (names, drawing) in [
        (
            [CursorIcon::NwResize, CursorIcon::SeResize, CursorIcon::NwseResize],
            Drawing::Falling,
        ),
        (
            [CursorIcon::NeResize, CursorIcon::SwResize, CursorIcon::NeswResize],
            Drawing::Rising,
        ),
    ] {
        for icon in names {
            assert_eq!(drawing_for(icon), drawing, "{}", icon.name());
        }
    }
}

/// **The owner's choice is what ships**, and each arrow's hotspot is the first
/// point of its own outline, so the tip clicks.
#[test]
fn the_kite_ships_and_each_arrow_clicks_at_its_tip() {
    assert_eq!(Silhouette::default(), Silhouette::Kite);
    for silhouette in Silhouette::ALL {
        let first: Vec<f32> = silhouette
            .path()
            .trim_start_matches('M')
            .split_whitespace()
            .take(2)
            .map(|number| number.parse().unwrap_or(f32::NAN))
            .collect();
        let (x, y) = silhouette.tip();
        assert_eq!(first, [x, y], "{silhouette:?}'s tip is not where its outline starts");
        let art = artwork(Drawing::Arrow, silhouette);
        assert_eq!(art.hotspot, silhouette.tip());
        assert_eq!(art.parts.first().map(|part| part.d.as_str()), Some(silhouette.path()));
    }
    assert_ne!(Silhouette::Kite.path(), Silhouette::Classic.path());
}

/// **The arrows that point two ways click in their middle** (ADR 0071).
#[test]
fn a_double_arrow_clicks_in_its_middle() {
    for drawing in [Drawing::Across, Drawing::Down, Drawing::Falling, Drawing::Rising, Drawing::Move] {
        assert_eq!(artwork(drawing, Silhouette::Kite).hotspot, (16.0, 16.0), "{drawing:?}");
    }
}
