//! That the task status's artwork is the design file's own, read whole,
//! recorded with the node it came from, and in the colour the file names.

use super::AStatusIcon;

/// The record of where they came from.
const WHERE_THEY_CAME_FROM: &str =
    include_str!("../icons/the-task-status/where-they-came-from.toml");

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
    assert_eq!(recorded.len(), AStatusIcon::ALL.len(), "{recorded:?}");
    for icon in AStatusIcon::ALL {
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
        // **Drawn at 16 by the design, in a box of 14.5**: the group sits
        // 8.33% in from each side and its stroke reaches back out 4.38%.
        assert_eq!(shape.its_box(), (14.5, 14.5), "{}", icon.file_name());
    }
}
