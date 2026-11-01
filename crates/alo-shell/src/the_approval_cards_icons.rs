//! **The approval card's artwork**: alo's mark beside *alo is asking to*
//! (`549:195`), and the file icon in the tile of each row naming a file the
//! change touches (`549:162`).
//!
//! The files are in `crates/alo-shell/icons/the-approval-card/`, byte for
//! byte as the design file exported them, with where each came from in
//! `where-they-came-from.toml`. `alo_vector_shapes` reads and draws them at the
//! display's scale, as the reply box's icons are drawn
//! (`crate::the_reply_boxs_icons`), each inked in the palette's colour.

use std::sync::OnceLock;

use alo_vector_shapes::AShape;
use smithay::utils::{Physical, Rectangle};

use crate::painted::Washed;

/// alo's mark, as the design exported it.
pub(crate) const THE_MARK: &str = include_str!("../icons/the-approval-card/alo-mark.svg");

/// The file icon, as the design exported it.
pub(crate) const THE_FILE: &str = include_str!("../icons/the-approval-card/file.svg");

/// One of the card's two pieces of artwork.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ACardsIcon {
    /// alo's mark.
    TheMark,
    /// A file.
    AFile,
}

impl ACardsIcon {
    /// Both, in the order the card draws them.
    pub(crate) const ALL: [Self; 2] = [Self::TheMark, Self::AFile];

    /// The file, as the design exported it.
    pub(crate) const fn svg(self) -> &'static str {
        match self {
            Self::TheMark => THE_MARK,
            Self::AFile => THE_FILE,
        }
    }

    /// The file's name, as `where-they-came-from.toml` records it.
    pub(crate) const fn file_name(self) -> &'static str {
        match self {
            Self::TheMark => "alo-mark.svg",
            Self::AFile => "file.svg",
        }
    }

    /// The artwork, read once for the life of the shell.
    ///
    /// [`None`] for a file `alo_vector_shapes` refused, which is said once on
    /// the error stream and which this module's tests refuse on every build.
    pub(crate) fn shape(self) -> Option<&'static AShape> {
        static READ: OnceLock<[Option<AShape>; 2]> = OnceLock::new();
        let [mark, file] = READ.get_or_init(|| {
            ACardsIcon::ALL.map(|icon| match AShape::read(icon.svg()) {
                Ok(shape) => Some(shape),
                Err(why) => {
                    eprintln!(
                        "alo-shell: the approval card's {} could not be read and is not drawn: {why}",
                        icon.file_name()
                    );
                    None
                }
            })
        });
        match self {
            Self::TheMark => mark.as_ref(),
            Self::AFile => file.as_ref(),
        }
    }
}

/// Ink `icon` into `panel`, filling `at` (in the display's pixels), in `ink`.
/// Pixels outside the raster are left alone.
pub(crate) fn ink_a_cards_icon(
    panel: &mut Washed,
    icon: ACardsIcon,
    at: Rectangle<i32, Physical>,
    ink: [u8; 3],
) {
    if let Some(shape) = icon.shape() {
        crate::inking_a_shape::ink_a_shape(panel, shape, at, ink);
    }
}

#[cfg(test)]
#[path = "the_approval_cards_icons_tests.rs"]
mod tests;
