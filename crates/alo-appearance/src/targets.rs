//! How big a thing a person presses, drags or grabs has to be.
//!
//! These are not this crate's design and not any crate's. They are two figures
//! from **WCAG**, carried into European procurement by **EN 301 549**, plus one
//! decision the owner took that is built out of them.
//!
//! # Why they live here rather than beside a dock
//!
//! `alo_dock::measures` held the first of them, because a dock icon was the
//! first thing that needed it. That file's own header says it holds *the
//! numbers this crate is built out of*, and a standard's floor was never that:
//! it is a rule about hands, and it binds a dock icon, a window's name band, a
//! status item and anything else somebody has to hit.
//!
//! This crate is the common ancestor of every surface that draws — `alo-dock`,
//! `alo-shell` and the rest already depend on it — and it is where
//! [`crate::text::TextScale`] lives, which matters because these floors are
//! scaled rather than fixed. *Moved here on 2026-09-30, after the Mac lane
//! pointed out that a second surface needing a floor was about to produce a
//! second copy of it with a different name.*
//!
//! # The unit, and the one-way scaling
//!
//! **Logical pixels**, like everything else a layout is written in. A display
//! converts them through its own scale, so a dense screen draws the same target
//! sharper rather than smaller.
//!
//! **That conversion happens once, elsewhere, and nothing here repeats it.** It
//! is `alo_displays::Scale::laid_out`, applied by `room_at` in
//! `alo_shell::screens` at the rendering boundary. A floor in this file is a
//! logical figure and stays one; a surface that multiplied it by the display's
//! scale as well would reserve twice the room on a dense screen.
//!
//! **Where that would be caught, and where it would not.**
//! `alo_shell::screens`' own tests do exercise a scale other than one — two
//! screens, neither at a hundred per cent, each sized from its own glass — and
//! they hold `place.room()` to `laid_out(pixels)` and to *not* the raw pixels.
//! So **the conversion itself is tested.** What is not tested at any scale but
//! one is everything downstream of it: layout, pointer hit-testing, control
//! exclusion and this drag band. A double conversion in a surface would pass
//! today, and the gap is one step further along than it looks.
//!
//! *Written this way after the first draft of this paragraph claimed that no
//! test in `alo-shell` uses a scale other than one. That was measured as `no
//! test calls `Scale::per_cent` with a number` and stated as a fact about
//! scales — and the tests that do use another scale get it from
//! `worked_out_for` and a screen's glass instead. The narrower claim is the
//! true one and it is the one that locates the gap.*
//!
//! **Accessibility may enlarge these and may never reduce them.** The direction
//! belongs to the measure rather than to a convention around it, which is why
//! [`at_least`] exists and why a scale below one cannot shrink a floor through
//! it.
//!
//! # Who is held to this, and who is not yet
//!
//! The owner asked on 2026-09-30 for these to be tested against the Dock, the
//! status area and the minimized panel. Two of the three are:
//!
//! - **The Dock.** `alo_dock::places` holds every icon to
//!   [`ENHANCED_TARGET`], and `alo_dock::room` holds the icon's own room to it,
//!   both against the rectangles those files produce rather than against the
//!   constant, because an assertion written directly about a constant is folded
//!   away before it can fail.
//! - **The minimized panel.** `alo_shell::panel_raster` holds every preview
//!   slot to it, on the rectangles it lays out.
//!
//! **The status area is not, and that is a fact about the status area rather
//! than an omission here.** Nothing in it is pressable: `alo_shell::status_row`
//! draws an edge, a ground, a mark and words, and neither it nor
//! `status_items` has any notion of a press, a click or a target. A test
//! asserting a floor over nothing would pass for the wrong reason and go on
//! passing after somebody added the first control — which is the family
//! `docs/decisions/0080` names. **The status area is held to this on the day it
//! gains something to press**, and whoever adds that is the one who owes the
//! test.

/// The smallest a thing a person presses may be: **WCAG 2.5.8, Target Size
/// (Minimum)**, 24×24.
///
/// **A floor a layout may not go below, not a size to build to.** No target in
/// the alo OS design file is 24 — they are 29, 40, 42, 44 and 52 — and the file
/// draws 24×24 *glyphs* inside larger targets, hundreds of them. A control
/// sized to this number would be compliant and still wrong. Where a thing is a
/// control, [`ENHANCED_TARGET`] is the number to reach for.
pub const SMALLEST_TARGET: u32 = 24;

/// What a control is built to: **WCAG 2.5.5, Target Size (Enhanced)**, 44×44.
///
/// Nearly twice [`SMALLEST_TARGET`], and deliberately. That one is what a
/// standard will not let a layout go below; this is what this product builds
/// to, and the owner set it for the shell's controls on 2026-09-30. The design
/// agrees: its largest hit areas are 44 and 52.
pub const ENHANCED_TARGET: u32 = 44;

/// The unobstructed area a person must always be able to grab a frame by, along
/// its name band.
///
/// **Not a button and not a glyph**, which is the distinction the owner drew on
/// 2026-09-30 after two lanes read it as one or the other. It is *the
/// unobstructed area needed to grab and recover a frame* — the part of a name
/// band that no control may cover, so a window can always be picked up and
/// brought back however its chrome is arranged.
///
/// **Both of its numbers are the standards' and neither is new.** It is
/// [`ENHANCED_TARGET`] along the band and [`SMALLEST_TARGET`] across it: the
/// enhanced figure on the edge a gesture runs along, and the minimum on the
/// edge it does not. A band is not a square, and pressing a 44×24 strip is not
/// the act that 24×24 is the floor for — the strip is nearly twice the area and
/// is reached along its length rather than at a point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragBand;

