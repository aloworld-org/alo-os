//! **What happens when a person presses save on a provider they filled in**:
//! the key goes to the keyring, and the settings file says it answers them.
//!
//! Two writes with two owners, joined here because only a binary sees both —
//! which is `crate::what_this_machine_offers`'s reason for existing in this
//! directory, and the same shape.
//!
//! | | |
//! |---|---|
//! | `alo_secrets::TheKeyring::keep` | where the key goes |
//! | `alo_choosing::Choosing::adding` | the provider on the person's own list |
//! | `alo_choosing::Choosing::setting_up` | and that it is what answers them |
//!
//! `alo-shell` does none of it and must not: that crate *shows rather than
//! measures*, and `alo-models` is a dev-dependency of it with the note **never
//! a dependency of the shipped surface, which only draws.** A screen asks for
//! this to happen; this is what happens.
//!
//! # The key is written first, and that order is the whole of what can go wrong
//!
//! Neither write can carry the other, so one of them lands first and the
//! machine can be interrupted between them. Both orders leave something
//! behind, and they are not equally bad:
//!
//! - **key first, settings second** — a key in the keyring that nothing refers
//!   to. Nothing reads it, nothing is slower, and the next save under the same
//!   name replaces it. A person sees a provider that was not added, which is
//!   true, and adding it again works.
//! - **settings first, key second** — a settings file naming a provider whose
//!   key is not there. The machine then *looks like* it is set up and fails at
//!   the moment a person asks it something, with `NotStored::Missing` — and
//!   the obvious thing for them to do about it, adding the key, is the thing
//!   they just did.
//!
//! So the key goes first. This is not an atomicity claim: it is a choice about
//! which half-written machine is easier to understand and to put right.
//!
//! # Nothing is undone on failure, and that is deliberate
//!
//! A key written and then a settings write that failed leaves the key. Taking
//! it back out would be a second write that can also fail, on the way out of a
//! failure, and the thing it would protect against is a stored credential that
//! nothing refers to. `alo_choosing::Choosing` already works this way for its
//! own two writes and says so: *each write is whole or not at all; what a
//! refusal leaves behind is a provider on their list and an unanswered setup,
//! which is true of the machine and is the state they can act on.*

use alo_choosing::{Choosing, NoProvider, NotSet, NotWritten};
use alo_models::ProviderError;
use alo_secrets::{NotStored, TheKeyring};
use alo_setting_up::a_provider_is_filled_in::WhatToSave;

/// Why a provider a person filled in was not saved.
///
/// **Each names which of the three writes did not happen**, because what a
/// person should do about it differs: a locked keyring is theirs to unlock, a
/// settings file that will not read is theirs to look at, and a provider their
/// list refused is a name they can change.
///
/// **`Debug` only.** `alo_choosing::NotWritten` is neither `Clone` nor
/// `PartialEq`, and making this one would mean either cloning a refusal or
/// inventing an equality for somebody else's type. A caller tells these apart
/// by matching, which is what the three names are for.
#[derive(Debug)]
pub enum NotSaved {
    /// The key did not reach the keyring. **Nothing else was attempted**, so
    /// the settings file is exactly as it was.
    KeyNotKept(NotStored),
    /// The settings file is there and would not read.
    ///
    /// **Its own name rather than one of `NotWritten`'s.** The first version
    /// of this reported it as `NoSuchProvider`, which is a refusal about a
    /// name on a list — a sentence that would have sent a person to look at
    /// their provider's name when what is wrong is the file. Borrowing
    /// somebody else's refusal to stand in for your own is how a machine tells
    /// a person to fix the wrong thing.
    ///
    /// Nothing is written over it, which is `alo_choosing::Choosing`'s rule
    /// and ADR 0038's reason: a surface that read a typo as *nothing chosen*
    /// and then wrote would take away the edit that was about to fix it.
    SettingsWouldNotRead(NotSet),
    /// The key was kept and the provider would not go on the person's list.
    ProviderNotListed(NotWritten),
    /// The key was kept, the provider was added, and the choice was not
    /// written — so the machine knows the provider and has not been told it
    /// answers.
    ChoiceNotWritten(NotWritten),
    /// The answer named a provider with no name.
    ///
    /// **Unreachable while `WhatToSave` comes from a checked provider**, whose
    /// name cannot be empty — and handled rather than asserted, because the
    /// alternative is a panic in a settings screen on somebody else's
    /// invariant.
    ChoiceNamedNothing(NoProvider),
}

