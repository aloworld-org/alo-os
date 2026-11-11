//! **The field in a window's panel**: *alo · Ask about this window…*, the
//! words a person types there, and where the caret is.
//!
//! The owner's ruling of 2026-11-11 on *Help with this window*: **"I think the
//! user should be able to type in here not to open another window"**
//! (`docs/design/the-window-controls.md`, *How Help is asked next*). So the
//! panel's alo row is a field, and reaching it — by the arrows, or a click —
//! is what focuses it. This file is only the field: what is typed, the caret,
//! and what one [`AFieldKey`](crate::the_window_panels_field::AFieldKey) does to both. Whether the field has the keyboard,
//! and what a sent request becomes, are `crate::a_key_reaches_the_window_field`'s
//! and `crate::alo_in_a_window`'s.
//!
//! # Bounded as the composer is
//!
//! A held key must not fill memory, so what can be typed is bounded in
//! **graphemes** — the same number of characters in every language — by the
//! composer's own bound, [`crate::the_composer_a_person_types_in::THE_MOST_THAT_CAN_BE_TYPED`].
//! The caret moves by grapheme too, so Left never lands inside a letter
//! built of several code points.

use unicode_segmentation::UnicodeSegmentation;

use crate::the_composer_a_person_types_in::THE_MOST_THAT_CAN_BE_TYPED;

/// One key press, as the field understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AFieldKey {
    /// A letter, typed at the caret. Space is a letter here.
    Letter(char),
    /// Take back the grapheme before the caret. Backspace.
    Erase,
    /// Take back the grapheme after the caret. Delete.
    EraseForwards,
    /// The caret one grapheme back. Left.
    Left,
    /// The caret one grapheme on. Right.
    Right,
    /// The caret to the start. Home.
    Start,
    /// The caret to the end. End.
    End,
    /// **Send what is typed to alo, about this window.** Enter.
    Send,
    /// Clear what is typed, or, with nothing typed, close the panel. Escape.
    Escape,
    /// Leave the field for the choice above it. Up.
    Up,
    /// Leave the field for the choice below it. Down.
    Down,
    /// Anything else, which the field does nothing with.
    Nothing,
}

/// What a key did to the field itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatTheFieldDid {
    /// Nothing changed.
    Nothing,
    /// What is typed or where the caret is changed, so it is drawn again.
    ItChanged,
    /// The person asked to send this. **The field is not emptied yet**: the
    /// caller empties it once the request has really gone, so words a person
    /// typed about a window that has just closed are not lost.
    SendThis(String),
    /// Escape with nothing typed: the panel closes.
    CloseThePanel,
    /// Up or Down: the keyboard leaves the field for another choice. What is
    /// typed stays, for coming back to.
    Leave(AFieldKey),
}

/// The words typed into a window panel's field, and the caret among them.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TheWindowField {
    /// What has been typed.
    typed: String,
    /// The caret, as a byte offset into `typed` that is always on a grapheme
    /// boundary.
    caret: usize,
}

impl TheWindowField {
    /// What has been typed.
    #[must_use]
    pub fn typed(&self) -> &str {
        &self.typed
    }

    /// The caret, as a byte offset into [`Self::typed`], on a grapheme
    /// boundary.
    #[must_use]
    pub const fn caret(&self) -> usize {
        self.caret
    }

    /// Empty the field.
    pub fn emptied(&mut self) {
        self.typed.clear();
        self.caret = 0;
    }

    /// The grapheme boundaries of what is typed, start and end included.
    fn boundaries(&self) -> Vec<usize> {
        self.typed
            .grapheme_indices(true)
            .map(|(at, _)| at)
            .chain(std::iter::once(self.typed.len()))
            .collect()
    }

    /// One key.
    pub fn pressed(&mut self, key: AFieldKey) -> WhatTheFieldDid {
        match key {
            AFieldKey::Letter(letter) => {
                if self.typed.graphemes(true).count() >= THE_MOST_THAT_CAN_BE_TYPED {
                    return WhatTheFieldDid::Nothing;
                }
                self.typed.insert(self.caret, letter);
                // A combining mark joins the grapheme before it, so the caret
                // goes to the first boundary at or after the inserted letter.
                let after = self.caret + letter.len_utf8();
                self.caret = self
                    .boundaries()
                    .into_iter()
                    .find(|at| *at >= after)
                    .unwrap_or(self.typed.len());
                WhatTheFieldDid::ItChanged
            }
            AFieldKey::Erase => {
                let Some(before) = self
                    .boundaries()
                    .into_iter()
                    .rev()
                    .find(|at| *at < self.caret)
                else {
                    return WhatTheFieldDid::Nothing;
                };
                self.typed.replace_range(before..self.caret, "");
                self.caret = before;
                WhatTheFieldDid::ItChanged
            }
            AFieldKey::EraseForwards => {
                let Some(after) = self.boundaries().into_iter().find(|at| *at > self.caret) else {
                    return WhatTheFieldDid::Nothing;
                };
                self.typed.replace_range(self.caret..after, "");
                WhatTheFieldDid::ItChanged
            }
            AFieldKey::Left => {
                let to = self
                    .boundaries()
                    .into_iter()
                    .rev()
                    .find(|at| *at < self.caret);
                self.caret_to(to)
            }
            AFieldKey::Right => {
                let to = self.boundaries().into_iter().find(|at| *at > self.caret);
                self.caret_to(to)
            }
            AFieldKey::Start => self.caret_to((self.caret != 0).then_some(0)),
            AFieldKey::End => {
                let end = self.typed.len();
                self.caret_to((self.caret != end).then_some(end))
            }
            AFieldKey::Send => {
                // **Nothing to send is nothing sent**: Enter on an empty field,
                // or one of spaces, asks alo nothing.
                let request = self.typed.trim();
                if request.is_empty() {
                    WhatTheFieldDid::Nothing
                } else {
                    WhatTheFieldDid::SendThis(request.to_owned())
                }
            }
            AFieldKey::Escape => {
                // **Escape dismisses the deepest layer first**
                // (`docs/design/the-window-controls.md` §9): the words, then
                // the panel.
                if self.typed.is_empty() {
                    WhatTheFieldDid::CloseThePanel
                } else {
                    self.emptied();
                    WhatTheFieldDid::ItChanged
                }
            }
            AFieldKey::Up | AFieldKey::Down => WhatTheFieldDid::Leave(key),
            AFieldKey::Nothing => WhatTheFieldDid::Nothing,
        }
    }

    /// Move the caret to `to`, where there was somewhere to move to.
    fn caret_to(&mut self, to: Option<usize>) -> WhatTheFieldDid {
        match to {
            Some(at) => {
                self.caret = at;
                WhatTheFieldDid::ItChanged
            }
            None => WhatTheFieldDid::Nothing,
        }
    }
}

#[cfg(test)]
#[path = "the_window_panels_field_tests.rs"]
mod tests;
