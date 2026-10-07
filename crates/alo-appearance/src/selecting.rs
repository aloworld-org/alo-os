//! How a chosen thing is marked, and how that differs from a focused one.
//!
//! One rule, settled by the owner on 2026-10-07 after the design file had
//! carried two: **selected is two logical pixels of navy, plus the word.** The
//! 1.5px edge that used to sit on the first-start access screens was an
//! unfinished edit, and `docs/design/the-first-start.md` records how it was
//! found — a card measures 85 logical pixels with an ordinary edge and 87 with
//! a selected one, so a 1.5px survivor would measure 86, and no card in the
//! refreshed export does.
//!
//! # Why the edge and the ring are separate numbers
//!
//! **Moving focus must never commit a choice.** A person tabbing through four
//! processing destinations has not picked one, and a surface that drew focus
//! the way it draws selection would make arriving at a card indistinguishable
//! from choosing it — on the one screen in the system where the difference is
//! whether somebody's words leave the building.
//!
//! So [`Mark`] carries the two independently, and [`Mark::commits`] answers
//! from the selected half alone. The ring sits **outside** the edge with a gap,
//! which is what lets both be visible at once on a card that is focused and
//! selected together.
//!
//! # Colour never carries this alone
//!
//! The edge is accompanied by the word *Selected*, which translates. Colour
//! alone would fail anybody who cannot see the difference between navy and the
//! ordinary border, and the word is also what a screen reader has.
//!
//! **Teal is not available here.** [ADR 0067](../../../docs/decisions/0067-the-agents-colour-is-deep-teal-and-the-colour-it-vacates-is-given-back.md)
//! reserved deep teal for alo being present or acting, and
//! `crate::words::DEEP_TEAL_IS_NOT_AN_ACCENT` says so to a translator.
//! A selection drawn in teal would say *alo chose this*, on a screen whose
//! entire purpose is that the person chose it. [`marking`] returns
//! [`Token::Navy`] and there is no way to ask this module for another colour.
//!
//! # The unit, and the conversion that happens once
//!
//! **Logical pixels**, like [`crate::targets`], and for the same reason: a
//! display converts them through its own scale, so a dense screen draws the
//! same edge sharper rather than thinner. That conversion happens **once,
//! elsewhere** — `alo_displays::Scale::laid_out`, applied at the rendering
//! boundary — and nothing here repeats it. A surface that multiplied these by
//! the display scale as well would draw a four-pixel edge on a dense screen
//! and call it two.

use crate::token::Token;

/// The ordinary edge of a card nobody has chosen.
pub const ORDINARY_EDGE: u32 = 1;

/// The edge of a chosen card.
///
/// Two logical pixels, navy, **and never without the word.**
pub const SELECTED_EDGE: u32 = 2;

/// The keyboard focus ring, drawn outside the edge.
pub const FOCUS_RING: u32 = 2;

/// The gap between a card and its focus ring.
///
/// The ring is drawn outside the card, so a focused card occupies
/// `FOCUS_GAP + FOCUS_RING` more on each side than an unfocused one. That is
/// room a layout has to leave whether or not anything is focused yet —
/// otherwise the first Tab press moves everything.
pub const FOCUS_GAP: u32 = 4;

/// The one colour a selection is marked in.
///
/// There is deliberately no argument: see this module's header on teal.
#[must_use]
pub const fn marking() -> Token {
    Token::Navy
}

/// Whether a card is focused, chosen, both or neither.
///
/// **The two are independent**, which is the whole reason this is a pair of
/// flags rather than one enum of three states: a card can be focused and
/// selected at the same time, and the design draws both marks on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Mark {
    /// Keyboard focus is here.
    pub focused: bool,
    /// The person has chosen this one.
    pub selected: bool,
}

impl Mark {
    /// Nothing marked, which is how a choice screen arrives.
    ///
    /// ADR 0025 settled that nothing is pre-selected, so this is the state a
    /// person meets rather than a degenerate case.
    #[must_use]
    pub const fn nothing() -> Self {
        Self {
            focused: false,
            selected: false,
        }
    }

    /// This card's edge, in logical pixels.
    ///
    /// ```
    /// use alo_appearance::selecting::{Mark, ORDINARY_EDGE, SELECTED_EDGE};
    ///
    /// assert_eq!(Mark::nothing().edge(), ORDINARY_EDGE);
    /// assert_eq!(Mark::nothing().focused().edge(), ORDINARY_EDGE);
    /// assert_eq!(Mark::nothing().chosen().edge(), SELECTED_EDGE);
    /// ```
    ///
    /// **Focus does not thicken the edge.** It adds a ring outside it, which
    /// is [`Mark::ring`].
    #[must_use]
    pub const fn edge(self) -> u32 {
        if self.selected {
            SELECTED_EDGE
        } else {
            ORDINARY_EDGE
        }
    }

