//! **The four fields a person fills in, as words Settings already knows how to
//! draw.**
//!
//! The last thing between a person and a machine that answers them. Everything
//! under this existed and none of it was on a screen: the form, the keys that
//! type into it, the clipboard, both writes.
//!
//! # A section, not a new kind of drawing
//!
//! `crate::settings_raster` lays out sections of `SectionWords`, moves the view
//! to keep the focused line in it, draws a rule between sections and an edge
//! around the focused row. All of that is right for a form, so a form is made
//! out of the same words rather than drawn beside them — which means the view
//! scrolls, the focus ring lands and nothing is cut, with no code here at all.
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

use crate::settings_lines::{LineWords, SectionWords};
use crate::settings_window::SettingsSection;

/// What one hidden character is drawn as.
///
/// **A bullet, which is what every other system uses**, so a person who has
/// typed a password anywhere else already knows what they are looking at.
const A_HIDDEN_LETTER: char = '•';

/// The form as the section Settings draws.
///
/// `which` is the field with the keyboard; it is the one a caret belongs in
/// and the one the focus ring goes round.
pub(crate) fn the_form(form: &AProviderBeingAddded, strings: &Strings) -> SectionWords {
    let which = form.which_field();
    SectionWords {
        // **The section it belongs under**, so a person reads it where they
        // were: this is how their questions get answered, and the list of what
        // can answer them is what this form is adding to.
        section: SettingsSection::Answering,
        said: vec![
            said(strings, &crate::the_providers_words::ADDING_A_PROVIDER),
            said(strings, &crate::the_providers_words::WHAT_TO_DO_WITH_IT),
        ],
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
) -> LineWords {
    let typed = form.what_is_in(field);
    LineWords {
        // **No row.** `SettingsRow` names a row a person acts on with Enter,
        // and Enter in a form saves the whole of it rather than acting on the
        // line the keyboard is in. A row here would put the form's one action
        // on four lines.
        row: None,
        parts: vec![label(field, strings), shown(typed, field, which)],
        // **The field with the keyboard is marked as chosen**, which is the
        // one piece of state this section has and the one `settings_raster`
        // already draws without being told anything new.
        chosen: field == which,
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
