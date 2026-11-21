//! **What the shell says on the journal about where Settings' keyboard is, and
//! the names it says it by.**
//!
//! A walk cannot read a row's name off a picture, and counting rows broke as
//! soon as a provider brought a row of its own (main's walk at 9ced5c40, run
//! 44851000278: five Downs landed on *alo*, not *Add a provider*). So the shell
//! says, each time Settings' keyboard moves, the name of the row or field it is
//! on — the name a screen reader is told, which is the word the screen draws —
//! and a walk presses until it is the one it wants.
//!
//! **One source for names.** The names a walk finds are declared here, once:
//! the shell's own words (`alo_shell`'s `the_providers_words`) draw these very
//! strings, and the walk reads them from here. A row renamed is renamed for
//! both, or the shell's test that the two agree fails.
//!
//! **Only names, never what was typed.** In the form that adds a provider the
//! keyboard is on a *field*, and the field is said by its label.

/// The start of every line that says where Settings' keyboard is.
pub const IN_SETTINGS_THE_KEYBOARD_IS_ON: &str = "alo-shell: in Settings the keyboard is on";

/// The row that opens the form to add a provider, on *alo and AI*.
pub const ADD_A_PROVIDER: &str = "Add a provider";

/// The form's field for what to call the provider.
pub const NAME: &str = "Name";

/// The form's field for the provider's web address.
pub const ADDRESS: &str = "Address";

/// The form's field for the provider's secret key.
pub const KEY: &str = "Key";

/// The form's field for which model to ask for.
pub const MODEL: &str = "Model";

/// The form's switch for a service on this computer that needs no key.
pub const NEEDS_NO_KEY: &str = "This service needs no key";

/// The line saying the keyboard is on the row or field called `name`, the name
/// quoted so a newline in it cannot start a line of its own.
#[must_use]
pub fn the_keyboard_is_on(name: &str) -> String {
    format!("{IN_SETTINGS_THE_KEYBOARD_IS_ON} {name:?}")
}

#[cfg(test)]
#[path = "said_in_settings_tests.rs"]
mod tests;
