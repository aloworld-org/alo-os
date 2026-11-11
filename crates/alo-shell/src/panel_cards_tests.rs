//! The cards in the collapsed rail, held to the design they were measured from
//! and to the walk of 2026-11-11 that found them blank.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_canvas::Zoom;
use alo_dock::artwork::{APicture, TheArtwork};
use alo_dock::{AppId, HowItSits, Patch, Spot, Window, WindowId};
use alo_put_aside::Panel;
use alo_put_aside::the_region_the_panel_claims::WhichEdge;
use alo_put_aside::whether_it_is_private::Privacy;
use alo_strings::Direction;

use super::drawn;
use crate::desktop_look::{DesktopLook, DesktopPalette};
use crate::desktop_testing::{an_appearance, noon_look};
use crate::display_scale::DisplayScale;
use crate::painted::Inked;
use crate::panel_raster::{WhetherRevealed, picture};

/// The design's own frame.
const AS_DRAWN: (i32, i32) = (1440, 960);

/// The bundled face, as the shell loads it.
fn fonts() -> cosmic_text::FontSystem {
    let mut fonts = cosmic_text::FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
    fonts
}

/// The look the panel is drawn in at noon.
fn a_look() -> DesktopLook {
    noon_look(&an_appearance(), Direction::LeftToRight)
}

/// That look's colours.
fn the_palette() -> DesktopPalette {
    a_look().palette().unwrap()
}

/// A panel holding one window of each application named, in order.
fn holding(apps: &[&str]) -> Panel {
    let mut panel = Panel::new();
    for (which, app) in apps.iter().enumerate() {
        let window = Window::of(
            WindowId::numbered(u64::try_from(which).unwrap() + 1),
            Some(AppId::named(app).expect("a named application")),
            "Home",
            Patch::of(Spot::at(0, 0), 800, 600).expect("a window has extent"),
            HowItSits::OnTheCanvas,
        );
        panel
            .put_aside(
                &window,
                Zoom::LIFE_SIZE,
                alo_canvas::Place::FIRST,
                Privacy::Ordinary,
            )
            .unwrap();
    }
    panel
}

/// The tiles for `panel` on the design's frame at `edge`.
fn tiles(panel: &Panel, edge: WhichEdge, artwork: &TheArtwork) -> Vec<Inked> {
    let laid_out = picture(
        panel,
        a_look(),
        AS_DRAWN,
        edge,
        WhetherRevealed::Revealed,
        DisplayScale::ONE,
    )
    .unwrap();
    drawn(
        panel,
        &laid_out,
        edge,
        artwork,
        &the_palette(),
        &mut fonts(),
    )
}

/// The pixel at (`x`, `y`) of a tile.
fn pixel(tile: &Inked, x: i32, y: i32) -> [u8; 3] {
    *tile
        .pixels
        .get(usize::try_from(y * tile.area.size.w + x).unwrap())
        .unwrap()
}

/// **The walk's fault: one window put aside drew two identical accent
/// squares.** Now the control and the card are two different things, and
/// neither is the accent.
#[test]
fn one_window_put_aside_is_one_card_under_a_control_that_is_not_a_card() {
    let palette = the_palette();
    let drawn = tiles(
        &holding(&["org.kde.dolphin"]),
        WhichEdge::Right,
        &TheArtwork::none(),
    );
    assert_eq!(drawn.len(), 2, "the control and one card");
    let (control, card) = (drawn.first().unwrap(), drawn.get(1).unwrap());
    for tile in [control, card] {
        assert!(
            !tile.pixels.contains(&palette.accent),
            "an accent square is what read as a window"
        );
    }
    assert!(
        !control.pixels.contains(&palette.badge),
        "the control has no hit area: the design draws it unfilled"
    );
    assert!(
        card.pixels.contains(&palette.badge),
        "a card has its hit area"
    );
    assert_ne!(control.pixels, card.pixels);
}

