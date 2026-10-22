//! **A person fills in a provider inside Settings**, and the keys they press
//! go to the form rather than to the rows behind it.
//!
//! This is the link that made the other five real. `alo-setting-up` holds the
//! form, `alo-desktop` holds both writes, `crate::a_key_fills_in_a_provider`
//! reads a press as a field would — and until this file nothing opened a form,
//! nothing routed a press to one, and nothing asked for it to be saved.
//!
//! # The mode is the form's own existence
//!
//! There is no flag. A form is open or there is no form, and
//! [`Server::a_provider_is_being_filled_in`] answers which — so a press cannot
//! be read against a mode that disagrees with what is on the screen. The press
//! itself carries both readings already (`SettingsPress`), taken from one
//! translation of one key, so choosing between them here costs nothing and
//! cannot answer about a different key.
//!
//! # Escape leaves Settings open
//!
//! `SettingsKey::Close` closes the window and `FillingInKey::GiveUp` closes
//! the form. The same press means the second while a form is open, which is
//! why the window never sees it: a person getting out of a half-typed key has
//! not asked to leave Settings.
//!
//! # What is not here
//!
//! **Paste.** `FillingInKey::Paste` arrives and nothing happens yet, and that
//! is said out loud rather than left to be found: the selection belongs to
//! whichever client owns it, reading it is a conversation over the bus, and
//! nothing in alo OS has ever read one. Until it is built, **a person cannot
//! put a real API key in this form** — 164 characters is not something anybody
//! types. The road is `smithay`'s `request_data_device_client_selection`,
//! which this tree vendors and nothing calls.

use crate::Server;
use crate::a_key_fills_in_a_provider::FillingInKey;
use alo_setting_up::a_provider_is_filled_in::{AProviderBeingAddded, WhichField};

/// What came of one press while a form was open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatFillingInDid {
    /// The form changed and should be drawn again.
    ItChanged,
    /// Nothing happened.
    Nothing,
    /// They asked for it to be saved, and every field was filled.
    ///
    /// **The caller does the saving**, because saving is two writes to the
    /// machine and this crate shows rather than measures — see
    /// `crate::TheDesktop::a_provider_was_filled_in`.
    SaveIt,
    /// They asked for it to be saved and something is still wrong.
    ///
    /// The form stays open with what they typed still in it, which is the
    /// whole point: a form that cleared itself on a refusal would take away a
    /// 164-character key because an address was missing a colon.
    NotFinished,
    /// They gave up. The form is gone and Settings is still open.
    GaveUp,
    /// They asked to paste, and nothing can do that yet.
    ///
    /// **Its own answer rather than `Nothing`**, so that a surface can say so
    /// and a test can see the difference between a press that meant nothing
    /// and one that meant something this machine cannot do.
    CannotPasteYet,
}

impl Server {
    /// Start filling in a provider, on an empty form.
    ///
    /// **Replaces whatever was being filled in**, which is what a person
    /// pressing *add a provider* means. There is no second form.
    pub fn a_provider_is_added(&mut self) {
        self.filling_in = Some(AProviderBeingAddded::empty());
    }

    /// The form a person is filling in, if they are.
    #[must_use]
    pub const fn a_provider_is_being_filled_in(&self) -> Option<&AProviderBeingAddded> {
        self.filling_in.as_ref()
    }

    /// Which field has the keyboard, if a form is open.
    ///
    /// For the drawing: it decides which field carries the caret, and
    /// [`WhichField::is_a_secret`] decides which is shown as marks.
    #[must_use]
    pub fn the_field_being_filled_in(&self) -> Option<WhichField> {
        self.filling_in
            .as_ref()
            .map(AProviderBeingAddded::which_field)
    }

    /// Give up on the form, leaving Settings as it was.
    pub fn that_provider_is_not_being_added(&mut self) {
        self.filling_in = None;
    }

    /// Carry one press out against the form, and say what came of it.
    ///
    /// Answers [`None`] when no form is open, which is the caller's signal to
    /// read the press as a row's instead.
    pub fn a_key_fills_in_the_provider(&mut self, key: FillingInKey) -> Option<WhatFillingInDid> {
        let form = self.filling_in.as_mut()?;
        Some(match key {
            FillingInKey::Typed(letter) => {
                if form.typed(letter) {
                    WhatFillingInDid::ItChanged
                } else {
                    WhatFillingInDid::Nothing
                }
            }
            FillingInKey::RubOut => {
                if form.rubbed_out() {
                    WhatFillingInDid::ItChanged
                } else {
                    WhatFillingInDid::Nothing
                }
            }
            FillingInKey::NextField => {
                if form.to_the_next_field() {
                    WhatFillingInDid::ItChanged
                } else {
                    WhatFillingInDid::Nothing
                }
            }
            FillingInKey::FieldBefore => {
                if form.to_the_field_before() {
                    WhatFillingInDid::ItChanged
                } else {
                    WhatFillingInDid::Nothing
                }
            }
            // **Asked of the form rather than of what it would save**, because
            // these are two questions and the file that owns them says so: one
            // is *have they finished typing*, the other is *is it right*. A
            // caller that checked only the second would refuse a person who
            // had filled nothing in with a sentence about an address.
            FillingInKey::Save => {
                if form.every_field_is_filled() {
                    WhatFillingInDid::SaveIt
                } else {
                    WhatFillingInDid::NotFinished
                }
            }
            FillingInKey::GiveUp => {
                self.filling_in = None;
                WhatFillingInDid::GaveUp
            }
            FillingInKey::Paste => WhatFillingInDid::CannotPasteYet,
            FillingInKey::Nothing => WhatFillingInDid::Nothing,
        })
    }
}

#[cfg(test)]
#[path = "a_provider_is_added_in_settings_tests.rs"]
mod tests;