impl NotSaved {
    /// Which of the writes did not happen, for the service log.
    ///
    /// **Not for a person to read.** Every sentence a person reads belongs to
    /// the crate that owns the thing it is about, and these carry refusals
    /// from three different crates; turning them into one English sentence
    /// here would be this file wording somebody else's refusal. What this is
    /// for is whoever opens `journalctl` after a provider did not save, and it
    /// says which of four things to go and look at.
    ///
    /// It also makes the reasons **read** rather than only carried. Clippy
    /// noticed they were not, and it was right in a way worth keeping: a
    /// refusal that nothing ever looks at is a refusal that could be wrong for
    /// a year without anybody finding out.
    #[must_use]
    pub fn what_did_not_happen(&self) -> String {
        match self {
            Self::KeyNotKept(why) => {
                format!("the key did not reach the keyring, and nothing else was tried: {why:?}")
            }
            Self::SettingsWouldNotRead(why) => format!(
                "the key was kept, and this person's settings file is there and would not \
                 read, so nothing was written over it: {why:?}"
            ),
            Self::ProviderNotListed(why) => {
                format!("the key was kept and the provider would not go on their list: {why:?}")
            }
            Self::ChoiceNotWritten(why) => format!(
                "the key was kept and the provider is on their list, and it was not written \
                 down as what answers them: {why:?}"
            ),
            Self::ChoiceNamedNothing(why) => format!(
                "the answer named a provider with no name, which should not be reachable from \
                 a checked provider: {why:?}"
            ),
        }
    }
}

/// Save a provider a person filled in: its key, then its place in settings.
///
/// `at` is the person's own settings file, which is
/// `alo_choosing::where_it_is`'s answer.
///
/// # Errors
///
/// [`NotSaved`], naming which of the three writes did not happen.
pub fn a_provider_a_person_added_is_saved(
    keyring: &TheKeyring,
    at: &std::path::Path,
    saving: &WhatToSave,
) -> Result<(), NotSaved> {
    // **First, and the file above says why.**
    keyring
        .keep(&saving.filed_under, &saving.key)
        .map_err(NotSaved::KeyNotKept)?;

    // **Opened after the key is kept, not before.** `Choosing::at` reads the
    // person's settings, and reading them earlier would widen the window in
    // which another surface could write between the read and the write.
    let mut choosing = Choosing::at(at).map_err(NotSaved::SettingsWouldNotRead)?;

    // **Added before chosen, which is `Choosing::setting_up`'s own rule**: a
    // choice naming a provider is a reference into the person's own list, so
    // the provider has to be on it before anything can refer to it.
    //
    // **And changed rather than refused when the name is already theirs.**
    // This was found by a test and is a real thing a person does: they paste a
    // key, it is wrong, and they come back and paste the right one. `adding`
    // answers `AlreadyAdded` for that, which would have meant **a person who
    // mistypes their key once can never fix it** — the only way out being a
    // settings file they would have to find and edit by hand.
    //
    // `Choosing::changing` is the door for it and already exists. Which of the
    // two to use is decided by what the list says, not by a flag the screen
    // passes: a screen that had to know would be a second place that can be
    // wrong about what is on the person's list.
    //
    // **What this decides on their behalf**, said plainly: saving a provider
    // under a name they already use replaces that one. The alternative is
    // refusing and telling them to pick another name, which for the common
    // case — one provider, a corrected key — is the machine arguing with
    // somebody fixing a typo.
    let provider = saving.provider.clone();
    match choosing.adding(provider.clone()) {
        Ok(()) => {}
        Err(NotWritten::NotAProvider {
            why: ProviderError::AlreadyAdded(_),
            ..
        }) => choosing
            .changing(provider)
            .map_err(NotSaved::ProviderNotListed)?,
        Err(why) => return Err(NotSaved::ProviderNotListed(why)),
    }

    // **And the answer is built here rather than passed in**, so that the
    // provider written to the list and the provider named in the choice are
    // one value. Two call sites building it separately is how a machine ends
    // up choosing a provider it did not add.
    let answer = alo_setting_up::Answer::FromAProvider {
        provider: saving.provider.clone(),
        model: saving.model.clone(),
    };
    let picked = answer.chosen().map_err(NotSaved::ChoiceNamedNothing)?;
    choosing
        .setting_up(picked)
        .map_err(NotSaved::ChoiceNotWritten)
}

#[cfg(test)]
#[path = "a_provider_a_person_added_is_saved_tests.rs"]
mod tests;
