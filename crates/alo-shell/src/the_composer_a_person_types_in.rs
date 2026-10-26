//! The Dock's composer as a field a person types in, and the one key that turns
//! what they typed into a request.
//!
//! **Before this the composer was drawn and inert.** `alo-dock` had its width,
//! its height, its corner radius and its prompt — *Find, open, or ask…* — and
//! `crate::a_press_on_the_dock` answered a press on it with
//! `WhatAPressOnTheDockDoes::Nothing`, under a comment saying the field was a
//! surface somebody had to build. Nothing in this repository had ever accepted
//! a keystroke there, which made *the composer is the agent's only way in* a
//! promise with no door behind it.
//!
//! # What this file is and is not
//!
//! It is the field: what has been typed, whether the field has the keyboard, and
//! what one [`crate::composer_keys::ComposerKey`] does to both. It **does not**
//! decide what a request means, which model answers it, or what a task window
//! shows — those are `alo-tasks`' and the binary's. The one thing it hands out
//! is a request somebody finished typing, through
//! [`crate::Server::a_person_asked_alo`].
//!
//! # Why typing is bounded
//!
//! A field with no bound is a field a held key fills memory with, and this one
//! is reachable before anybody has signed in to anything. The bound is on
//! **graphemes** rather than bytes, so it is the same number of characters in
//! every language — a limit counted in bytes would cut a Japanese request to a
//! third of an English one.

use unicode_segmentation::UnicodeSegmentation;

use crate::composer_keys::ComposerKey;

/// The most a person can type into the composer, in graphemes.
///
/// **Generous, because a request is a sentence and may be a paragraph.** *Make
/// a six-slide launch presentation from these two files, keep it clear and
/// concise* is 86, and somebody describing a goal properly will write ten times
/// that. The number exists to stop a stuck key, not to ration what a person may
/// ask for, so it is far above any real request rather than close to one.
pub const THE_MOST_THAT_CAN_BE_TYPED: usize = 4_000;

/// What a key press did to the composer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatTheComposerDid {
    /// Nothing at all.
    Nothing,
    /// What is in the field changed, so it has to be drawn again.
    ItChanged,
    /// **A person finished asking**, and this is what they asked for.
    ///
    /// The field is empty again by the time this is handed out: a request that
    /// stayed in the field after being sent is one a person sends twice.
    TheyAsked(String),
    /// The field gave the keyboard up, with nothing asked.
    ItWasLeft,
}

/// The composer, as a person types in it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TheComposer {
    /// What has been typed and not yet asked for.
    typed: String,
    /// Whether the field has the keyboard.
    has_the_keyboard: bool,
}

impl TheComposer {
    /// An empty composer that nobody has typed in.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// What has been typed so far.
    #[must_use]
    pub fn typed(&self) -> &str {
        &self.typed
    }

    /// Whether anything has been typed.
    ///
    /// **Whitespace alone is nothing.** A person who pressed space and Enter has
    /// not asked for anything, and a task window holding one space is a window
    /// they have to close.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.typed.trim().is_empty()
    }

    /// Whether the field has the keyboard, and so whether it draws a caret.
    #[must_use]
    pub const fn has_the_keyboard(&self) -> bool {
        self.has_the_keyboard
    }

    /// A person pressed on the field, or reached it with the keyboard.
    ///
    /// **What was typed is kept.** Somebody who clicks away to read a document
    /// and clicks back is still writing the same request.
    pub const fn it_took_the_keyboard(&mut self) {
        self.has_the_keyboard = true;
    }

    /// The keyboard went somewhere else.
    pub const fn it_gave_the_keyboard_up(&mut self) {
        self.has_the_keyboard = false;
    }

    /// One key press.
    ///
    /// **A key press with the keyboard elsewhere does nothing**, which is what
    /// stops the Dock's field quietly collecting what somebody is typing into a
    /// window. Law 2's shape applied to a text field: the field acts because a
    /// person chose it, not because a key was available.
    pub fn pressed(&mut self, key: ComposerKey) -> WhatTheComposerDid {
        if !self.has_the_keyboard {
            return WhatTheComposerDid::Nothing;
        }
        match key {
            ComposerKey::Letter(letter) => self.typed_a_letter(letter),
            ComposerKey::NewLine => self.typed_a_letter('\n'),
            ComposerKey::Erase => self.erased(),
            ComposerKey::Ask => self.asked(),
            ComposerKey::Clear => {
                self.typed.clear();
                self.has_the_keyboard = false;
                WhatTheComposerDid::ItWasLeft
            }
            ComposerKey::Nothing => WhatTheComposerDid::Nothing,
        }
    }

    /// Add one letter, if there is room for it.
    fn typed_a_letter(&mut self, letter: char) -> WhatTheComposerDid {
        if self.typed.graphemes(true).count() >= THE_MOST_THAT_CAN_BE_TYPED {
            return WhatTheComposerDid::Nothing;
        }
        self.typed.push(letter);
        WhatTheComposerDid::ItChanged
    }

    /// Take the last grapheme back.
    ///
    /// **A grapheme and not a `char`**, so one backspace takes back one thing a
    /// person sees: an emoji built of several code points, or a letter and its
    /// accent, goes in one press rather than leaving half of itself behind.
    fn erased(&mut self) -> WhatTheComposerDid {
        let mut graphemes = self.typed.graphemes(true);
        let Some(last) = graphemes.next_back() else {
            return WhatTheComposerDid::Nothing;
        };
        let keep = self.typed.len() - last.len();
        self.typed.truncate(keep);
        WhatTheComposerDid::ItChanged
    }

    /// Enter, with something in the field.
    fn asked(&mut self) -> WhatTheComposerDid {
        if self.is_empty() {
            return WhatTheComposerDid::Nothing;
        }
        let asked = std::mem::take(&mut self.typed);
        // **And the keyboard leaves the field**, which the open-an-application
        // path's comment already said asking alo does — and it did not. Kept,
        // the next thing a person typed went back into the Bar instead of the
        // task window they had just asked for: seen on a booted machine on
        // 2026-10-25.
        self.has_the_keyboard = false;
        WhatTheComposerDid::TheyAsked(asked)
    }
}

