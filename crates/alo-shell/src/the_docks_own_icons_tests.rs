//! That Settings and Applications are drawn on the Dock as the owner's two
//! line drawings and not as letters: each in its own slot, in the palette's
//! ink, in light and dark, at one to one and at twice the pixels, on the bar
//! and in the list *Show all* opens.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported, and \
              a formatted panic names the case it was on"
)]

use alo_appearance::{Following, Scheme};
use alo_dock::places::WhatIsHere;
use alo_dock::{ADockEntry, ANativeAction, Dock, Edge, OnTheDock};
use alo_strings::Direction;
use smithay::utils::{Physical, Rectangle};

use super::ADocksOwnIcon;
use crate::desktop_testing::{an_appearance, noon_look};
use crate::display_scale::DisplayScale;
use crate::dock_raster::DockPicture;

/// The record of where the two icons came from.
const WHERE_THEY_CAME_FROM: &str = include_str!("../icons/the-dock/where-they-came-from.toml");

/// The cog as the owner approved it, byte for byte.
const THE_APPROVED_COG: &str = include_str!("../icons/the-dock/as-approved/gear-1-classic.svg");

/// The squares as the owner approved them, byte for byte.
const THE_APPROVED_SQUARES: &str =
    include_str!("../icons/the-dock/as-approved/applications-a-line.svg");

/// Both native entries, with the icon each is drawn with.
const BOTH: [(ANativeAction, ADocksOwnIcon); 2] = [
    (ANativeAction::Settings, ADocksOwnIcon::Settings),
    (ANativeAction::Applications, ADocksOwnIcon::Applications),
];

/// The faces the shell draws with, and only those.
fn fonts() -> cosmic_text::FontSystem {
    crate::WindowControlLabels::new()
        .expect("the bundled faces")
        .fonts
}

/// A Dock holding Settings and Applications and nothing else.
fn the_two_entries() -> Vec<OnTheDock> {
    let mut holding = alo_dock::Holding::nothing();
    holding.pin_entry(ADockEntry::Native(ANativeAction::Settings));
    holding.pin_entry(ADockEntry::Native(ANativeAction::Applications));
    holding.showing(&alo_dock::Windows::none())
}

/// The look for `scheme`, always.
fn a_look_in(scheme: Scheme) -> crate::desktop_look::DesktopLook {
    let mut always = an_appearance();
    always.follow(Following::from(scheme));
    let look = noon_look(&always, Direction::LeftToRight);
    assert_eq!(
        look.scheme(),
        scheme,
        "the premise: this look is {scheme:?}"
    );
    look
}

/// The bottom Dock holding the two entries, in `scheme` at `scale`, with
/// `open` open.
fn drawn(scheme: Scheme, scale: DisplayScale, open: &crate::WhatTheDockHasOpen) -> DockPicture {
    let mut dock = Dock::shipped();
    dock.set_edge(Edge::Bottom);
    let strings = alo_strings::Strings::of(
        alo_saying::everything_this_machine_can_say().expect("the vocabulary"),
    );
    let words = crate::the_docks_words::TheDocksWords::said(&strings, open);
    let per_cent = i32::from(scale.per_cent());
    let entries = the_two_entries();
    crate::dock_raster::picture(
        &dock,
        a_look_in(scheme),
        (1440 * per_cent / 100, 960 * per_cent / 100),
        &entries,
        &entries,
        (open, &words),
        None,
        &alo_dock::artwork::NOTHING,
        &mut fonts(),
        scale,
    )
    .unwrap_or_else(|why| panic!("{scheme:?} at {scale:?} would not draw: {why:?}"))
}

/// `action`'s slot on the bar, in output pixels.
fn its_slot(picture: &DockPicture, action: ANativeAction) -> Rectangle<i32, Physical> {
    let dock = &picture.standing;
    let place = dock
        .places
        .each()
        .iter()
        .find(|place| matches!(place.what(), WhatIsHere::ANativeAction(it) if *it == action))
        .unwrap_or_else(|| panic!("{action:?} is on this Dock"));
    let icon = dock.scale.px(alo_dock::measures::ICON);
    let along = dock.scale.px(place.from_the_start());
    let (left, top) = crate::dock_raster::a_slot_at(dock.band, dock.layout.edge(), along, icon);
    Rectangle::new(
        (left, top).into(),
        (dock.scale.px(place.across()), icon).into(),
    )
}

/// One box of ink, where it is and its pixels.
type ABoxOfInk<'a> = (Rectangle<i32, Physical>, &'a [[u8; 3]]);

