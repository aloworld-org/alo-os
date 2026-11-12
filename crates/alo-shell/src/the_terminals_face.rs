//! **How a terminal asks for its text**: alo's monospace face at the
//! terminal's own weight, and heavier or slanted when a program asks.
//!
//! In the shell rather than `alo-drawing` because the weight is
//! `alo_terminal::measures::THE_WEIGHT` — the design's figure, kept in one
//! place — and a drawing crate an application builds on should not link a
//! terminal engine to know it. The face itself, and why every substitution by a
//! neighbour is off, is `alo_drawing::painted_text::alos_own_mono_face`.

use cosmic_text::{Attrs, Weight};

/// How alo asks for a terminal's text.
///
/// **The weight comes from `alo_terminal::measures`**, not from a literal here,
/// because it is the design's figure and a second copy of it is a second thing
/// to keep in step. It is also load-bearing in a way a weight usually is not:
/// at any other weight the family is thrown away entirely and the transcript is
/// drawn in a proportional face. `fonts/GeistMono-Medium.ttf` is the static cut
/// that makes this weight an exact match; before it was bundled, this ask drew
/// **Inter**.
///
/// **With every substitution by a neighbour switched off.** Geist Mono joins
/// and swaps characters by what stands beside them — ligatures and contextual
/// alternates — and a run is shaped whole, so a terminal printed `-rw-r--r--`
/// as `-IW-F-F-` on the twelfth Demo 1 walk. A cell holds exactly what the
/// program wrote; `crate::each_letter_keeps_its_own_cell_tests` holds it.
pub(crate) fn alos_own_mono<'a>() -> Attrs<'a> {
    crate::painted_text::alos_own_mono_face().weight(Weight(alo_terminal::measures::THE_WEIGHT))
}

/// How alo asks for a terminal's text when a program has asked for bold or
/// italic.
///
/// **Bold is a heavier face and not only a brighter colour.** A program asking
/// for bold means both — `alo_terminal`'s palette brightens the colour and this
/// asks for the weight — and a terminal doing one of the two is wrong in a way
/// a person notices by comparison.
///
/// **Bold is 700 and not the design's 500**, because `Terminal/Command` is the
/// ordinary weight and bold is heavier than ordinary. `GeistMono-Bold.ttf` is
/// bundled for it, and `bold_is_drawn_in_the_terminals_own_face` reads the
/// shaper's own `font_id` back to hold that it is used.
///
/// **This comment used to say a bold cell *resolves to the one weight that
/// is*, and that was the fault rather than a description of it.** A family is
/// honoured only among faces whose weight matches exactly, so a missed weight
/// does not fall back within the family — it leaves the family. Bold cells
/// were drawn in a **proportional** face, in a grid where every cell must be
/// one width, for the day between bold being built and the cut being bundled.
///
/// The sentence mattered because of how it read: *resolves to the one weight
/// that is* sounds like a cell a little too light, which is a thing to fix
/// later. *Draws in another typeface and breaks the grid* is not. A note that
/// understates what is wrong is worse than no note, because it is read as
/// having been assessed.
pub(crate) fn alos_own_mono_at<'a>(
    looks: alo_terminal::what_a_cell_looks_like::HowItLooks,
) -> Attrs<'a> {
    let mut attrs = alos_own_mono();
    if looks.bold {
        attrs = attrs.weight(Weight(700));
    }
    if looks.italic {
        attrs = attrs.style(cosmic_text::Style::Italic);
    }
    attrs
}
