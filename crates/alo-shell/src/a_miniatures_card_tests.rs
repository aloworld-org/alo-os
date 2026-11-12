//! The square an application's window put aside shows: a card with a hairline
//! for its miniature and the application's badge in the lower corner — and a
//! private window's blank card with the icon and no badge.

use alo_canvas::{Place, Zoom};
use alo_dock::artwork::{APicture, TheArtwork};
use alo_dock::{AppId, HowItSits, Patch, Spot, Window, WindowId};
use alo_put_aside::whether_it_is_private::Privacy;
use alo_put_aside::{Panel, SafeName};
use alo_strings::Direction;
use smithay::utils::{Physical, Rectangle};

use super::a_windows_square;
use crate::desktop_look::DesktopPalette;
use crate::desktop_testing::{an_appearance, noon_look};
use crate::painted::Inked;

fn fonts() -> cosmic_text::FontSystem {
    let mut fonts = cosmic_text::FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(alo_drawing::fonts::MANROPE.to_vec());
    fonts
}

fn palette() -> DesktopPalette {
    noon_look(&an_appearance(), Direction::LeftToRight)
        .palette()
        .unwrap()
}

fn a_preview(privacy: Privacy) -> alo_put_aside::Preview {
    let mut panel = Panel::new();
    let window = Window::of(
        WindowId::numbered(3),
        Some(AppId::named("org.kde.dolphin").unwrap()),
        "Home",
        Patch::of(Spot::at(0, 0), 800, 600).unwrap(),
        HowItSits::OnTheCanvas,
    );
    panel
        .put_aside(&window, Zoom::LIFE_SIZE, Place::FIRST, privacy)
        .unwrap();
    panel.previews().first().unwrap().clone()
}

fn private() -> Privacy {
    Privacy::Private(SafeName::chosen("Bank").expect("a safe name"))
}

/// The design's own 48 slot.
fn slot() -> Rectangle<i32, Physical> {
    Rectangle::new((0, 0).into(), (48, 48).into())
}

fn pixel(tile: &Inked, x: i32, y: i32) -> [u8; 3] {
    *tile
        .pixels
        .get(usize::try_from(y * tile.area.size.w + x).unwrap())
        .unwrap()
}

/// **An ordinary window: a card with its hairline, the window's ground inside,
/// and a badge in the lower corner carrying the application's letter.**
#[test]
fn an_ordinary_window_is_a_card_with_a_badge() {
    let palette = palette();
    let tile = a_windows_square(
        slot(),
        &a_preview(Privacy::Ordinary),
        &TheArtwork::none(),
        &palette,
        &mut fonts(),
    );
    assert_eq!((tile.area.size.w, tile.area.size.h), (48, 48));
    // Above the card (y < 8): the rail.
    assert_eq!(pixel(&tile, 24, 3), palette.dock);
    // The card's hairline on its top edge, its ground just inside.
    assert_eq!(pixel(&tile, 24, 8), palette.edge, "the hairline");
    assert_eq!(pixel(&tile, 20, 20), palette.ground, "the card's ground");
    // The badge (30..48): its fallback ground, and the letter on it.
    assert_eq!(pixel(&tile, 33, 33), palette.badge);
    // A 9-pixel letter is mostly edges, so its ink is counted as anything
    // inside the badge's flat middle that is not the badge's ground.
    let letter = (34..44)
        .flat_map(|y| (34..44).map(move |x| (x, y)))
        .filter(|&(x, y)| pixel(&tile, x, y) != palette.badge)
        .count();
    assert!(letter > 4, "the badge carries the letter: {letter}");
}

/// **An application with its own icon is badged with it**, inside the badge's
/// white border.
#[test]
fn an_application_with_an_icon_is_badged_with_it() {
    let palette = palette();
    let red = [200_u8, 30, 40];
    let mut artwork = TheArtwork::none();
    artwork.draws(
        AppId::named("org.kde.dolphin").unwrap(),
        APicture::of(2, 2, [red[0], red[1], red[2], 255].repeat(4)).unwrap(),
    );
    let tile = a_windows_square(
        slot(),
        &a_preview(Privacy::Ordinary),
        &artwork,
        &palette,
        &mut fonts(),
    );
    assert_eq!(pixel(&tile, 39, 39), red, "the icon in the badge");
    assert_eq!(pixel(&tile, 20, 20), palette.ground, "not in the card");
    assert!(!tile.pixels.contains(&palette.badge_ink), "no letter");
}

/// **The refusal path: a private window shows the application's mark on a
/// blank card, and no badge** — there is no picture for one to sit over.
#[test]
fn a_private_window_is_a_blank_card_with_its_mark_and_no_badge() {
    let palette = palette();
    let red = [200_u8, 30, 40];
    let mut artwork = TheArtwork::none();
    artwork.draws(
        AppId::named("org.kde.dolphin").unwrap(),
        APicture::of(2, 2, [red[0], red[1], red[2], 255].repeat(4)).unwrap(),
    );
    let tile = a_windows_square(
        slot(),
        &a_preview(private()),
        &artwork,
        &palette,
        &mut fonts(),
    );
    assert_eq!(
        pixel(&tile, 24, 24),
        red,
        "the icon in the middle of the card"
    );
    // Where the badge would be, below the card: the rail, not a badge.
    assert_eq!(pixel(&tile, 44, 44), palette.dock, "no badge");
    assert_eq!(pixel(&tile, 24, 8), palette.edge, "the same card");
}