/// Every box of ink on `picture` that touches `slot`, with its pixels.
fn ink_on(picture: &DockPicture, slot: Rectangle<i32, Physical>) -> Vec<ABoxOfInk<'_>> {
    picture
        .inked
        .iter()
        .map(|inked| {
            (
                crate::painted::on_the_frame(inked.area),
                inked.pixels.as_slice(),
            )
        })
        .filter(|(area, _)| area.overlaps(slot))
        .collect()
}

/// **Each file reads whole, is recorded, and is the approved geometry**: the
/// cog's outline and the plus are the approved paths character for character,
/// and both files are in the design's light `text/primary`.
#[test]
fn each_icon_reads_whole_is_recorded_and_is_what_the_owner_approved() {
    let record: toml::Table = WHERE_THEY_CAME_FROM
        .parse()
        .unwrap_or_else(|why| panic!("the record reads: {why}"));
    let icons = record
        .get("icon")
        .and_then(toml::Value::as_array)
        .expect("the record lists the icons");
    assert_eq!(icons.len(), ADocksOwnIcon::BOTH.len());
    let primary = alo_appearance::Role::TextPrimary.colour();
    for (icon, file) in [
        (ADocksOwnIcon::Settings, "settings.svg"),
        (ADocksOwnIcon::Applications, "applications.svg"),
    ] {
        let shape = icon
            .shape()
            .unwrap_or_else(|| panic!("{icon:?} does not read"));
        assert_eq!(shape.its_box(), (28.0, 28.0), "{file}");
        assert!(
            shape
                .colours()
                .iter()
                .all(|one| *one == [primary.red(), primary.green(), primary.blue()]),
            "{file} is not in text/primary"
        );
        assert!(
            icons
                .iter()
                .any(|it| it.get("svg").and_then(toml::Value::as_str) == Some(file)),
            "{file} is drawn and not recorded"
        );
    }
    // The approved paths, read out of the approved files and found in the
    // drawn ones unchanged.
    let first_path = |svg: &'static str, after: &str| -> &'static str {
        let from = svg.find(after).expect("the approved file has the path") + after.len();
        let to = svg[from..].find('"').expect("the path ends") + from;
        &svg[from..to]
    };
    let cog = first_path(THE_APPROVED_COG, "<path d=\"");
    assert!(
        ADocksOwnIcon::Settings.svg().contains(cog),
        "the cog is not the approved cog"
    );
    let plus = first_path(THE_APPROVED_SQUARES, "<path d=\"");
    assert!(
        ADocksOwnIcon::Applications.svg().contains(plus),
        "the plus is not the approved plus"
    );
}

/// **Every native entry has its own icon, and the two are different
/// drawings.**
#[test]
fn each_entry_has_its_own_icon() {
    for (action, icon) in BOTH {
        assert_eq!(ADocksOwnIcon::of(action), icon);
    }
    let (ground, ink) = ([255, 255, 255], [0x10, 0x2a, 0x43]);
    assert_ne!(
        ADocksOwnIcon::Settings.drawn(28, ground, ink),
        ADocksOwnIcon::Applications.drawn(28, ground, ink)
    );
}

/// **The drawing is ink on its strokes and ground between them, and nothing
/// else**: every pixel lies between the two colours, the corners are bare
/// ground, the strokes reach the ink, and twice the side is about four times
/// the ink. A box with no area draws nothing.
#[test]
fn the_drawing_is_ink_on_ground_and_grows_with_its_square() {
    let (ground, ink) = ([247, 249, 250], [0x10, 0x2a, 0x43]);
    for icon in ADocksOwnIcon::BOTH {
        let once = icon.drawn(28, ground, ink);
        assert_eq!(once.len(), 28 * 28, "{icon:?}");
        for pixel in &once {
            for c in 0..3 {
                let (low, high) = (ground[c].min(ink[c]), ground[c].max(ink[c]));
                assert!(
                    (low..=high).contains(&pixel[c]),
                    "{icon:?}: {pixel:?} is not a mix of the ground and the ink"
                );
            }
        }
        assert_eq!(once[0], ground, "{icon:?}: ink in the corner");
        assert_eq!(once[28 * 28 - 1], ground, "{icon:?}: ink in the corner");
        assert!(
            once.iter()
                .any(|pixel| (0..3).all(|c| pixel[c].abs_diff(ink[c]) <= 8)),
            "{icon:?}: no stroke reaches the ink"
        );
        let inked = once.iter().filter(|pixel| **pixel != ground).count();
        let twice = icon.drawn(56, ground, ink);
        let inked_twice = twice.iter().filter(|pixel| **pixel != ground).count();
        assert!(
            (3 * inked..=5 * inked).contains(&inked_twice),
            "{icon:?}: twice the side should be about four times the ink: {inked} and {inked_twice}"
        );
        assert!(icon.drawn(0, ground, ink).is_empty(), "{icon:?}");
        assert!(icon.drawn(-3, ground, ink).is_empty(), "{icon:?}");
    }
}

