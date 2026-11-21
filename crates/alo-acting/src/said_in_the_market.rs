//! **What the shell says on the journal on the road a person installs an
//! application by**: the Dock's slot the keyboard is on, the market's card the
//! keyboard is on, and what became of an installation the person asked for.
//!
//! No walk had ever installed an application (the launch audit of 2026-11-21,
//! item 6): the market opens from one slot on the Dock, its cards are moved
//! through by the arrows, and nothing said on the journal which slot or card
//! the keyboard was on or whether an installation arrived. A walk cannot read
//! that off a picture, and counting presses broke as soon as a row was added
//! (`crate::said_in_settings`' header). So the shell says each, here declared
//! once and read by the walk.
//!
//! # The lines
//!
//! - `alo-shell: on the Dock the keyboard is on "<name>"` — after F6 or Tab
//!   moves it, by the slot's English source word (an application by its id);
//! - `alo-shell: the market opened, listing <n> applications`;
//! - `alo-shell: in the market the keyboard is on "<id>"` — after it opens and
//!   after each key that moves it, by the catalogue's identifier;
//! - `alo-shell: in the market a person asked to install "<id>"`;
//! - `alo-shell: in the market an installation arrived: "<id>"`, or
//!   `alo-shell: in the market an installation was not done: "<id>": <why>`.
//!
//! **Never anything a person typed**: names and identifiers off the Dock and
//! the catalogue, the same on every machine, quoted as Rust quotes a string so
//! nothing in one can start a line of its own.

/// The start of every line that says which Dock slot the keyboard is on.
pub const ON_THE_DOCK_THE_KEYBOARD_IS_ON: &str = "alo-shell: on the Dock the keyboard is on";

/// The start of the line that says the market opened.
pub const THE_MARKET_OPENED: &str = "alo-shell: the market opened, listing";

/// The start of every line that says which market card the keyboard is on.
pub const IN_THE_MARKET_THE_KEYBOARD_IS_ON: &str = "alo-shell: in the market the keyboard is on";

/// The start of the line that says a person pressed Install.
pub const A_PERSON_ASKED_TO_INSTALL: &str = "alo-shell: in the market a person asked to install";

/// The start of the line that says an installation arrived.
pub const AN_INSTALLATION_ARRIVED: &str = "alo-shell: in the market an installation arrived:";

/// The start of the line that says an installation was not done.
pub const AN_INSTALLATION_WAS_NOT_DONE: &str =
    "alo-shell: in the market an installation was not done:";

/// The line saying the Dock's keyboard is on the slot called `name`.
#[must_use]
pub fn on_the_dock_the_keyboard_is_on(name: &str) -> String {
    format!("{ON_THE_DOCK_THE_KEYBOARD_IS_ON} {name:?}")
}

/// The line saying the market opened with `how_many` applications listed.
#[must_use]
pub fn the_market_opened(how_many: usize) -> String {
    format!("{THE_MARKET_OPENED} {how_many} applications")
}

/// The line saying the market's keyboard is on the card for `identifier`.
#[must_use]
pub fn in_the_market_the_keyboard_is_on(identifier: &str) -> String {
    format!("{IN_THE_MARKET_THE_KEYBOARD_IS_ON} {identifier:?}")
}

/// The line saying a person asked for `identifier` to be installed.
#[must_use]
pub fn a_person_asked_to_install(identifier: &str) -> String {
    format!("{A_PERSON_ASKED_TO_INSTALL} {identifier:?}")
}

/// The line saying `identifier` arrived.
#[must_use]
pub fn an_installation_arrived(identifier: &str) -> String {
    format!("{AN_INSTALLATION_ARRIVED} {identifier:?}")
}

/// The line saying `identifier` was not installed, and why, on one line.
#[must_use]
pub fn an_installation_was_not_done(identifier: &str, why: &str) -> String {
    let why = why.replace(['\n', '\r'], " ");
    format!("{AN_INSTALLATION_WAS_NOT_DONE} {identifier:?}: {why}")
}

#[cfg(test)]
#[path = "said_in_the_market_tests.rs"]
mod tests;
