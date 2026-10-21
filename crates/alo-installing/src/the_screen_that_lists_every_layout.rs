//! The screen behind *Something else* on the keyboard question: every layout
//! this computer knows, as a list a person scrolls.
//!
//! # Why it is a second screen and not the first one
//!
//! `crate::words` settles this and it is worth reading before changing
//! anything here. The keyboard question is **two cards**: the layout the
//! environment thinks is right, carrying *Press the keys shown to check*, and
//! *Something else*. The note on the first of those says what the design is
//! doing:
//!
//! > This card asks the keyboard to prove itself rather than asking the person
//! > to recognise a name: somebody who cannot tell one layout from another in
//! > a list can tell whether what appeared matches what is printed on their
//! > key.
//!
//! So the list is **the way out of a wrong guess**, not the question. A person
//! who presses the keys and sees what they expected never comes here at all.
//! Building the list as the first screen would have put eight names in front
//! of everybody and asked the one question the design deliberately avoids.
//!
//! # What this screen is made of
//!
//! Nothing new. The eyebrow, the question and the quiet line are the keyboard
//! screen's own words, because this is the same question asked a second way —
//! and a screen that re-worded them would read as somewhere else in the
//! installer rather than as one step further into the same choice.
//!
//! The layout names are **not** words. They are endonyms — *Belgisch*,
//! *Deutsch*, *Nederlands* — the same in every interface language, and this
//! crate treats them exactly as it treats a disk's own name: data that arrives
//! and is never translated. They are passed in rather than held here, so that
//! whichever crate owns the layouts owns their names too.

use crate::what_display_there_is::Glass;
use crate::where_a_list_screen_goes::{how_many_cards_fit, where_a_list_screen_goes};
use crate::where_a_screen_goes::{AScreensPlaces, AnArea};
use crate::which_choices_are_shown::WhereInTheList;
use crate::words;
use alo_strings::Word;

/// One layout's card, as this screen draws it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ALayoutsCard<'a> {
    /// Where the card goes.
    pub at: AnArea,
    /// The layout's own name for itself, never translated.
    pub called: &'a str,
    /// Whether the keyboard is on this card.
    pub has_the_ring: bool,
}

/// What the full list of layouts shows, and where in it a person is.
#[derive(Debug, Clone)]
pub struct EveryLayoutThisComputerKnows<'a> {
    /// Where the parts of the screen go.
    places: AScreensPlaces,
    /// How many cards this display has room for.
    fit: usize,
    /// Which layout the cursor is on.
    where_in: WhereInTheList,
    /// Every layout's own name, in the order they are offered.
    names: &'a [&'a str],
}

impl<'a> EveryLayoutThisComputerKnows<'a> {
    /// Open the screen on this display, for these layouts.
    ///
    /// [`None`] on a display too small to lay out, which is the same answer
    /// [`where_a_list_screen_goes`] gives and at the same size — asked of it
    /// rather than decided again here.
    #[must_use]
    pub fn opened(
        across: u32,
        down: u32,
        glass: Option<Glass>,
        names: &'a [&'a str],
    ) -> Option<Self> {
        let places = where_a_list_screen_goes(across, down, glass)?;
        let fit = how_many_cards_fit(&places);
        Some(Self {
            places,
            fit,
            where_in: WhereInTheList::of(names.len()),
            names,
        })
    }

    /// Where the parts of the screen go.
    #[must_use]
    pub const fn places(&self) -> &AScreensPlaces {
        &self.places
    }

    /// The quiet label above the question.
    ///
    /// The keyboard screen's own, deliberately: see this module's header.
    ///
    /// Handed back as a [`Word`] rather than said here, which is this
    /// crate's own direction: a screen holds no `Strings`, so it cannot say
    /// anything in the wrong language by accident.
    #[must_use]
    pub const fn eyebrow(&self) -> Word {
        words::BEFORE_ANYTHING_IS_TYPED
    }

    /// The question.
    #[must_use]
    pub const fn title(&self) -> Word {
        words::EVERY_LAYOUT_THIS_COMPUTER_KNOWS
    }

    /// The quiet line under the cards.
    #[must_use]
    pub const fn under_the_cards(&self) -> Word {
        words::IF_IT_DOES_NOT_MATCH
    }

    /// The cards to draw now, in the order they go down the screen.
    #[must_use]
    pub fn cards(&self) -> Vec<ALayoutsCard<'a>> {
        let shown = self.where_in.shown(self.fit);
        let ring = self.where_in.cursor_among_those_shown(self.fit);
        shown
            .clone()
            .enumerate()
            .filter_map(|(down_the_screen, which)| {
                let called = self.names.get(which)?;
                let at = AnArea {
                    across: self.places.first_card.across,
                    down: self.places.first_card.down
                        + (down_the_screen as u32) * self.places.card_to_card,
                    wide: self.places.first_card.wide,
                    tall: self.places.first_card.tall,
                };
                Some(ALayoutsCard {
                    at,
                    called,
                    has_the_ring: ring == Some(down_the_screen),
                })
            })
            .collect()
    }

    /// Whether there is more list above what is drawn, so the screen can say so.
    #[must_use]
    pub const fn more_above(&self) -> bool {
        self.where_in.more_above(self.fit)
    }

    /// Whether there is more list below what is drawn.
    #[must_use]
    pub const fn more_below(&self) -> bool {
        self.where_in.more_below(self.fit)
    }

    /// Move the keyboard down one card.
    pub const fn down(&mut self) {
        self.where_in.down();
    }

    /// Move the keyboard up one card.
    pub const fn up(&mut self) {
        self.where_in.up();
    }

    /// Which layout the keyboard is on, by its own name.
    ///
    /// [`None`] only where no layout was offered at all, which would be a bug
    /// upstream rather than a state this screen puts a person in — and it is
    /// answered rather than asserted, because a screen that named a layout out
    /// of an empty list would be naming one nobody can choose.
    #[must_use]
    pub fn on(&self) -> Option<&'a str> {
        self.names.get(self.where_in.on()?).copied()
    }
}

#[cfg(test)]
#[path = "the_screen_that_lists_every_layout_tests.rs"]
mod tests;