/// **On the bar, each entry is its icon and not a letter**: one box of ink in
/// its slot, the artwork's size through the display's scale, centred, holding
/// exactly the icon drawn in the palette's ink on the Dock's ground — and no
/// badge circle under it. In light and dark, at one to one and at twice the
/// pixels.
#[test]
fn on_the_bar_each_entry_is_its_icon_in_the_palettes_ink_at_every_scale() {
    let mut inks = Vec::new();
    for scheme in [Scheme::Light, Scheme::Dark] {
        for scale in [DisplayScale::ONE, DisplayScale::of_per_cent(200)] {
            let picture = drawn(scheme, scale, &crate::WhatTheDockHasOpen::Nothing);
            let palette = a_look_in(scheme).palette().expect("a palette");
            assert_ne!(
                palette.badge, palette.dock,
                "the premise: a badge would show"
            );
            let side = scale.px(alo_dock::measures::GLYPH);
            assert_eq!(side, 28 * i32::from(scale.per_cent()) / 100);
            for (action, icon) in BOTH {
                let slot = its_slot(&picture, action);
                let ink = ink_on(&picture, slot);
                assert_eq!(ink.len(), 1, "{scheme:?} {scale:?} {action:?}: {ink:?}");
                let (area, pixels) = ink[0];
                assert_eq!(
                    area,
                    Rectangle::new(
                        (
                            slot.loc.x + (slot.size.w - side) / 2,
                            slot.loc.y + (slot.size.h - side) / 2
                        )
                            .into(),
                        (side, side).into()
                    ),
                    "{scheme:?} {scale:?} {action:?}: not the artwork's square, centred"
                );
                assert_eq!(
                    pixels,
                    icon.drawn(side, palette.dock, palette.ink).as_slice(),
                    "{scheme:?} {scale:?} {action:?}: not the icon in the Dock's ink"
                );
                assert!(
                    pixels.iter().all(|pixel| *pixel != palette.badge),
                    "{scheme:?} {scale:?} {action:?}: a badge's colour in the slot"
                );
                // Nothing under it is a badge: the surface at the slot's centre
                // and round where a circle's rim would be is the Dock's own.
                let surface = picture.surface.as_ref().expect("the Dock has a surface");
                let at = |x: i32, y: i32| -> [u8; 3] {
                    let (x, y) = (x - surface.area.loc.x, y - surface.area.loc.y);
                    let n = usize::try_from(y * surface.area.size.w + x).expect("on it") * 4;
                    [
                        surface.pixels[n],
                        surface.pixels[n + 1],
                        surface.pixels[n + 2],
                    ]
                };
                let (cx, cy) = (area.loc.x + side / 2, area.loc.y + side / 2);
                let r = side / 2 - 3;
                for (dx, dy) in [(0, 0), (r, 0), (-r, 0), (0, r), (0, -r)] {
                    assert_ne!(
                        at(cx + dx, cy + dy),
                        palette.badge,
                        "{scheme:?} {scale:?} {action:?}: a badge under the icon"
                    );
                }
            }
            if scale == DisplayScale::ONE {
                inks.push((scheme, palette.ink));
            }
        }
    }
    let primary = alo_appearance::Role::TextPrimary.colour();
    assert_eq!(
        inks[0],
        (
            Scheme::Light,
            [primary.red(), primary.green(), primary.blue()]
        ),
        "light is drawn in text/primary"
    );
    assert_ne!(inks[0].1, inks[1].1, "dark is drawn in its own ink");
}

/// **In the list *Show all* opens, each entry is its icon too**, and not its
/// first letter on a circle.
#[test]
fn in_the_overview_each_entry_is_its_icon() {
    for scheme in [Scheme::Light, Scheme::Dark] {
        for scale in [DisplayScale::ONE, DisplayScale::of_per_cent(200)] {
            let picture = drawn(
                scheme,
                scale,
                &crate::WhatTheDockHasOpen::MoreApps { focused: None },
            );
            assert!(
                picture.overflow_list.is_some(),
                "{scheme:?} {scale:?}: the premise, the overview is open"
            );
            let palette = a_look_in(scheme).palette().expect("a palette");
            let side = scale.px(alo_dock::measures::GLYPH);
            for (action, icon) in BOTH {
                let wanted = icon.drawn(side, palette.dock, palette.ink);
                let found = picture
                    .inked
                    .iter()
                    .filter(|inked| inked.pixels == wanted)
                    .count();
                // Once on the bar and once in the list.
                assert_eq!(
                    found, 2,
                    "{scheme:?} {scale:?} {action:?}: drawn {found} times, not on the bar and in the list"
                );
            }
        }
    }
}
