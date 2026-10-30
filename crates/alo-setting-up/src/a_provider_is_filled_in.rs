//! **What a person types to add a provider**, before any of it is drawn.
//!
//! The one thing standing between somebody with an API key and a machine that
//! answers them. `alo_setting_up::Answer::FromAProvider` has modelled *a
//! provider the person is adding here* since that crate was written, and
//! `alo_choosing::Choosing::setting_up` has written both halves of the settings
//! file — **and nothing ever drove either**, because Settings takes no typing
//! at all. It has `Previous`, `Next`, `Choose` and `Close`, and no way to enter
//! a character.
//!
//! So `crates/alo-shell/src/settings_answering.rs` says in its own words what
//! a person is left with: *adding a provider, bringing weights and pairing are
//! flows of their own … and none is reachable from this list.* Only *Not at
//! all* is always offered. That is why asking alo does nothing on a fresh
//! machine.
//!
//! # A value, and no drawing
//!
//! The same shape `alo-setting-up` took, in its own words: *decided without
//! drawing one, so that the drawing is the compositor lane's and the rules are
//! testable now*. Everything here can be tested by typing into it and reading
//! back what it would save — no compositor, no font, no display.
//!
//! # Four fields, and why each is asked
//!
//! | | |
//! |---|---|
//! | the name | what they will see when it answers, and what the key is filed under |
//! | the address | where to ask. `Provider::checked` refuses one that is not `https://`, unless it is this machine |
//! | the key | what the provider gave them. Held as [`alo_models::Secret`], which exists for exactly this moment |
//! | the model | what to ask for, *exactly as they wrote it*, because a provider's model names are the provider's |
//!
//! **The region is not asked.** `Provider` carries one *as whoever added it
//! stated*, and a person pasting a key has not stated anything — so it is
//! [`Region::Unknown`], which that type's own documentation calls a fact rather
//! than a failure. Inventing one here would be this screen claiming to know
//! where somebody else's service runs.
//!
//! # The key is typed into a `Secret` and never into a `String`
//!
//! `alo_models::Secret`'s header: *a key as it was just typed, for the length
//! of one call … it exists because of a single moment: **a provider is tested
//! before it is saved**, so at that moment the key is not in the keyring yet.*
//! That moment is this one. What this holds is the characters as they arrive,
//! and [`AProviderBeingAddded::what_it_would_save`] is where they become a
//! `Secret` — because a field a person is still typing into has to be able to
//! go backwards, and a `Secret` deliberately cannot.
//!
//! **So the characters do live in this struct while the person types**, and
//! saying so is better than implying otherwise: there is no way to accept
//! typing without holding what was typed. What is avoided is the key becoming
//! a value that travels — it is turned into a `Secret` at the edge and never
//! returned, logged or rendered. Its `Debug` is written by hand to say
//! nothing, for the reason `Secret`'s is.

use alo_models::source::Region;
use alo_models::{Provider, ProviderError, Secret, SecretError, SecretRef};
use unicode_segmentation::UnicodeSegmentation;

/// Which of the four fields a person is typing into.
///
/// **In the order they are asked**, which is the order a person has the
/// answers in: they know what they are adding, then where it is, then the key
/// they were given, then which model they want.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhichField {
    /// What they call it.
    #[default]
    Name,
    /// Where to ask.
    Address,
    /// The key the provider gave them.
    Key,
    /// What to ask for.
    Model,
}

impl WhichField {
    /// Every field, in the order they are asked.
    pub const EVERY: [Self; 4] = [Self::Name, Self::Address, Self::Key, Self::Model];

    /// The field after this one, or [`None`] at the last.
    ///
    /// **[`None`] rather than wrapping to the first.** Tab at the end of a form
    /// that jumps back to the top is a person losing their place, and the thing
    /// they most likely wanted was to save.
    #[must_use]
    pub const fn after(self) -> Option<Self> {
        match self {
            Self::Name => Some(Self::Address),
            Self::Address => Some(Self::Key),
            Self::Key => Some(Self::Model),
            Self::Model => None,
        }
    }

    /// The field before this one, or [`None`] at the first.
    #[must_use]
    pub const fn before(self) -> Option<Self> {
        match self {
            Self::Name => None,
            Self::Address => Some(Self::Name),
            Self::Key => Some(Self::Address),
            Self::Model => Some(Self::Key),
        }
    }

    /// Whether what is typed here is a credential.
    ///
    /// **Asked rather than assumed by position**, because the one thing a
    /// drawing must get right about this form is which field it must not show
    /// in clear — and a surface that decided that by counting rows would show a
    /// key the day somebody adds a field above it.
    #[must_use]
    pub const fn is_a_secret(self) -> bool {
        matches!(self, Self::Key)
    }
}

