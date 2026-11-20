//! **A move proposed for a slider the window does not show is refused before
//! the card** (2026-11-17).
//!
//! *What a person approves is that sentence* (`CLAUDE.md`), and an approval
//! should mean the thing can happen. Asked to *set the Zoom slider to 200*, a
//! model names the control "Zoom slider" where Loupe calls it "Zoom"; until
//! this, the card *move “Zoom slider” in Loupe to 200* was shown, approved, and
//! only then refused when the move ran. So the window is read once here,
//! **after** the grants have allowed the application and before the card, and
//! a control it does not show is refused in the move's own words
//! (`alo_adapters::not_there_to_set`). The model reads the refusal, as it reads
//! every refusal before asking, and may name the control again.
//!
//! Only `accessible.set_value` is asked here. A window that cannot be read is
//! not a refusal: the card is asked, and the move looks again when it runs.

use alo_adapters::{SET_VALUE, not_there_to_set};
use alo_capability::{Call, Refused, Value};

use crate::machine::Machine;

/// Refuse `call` where it moves a slider or number box that the window, read
/// now, does not show by that name; pass it otherwise.
///
/// # Errors
/// [`Refused`], carrying the call, in the words the move would have said.
pub(crate) fn the_control_to_set_is_there(
    call: &Call,
    machine: &mut Machine<'_>,
) -> Result<(), Refused> {
    if call.verb() != SET_VALUE {
        return Ok(());
    }
    let (Some(Value::Application(application)), Some(Value::Name(name)), Some(Value::Count(value))) = (
        call.value(alo_adapters::fallback_verbs::APPLICATION),
        call.value(alo_adapters::fallback_verbs::NAME),
        call.value(alo_adapters::fallback_verbs::VALUE),
    ) else {
        return Ok(());
    };
    let strings = machine.strings();
    let Some(windows) = machine.the_windows_on_the_screen() else {
        return Ok(());
    };
    let Ok(tree) = windows.opening() else {
        return Ok(());
    };
    match not_there_to_set(tree.as_ref(), application, name, *value, strings) {
        Some(said) => Err(Refused::worded_elsewhere(call.clone(), said)),
        None => Ok(()),
    }
}
