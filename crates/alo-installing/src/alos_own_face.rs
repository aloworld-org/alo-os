//! The typeface the environment draws in, and how wide it draws things.
//!
//! # Why the installer carries a font at all
//!
//! The install environment is an initramfs. It has no fontconfig, no
//! `/usr/share/fonts`, and no package manager to get one — and
//! `alo-installing` is built static against musl precisely so that what is
//! copied into that environment depends on nothing in it. A font read from a
//! path would be a font that is not there.
//!
//! Measured on 2026-10-20: the base the environment is built from carries
//! `AdwaitaSans-Regular.ttf` and `AdwaitaMono-Regular.ttf` and no Manrope at
//! all, so the environment would otherwise have drawn its screens in a
//! typeface this product's design does not use.
//!
//! # Why this exact file, and why it is not copied
//!
//! `Manrope-SemiBold.ttf`, the static cut, reached across to
//! `crates/alo-shell/fonts/` rather than duplicated here. **One file, so the
//! installer and the desktop cannot drift onto different cuts of the same
//! typeface** — which is the same reason the renderer is shared, and the thing
//! a second copy would quietly undo the first time somebody updated one of
//! them.
//!
//! SemiBold rather than `Manrope.ttf`, which is the variable cut: a variable
//! font needs its weight axis set, and a weight axis that is set wrongly draws
//! ExtraLight while every test about *which family* still passes. This
//! repository has already found that once — `docs/` records the shell's own
//! tables saying ExtraLight while its ink said SemiBold. A static cut cannot
//! be wrong about its weight.
//!
//! # What this does not do, said plainly
//!
//! **No shaping, and so no kerning and no ligatures.** Each character is
//! looked up in the font's own character map and advanced by its own width.
//! For the Latin sentences this environment says — including *Müller* and
//! *Liège*, whose accented letters are single characters in every one of the
//! 24 languages `alo-strings` carries — that is correct, and the only visible
//! cost is a pair like *AV* sitting very slightly apart.
//!
//! What it would be wrong for is a script where a character's shape depends on
//! its neighbours, or text written with combining marks rather than composed
//! characters. Neither is among the 24, and **whoever adds a language that
//! needs one should start at this paragraph**: it needs `rustybuzz`, which is
//! already in the lock file, and not a change to anything else here.

use swash::{FontRef, GlyphId};

/// The typeface, as bytes compiled into this program.
///
/// Reached across to the crate that owns the product's typefaces. See this
/// module's header for why it is not a second copy.
const MANROPE_SEMIBOLD: &[u8] = include_bytes!("../../alo-shell/fonts/Manrope-SemiBold.ttf");

/// What alo's own surfaces are drawn in.
#[derive(Clone, Copy)]
pub struct AlosOwnFace<'a> {
    /// What swash reads the tables through.
    face: FontRef<'a>,
}

/// Written by hand because swash's own `FontRef` is not `Debug`, and a face
/// that could not be printed would make every assertion about one unreadable.
/// It says what the face is rather than dumping its tables, which is all a
/// failing test needs.
impl core::fmt::Debug for AlosOwnFace<'_> {
    fn fmt(&self, to: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        to.debug_struct("AlosOwnFace")
            .field("weight", &self.face.attributes().weight().0)
            .finish()
    }
}

impl AlosOwnFace<'static> {
    /// The typeface this program carries.
    ///
    /// # Errors
    ///
    /// [`NoFace`] where the compiled-in bytes are not a font swash can read,
    /// which would mean the file in the repository is damaged. It is an error
    /// rather than a panic because this program runs as root against a disk it
    /// is about to overwrite, and the one thing it must always be able to do
    /// is say why it is not going on.
    pub fn carried() -> Result<Self, NoFace> {
        FontRef::from_index(MANROPE_SEMIBOLD, 0)
            .map(|face| Self { face })
            .ok_or(NoFace)
    }
}

impl<'a> AlosOwnFace<'a> {
    /// What swash reads this face through.
    #[must_use]
    pub const fn as_swash_reads_it(self) -> FontRef<'a> {
        self.face
    }

    /// Which glyph this character is drawn with, and `None` where the face has
    /// none for it.
    ///
    /// Glyph nought is a font's *missing glyph* — the empty box — and it is
    /// reported as `None` rather than drawn, because a row of empty boxes on a
    /// screen asking whether to erase a disk is worse than a gap.
    #[must_use]
    pub fn the_glyph_for(self, character: char) -> Option<GlyphId> {
        let glyph = self.face.charmap().map(character);
        (glyph != 0).then_some(glyph)
    }

    /// How wide this text is drawn at this many pixels to the em.
    ///
    /// This is the `measure` [`crate::wrapped`] asks for, and the two have to
    /// agree or the wrapping is wrong by whatever they disagree by — so it is
    /// the same arithmetic that [`crate::a_line_as_coverage`] advances by when
    /// it draws, rather than a second estimate of it.
    ///
    /// A character the face has no glyph for adds nothing, which matches it
    /// not being drawn.
    #[must_use]
    pub fn how_wide(self, text: &str, em: u32) -> u32 {
        let metrics = self.face.glyph_metrics(&[]).scale(em as f32);
        let across: f32 = text
            .chars()
            .filter_map(|character| self.the_glyph_for(character))
            .map(|glyph| metrics.advance_width(glyph))
            .sum();
        // Rounded up, because a line measured a fraction narrow is a line that
        // overflows the block by that fraction, and the only direction this
        // can be wrong in safely is wide.
        across.ceil().max(0.0) as u32
    }

    /// How far below the top of a line its baseline sits.
    ///
    /// Needed by anything that draws: a glyph is positioned from its baseline,
    /// and a layout positions lines from their tops.
    ///
    /// # Why the line height is asked for and not just the em
    ///
    /// **Because Manrope ascends higher than its own em.** Measured on
    /// 2026-10-20, at 2000 units to the em: ascent 2132, descent 600, leading
    /// nought — so a line of it is 1.37 em tall and the ascent alone is 1.07.
    /// Treating the em as the line height put the baseline at the very bottom
    /// of the line and clipped every capital and every descender, which two
    /// tests caught by asking whether a `p` reaches below an `H`. It did not:
    /// both ended on the last row.
    ///
    /// So the line height comes from the layout, which already has it —
    /// `TheType::apart`, 25 for the environment's body text against an em of
    /// 16 — and the typeface's own box is centred in it.
    ///
    /// One corroboration worth recording: this face's cap height is 1440 of
    /// 2000 units, **exactly the 0.72 of an em that
    /// `where_a_screen_goes` floors body legibility by**. That constant was
    /// taken from the design rather than from the font, and the two agree.
    #[must_use]
    pub fn the_baseline_in(self, em: u32, line_is_tall: u32) -> u32 {
        let metrics = self.face.metrics(&[]).scale(em as f32);
        let the_faces_own_box = metrics.ascent + metrics.descent;
        let slack = (line_is_tall as f32 - the_faces_own_box).max(0.0) / 2.0;
        (slack + metrics.ascent).round().max(0.0) as u32
    }
}

/// The compiled-in typeface could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the typeface compiled into this program is not a font that can be read")]
pub struct NoFace;

#[cfg(test)]
#[path = "alos_own_face_tests.rs"]
mod tests;
