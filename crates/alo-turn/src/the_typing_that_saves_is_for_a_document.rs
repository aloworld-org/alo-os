//! **Typing that saves, aimed at a one-line field, is refused before the
//! card** (the owner, 2026-11-20).
//!
//! `accessible.type_text` types and then saves; in a search box the save is
//! the browser's, and saves the web page. So the window is read once here,
//! **after** the grants have allowed the application and before the card, and
//! a one-line field is refused in the words the typing itself would say when
//! it runs (`alo_adapters::a_one_line_field_to_save`) — words that name
//! `accessible.type_without_saving`, which the model reads, as it reads every
//! refusal before asking, and may use instead.
//!
//! Only `accessible.type_text` is asked here, and never for a terminal the
//! person allowed: that is refused before any tree is read
//! (`alo_adapters::Typing::of`). A window that cannot be read, or a name that
//! means no one place, is not a refusal: the card is asked, and the typing
//! looks again when it runs. At full access there is no card, and the typing
//! refuses the same field in the same words before it types anything.

use alo_adapters::fallback_verbs::{APPLICATION, NAME};
use alo_adapters::{TYPE_TEXT, a_one_line_field_to_save};
use alo_capability::{Call, Refused, Value};

use crate::machine::Machine;

/// Refuse `call` where it types and saves into a field that the window, read
/// now, shows is one line; pass it otherwise.
///
/// # Errors
/// [`Refused`], carrying the call, in the words the typing would have said.
pub(crate) fn the_typing_that_saves_is_for_a_document(
    call: &Call,
    machine: &mut Machine<'_>,
) -> Result<(), Refused> {
    if call.verb() != TYPE_TEXT {
        return Ok(());
    }
    let (Some(Value::Application(application)), Some(Value::Name(name))) =
        (call.value(APPLICATION), call.value(NAME))
    else {
        return Ok(());
    };
    if alo_capability::is_a_persons_own(application) {
        return Ok(());
    }
    let strings = machine.strings();
    let Some(windows) = machine.the_windows_on_the_screen() else {
        return Ok(());
    };
    let Ok(tree) = windows.opening() else {
        return Ok(());
    };
    match a_one_line_field_to_save(tree.as_ref(), application, name, strings) {
        Some(said) => Err(Refused::worded_elsewhere(call.clone(), said)),
        None => Ok(()),
    }
}
