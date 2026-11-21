//! **Typing that saves is for a document, not a one-line field** (the owner,
//! 2026-11-20).
//!
//! `accessible.type_text` types and then saves. In a search box or an address
//! bar that save is the browser's, and saves the whole web page; typing into a
//! field is `accessible.type_without_saving`'s. So `type_text` aimed at a
//! one-line field is refused — when it runs ([`crate::Typing::the_field`]) and,
//! through [`a_one_line_field_to_save`], **before anybody is asked**, so no
//! card asks a person to approve a save that should not happen.
//!
//! Which places are one line is [`crate::places_for_text::is_one_line`]'s,
//! written once. **This refuses only what it saw**: one place of that name,
//! that typing would otherwise go into, and that the tree says is one line. A
//! window it could not read, a name meaning none or several, or a place typing
//! refuses for another reason is left to the typing itself, which says why
//! when it runs. Nothing is typed or pressed here — the walk reads.

use alo_strings::{Filling, Said, Strings};

use crate::accessibility_tree::AccessibilityTree;
use crate::fallback_verbs::TYPE_WITHOUT_SAVING;
use crate::fallback_words as fallback;
use crate::places_for_text::{
    is_one_line, the_ones_meant, the_places_for_text, why_typing_refuses,
};
use crate::typing_words as words;

/// The refusal for typing that saves into `name` in `application`, a one-line
/// field — in the same words before the card and when it runs.
#[must_use]
pub fn a_one_line_field_said(application: &str, name: &str, strings: &Strings) -> Said {
    strings.say(
        &words::A_ONE_LINE_FIELD.key(),
        &Filling::of(fallback::APPLICATION, application.to_owned())
            .and(fallback::NAME, name.to_owned())
            .and(fallback::VERB, TYPE_WITHOUT_SAVING.to_owned()),
    )
}

/// [`a_one_line_field_said`] where `tree` shows exactly one place `name` means
/// in `application`, typing would go into it, and it is one line; [`None`]
/// wherever that is not seen.
#[must_use]
pub fn a_one_line_field_to_save(
    tree: &dyn AccessibilityTree,
    application: &str,
    name: &str,
    strings: &Strings,
) -> Option<Said> {
    let places = the_places_for_text(tree, application).ok()?;
    let meant = the_ones_meant(&places, name);
    let [one] = meant.as_slice() else {
        return None;
    };
    // The order `Typing::the_field` asks in: a place it refuses for another
    // reason is refused in that reason's words when it runs.
    if why_typing_refuses(tree, &one.at, one.role, &one.states).is_some() {
        return None;
    }
    is_one_line(one.role, &one.states).then(|| a_one_line_field_said(application, name, strings))
}