/// **The hit area is the design's 42 inside the 48, with rounded corners**:
/// the slot's corner is the rail, its middle the hit area.
#[test]
fn the_hit_area_is_inset_and_rounded_as_measured() {
    let palette = the_palette();
    let drawn = tiles(
        &holding(&["org.kde.dolphin"]),
        WhichEdge::Right,
        &TheArtwork::none(),
    );
    let card = drawn.get(1).unwrap();
    assert_eq!(card.area.size.w, 48);
    // Inside the 3-pixel inset: the rail.
    assert_eq!(pixel(card, 1, 24), palette.dock);
    // Just inside the hit area's straight edge: the hit area.
    assert_eq!(pixel(card, 4, 24), palette.badge);
    // At the hit area's corner, which is rounded away: the rail.
    assert_eq!(pixel(card, 3, 3), palette.dock);
}

/// **An application with no artwork is marked by its letter**, in the badge's
/// ink, near the middle of the card.
#[test]
fn an_application_with_no_icon_is_marked_by_its_letter() {
    let palette = the_palette();
    let drawn = tiles(
        &holding(&["org.kde.dolphin"]),
        WhichEdge::Right,
        &TheArtwork::none(),
    );
    let card = drawn.get(1).unwrap();
    let inked = (14..34)
        .flat_map(|y| (14..34).map(move |x| (x, y)))
        .filter(|&(x, y)| pixel(card, x, y) == palette.badge_ink)
        .count();
    assert!(inked > 3, "a letter is drawn in the middle: {inked} pixels");
}

/// **An application that draws its own icon is marked by it**, at the Dock's
/// artwork size and centred — and no letter is put across it.
#[test]
fn an_application_with_an_icon_is_marked_by_it() {
    let palette = the_palette();
    let red = [200_u8, 30, 40];
    let mut artwork = TheArtwork::none();
    artwork.draws(
        AppId::named("org.kde.dolphin").unwrap(),
        APicture::of(2, 2, [red[0], red[1], red[2], 255].repeat(4)).unwrap(),
    );
    let drawn = tiles(&holding(&["org.kde.dolphin"]), WhichEdge::Right, &artwork);
    let card = drawn.get(1).unwrap();
    assert_eq!(pixel(card, 24, 24), red, "the icon is in the middle");
    // `GLYPH` is 28, centred in 48: columns 10 to 37.
    assert_eq!(pixel(card, 10, 24), red);
    assert_eq!(pixel(card, 37, 24), red);
    assert_eq!(pixel(card, 8, 24), palette.badge, "and no wider");
    assert!(
        !card.pixels.contains(&palette.badge_ink),
        "no letter over it"
    );
}

/// **Two windows are two cards that can be told apart**: two applications'
/// marks, not two of the same square.
#[test]
fn two_windows_of_two_applications_are_two_different_cards() {
    let drawn = tiles(
        &holding(&["Docs", "Mail"]),
        WhichEdge::Right,
        &TheArtwork::none(),
    );
    assert_eq!(drawn.len(), 3);
    assert_ne!(drawn.get(1).unwrap().pixels, drawn.get(2).unwrap().pixels);
}

/// **The chevron points inward**: left of the middle on a right-hand panel,
/// right of it on a left-hand one.
#[test]
fn the_control_points_away_from_its_edge() {
    let palette = the_palette();
    let tip_of = |edge| {
        let drawn = tiles(&holding(&["org.kde.dolphin"]), edge, &TheArtwork::none());
        let control = drawn.first().unwrap().clone();
        let row = control.area.size.h / 2;
        (0..control.area.size.w)
            .filter(|&x| pixel(&control, x, row) == palette.ink)
            .collect::<Vec<i32>>()
    };
    let right = tip_of(WhichEdge::Right);
    let left = tip_of(WhichEdge::Left);
    assert!(
        !right.is_empty() && !left.is_empty(),
        "the chevron is drawn"
    );
    assert!(right.iter().all(|&x| x < 24), "{right:?}");
    assert!(left.iter().all(|&x| x >= 24), "{left:?}");
}

/// **Nothing when nothing is put aside**: no rail, no control, no card.
#[test]
fn an_empty_panel_draws_no_card() {
    assert!(tiles(&Panel::new(), WhichEdge::Right, &TheArtwork::none()).is_empty());
}
