//! **An application that draws its own icon has no circle behind it; one that
//! draws none has the circle** — on the bar and in *More apps*, on every edge,
//! at one to one and at twice the pixels.
//!
//! The owner's decision of 2026-11-21: the Dock's application icons stand on
//! the Dock's own ground, as the design draws them, and the circle is only the
//! mark of an application with no icon (`docs/design/the-docks-visual-specification.md`,
//! *The circle is for an application with no icon*).
//!
//! **Asked with a picture that is wholly transparent**, so whatever is under an
//! icon is what the raster shows at its middle: the Dock's ground if nothing
//! was drawn there, the badge's colour if a circle was.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported, and \
              a formatted panic names the case it was on"
)]

use alo_dock::artwork::{APicture, TheArtwork};
use alo_dock::places::WhatIsHere;
use alo_dock::{AppId, Dock, Edge, OnTheDock};
use alo_strings::Direction;

use crate::desktop_testing::{an_appearance, noon_look};
use crate::display_scale::DisplayScale;
use crate::dock_raster::DockPicture;

/// An application the Dock takes.
fn app(name: &str) -> AppId {
    AppId::named(name).expect("a plain identifier is an application")
}

/// A picture with nothing in it: every pixel wholly transparent.
fn a_clear_picture() -> APicture {
    let side = alo_dock::measures::GLYPH;
    APicture::of(side, side, vec![0; side as usize * side as usize * 4]).expect("a picture")
}

/// Two applications that draw an icon and two that draw none, each with a
/// window open.
fn four_applications() -> (Vec<OnTheDock>, TheArtwork) {
    let mut windows = alo_dock::Windows::none();
    let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).expect("a patch");
    for (number, name) in ["icon-one", "letter-one", "icon-two", "letter-two"]
        .into_iter()
        .enumerate()
    {
        windows.opened(alo_dock::Window::of(
            alo_dock::WindowId::numbered(u64::try_from(number).expect("small")),
            Some(app(name)),
            "",
            patch,
            alo_dock::HowItSits::OnTheCanvas,
        ));
    }
    let mut artwork = TheArtwork::none();
    artwork.draws(app("icon-one"), a_clear_picture());
    artwork.draws(app("icon-two"), a_clear_picture());
    (alo_dock::Holding::nothing().showing(&windows), artwork)
}

/// The Dock on `edge` at `scale`, holding `rows`, with `open` open.
fn drawn(
    edge: Edge,
    scale: DisplayScale,
    rows: &[OnTheDock],
    artwork: &TheArtwork,
    open: &crate::WhatTheDockHasOpen,
) -> DockPicture {
    let mut dock = Dock::shipped();
    dock.set_edge(edge);
    let strings = alo_strings::Strings::of(
        alo_saying::everything_this_machine_can_say().expect("the vocabulary"),
    );
    let words = crate::the_docks_words::TheDocksWords::said(&strings, open);
    let per_cent = i32::from(scale.per_cent());
    crate::dock_raster::picture(
        &dock,
        noon_look(&an_appearance(), Direction::LeftToRight),
        (1440 * per_cent / 100, 960 * per_cent / 100),
        rows,
        rows,
        (open, &words),
        None,
        artwork,
        &mut crate::WindowControlLabels::new()
            .expect("the bundled faces")
            .fonts,
        scale,
    )
    .unwrap_or_else(|why| panic!("{edge:?} at {scale:?} would not draw: {why:?}"))
}

/// The colour of the Dock's surface at a point of the display, if the surface
/// covers it.
fn the_surface_at(picture: &DockPicture, x: i32, y: i32) -> Option<[u8; 3]> {
    let surface = picture.surface.as_ref()?;
    let (x, y) = (x - surface.area.loc.x, y - surface.area.loc.y);
    let at = usize::try_from(y * surface.area.size.w + x).ok()? * 4;
    let pixel = surface.pixels.get(at..at + 3)?;
    <[u8; 3]>::try_from(pixel).ok()
}

