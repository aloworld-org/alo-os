//! **The four fields a person fills in, as words Settings already knows how to
//! draw.**
//!
//! The last thing between a person and a machine that answers them. Everything
//! under this existed and none of it was on a screen: the form, the keys that
//! type into it, the clipboard, both writes.
//!
//! # A section, not a new kind of drawing
//!
//! `crate::settings_page::the_form_page` turns these words into a page of
//! plain rows, which `crate::settings_raster` lays out and scrolls like every
//! other page — so nothing is cut, with no drawing code here at all. **The
//! field with the keyboard is the one with the caret in it**, which is the only
//! mark it has: since the redesign of 2026-10-31 a plain row has no filled
//! square to draw.
//!
//! **It replaces the rows rather than joining them.** A person filling a form
//! in is doing one thing, and the rows behind would be rows they cannot reach:
//! every key is the form's while one is open (`crate::a_key_reaches_settings`),
//! so drawing them would be drawing a list that does not respond.
//!
//! # The key is shown as marks, and the rule is asked rather than counted
//!
//! `WhichField::is_a_secret` decides it. A surface that masked by position
//! would show a key the day somebody adds a field above it — which is why that
//! question lives on the field and not here.
//!
//! **One mark per grapheme, not per byte.** A key of 164 characters drawn as
//! 164 bullets says the same thing to a person as the key does: it is long,
//! and it is there. Counting bytes would draw a different number for the same
//! key depending on what is in it.
//!
//! # Nothing here is worded
//!
//! Every label is a sentence from `alo-saying`, like every other row in
//! Settings. What a person typed is their own and is never translated.

use alo_setting_up::a_provider_is_filled_in::{AProviderBeingAddded, WhichField};
use alo_strings::{Filling, Strings};

/// The form's words: what it says above its fields, and the fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheFormsWords {
    /// What the form says above its fields.
    pub(crate) said: Vec<String>,
    /// Its fields, in order.
    pub(crate) lines: Vec<AFieldsWords>,
}

/// One field: its label, then what is in it.
///
/// **No [`crate::settings_window::SettingsRow`].** A row names something a
/// person acts on with Enter, and Enter in a form saves the whole of it rather
/// than acting on the field the keyboard is in. A row here would put the form's
/// one action on four lines, so a field has none to give.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AFieldsWords {
    /// The pieces of text, in reading order: the label, then what is in it.
    pub(crate) parts: Vec<String>,
}

/// What one hidden character is drawn as.
///
/// **A bullet, which is what every other system uses**, so a person who has
/// typed a password anywhere else already knows what they are looking at.
const A_HIDDEN_LETTER: char = '•';

/// The form as the section Settings draws.
///
/// `which` is the field with the keyboard; it is the one a caret belongs in
/// and the one the focus ring goes round.
pub(crate) fn the_form(
    form: &AProviderBeingAddded,
    not_saved: Option<crate::a_provider_is_added_in_settings::WhyNotSaved>,
    strings: &Strings,
) -> TheFormsWords {
    let which = form.which_field();
    // **The refusal goes above the fields rather than below them.** Somebody
    // who pressed Enter is looking at the top of the form for what happened,
    // and a sentence under the last field is one they reach after deciding
    // nothing happened at all — which is what a machine told a person on
    // 2026-10-23: the save ran, the keyring refused, and the screen did not
    // change.
    let mut lines_said = vec![
        said(strings, &crate::the_providers_words::ADDING_A_PROVIDER),
        said(strings, &crate::the_providers_words::WHAT_TO_DO_WITH_IT),
    ];
    // **Said while the address is this machine's own, before anything is
    // saved**, and saving still works (owner's decision, 2026-10-29; ADR
    // 0021). The indicator cannot show what a local service does with a
    // question, so the person is told so at the one moment they can still
    // choose another address.
    if form.its_address_is_this_machine() {
        lines_said.push(said(
            strings,
            &crate::the_providers_words::THIS_MACHINE_CANNOT_SEE_WHERE_IT_GOES,
        ));
    }
    // **And while it is plain http to another computer on the person's own
    // network** (owner's decision, 2026-10-30; ADR 0021): such a provider can
    // be saved, and its questions and key cross that network unencrypted. The
    // person reads that here, before Enter, so saving is a choice made knowing
    // its cost. The address of this machine's own, over http, gets both: by
    // the time a question is asked that address may be another computer's.
    if form.its_address_crosses_the_network_unencrypted() {
        lines_said.push(said(
            strings,
            &crate::the_providers_words::OTHERS_ON_THIS_NETWORK_COULD_READ_IT,
        ));
    }
    if let Some(why) = not_saved {
        lines_said.push(said(strings, &why.the_word()));
    }
    TheFormsWords {
        said: lines_said,
        lines: WhichField::EVERY
            .into_iter()
            .map(|field| one_field(form, field, which, strings))
            .collect(),
    }
}

/// One field, as a label and what is in it.
fn one_field(
    form: &AProviderBeingAddded,
    field: WhichField,
    which: WhichField,
    strings: &Strings,
) -> AFieldsWords {
    let typed = form.what_is_in(field);
    AFieldsWords {
        parts: vec![label(field, strings), shown(typed, field, which)],
    }
}

/// What is drawn to the right of a field's label.
///
/// A secret is marks; everything else is what they typed. An empty field with
/// the keyboard shows the caret alone, so a person can see where they are
/// before they have typed anything.
fn shown(typed: &str, field: WhichField, which: WhichField) -> String {
    let body = if field.is_a_secret() {
        // One mark per grapheme. See this file's header for why not per byte.
        unicode_segmentation::UnicodeSegmentation::graphemes(typed, true)
            .map(|_| A_HIDDEN_LETTER)
            .collect()
    } else {
        typed.to_owned()
    };
    if field == which {
        // **The caret is a character and not a shape**, because this section
        // is words and `settings_raster` draws words. A shape would be a
        // second kind of drawing for one line of one surface.
        format!("{body}▏")
    } else {
        body
    }
}

/// What a field is called, in this person's language.
fn label(field: WhichField, strings: &Strings) -> String {
    let word = match field {
        WhichField::Name => &crate::the_providers_words::ITS_NAME,
        WhichField::Address => &crate::the_providers_words::ITS_ADDRESS,
        WhichField::Key => &crate::the_providers_words::ITS_KEY,
        WhichField::Model => &crate::the_providers_words::ITS_MODEL,
    };
    said(strings, word)
}

/// One sentence, looked up.
fn said(strings: &Strings, word: &alo_strings::Word) -> String {
    strings.say(&word.key(), &Filling::nothing()).into_text()
}

#[cfg(test)]
#[path = "a_provider_being_added_is_drawn_tests.rs"]
mod tests;
