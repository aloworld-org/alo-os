//! A list longer than the screen, and the few of it a person is looking at.
//!
//! Used by both of the environment's long lists — the keyboards, of which
//! there are nine, and the networks, of which there are as many as the room a
//! person is standing in.
//!
//! # It behaves exactly like the one on the installed system, deliberately
//!
//! `alo_shell::the_languages_offered` does this for the twenty-four languages
//! at first start, and every rule below is copied from it rather than decided
//! again:
//!
//! - **the cursor stops at each end**, and does not wrap. A list that wraps
//!   takes a person who is holding the down key past the end and back to the
//!   top, which reads as the list having reset;
//! - **where the list is scrolled to is derived from the cursor**, never kept
//!   beside it, so there is one answer to *where am I* and the two cannot
//!   disagree;
//! - **the cursor sits in the middle where there is room on both sides**, and
//!   the window stops at each end rather than scrolling past it into blank
//!   rows;
//! - **a cursor is not a choice.** Moving through the list chooses nothing.
//!
//! The point of copying rather than deciding is that a person meets this list
//! twice — once in the installer and once at first start — and two lists that
//! behaved differently would be two lists to learn.
//!
//! # What is not here
//!
//! **Searching.** It cannot work on either of these screens, for the reason
//! the language sheet already records: typing presupposes the layout, and on
//! the keyboard screen the layout is the thing being chosen. On the network
//! screen it would be worse than useless — a person cannot search for a name
//! they are reading off the screen for the first time.

/// A list of choices, and where in it a person is.
///
/// How many are visible is passed in rather than kept, because it belongs to
/// the display: [`crate::where_a_list_screen_goes::how_many_cards_fit`]
/// answers it, and it changes with the screen the firmware left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhereInTheList {
    /// How many choices there are altogether.
    all: usize,
    /// Which one the cursor is on.
    on: usize,
}

impl WhereInTheList {
    /// A list of this many, with the cursor on the first.
    #[must_use]
    pub const fn of(all: usize) -> Self {
        Self { all, on: 0 }
    }

    /// How many choices there are altogether.
    #[must_use]
    pub const fn how_many(self) -> usize {
        self.all
    }

    /// Which choice the cursor is on, and [`None`] when the list is empty.
    ///
    /// An empty list is a real state on the network screen — a card with no
    /// firmware finds nothing — and it is answered rather than asserted away,
    /// because a screen drawing a cursor on row nought of nothing would be
    /// drawing a lie.
    #[must_use]
    pub const fn on(self) -> Option<usize> {
        if self.all == 0 { None } else { Some(self.on) }
    }

    /// Move the cursor down one, stopping at the last choice.
    pub const fn down(&mut self) {
        let last = self.all.saturating_sub(1);
        let next = self.on.saturating_add(1);
        self.on = if next < last { next } else { last };
    }

    /// Move the cursor up one, stopping at the first choice.
    pub const fn up(&mut self) {
        self.on = self.on.saturating_sub(1);
    }

    /// Which choice is drawn at the top, given how many are visible.
    ///
    /// Derived from the cursor, so *where is this scrolled to* has one answer.
    #[must_use]
    pub const fn first_one_shown(self, how_many_fit: usize) -> usize {
        let last_start = self.all.saturating_sub(how_many_fit);
        let centred = self.on.saturating_sub(how_many_fit / 2);
        if centred < last_start {
            centred
        } else {
            last_start
        }
    }

    /// Which choices are drawn now, as a range into the whole list.
    #[must_use]
    pub const fn shown(self, how_many_fit: usize) -> std::ops::Range<usize> {
        let from = self.first_one_shown(how_many_fit);
        let to = from.saturating_add(how_many_fit);
        from..if to < self.all { to } else { self.all }
    }

    /// Where in what is drawn the cursor is, for putting the ring on it.
    ///
    /// Always [`Some`] while the list has anything in it, because the window
    /// is derived from the cursor and so contains it by construction.
    #[must_use]
    pub const fn cursor_among_those_shown(self, how_many_fit: usize) -> Option<usize> {
        if self.all == 0 {
            return None;
        }
        self.on.checked_sub(self.first_one_shown(how_many_fit))
    }

    /// Whether there is more list above what is drawn.
    ///
    /// For saying so on the screen. A list that silently continues past the
    /// edge is a list a person does not know to scroll.
    #[must_use]
    pub const fn more_above(self, how_many_fit: usize) -> bool {
        self.first_one_shown(how_many_fit) > 0
    }

    /// Whether there is more list below what is drawn.
    #[must_use]
    pub const fn more_below(self, how_many_fit: usize) -> bool {
        self.first_one_shown(how_many_fit)
            .saturating_add(how_many_fit)
            < self.all
    }
}

#[cfg(test)]
#[path = "which_choices_are_shown_tests.rs"]
mod tests;
