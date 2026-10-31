//! **The egress indicator's two arrows**, read from the design file's own SVG.
//!
//! The lit lamp's white arrow and the quiet circle's grey one, from the alo
//! window the owner approved on 2026-10-31 (`549:129` and `549:368`). The
//! files are in `crates/alo-shell/icons/the-egress-indicator/`, byte for byte
//! as the design file exported them, with the node each came from beside them.
//!
//! **A file of its own and not one of `egress_status*`**, because those are
//! held by `tests/egress_status_source.rs` to carrying no string at all, and
//! naming a file is a string. What is read here is artwork, never words.

use std::sync::OnceLock;

use alo_vector_shapes::AShape;

/// Which of the two arrows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnArrow {
    /// On the lit lamp, while something is leaving.
    Lit,
    /// In the quiet circle, while nothing is.
    Quiet,
}

impl AnArrow {
    /// Both, in the order [`Self::shape`] keeps them.
    pub(crate) const BOTH: [Self; 2] = [Self::Lit, Self::Quiet];

    /// The file, as the design exported it.
    pub(crate) const fn svg(self) -> &'static str {
        match self {
            Self::Lit => include_str!("../icons/the-egress-indicator/lit.svg"),
            Self::Quiet => include_str!("../icons/the-egress-indicator/quiet.svg"),
        }
    }

    /// The arrow's own box inside its frame, in hundredths of a logical pixel —
    /// the file's `viewBox`, which the design places in the middle of the
    /// frame.
    pub(crate) const fn drawn_in_hundredths(self) -> u32 {
        match self {
            Self::Lit => 715,
            Self::Quiet => 783,
        }
    }

    /// Its place in [`Self::BOTH`].
    const fn at(self) -> usize {
        match self {
            Self::Lit => 0,
            Self::Quiet => 1,
        }
    }

    /// The artwork, read once for the life of the shell, or [`None`] for a file
    /// `alo_vector_shapes` refused — said once on the error stream, and refused
    /// on every build by this file's tests.
    pub(crate) fn shape(self) -> Option<&'static AShape> {
        static READ: OnceLock<Vec<Option<AShape>>> = OnceLock::new();
        READ.get_or_init(|| {
            AnArrow::BOTH
                .iter()
                .map(|arrow| match AShape::read(arrow.svg()) {
                    Ok(shape) => Some(shape),
                    Err(why) => {
                        eprintln!(
                            "alo-shell: the egress indicator's {arrow:?} arrow could not be read: {why}"
                        );
                        None
                    }
                })
                .collect()
        })
        .get(self.at())
        .and_then(Option::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::AnArrow;

    /// The record of where the arrows came from.
    const WHERE_THEY_CAME_FROM: &str =
        include_str!("../icons/the-egress-indicator/where-they-came-from.toml");

    /// **Both arrows read whole**, are the size the record says, and are in the
    /// design's colours: white on the lamp, `text/muted` in the quiet circle.
    #[test]
    fn both_arrows_read_whole_at_their_recorded_size_in_the_designs_colours() {
        let record: toml::Table = WHERE_THEY_CAME_FROM
            .parse()
            .unwrap_or_else(|why| unreachable!("the record reads: {why}"));
        let icons = record
            .get("icon")
            .and_then(toml::Value::as_array)
            .unwrap_or_else(|| unreachable!("the record lists the arrows"));
        assert_eq!(icons.len(), AnArrow::BOTH.len());
        for (arrow, colour, file) in [
            (AnArrow::Lit, [255, 255, 255], "lit.svg"),
            (AnArrow::Quiet, [0x59, 0x6b, 0x78], "quiet.svg"),
        ] {
            let shape = arrow
                .shape()
                .unwrap_or_else(|| unreachable!("{arrow:?} does not read"));
            assert!(
                shape.colours().iter().all(|one| *one == colour),
                "{arrow:?}"
            );
            let recorded = icons
                .iter()
                .find(|icon| icon.get("svg").and_then(toml::Value::as_str) == Some(file))
                .unwrap_or_else(|| unreachable!("{file} is drawn and not recorded"));
            let frame = recorded
                .get("frame_is")
                .and_then(toml::Value::as_integer)
                .unwrap_or_default();
            let drawn = recorded
                .get("drawn_is")
                .and_then(toml::Value::as_float)
                .unwrap_or_default();
            #[expect(
                clippy::cast_precision_loss,
                reason = "an icon frame of 14 or 16 is exact in f64"
            )]
            let frame_is = frame as f64;
            assert!(
                drawn > 0.0 && drawn < frame_is,
                "{file}: the arrow's box {drawn} is not inside its frame {frame}"
            );
            #[expect(
                clippy::cast_possible_truncation,
                reason = "a box of a few logical pixels, in hundredths"
            )]
            let hundredths = (drawn * 100.0).round() as u32;
            assert_eq!(hundredths, arrow.drawn_in_hundredths(), "{file}");
            assert_eq!(
                (f64::from(shape.its_box().0) * 100.0).round(),
                (drawn * 100.0).round(),
                "{file}'s own box is not the size recorded"
            );
        }
    }
}
