//! That the approval card's artwork is the design file's own, read whole,
//! recorded with the node it came from, and in the colour the file names.

use smithay::utils::Rectangle;

use super::{ACardsIcon, ink_a_cards_icon};
use crate::painted::Washed;

/// The record of where they came from.
const WHERE_THEY_CAME_FROM: &str =
    include_str!("../icons/the-approval-card/where-they-came-from.toml");

#[test]
fn every_icon_reads_whole_and_is_recorded() {
    let record: toml::Table = WHERE_THEY_CAME_FROM
        .parse()
        .unwrap_or_else(|why| unreachable!("the record reads: {why}"));
    assert_eq!(
        record.get("file").and_then(toml::Value::as_str),
        Some("nDxyF5Ho9oC4RObjVzwBNJ")
    );
    let recorded = record
        .get("icon")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(recorded.len(), ACardsIcon::ALL.len(), "{recorded:?}");
    for icon in ACardsIcon::ALL {
        let shape = icon.shape();
        assert!(shape.is_some(), "{} does not read", icon.file_name());
        let shape = shape.unwrap_or_else(|| unreachable!());
        let Some(entry) = recorded
            .iter()
            .find(|it| it.get("svg").and_then(toml::Value::as_str) == Some(icon.file_name()))
        else {
            unreachable!("{} is drawn and not recorded", icon.file_name());
        };
        let colour = entry
            .get("colour")
            .and_then(toml::Value::as_str)
            .and_then(|it| it.strip_prefix('#'))
            .and_then(|hex| u32::from_str_radix(hex, 16).ok())
            .unwrap_or_else(|| unreachable!("{} has no colour", icon.file_name()));
        let [_, red, green, blue] = colour.to_be_bytes();
        assert!(
            shape.colours().iter().all(|it| *it == [red, green, blue]),
            "{} is drawn in {:?}",
            icon.file_name(),
            shape.colours()
        );
    }
    let mark = ACardsIcon::TheMark
        .shape()
        .unwrap_or_else(|| unreachable!());
    assert_eq!(mark.its_box(), (12.0, 12.0));
}

/// **Each draws**: ink lands inside the box it is given and nowhere else.
#[test]
fn every_icon_is_inked_inside_its_box() {
    for icon in ACardsIcon::ALL {
        let mut panel = Washed {
            area: Rectangle::new((0, 0).into(), (32, 32).into()),
            pixels: vec![255; 32 * 32 * 4],
        };
        ink_a_cards_icon(
            &mut panel,
            icon,
            Rectangle::new((10, 10).into(), (12, 12).into()),
            [0, 0, 0],
        );
        let mut inside = 0;
        let mut outside = 0;
        for (at, [red, ..]) in panel.pixels.as_chunks::<4>().0.iter().enumerate() {
            let (x, y) = (at % 32, at / 32);
            if *red != 255 {
                if (10..22).contains(&x) && (10..22).contains(&y) {
                    inside += 1;
                } else {
                    outside += 1;
                }
            }
        }
        assert!(inside > 0, "{} drew nothing", icon.file_name());
        assert_eq!(outside, 0, "{} drew outside its box", icon.file_name());
    }
}
