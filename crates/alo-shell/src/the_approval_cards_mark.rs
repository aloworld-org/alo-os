//! **The alo mark** beside *alo is asking to*, at the top of the approval card
//! (`549:195` in the approved frame `549:135`).
//!
//! The file is `crates/alo-shell/icons/the-approval-card/alo-mark.svg`, byte
//! for byte as the design file exported it, with where it came from beside it
//! in `where-it-came-from.toml`. `alo_vector_shapes` reads it and draws it at
//! the display's scale, as the reply box's icons are drawn
//! (`crate::the_reply_boxs_icons`); it is inked in alo's teal, which is the
//! colour the file names and the colour ADR 0067 reserves for alo acting.

use std::sync::OnceLock;

use alo_vector_shapes::AShape;
use smithay::utils::{Physical, Rectangle};

use crate::painted::Washed;

/// The file, as the design exported it.
pub(crate) const THE_MARK: &str = include_str!("../icons/the-approval-card/alo-mark.svg");

/// The mark, read once for the life of the shell.
///
/// [`None`] for a file `alo_vector_shapes` refused, which is said once on the
/// error stream and which this module's tests refuse on every build.
pub(crate) fn the_mark() -> Option<&'static AShape> {
    static READ: OnceLock<Option<AShape>> = OnceLock::new();
    READ.get_or_init(|| match AShape::read(THE_MARK) {
        Ok(shape) => Some(shape),
        Err(why) => {
            eprintln!(
                "alo-shell: the approval card's mark could not be read and is not drawn: {why}"
            );
            None
        }
    })
    .as_ref()
}

/// Ink the mark into `panel`, filling `at` (in the display's pixels), in
/// `ink`. Pixels outside the raster are left alone.
pub(crate) fn ink_the_mark(panel: &mut Washed, at: Rectangle<i32, Physical>, ink: [u8; 3]) {
    if let Some(shape) = the_mark() {
        crate::inking_a_shape::ink_a_shape(panel, shape, at, ink);
    }
}

#[cfg(test)]
#[path = "the_approval_cards_mark_tests.rs"]
mod tests;
