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
    /// They asked to paste, and the selection's owner has been asked for it.
    ///
    /// **Not `ItChanged`.** Nothing is in the field yet and may never be: the
    /// selection belongs to another client, which writes when it gets round to
    /// it and may not write at all. What lands in the field is
    /// [`Server::that_paste_arrived`]'s business, a frame or several later.
    AskedForThePaste,
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
            // **Asked for here and arriving later**, which is why this is not
            // `ItChanged`: the selection belongs to another client and the
            // field does not change until that client writes. See
            // [`Server::that_paste_arrived`], which the frame loop calls.
            FillingInKey::Paste => {
                self.ask_for_what_is_on_the_clipboard();
                WhatFillingInDid::AskedForThePaste
            }
            FillingInKey::Nothing => WhatFillingInDid::Nothing,
        })
    }

    /// Ask whoever owns the selection to write it down a pipe.
    ///
    /// **Nothing waits here.** The pipe's reading end goes to a thread of its
    /// own (`crate::what_is_on_the_clipboard`), because a client that is busy,
    /// swapped out or hostile can take as long as it likes to write — and a
    /// compositor that blocked on that would hang the whole machine on a press
    /// of Control+V.
    ///
    /// Every way this can fail is **silence**, and that is deliberate rather
    /// than lazy: there is no selection, or it holds no text, or the pipe will
    /// not open. A person who presses paste with nothing copied sees nothing
    /// happen, which is what every other system does and what they already
    /// expect. Saying *there is nothing on your clipboard* would be a sentence
    /// nobody asked for in the middle of typing a key.
    fn ask_for_what_is_on_the_clipboard(&mut self) {
        let Some(keyboard) = self.surfaces.keyboard.as_ref() else {
            return;
        };
        let seat = keyboard.seat.clone();
        let Ok((reading, writing)) = std::io::pipe() else {
            return;
        };
        // **The reading end is handed over before the ask**, so that a client
        // which writes and closes immediately has somewhere for its bytes to
        // go. The other order loses a fast client's paste.
        self.clipboard
            .reading(std::os::unix::io::OwnedFd::from(reading));
        let _ = smithay::wayland::selection::data_device::request_data_device_client_selection(
            &seat,
            crate::what_is_on_the_clipboard::TheClipboard::as_text().to_owned(),
            std::os::unix::io::OwnedFd::from(writing),
        );
    }

    /// Put anything that has been pasted into the field it was asked for in.
    ///
    /// **Asked once a frame and never blocking**, which is
    /// `crate::what_a_person_asked_goes_to_the_agent`'s shape: it answers with
    /// nothing almost every time.
    ///
    /// Answers whether the form changed, so a caller knows whether to draw
    /// again.
    ///
    /// **It goes into whichever field has the keyboard now**, not the one that
    /// had it when the press happened. A person who pressed paste and then
    /// pressed Tab meant the field they are looking at; holding the earlier
    /// one would put a key into an address while they watched.
    pub fn that_paste_arrived(&mut self) -> bool {
        let pasted = self.clipboard.whatever_was_pasted();
        if pasted.is_empty() {
            return false;
        }
        let Some(form) = self.filling_in.as_mut() else {
            // The form closed while a client was being asked. What came back
            // is dropped rather than kept for a later form: a person who gave
            // up and came back has not asked for this again.
            return false;
        };
        let mut changed = false;
        for came in pasted {
            for letter in came.text().chars() {
                changed |= form.typed(letter);
            }
        }
        changed
    }
}

#[cfg(test)]
#[path = "a_provider_is_added_in_settings_tests.rs"]
mod tests;
