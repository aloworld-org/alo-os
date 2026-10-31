//! That every icon the reply box draws is the design file's own, read whole,
//! recorded with the node it came from, in the design's colours, and drawn at
//! the display's scale.

use alo_tasks::TheOneButton;
use smithay::utils::Rectangle;

use super::{AnIcon, ink_an_icon};
use crate::display_scale::DisplayScale;
use crate::painted::Washed;

/// The record of where the icons came from.
const WHERE_THEY_CAME_FROM: &str = include_str!("../icons/the-reply-box/where-they-came-from.toml");

#[test]
fn every_icon_reads_whole() {
    for icon in AnIcon::ALL {
        assert!(
            alo_vector_shapes::AShape::read(icon.svg()).is_ok(),
            "{} does not read: {:?}",
            icon.file_name(),
            alo_vector_shapes::AShape::read(icon.svg())
        );
        assert!(icon.shape().is_some(), "{} is not drawn", icon.file_name());
    }
}

#[test]
fn every_icon_is_recorded_with_the_node_it_came_from() {
    let record: toml::Table = WHERE_THEY_CAME_FROM
        .parse()
        .unwrap_or_else(|why| unreachable!("the record reads: {why}"));
    assert_eq!(
        record.get("file").and_then(toml::Value::as_str),
        Some("nDxyF5Ho9oC4RObjVzwBNJ")
    );
    assert_eq!(
        record.get("fetched").and_then(toml::Value::as_str),
        Some("2026-10-30")
    );
    let recorded: Vec<(String, i64, String)> = record
        .get("icon")
        .and_then(toml::Value::as_array)
        .map(|icons| {
            icons
                .iter()
                .filter_map(|icon| {
                    Some((
                        icon.get("svg")?.as_str()?.to_owned(),
                        icon.get("artwork_is")?.as_integer()?,
                        icon.get("component")?.as_str()?.to_owned(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(recorded.len(), AnIcon::ALL.len(), "{recorded:?}");
    for icon in AnIcon::ALL {
        let Some((_, is, component)) = recorded.iter().find(|(svg, _, _)| svg == icon.file_name())
        else {
            unreachable!("{} is drawn and not recorded", icon.file_name());
        };
        assert_eq!(
            i64::from(icon.is()),
            *is,
            "{} is drawn at another size",
            icon.file_name()
        );
        assert!(!component.is_empty());
        // The file's own box is the size the design drew it at: the icon
        // frame itself, or — for the cloud — a box inside that frame.
        let shape = icon.shape().unwrap_or_else(|| unreachable!("read above"));
        #[expect(clippy::cast_precision_loss, reason = "28, 24 and 16 are exact in f32")]
        let frame = icon.is() as f32;
        let (wide, tall) = shape.its_box();
        match icon {
            AnIcon::Cloud => assert!(
                wide <= frame && tall <= frame,
                "the cloud's box {wide} × {tall} is bigger than its frame"
            ),
            AnIcon::Attach
            | AnIcon::TheModelsChevron
            | AnIcon::Permissions
            | AnIcon::Send
            | AnIcon::Stop => assert_eq!((wide, tall), (frame, frame), "{}", icon.file_name()),
        }
    }
}

/// **The file's colours are the design's light tokens**: `text/primary` on the
/// cool ground, white on the navy button — so drawing them in the palette's ink
/// is drawing what the design drew.
#[test]
fn every_icon_is_in_the_colour_the_design_names() {
    const TEXT_PRIMARY: [u8; 3] = [0x10, 0x2a, 0x43];
    const TEXT_MUTED: [u8; 3] = [0x59, 0x6b, 0x78];
    const WHITE: [u8; 3] = [255, 255, 255];
    for icon in AnIcon::ALL {
        let shape = icon
            .shape()
            .unwrap_or_else(|| unreachable!("every icon reads"));
        let expected = match icon {
            AnIcon::Attach | AnIcon::TheModelsChevron | AnIcon::Permissions => TEXT_PRIMARY,
            AnIcon::Send | AnIcon::Stop => WHITE,
            // In the destination's own quiet ink.
            AnIcon::Cloud => TEXT_MUTED,
        };
        assert!(
            shape.colours().iter().all(|colour| *colour == expected),
            "{} is drawn in {:?}",
            icon.file_name(),
            shape.colours()
        );
    }
}

#[test]
fn stop_while_alo_is_working_and_send_otherwise() {
    assert_eq!(AnIcon::on_the_one_button(TheOneButton::Stop), AnIcon::Stop);
    assert_eq!(
        AnIcon::on_the_one_button(TheOneButton::Stopping),
        AnIcon::Stop
    );
    assert_eq!(AnIcon::on_the_one_button(TheOneButton::Send), AnIcon::Send);
    assert_eq!(
        AnIcon::on_the_one_button(TheOneButton::NothingToSend),
        AnIcon::Send
    );
}

/// A white raster `side` pixels square at the origin.
fn white(side: i32) -> Washed {
    Washed {
        area: Rectangle::new((0, 0).into(), (side, side).into()),
        pixels: vec![255; usize::try_from(side * side * 4).unwrap_or(0)],
    }
}

/// How many pixels of the raster are no longer white.
fn inked(panel: &Washed) -> usize {
    panel
        .pixels
        .chunks(4)
        .filter(|pixel| pixel.get(..3) != Some(&[255, 255, 255][..]))
        .count()
}

/// **At 200 per cent an icon covers four times the pixels**, centred in a box
/// twice the size: the same artwork, sharper, not smaller.
#[test]
fn an_icon_is_drawn_at_the_displays_scale() {
    let mut one = white(48);
    ink_an_icon(
        &mut one,
        AnIcon::Stop,
        Rectangle::new((0, 0).into(), (48, 48).into()),
        DisplayScale::ONE,
        [0, 0, 0],
    );
    let mut two = white(96);
    ink_an_icon(
        &mut two,
        AnIcon::Stop,
        Rectangle::new((0, 0).into(), (96, 96).into()),
        DisplayScale::of_per_cent(200),
        [0, 0, 0],
    );
    let (one, two) = (inked(&one), inked(&two));
    assert!(one > 0, "nothing was inked");
    #[expect(
        clippy::cast_precision_loss,
        reason = "pixel counts in the thousands are exact in f64"
    )]
    let ratio = two as f64 / one as f64;
    assert!(
        (3.6..=4.4).contains(&ratio),
        "200 per cent inked {ratio} times as many pixels"
    );
}

/// **Centred in its box**: the Stop square's middle is the box's middle, and a
/// pixel in the box's corner is untouched.
#[test]
fn an_icon_is_centred_in_its_box() {
    let mut panel = white(48);
    ink_an_icon(
        &mut panel,
        AnIcon::Stop,
        Rectangle::new((0, 0).into(), (48, 48).into()),
        DisplayScale::ONE,
        [0, 0, 0],
    );
    let pixel = |x: usize, y: usize| panel.pixels.get((y * 48 + x) * 4).copied();
    assert_eq!(pixel(24, 24), Some(0), "the middle is not inked");
    assert_eq!(pixel(2, 2), Some(255), "the corner is inked");
}

/// **An icon on a raster's edge is cut, not refused** and not written outside.
#[test]
fn an_icon_past_the_raster_is_cut() {
    let mut panel = white(20);
    ink_an_icon(
        &mut panel,
        AnIcon::Stop,
        Rectangle::new((0, 0).into(), (48, 48).into()),
        DisplayScale::ONE,
        [0, 0, 0],
    );
    assert_eq!(panel.pixels.len(), 20 * 20 * 4);
    assert!(inked(&panel) > 0);
}