impl crate::Server {
    /// One key press, while the composer has the keyboard.
    ///
    /// **This is the join**, and the whole point of the file: Enter in the
    /// Dock's field becomes a request recorded on the `Server`, which the
    /// binary drains to build a task. Without it the field and
    /// [`crate::Server::a_person_asked_alo`] were two halves with nothing
    /// between them — the field had no road out and the recorder had no caller.
    ///
    /// Answers whether anything happened, so a caller knows whether to draw
    /// again.
    /// What the alo Bar would open if Enter were pressed now, if anything.
    ///
    /// **The row the keyboard is on, and the first one when it is on none.**
    /// That is what every other system does — Windows' Start and a Mac's
    /// Spotlight both act on the top result until somebody moves off it — and
    /// it is why a person can type three letters and press Enter without ever
    /// touching an arrow.
    ///
    /// [`None`] when nothing was typed or nothing matched, which is when Enter
    /// means what it has always meant: ask alo.
    #[must_use]
    pub fn what_the_bar_would_open(&self) -> Option<String> {
        let matches = crate::the_bar_finds::what_matches(
            self.the_composer.typed(),
            &self.what_this_machine_has,
        );
        matches
            .get(self.which_row_in_the_bar.unwrap_or(0))
            .map(|identifier| (*identifier).clone())
    }

    /// Carry out one key in the composer.
    ///
    /// **Enter opens what the Bar found, and asks alo only when it found
    /// nothing.** `docs/features.md` promises the Bar *works with no model at
    /// all*: application names open applications. A person who typed the name
    /// of something they have installed and pressed Enter has asked for that
    /// thing, and sending it to a model instead would be answering a different
    /// question — and on a machine with no model, answering none.
    pub fn a_key_reached_the_composer(&mut self, key: ComposerKey) -> WhatTheComposerDid {
        // **The field's own guard, repeated here deliberately.** This arm runs
        // *before* `TheComposer::pressed`, which is what refuses every key for
        // a field that does not have the keyboard — so without this an Enter
        // routed here while the Bar is shut would open an application from
        // text nobody is looking at. Tested rather than argued:
        // `enter_does_nothing_when_the_bar_does_not_have_the_keyboard`.
        if key == ComposerKey::Ask
            && self.the_composer.has_the_keyboard()
            && let Some(identifier) = self.what_the_bar_would_open()
            && let Ok(application) = alo_dock::AppId::named(&identifier)
        {
            // **Pushed and not started here**, which is
            // `Server::asked_to_start`'s whole reason: the bus starts an
            // application and that is not fast, and sent from the input path it
            // would hold every window on the machine still while one started.
            self.asked_to_start.push(application);
            // **The field empties and the keyboard leaves it**, which is what
            // asking alo does and for the same reason: a request still in the
            // field is one a person sends twice. `Clear` is the key that means
            // exactly this, so it is the one used rather than a second way of
            // emptying the same field.
            self.which_row_in_the_bar = None;
            // **A task window lets go as well**: launching something is
            // choosing it, so what is typed next belongs to it.
            self.the_task_gave_the_keyboard_up();
            return self.the_composer.pressed(ComposerKey::Clear);
        }
        let what = self.the_composer.pressed(key);
        if let WhatTheComposerDid::TheyAsked(ref request) = what {
            // **The field already emptied itself**, and this records the asking.
            // Two steps rather than one because they belong to different things:
            // the field owns what is in it, and the `Server` owns what is owed
            // to the binary.
            self.a_person_asked_alo(request);
        }
        what
    }

    /// The composer, to read when drawing it.
    #[must_use]
    pub const fn composer(&self) -> &TheComposer {
        &self.the_composer
    }
}

#[cfg(test)]
#[path = "the_composer_a_person_types_in_tests.rs"]
mod tests;
