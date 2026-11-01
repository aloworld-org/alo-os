//! That the approval card's mark is the design file's own, read whole,
//! recorded with the node it came from, and in alo's teal.

use smithay::utils::Rectangle;

use super::{ink_the_mark, the_mark};
use crate::painted::Washed;

/// The record of where it came from.
const WHERE_IT_CAME_FROM: &str = include_str!("../icons/the-approval-card/where-it-came-from.toml");

#[test]
fn the_mark_reads_whole_at_the_size_the_design_drew_it() {
    let shape = the_mark();
    assert!(shape.is_some(), "the mark does not read");
    let shape = shape.unwrap_or_else(|| unreachable!());
    let record: toml::Table = WHERE_IT_CAME_FROM
        .parse()
        .unwrap_or_else(|why| unreachable!("the record reads: {why}"));
    assert_eq!(
        record.get("file").and_then(toml::Value::as_str),
        Some("nDxyF5Ho9oC4RObjVzwBNJ")
    );
    assert_eq!(
        record.get("node").and_then(toml::Value::as_str),
        Some("549:195")
    );
    assert_eq!(
        record.get("artwork_is").and_then(toml::Value::as_integer),
        Some(12)
    );
    assert_eq!(shape.its_box(), (12.0, 12.0));
    assert_eq!(
        shape.colours(),
        [[0x0f, 0x6b, 0x72]],
        "the mark is not in alo's teal"
    );
}

/// **It draws**: ink lands inside the box it is given and nowhere else.
#[test]
fn the_mark_is_inked_inside_its_box() {
    let mut panel = Washed {
        area: Rectangle::new((0, 0).into(), (32, 32).into()),
        pixels: vec![255; 32 * 32 * 4],
    };
    ink_the_mark(
        &mut panel,
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
    assert!(inside > 0, "the mark drew nothing");
    assert_eq!(outside, 0, "the mark drew outside its box");
}
