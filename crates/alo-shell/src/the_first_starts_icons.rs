//! The first start's icons, from the design's own files: the language, the
//! region and the keyboard on the overview's rows, the chevron at a row's end,
//! the three notes' marks, and *Accessibility*'s.
//!
//! **Inked onto the ground they sit on**, as the sheets' words are
//! (`crate::painted::Inked`): every icon here sits on a surface of one colour —
//! a cool tile, a card, the sheet — so it is drawn as opaque pixels already
//! blended with that colour, and painted after the sheet's surface.

use std::sync::OnceLock;

use alo_vector_shapes::AShape;
use smithay::utils::{Physical, Rectangle};

use crate::painted::Inked;

/// One of the first start's icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AFirstStartIcon {
    /// The language row's.
    Language,
    /// The region row's.
    Region,
    /// The keyboard row's.
    Keyboard,
    /// At the end of a row that opens a list.
    Chevron,
    /// The overview's note.
    Info,
    /// The name's note: an account made on this computer.
    Computer,
    /// The PIN's and the password's note.
    Lock,
    /// *Accessibility*'s.
    Accessibility,
}

impl AFirstStartIcon {
    /// Every icon, in the record's order.
    pub(crate) const ALL: [Self; 8] = [
        Self::Language,
        Self::Region,
        Self::Keyboard,
        Self::Chevron,
        Self::Info,
        Self::Computer,
        Self::Lock,
        Self::Accessibility,
    ];

    /// The file, as the design exported it.
    const fn svg(self) -> &'static str {
        match self {
            Self::Language => include_str!("../icons/the-first-start/language.svg"),
            Self::Region => include_str!("../icons/the-first-start/region.svg"),
            Self::Keyboard => include_str!("../icons/the-first-start/keyboard.svg"),
            Self::Chevron => include_str!("../icons/the-first-start/chevron.svg"),
            Self::Info => include_str!("../icons/the-first-start/info.svg"),
            Self::Computer => include_str!("../icons/the-first-start/computer.svg"),
            Self::Lock => include_str!("../icons/the-first-start/lock.svg"),
            Self::Accessibility => include_str!("../icons/the-first-start/accessibility.svg"),
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
            AFirstStartIcon::ALL
                .iter()
                .map(|icon| match AShape::read(icon.svg()) {
                    Ok(shape) => Some(shape),
                    Err(why) => {
                        eprintln!(
                            "alo-shell: a first-start icon could not be read and is not drawn: {why}"
                        );
                        None
                    }
                })
                .collect()
        })
        .get(self.at())
        .and_then(Option::as_ref)
    }

    /// This icon in `at`, its square in the picture's pixels, in `ink` on
    /// `ground`. [`None`] for a square with no pixels or a file not read.
    pub(crate) fn inked(
        self,
        at: Rectangle<i32, Physical>,
        ground: [u8; 3],
        ink: [u8; 3],
    ) -> Option<Inked> {
        let shape = self.shape()?;
        let (wide, tall) = (u32::try_from(at.size.w).ok()?, u32::try_from(at.size.h).ok()?);
        if wide == 0 || tall == 0 {
            return None;
        }
        let pixels = shape
            .coverage(wide, tall)
            .into_iter()
            .map(|covered| crate::sign_in_surface::mixed(ground, ink, f64::from(covered) / 255.0))
            .collect();
        Some(Inked { area: at, pixels })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every icon is the design file's own, read whole, and recorded.**
    #[test]
    fn every_icon_reads_and_is_recorded() {
        let record = include_str!("../icons/the-first-start/where-they-came-from.toml");
        for icon in AFirstStartIcon::ALL {
            assert!(icon.shape().is_some(), "{icon:?} does not read");
        }
        for name in [
            "language.svg",
            "region.svg",
            "keyboard.svg",
            "chevron.svg",
            "info.svg",
            "computer.svg",
            "lock.svg",
            "accessibility.svg",
        ] {
            assert!(record.contains(name), "{name} is not recorded");
        }
    }

    /// **Inked on its ground**: the corner of a chevron's square is the ground
    /// and its stroke is the ink.
    #[test]
    fn an_icon_is_its_ink_on_its_ground() {
        let ground = [0xFF, 0xFF, 0xFF];
        let ink = [0x59, 0x6B, 0x78];
        let drawn = AFirstStartIcon::Chevron
            .inked(Rectangle::new((0, 0).into(), (18, 18).into()), ground, ink)
            .map(|it| it.pixels);
        let pixels = drawn.unwrap_or_default();
        assert_eq!(pixels.len(), 18 * 18);
        assert_eq!(pixels.first(), Some(&ground), "the corner is the ground");
        assert!(pixels.contains(&ink), "the stroke is the ink");
    }
}