/// What a person has typed so far, and which field they are in.
#[derive(Clone, Default)]
pub struct AProviderBeingAddded {
    /// What they call it.
    name: String,
    /// Where to ask.
    address: String,
    /// The key, as it has arrived so far.
    key: String,
    /// What to ask for.
    model: String,
    /// Which field the keyboard is in.
    which: WhichField,
}

/// **Says that somebody is adding a provider, and nothing about what they
/// typed.**
///
/// Written by hand for `alo_models::Secret`'s reason: a form holding a
/// credential must not be the thing that puts one in a log because somebody
/// derived `Debug` on the struct that holds it.
impl std::fmt::Debug for AProviderBeingAddded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AProviderBeingAddded")
            .field("which", &self.which)
            .finish_non_exhaustive()
    }
}

/// Why what a person typed is not yet a provider they can be asked with.
///
/// **Each says what to fix**, which is `ProviderError`'s own rule, and each is
/// its own name so that a surface can put the mark on the field that is wrong
/// rather than on the form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotYetAProvider {
    /// The name, the address or the kind of address. `alo-models`' own words.
    TheProvider(ProviderError),
    /// The key is empty, or is not one a key could be.
    TheKey(SecretError),
    /// No model was named.
    ///
    /// **Its own refusal rather than an empty string passed on.** A provider
    /// asked for a model called nothing answers with its own error, over the
    /// network, about something this machine knew before it asked.
    NoModel,
}

impl NotYetAProvider {
    /// Which field a surface should put the mark on.
    #[must_use]
    pub const fn which_field(&self) -> WhichField {
        match self {
            Self::TheProvider(ProviderError::Unnamed) => WhichField::Name,
            Self::TheProvider(_) => WhichField::Address,
            Self::TheKey(_) => WhichField::Key,
            Self::NoModel => WhichField::Model,
        }
    }
}

/// What a filled-in form becomes: a provider, its key, and the model wanted.
///
/// **The key is beside the provider rather than in it.** `Provider` holds a
/// `SecretRef` — where a key lives — and is written to a settings file; the key
/// itself goes to the keyring. Keeping them apart here is what stops a caller
/// writing the second where the first belongs.
pub struct WhatToSave {
    /// The provider, checked by the crate that owns what one is.
    pub provider: Provider,
    /// Its key, for `alo_secrets::TheKeyring::keep`.
    pub key: Secret,
    /// Where that key is to be filed, which is what the provider refers to.
    pub filed_under: SecretRef,
    /// The model to ask for, exactly as the person wrote it.
    pub model: String,
}

/// **Says what is to be saved, and nothing about the key.**
impl std::fmt::Debug for WhatToSave {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WhatToSave")
            .field("provider", &self.provider.name)
            .field("filed_under", &self.filed_under.as_str())
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

impl AProviderBeingAddded {
    /// A form nobody has typed into, with the keyboard in the first field.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Which field the keyboard is in.
    #[must_use]
    pub const fn which_field(&self) -> WhichField {
        self.which
    }

    /// Put the keyboard in this field.
    pub const fn the_keyboard_is_in(&mut self, which: WhichField) {
        self.which = which;
    }

    /// Move to the next field, if there is one after this.
    ///
    /// Answers whether it moved, so a caller knows whether to draw again.
    pub const fn to_the_next_field(&mut self) -> bool {
        match self.which.after() {
            Some(next) => {
                self.which = next;
                true
            }
            None => false,
        }
    }

    /// Move to the field before, if there is one.
    pub const fn to_the_field_before(&mut self) -> bool {
        match self.which.before() {
            Some(before) => {
                self.which = before;
                true
            }
            None => false,
        }
    }

    /// What is in this field, as a person would read it.
    ///
    /// **The key included, which is deliberate and is the surface's problem
    /// rather than this one's.** A form that would not say what is in its own
    /// key field could not offer *show it* — and a person who has pasted a key
    /// that will not work needs to be able to look at it. [`WhichField::is_a_secret`]
    /// is how a drawing knows to replace it with marks by default.
    #[must_use]
    pub fn what_is_in(&self, which: WhichField) -> &str {
        match which {
            WhichField::Name => &self.name,
            WhichField::Address => &self.address,
            WhichField::Key => &self.key,
            WhichField::Model => &self.model,
        }
    }

    /// Add this character to the field the keyboard is in.
    ///
    /// Answers whether anything changed.
    pub fn typed(&mut self, letter: char) -> bool {
        // **A control character is not typing.** A key a person pastes out of a
        // terminal can carry one, and a provider name holding an escape is a
        // name nothing will match and nobody can see is wrong.
        if letter.is_control() {
            return false;
        }
        let which = self.which;
        self.field_mut(which).push(letter);
        true
    }

