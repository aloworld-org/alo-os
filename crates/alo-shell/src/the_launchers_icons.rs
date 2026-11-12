//! The alo Bar's icons, from the design's own files: the search mark, the
//! Enter key, the cloud, and alo's mark. Close, Up and Down went with the
//! centred launcher, the only surface that drew them.
//!
//! **Each file's viewBox is the icon's whole frame** (`icons/the-launcher/
//! where-they-came-from.toml`), so an icon is drawn by stretching its file into
//! the square the launcher gives it, at the display's scale — nothing to centre
//! and no group to place, unlike `crate::the_settings_icons`, whose files were
//! taken from a group inside each frame.

use std::sync::OnceLock;

use alo_vector_shapes::AShape;

use crate::settings_canvas::{Area, Canvas};

/// One of the launcher's icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ALaunchersIcon {
    /// The search mark at the field's start.
    Search,
    /// Enter, on a key cap.
    Enter,
    /// Where an answer would come from.
    Cloud,
    /// alo's mark, on the row that asks alo.
    Alo,
}

impl ALaunchersIcon {
    /// Every icon, in the order the record lists them.
    pub(crate) const ALL: [Self; 4] = [Self::Search, Self::Enter, Self::Cloud, Self::Alo];

    /// The file, as the design exported it.
    const fn svg(self) -> &'static str {
        match self {
            Self::Search => include_str!("../icons/the-launcher/search.svg"),
            Self::Enter => include_str!("../icons/the-launcher/enter.svg"),
            Self::Cloud => include_str!("../icons/the-launcher/cloud.svg"),
            Self::Alo => include_str!("../icons/the-launcher/alo.svg"),
        }
    }

    /// Its place in [`Self::ALL`].
    fn at(self) -> usize {
        Self::ALL
            .iter()
            .position(|it| *it == self)
            .unwrap_or_default()
    }

    /// The artwork, read once for the life of the shell; [`None`] for a file
    /// `alo_vector_shapes` refused, said once and refused by this file's tests.
    fn shape(self) -> Option<&'static AShape> {
        static READ: OnceLock<Vec<Option<AShape>>> = OnceLock::new();
        READ.get_or_init(|| {
            ALaunchersIcon::ALL
                .iter()
                .map(|icon| match AShape::read(icon.svg()) {
                    Ok(shape) => Some(shape),
                    Err(why) => {
                        eprintln!(
                            "alo-shell: a launcher icon could not be read and is not drawn: {why}"
                        );
                        None
                    }
                })
                .collect()
        })
        .get(self.at())
        .and_then(Option::as_ref)
    }

    /// Draw this icon into `frame`, its square in the display's pixels, in
    /// `colour`.
    pub(crate) fn drawn(self, canvas: &mut Canvas, frame: Area, colour: [u8; 3]) {
        let Some(shape) = self.shape() else {
            return;
        };
        let (Ok(wide), Ok(tall)) = (u32::try_from(frame.size.w), u32::try_from(frame.size.h))
        else {
            return;
        };
        let covered = shape.coverage(wide, tall);
        canvas.artwork(frame.loc.x, frame.loc.y, frame.size.w, &covered, colour);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every icon is the design file's own, read whole, and recorded.**
    #[test]
    fn every_icon_reads_and_is_recorded_with_its_node() {
        let record = include_str!("../icons/the-launcher/where-they-came-from.toml");
        for icon in ALaunchersIcon::ALL {
            assert!(icon.shape().is_some(), "{icon:?} does not read");
        }
        for name in ["search.svg", "enter.svg", "cloud.svg", "alo.svg"] {
            assert!(record.contains(name), "{name} is not recorded");
        }
    }

    /// **Drawn into its square and no further.**
    #[test]
    fn an_icon_is_drawn_inside_its_square() {
        let area = Area::new((0, 0).into(), (40, 40).into());
        let mut canvas = Canvas::over(area);
        ALaunchersIcon::Enter.drawn(
            &mut canvas,
            Area::new((10, 10).into(), (16, 16).into()),
            [0x10, 0x2A, 0x43],
        );
        let inked = |x: i32, y: i32| canvas.at(x, y).is_some_and(|it| it[3] > 0);
        assert!(
            (10..26).any(|y| (10..26).any(|x| inked(x, y))),
            "the key is drawn inside its square"
        );
        assert!(!inked(5, 5), "nothing outside the square");
        assert!(!inked(30, 30), "nothing outside the square");
    }
}