    /// This card's focus ring, or [`None`] when focus is elsewhere.
    #[must_use]
    pub const fn ring(self) -> Option<u32> {
        if self.focused { Some(FOCUS_RING) } else { None }
    }

    /// How much further out this card reaches than its own box, on each side.
    ///
    /// Zero unless focused, and `FOCUS_GAP + FOCUS_RING` when it is.
    #[must_use]
    pub const fn reaches_beyond(self) -> u32 {
        match self.ring() {
            Some(ring) => FOCUS_GAP + ring,
            None => 0,
        }
    }

    /// Whether this mark means the person has chosen this card.
    ///
    /// **Answered from the selected half alone.** This is the method that keeps
    /// *Continue* disabled while somebody is only moving about, and it is
    /// written as its own function so that a change to it is visible in a diff
    /// rather than buried in a surface.
    #[must_use]
    pub const fn commits(self) -> bool {
        self.selected
    }

    /// The same mark, focused.
    #[must_use]
    pub const fn focused(mut self) -> Self {
        self.focused = true;
        self
    }

    /// The same mark, chosen.
    #[must_use]
    pub const fn chosen(mut self) -> Self {
        self.selected = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One rule. The figure the owner settled, where it fails if edited.
    #[test]
    fn a_chosen_card_has_a_two_pixel_edge() {
        assert_eq!(SELECTED_EDGE, 2);
        assert_eq!(ORDINARY_EDGE, 1);
    }

    /// **There is no second selection edge.** The 1.5px one was a leftover,
    /// and these being integers is part of why another cannot creep back:
    /// a fractional edge has nowhere to live in this module.
    #[test]
    fn there_is_exactly_one_edge_for_a_chosen_card() {
        let chosen: Vec<u32> = [Mark::nothing().chosen(), Mark::nothing().chosen().focused()]
            .iter()
            .map(|mark| mark.edge())
            .collect();
        for edge in chosen {
            assert_eq!(edge, SELECTED_EDGE);
        }
    }

    /// Navy, and nothing else is reachable from here.
    #[test]
    fn a_selection_is_marked_in_navy() {
        assert_eq!(marking(), Token::Navy);
        assert_eq!(marking().colour().to_string(), "#102A43");
    }

    /// **Teal means alo acting and may not mean chosen.** ADR 0067. Asserted
    /// rather than left to the absence of a code path, because the absence is
    /// what a later change would remove without noticing.
    #[test]
    fn a_selection_is_never_marked_in_teal() {
        assert_ne!(marking(), Token::DeepTeal);
        assert_ne!(marking().colour(), Token::DeepTeal.colour());
    }

    /// **The thing this module exists to prevent.** Focus moves; nothing is
    /// chosen.
    #[test]
    fn moving_focus_commits_nothing() {
        let arrived = Mark::nothing().focused();
        assert!(arrived.focused);
        assert!(!arrived.commits());
        assert_eq!(arrived.edge(), ORDINARY_EDGE);
    }

    /// And choosing does not require focus to have been anywhere — a pointer
    /// press selects without a ring.
    #[test]
    fn choosing_without_focus_still_commits() {
        let pressed = Mark::nothing().chosen();
        assert!(pressed.commits());
        assert_eq!(pressed.ring(), None);
        assert_eq!(pressed.reaches_beyond(), 0);
    }

    /// **Focused and selected is one card with both marks**, which the design
    /// file does not draw and which the rules settle. Both are visible because
    /// the ring is outside the edge.
    #[test]
    fn a_card_can_be_focused_and_chosen_at_once() {
        let both = Mark::nothing().focused().chosen();
        assert_eq!(both.edge(), SELECTED_EDGE);
        assert_eq!(both.ring(), Some(FOCUS_RING));
        assert!(both.commits());
        assert_eq!(both.reaches_beyond(), FOCUS_GAP + FOCUS_RING);
    }

    /// A screen arrives with nothing marked — ADR 0025.
    #[test]
    fn nothing_is_marked_to_begin_with() {
        let arriving = Mark::nothing();
        assert_eq!(arriving, Mark::default());
        assert!(!arriving.commits());
        assert_eq!(arriving.ring(), None);
        assert_eq!(arriving.edge(), ORDINARY_EDGE);
    }

    /// The ring is outside, so a focused card needs room on each side that an
    /// unfocused one does not.
    #[test]
    fn a_focused_card_reaches_further_than_its_own_box() {
        assert_eq!(Mark::nothing().reaches_beyond(), 0);
        assert_eq!(Mark::nothing().focused().reaches_beyond(), 6);
        assert_eq!(FOCUS_GAP + FOCUS_RING, 6);
    }
}