    /// Take back the last thing typed in this field.
    ///
    /// **A grapheme and not a `char`**, which is `the_composer_a_person_types_in`'s
    /// rule and for its reason: one backspace takes back one thing a reader
    /// calls a letter, and for most of the world's scripts that is not one
    /// `char`. A provider called `नमस्ते` is not an invented case — it is the
    /// i18n law reaching a place that is not a translated string.
    ///
    /// Answers whether anything changed.
    pub fn rubbed_out(&mut self) -> bool {
        let which = self.which;
        let field = self.field_mut(which);
        let Some(last) = field.graphemes(true).next_back() else {
            return false;
        };
        let keep = field.len() - last.len();
        field.truncate(keep);
        true
    }

    /// Whether every field a provider needs has something in it.
    ///
    /// **A question a surface asks to know whether to offer saving**, and it is
    /// deliberately not the same as [`Self::what_it_would_save`] succeeding: a
    /// person can have filled in four fields and typed an address that is not
    /// one. This answers *are they finished typing*, and that answers *is it
    /// right*.
    #[must_use]
    pub fn every_field_is_filled(&self) -> bool {
        WhichField::EVERY
            .iter()
            .all(|which| !self.what_is_in(*which).trim().is_empty())
    }

    /// Whether the address typed so far is this machine's own.
    ///
    /// **For a warning, not a refusal.** A service at this machine's address is
    /// asked there and its question goes on no road out, so the egress
    /// indicator has nothing to show — and if that service forwards the
    /// question to the internet, nothing in alo OS can see it go (ADR 0021,
    /// owner's note of 2026-10-29). A surface says so before saving, and the
    /// person may still save: it is their service on their machine (law 5).
    ///
    /// Asked of [`alo_models::address::is_on_this_machine`], the rule the
    /// daemon uses to send a question there, so the warning appears for exactly
    /// the addresses whose answers will say *a service at this machine's
    /// address*.
    #[must_use]
    pub fn its_address_is_this_machine(&self) -> bool {
        alo_models::address::is_on_this_machine(self.address.trim())
    }

    /// What this form would save, or what is still wrong with it.
    ///
    /// **Nothing is written here.** This is the form's answer, and a caller
    /// takes it to the keyring and to `alo_choosing::Choosing::setting_up` —
    /// which is what makes this testable without either.
    ///
    /// The key is filed under `provider/<name>`, which is the shape
    /// `alo-agentd`'s own live-provider test uses and what `look_up` searches
    /// for. The name is the person's, so two providers they called the same
    /// thing are one provider to this machine — which is also what `Providers`
    /// decides, so the two cannot disagree.
    ///
    /// # Errors
    ///
    /// [`NotYetAProvider`], naming the field to put the mark on.
    pub fn what_it_would_save(&self) -> Result<WhatToSave, NotYetAProvider> {
        let model = self.model.trim();
        if model.is_empty() {
            return Err(NotYetAProvider::NoModel);
        }
        let key = Secret::typed(&self.key).map_err(NotYetAProvider::TheKey)?;
        let filed_under = SecretRef::named(&format!("provider/{}", self.name.trim()));
        // **Checked last of the three, because its refusals are about the two
        // fields above and it is the only one that can rename what it is
        // given.** `Provider::checked` trims the name and strips a trailing
        // slash from the address, so what it hands back is what is saved —
        // and the reference above is built from the same trimmed name.
        let provider = Provider::checked(
            &self.name,
            &self.address,
            Region::Unknown,
            Some(filed_under.clone()),
        )
        .map_err(NotYetAProvider::TheProvider)?;
        // **The provider carries the model the person named**, and until
        // 2026-10-23 it did not: `Provider::checked` leaves `models` empty and
        // the model travelled beside the provider in `WhatToSave` instead of
        // in it. The provider was then written to the person's own list
        // knowing no models at all.
        //
        // What that cost is the whole point of filling this form in.
        // `alo_shell::settings_answering` offers **a model** under *my
        // provider* — `provider.models`, flattened — and *one of the four with
        // nothing under it is absent rather than drawn*. So a person added a
        // provider, the key was kept, the choice was written, and Settings
        // showed them no row to choose and no way to change their mind.
        // Measured on a booted machine: the save succeeded, and the answering
        // section still read *nothing is chosen yet to answer questions*.
        //
        // **The person naming one is the only thing this machine knows.**
        // Nothing here asks a provider what it offers — that is a request to
        // the wire and `alo_asking::Vetting` is the only door to it — so the
        // list is exactly what they typed, and it is one model long.
        let provider = Provider {
            models: vec![model.to_owned()],
            ..provider
        };
        Ok(WhatToSave {
            provider,
            key,
            filed_under,
            model: model.to_owned(),
        })
    }

    /// The field the keyboard is in, to be written into.
    fn field_mut(&mut self, which: WhichField) -> &mut String {
        match which {
            WhichField::Name => &mut self.name,
            WhichField::Address => &mut self.address,
            WhichField::Key => &mut self.key,
            WhichField::Model => &mut self.model,
        }
    }
}

#[cfg(test)]
#[path = "a_provider_is_filled_in_tests.rs"]
mod tests;