/// **On the bar**: at the middle of each icon's box, an application with an
/// icon shows the Dock's ground through it; near the top of the box, where
/// no letter reaches, an application without one shows its circle.
#[test]
fn on_the_bar_only_an_application_with_no_icon_has_a_circle() {
    let palette = noon_look(&an_appearance(), Direction::LeftToRight)
        .palette()
        .expect("a palette");
    let (rows, artwork) = four_applications();
    for edge in Edge::EVERY {
        for scale in [DisplayScale::ONE, DisplayScale::of_per_cent(200)] {
            let picture = drawn(
                edge,
                scale,
                &rows,
                &artwork,
                &crate::what_the_dock_has_open::NOTHING,
            );
            let dock = &picture.standing;
            let icon = scale.px(alo_dock::measures::ICON);
            let badge = scale.px(alo_dock::measures::GLYPH);
            let mut asked = 0;
            for place in dock.places.each() {
                let WhatIsHere::Application(it) = place.what() else {
                    continue;
                };
                let (left, top) = crate::dock_raster::a_slot_at(
                    dock.band,
                    dock.layout.edge(),
                    dock.scale.px(place.from_the_start()),
                    icon,
                );
                let inset = (icon - badge) / 2;
                let middle = left + icon / 2;
                let has_an_icon = artwork.of(it).is_some();
                let (y, wanted, what) = if has_an_icon {
                    (top + icon / 2, palette.dock, "the Dock's ground")
                } else {
                    (top + inset + badge / 6, palette.badge, "its circle")
                };
                let seen = the_surface_at(&picture, middle, y)
                    .unwrap_or_else(|| panic!("{edge:?} at {scale:?}: {it:?} is on the surface"));
                assert_eq!(
                    seen,
                    wanted,
                    "{edge:?} at {scale:?}: {it:?} {} an icon and does not show {what}",
                    if has_an_icon { "draws" } else { "draws no" }
                );
                asked += 1;
            }
            assert_eq!(asked, 4, "the premise: four applications on the bar");
        }
    }
}

/// How many pixels of *More apps*' own raster are the badge's colour.
fn badge_coloured(picture: &DockPicture, badge: [u8; 3]) -> usize {
    picture
        .panels
        .iter()
        .flat_map(|panel| panel.pixels.chunks_exact(4))
        .filter(|pixel| pixel.get(..3) == Some(&badge[..]) && pixel.get(3) == Some(&0xFF))
        .count()
}

/// **In *More apps***: giving one application an icon takes its circle away,
/// and only its circle — so each of the four is asked on its own, against the
/// same overview with no icons at all.
#[test]
fn in_more_apps_only_an_application_with_no_icon_has_a_circle() {
    let palette = noon_look(&an_appearance(), Direction::LeftToRight)
        .palette()
        .expect("a palette");
    let (rows, _) = four_applications();
    let open = &crate::what_the_dock_has_open::MORE_APPS;
    for scale in [DisplayScale::ONE, DisplayScale::of_per_cent(200)] {
        let none = badge_coloured(
            &drawn(Edge::Bottom, scale, &rows, &TheArtwork::none(), open),
            palette.badge,
        );
        // A circle's area, short of the real one on purpose — three for pi and
        // two pixels off the radius for its soft edge: what one circle must
        // at least remove.
        let radius = scale.px(alo_dock::measures::GLYPH) / 2 - 2;
        let a_circle = usize::try_from(3 * radius * radius).expect("an area");
        for one in ["icon-one", "letter-one", "icon-two", "letter-two"] {
            let mut artwork = TheArtwork::none();
            artwork.draws(app(one), a_clear_picture());
            let with = badge_coloured(
                &drawn(Edge::Bottom, scale, &rows, &artwork, open),
                palette.badge,
            );
            assert!(
                none >= with + a_circle,
                "at {scale:?}: {one} draws an icon in More apps and its circle is still there \
                 ({none} pixels of the badge's colour without its icon, {with} with it)"
            );
        }
        let mut every = TheArtwork::none();
        for one in ["icon-one", "letter-one", "icon-two", "letter-two"] {
            every.draws(app(one), a_clear_picture());
        }
        let all = badge_coloured(
            &drawn(Edge::Bottom, scale, &rows, &every, open),
            palette.badge,
        );
        assert!(
            none >= all + 4 * a_circle,
            "at {scale:?}: four applications with no icon must have four circles more than four \
             with one ({none} against {all})"
        );
    }
}
