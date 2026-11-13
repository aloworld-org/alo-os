//! **Settings and Applications on the Dock: line drawings, not first letters.**
//!
//! The owner's decision of 2026-11-13. The Dock drew these two entries as the
//! first letter of their translated names, *S* and *A*, which is the fallback
//! for an **application with no artwork**; these are alo's own entries, and
//! there is artwork for them now. Asked, the owner said *"Design icons for
//! both"*; shown the design file's own Settings artwork — three sliders,
//! `348:23618` — *"i think the gear icon can work best"*; and shown two gears,
//! *"Gear 1: classic, grid OK"*. So Settings is an eight-tooth cog, which
//! departs from the design file by the owner's choice, and Applications is
//! three rounded squares and a plus, which is new artwork approved that day.
//! `icons/the-dock/where-they-came-from.toml` has the files and the words.
//!
//! # Drawn the way the Dock's other drawing is
//!
//! Like History's mark (`crate::dock_history_mark`): one stroke, one colour,
//! in the Dock's ink on the Dock's ground, at the artwork's size
//! (`alo_dock::measures::GLYPH`) through the display's scale, centred in the
//! slot, and **no badge circle** — a circle is what says *this application has
//! no icon*. The name stays the tooltip (`crate::dock_tooltip`) and the
//! accessible name (`crate::the_dock_as_a_reader_meets_it`), so nobody loses
//! the word.
//!
//! **Unlike History's mark, the vector source is an SVG file per icon**, read
//! by `alo_vector_shapes` as every icon taken since ADR 0142 is. History's mark
//! is older than that crate and holds its path as numbers in Rust.
//!
//! **Not a `dock_*` file**, because those are held by `tests/desktop_source.rs`
//! to carrying no string at all, and naming a file is a string. What is read
//! here is artwork, never words.

use std::sync::OnceLock;

use alo_dock::ANativeAction;
use alo_vector_shapes::AShape;

/// One of the Dock's own two icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ADocksOwnIcon {
    /// Settings: the cog.
    Settings,
    /// Applications: three applications and a place for one more.
    Applications,
}

impl ADocksOwnIcon {
    /// Both, in the order [`Self::shape`] keeps them and the record lists them.
    pub(crate) const BOTH: [Self; 2] = [Self::Settings, Self::Applications];

    /// The icon a native entry is drawn with.
    ///
    /// **Exhaustive, with no catch-all**, so a third native entry is a compile
    /// error here rather than a Dock slot quietly drawn as a cog.
    #[must_use]
    pub(crate) const fn of(action: ANativeAction) -> Self {
        match action {
            ANativeAction::Settings => Self::Settings,
            ANativeAction::Applications => Self::Applications,
        }
    }

    /// The file drawn.
    pub(crate) const fn svg(self) -> &'static str {
        match self {
            Self::Settings => include_str!("../icons/the-dock/settings.svg"),
            Self::Applications => include_str!("../icons/the-dock/applications.svg"),
        }
    }

    /// Its place in [`Self::BOTH`].
    const fn at(self) -> usize {
        match self {
            Self::Settings => 0,
            Self::Applications => 1,
        }
    }

    /// The artwork, read once for the life of the shell, or [`None`] for a file
    /// `alo_vector_shapes` refused — said once on the error stream, and refused
    /// on every build by this file's tests.
    pub(crate) fn shape(self) -> Option<&'static AShape> {
        static READ: OnceLock<Vec<Option<AShape>>> = OnceLock::new();
        READ.get_or_init(|| {
            ADocksOwnIcon::BOTH
                .iter()
                .map(|icon| match AShape::read(icon.svg()) {
                    Ok(shape) => Some(shape),
                    Err(why) => {
                        eprintln!(
                            "alo-shell: the Dock's {icon:?} icon could not be read and is not drawn: {why}"
                        );
                        None
                    }
                })
                .collect()
        })
        .get(self.at())
        .and_then(Option::as_ref)
    }

    /// The icon in a square `side` pixels across, `ink` over `ground`, row by
    /// row from the top — opaque, as every inked box in this shell is.
    ///
    /// A square of `ground` alone if the file could not be read, so the slot is
    /// still the Dock's and nothing is drawn in it that is not the artwork; and
    /// empty for a square with no area.
    #[must_use]
    pub(crate) fn drawn(self, side: i32, ground: [u8; 3], ink: [u8; 3]) -> Vec<[u8; 3]> {
        let Ok(across) = u32::try_from(side) else {
            return Vec::new();
        };
        let covered = self.shape().map_or_else(
            || vec![0; usize::try_from(u64::from(across) * u64::from(across)).unwrap_or(0)],
            |shape| shape.coverage(across, across),
        );
        covered
            .into_iter()
            .map(|how_much| mixed(ground, ink, how_much))
            .collect()
    }
}

/// `ink` over `ground`, `covered` 255ths of the way.
fn mixed(ground: [u8; 3], ink: [u8; 3], covered: u8) -> [u8; 3] {
    let one = |g: u8, i: u8| {
        let (g, i, c) = (u32::from(g), u32::from(i), u32::from(covered));
        let value = (g * (255 - c) + i * c + 127) / 255;
        u8::try_from(value).unwrap_or(u8::MAX)
    };
    [
        one(ground[0], ink[0]),
        one(ground[1], ink[1]),
        one(ground[2], ink[2]),
    ]
}

#[cfg(test)]
#[path = "the_docks_own_icons_tests.rs"]
mod tests;