impl DragBand {
    /// How long it is, along the name band: [`ENHANCED_TARGET`].
    #[must_use]
    pub const fn along() -> u32 {
        ENHANCED_TARGET
    }

    /// How deep it is, across the name band: [`SMALLEST_TARGET`].
    ///
    /// **This is that constant and not a copy of it.** A second 24 with its own
    /// name is how two numbers that mean one thing drift apart while both look
    /// deliberate.
    #[must_use]
    pub const fn across() -> u32 {
        SMALLEST_TARGET
    }
}

/// A floor in **logical** pixels, which accessibility may raise and may not
/// lower.
///
/// `scale` is hundredths — 100 is one, 150 is half again — and is **the
/// person's text or accessibility scale only.**
///
/// # It is not the display's conversion, and it said it was until 2026-10-01
///
/// This read *covers both the display's own conversion and the person's text
/// size*. That was wrong, and wrong in the direction that doubles: a display's
/// conversion is applied **once**, at the rendering boundary —
/// `alo_displays::Scale::laid_out`, called by `room_at` in
/// `alo_shell::screens` — so a caller who multiplied by it here would convert
/// twice, and a reviewer reading the old sentence would have believed they were
/// obliged to.
///
/// *That conversion divides rather than multiplies — `pixels * 100 / per_cent`
/// — turning a screen's own pixels into the logical room a layout is written
/// in. Checked rather than described, because the first draft of this paragraph
/// named it as a free function in a module when it is a method on a type: a
/// path nobody could follow, inside a sentence whose only purpose is that a
/// reader can go and check where the conversion happens.*
///
/// The owner's requirement, in their words: *preserve at least 44 × 24 logical
/// pixels of unobstructed drag area; convert to physical coordinates once at
/// the established rendering boundary.* Where that conversion already happens,
/// it is reused. **No additional multiplication by display scale.**
///
/// *The earlier wording came from this lane telling the owner a handle needed
/// display scaling. It did not; the conversion was already there and had been
/// read off a type annotation rather than measured. The ruling that followed
/// carried the mistaken premise, and the owner removed the clause on
/// 2026-10-01 once the premise was corrected.*
///
/// **A scale below one returns the floor unchanged**, because these are floors:
/// a person who made their text smaller did not ask for a target they cannot
/// hit.
#[must_use]
pub fn at_least(floor: u32, scale: u32) -> u32 {
    let raised = u64::from(floor)
        .saturating_mul(u64::from(scale))
        .saturating_div(100);
    let raised = u32::try_from(raised).unwrap_or(u32::MAX);
    if raised > floor { raised } else { floor }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The two standards, named rather than described.** If either moves,
    /// something has been changed that a procurement answer depends on.
    #[test]
    fn the_two_floors_are_the_standards_own_figures() {
        assert_eq!(SMALLEST_TARGET, 24, "WCAG 2.5.8 is 24 by 24");
        assert_eq!(ENHANCED_TARGET, 44, "WCAG 2.5.5 is 44 by 44");
        // **Through `at_least` rather than compared directly.** Clippy refused
        // the direct form — *this assertion has a constant value* — and it was
        // right: a comparison between two constants is folded before it can
        // fail, which is the fault `measures.rs` already warns about two files
        // away. Routing it through a function the optimiser cannot fold makes
        // the claim survive into a test that can actually run.
        assert!(
            at_least(ENHANCED_TARGET, 100) > at_least(SMALLEST_TARGET, 100),
            "a control is built to more than the floor allows"
        );
    }

    /// **The drag band is built out of them and holds no number of its own.**
    /// The owner's 44 by 24 is the enhanced figure along and the minimum
    /// across, which is why neither had to be invented.
    #[test]
    fn the_drag_band_is_the_two_standards_and_not_a_third_number() {
        assert_eq!(DragBand::along(), ENHANCED_TARGET);
        assert_eq!(DragBand::across(), SMALLEST_TARGET);
        assert_eq!((DragBand::along(), DragBand::across()), (44, 24));
    }

    /// **A band is worth more than the square its short edge would allow.**
    /// The reason 24 is not too small here: the area is nearly twice a
    /// minimum square's, and the gesture runs along the long edge.
    #[test]
    fn the_band_is_larger_than_the_smallest_square_it_contains() {
        let band = at_least(DragBand::along(), 100) * at_least(DragBand::across(), 100);
        let square = at_least(SMALLEST_TARGET, 100) * at_least(SMALLEST_TARGET, 100);
        assert!(band > square, "{band} is not more than {square}");
    }

    /// **Accessibility raises a floor.**
    #[test]
    fn a_larger_scale_raises_the_floor() {
        assert_eq!(at_least(ENHANCED_TARGET, 200), 88);
        assert_eq!(at_least(SMALLEST_TARGET, 150), 36);
    }

    /// **And never lowers it**, which is the way this is violated without
    /// anybody noticing: a person who made their text smaller did not ask for
    /// a target they cannot hit.
    #[test]
    fn a_smaller_scale_never_lowers_a_floor() {
        assert_eq!(at_least(ENHANCED_TARGET, 50), ENHANCED_TARGET);
        assert_eq!(at_least(SMALLEST_TARGET, 1), SMALLEST_TARGET);
        assert_eq!(at_least(ENHANCED_TARGET, 0), ENHANCED_TARGET);
        assert_eq!(at_least(ENHANCED_TARGET, 100), ENHANCED_TARGET);
    }

    /// **An absurd scale does not wrap round to something small.** Saturating
    /// arithmetic rather than a panic, because a display reporting nonsense
    /// should give a person a target far too large rather than no target.
    #[test]
    fn an_absurd_scale_saturates_rather_than_wrapping() {
        assert!(at_least(ENHANCED_TARGET, u32::MAX) >= ENHANCED_TARGET);
    }
}
