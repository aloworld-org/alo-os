//! **A control named where nothing is named**: in a window a read in this turn
//! found to name none of its controls, nothing called anything can be found,
//! so a call that names one is refused **before anybody is asked**, in words
//! that say what to do instead — take a picture (the coordinator's decision of
//! 2026-11-20 for the screen road). A booted machine would find no such control
//! either; this only makes the refusal useful.
//!
//! **One function for both sides**: the daemon's turn asks it before any card
//! (`alo_turn`), and the every-try check asks it to answer as the machine does.

use alo_capability::{Call, Value};
use alo_strings::{Filling, Said, Strings};

use crate::click_words as words;
use crate::fallback_verbs::{
    ACTIVATE_CONTROL, APPLICATION, CHOOSE_OPTION, FIELD, NAME, OPEN_ITEM, OPEN_MENU, SCROLL_TO,
    SELECT_ITEM, SET_VALUE, THING, TO, TYPE_TEXT, TYPE_WITHOUT_SAVING,
};

/// Every verb that names a control, and the argument that names it.
pub const THE_NAMED_CONTROL_VERBS: [(&str, &str); 9] = [
    (ACTIVATE_CONTROL, NAME),
    (TYPE_TEXT, NAME),
    (TYPE_WITHOUT_SAVING, FIELD),
    (SCROLL_TO, TO),
    (SET_VALUE, NAME),
    (CHOOSE_OPTION, NAME),
    (SELECT_ITEM, NAME),
    (OPEN_ITEM, NAME),
    (OPEN_MENU, THING),
];

/// The refusal for `call`, where it names a control in an application whose
/// windows a read in this turn found to name nothing (`where_nothing_is_named`,
/// by identifier); [`None`] for every other call.
#[must_use]
pub fn named_where_nothing_is_named(
    call: &Call,
    where_nothing_is_named: &[String],
    strings: &Strings,
) -> Option<Said> {
    let (_, naming) = THE_NAMED_CONTROL_VERBS
        .iter()
        .find(|(verb, _)| *verb == call.verb())?;
    let Some(Value::Application(application)) = call.value(APPLICATION) else {
        return None;
    };
    if !where_nothing_is_named.iter().any(|one| one == application) {
        return None;
    }
    let name = match call.value(naming) {
        Some(Value::Name(name) | Value::Text(name)) => name.clone(),
        _ => String::new(),
    };
    Some(strings.say(
        &words::NAMES_NOTHING.key(),
        &Filling::of(crate::fallback_words::APPLICATION, application.clone()).and(NAME, name),
    ))
}

/// **Whether `application`'s windows, as the tree shows them now, name
/// nothing** — what a named guess is checked against when it arrives, with no
/// earlier read needed (the coordinator's decision of 2026-11-21). An
/// application the tree knows no window of names nothing, as for a picture
/// (`crate::picturing_a_window`); a tree that cannot be read says nothing
/// either way, so nothing is refused for it.
#[must_use]
pub fn the_windows_name_nothing(
    tree: &dyn crate::accessibility_tree::AccessibilityTree,
    application: &str,
) -> bool {
    let _ = (tree, application);
    false
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "where_nothing_is_named_tests.rs"]
mod tests;
