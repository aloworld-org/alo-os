//! **What `alo-access` decided, in the numbers the rented tree publishes.**
//!
//! `docs/autonomy/v0-5-the-shell-plan.md` task 12: *the shell exposes every
//! surface's role, name and state to AT-SPI as `alo-access` decides them*. The
//! deciding is that crate's; the numbers are at-spi2's — `AtspiRole` and
//! `AtspiStateType`, whose values are part of a published interface and do not
//! change (ADR 0011: the tree is rented, configured, never patched). This file
//! is the one place the two meet, so a role decided there and a number sent on
//! the bus cannot drift apart anywhere else.
//!
//! # Nothing is preselected, in the states as well as on the screen
//!
//! ADR 0001's *nothing is chosen for the person* has to survive the crossing on
//! to the bus. A reader announces `IS_DEFAULT` as *default*, `FOCUSED` as
//! *focused* and `CHECKED` as *on*, so a surface whose answers arrived carrying
//! any of the three would be telling somebody who cannot see the screen that a
//! choice had been made for them. **Nothing this file describes carries any of
//! them**, and the test reads that back off the bus rather than out of here.
//!
//! # Announced when it changes is an attribute, not a state
//!
//! AT-SPI has no *announce me* state: what a reader acts on is the `live`
//! attribute, the same one the web platform spells `aria-live`. So the two
//! indicators the first law makes visible — something is leaving, the agent is
//! working — are described with `live` set, and everything else has no
//! attributes at all.

use alo_access::{Role, State};

/// `ATSPI_ROLE_APPLICATION`: what this machine's own shell is, at the root of
/// its tree.
pub(crate) const APPLICATION: u32 = 75;

/// `ATSPI_ROLE_FILLER`: the place a surface's controls hang when `alo-access`
/// decided no container for them — the window controls are two buttons and no
/// box around them. A filler has no name, which is honest: it is not a control
/// anybody decided, it is where the decided ones are.
pub(crate) const FILLER: u32 = 20;

/// The `AtspiRole` number for a role `alo-access` decided.
///
/// A window is a **frame**, the role a top-level window of an application has,
/// rather than `ROLE_WINDOW`, which toolkits use for the popups and menus
/// inside one.
pub(crate) const fn number_of(role: Role) -> u32 {
    match role {
        // FRAME
        Role::Window => 23,
        // PUSH_BUTTON
        Role::Button => 43,
        // LABEL
        Role::Label => 29,
        // ENTRY
        Role::Entry => 79,
        // PASSWORD_TEXT
        Role::PasswordEntry => 40,
        // STATUS_BAR
        Role::StatusBar => 54,
        // SWITCH
        Role::Switch => 130,
        // LIST
        Role::List => 31,
        // LIST_ITEM
        Role::ListItem => 32,
        // DIALOG
        Role::Dialogue => 16,
    }
}

/// Whether a role is one other controls are read inside.
///
/// What it decides is where a surface's controls hang: a surface whose first
/// control is one of these is that control, with the rest inside it, and a
/// surface whose first control is not gets a [`FILLER`] with no name.
pub(crate) const fn holds_others(role: Role) -> bool {
    matches!(
        role,
        Role::Window | Role::Dialogue | Role::StatusBar | Role::List
    )
}

/// `ATSPI_STATE_ENABLED`.
const ENABLED: u32 = 8;
/// `ATSPI_STATE_FOCUSABLE`.
const FOCUSABLE: u32 = 11;
/// `ATSPI_STATE_SENSITIVE`.
const SENSITIVE: u32 = 24;
/// `ATSPI_STATE_SHOWING`.
const SHOWING: u32 = 25;
/// `ATSPI_STATE_VISIBLE`.
const VISIBLE: u32 = 30;
/// `ATSPI_STATE_CHECKABLE`.
const CHECKABLE: u32 = 41;
/// `ATSPI_STATE_READ_ONLY`.
const READ_ONLY: u32 = 43;

/// `ATSPI_STATE_FOCUSED` — never sent by this crate, and named so that the
/// test holding it to that has something to name.
#[cfg(test)]
pub(crate) const FOCUSED: u32 = 12;
/// `ATSPI_STATE_IS_DEFAULT` — never sent by this crate.
#[cfg(test)]
pub(crate) const IS_DEFAULT: u32 = 39;
/// `ATSPI_STATE_CHECKED` — never sent by this crate.
#[cfg(test)]
pub(crate) const CHECKED: u32 = 4;

/// The `live` attribute, for what is announced rather than looked for.
pub(crate) const LIVE: (&str, &str) = ("live", "assertive");

/// What a reader is told about a control, on the screen and nowhere else:
/// `SHOWING` as well as `VISIBLE`, because a thing a reader can reach and
/// nobody can see is a thing it will read out of nowhere.
const ON_THE_SCREEN: u64 = (1 << VISIBLE) | (1 << SHOWING) | (1 << ENABLED);

/// The two words the tree answers `GetState` with, low word first, for a state
/// `alo-access` decided.
///
/// `SENSITIVE` and `FOCUSABLE` follow `Control::can_be_used` exactly — the same
/// answer that decides where the keyboard stops (`alo_access::focus_order`), so
/// what a reader is told it can act on and where Tab goes are one decision.
pub(crate) const fn words_of(state: State) -> [u32; 2] {
    let set = match state {
        State::ReadOnly | State::AnnouncedWhenItChanges => ON_THE_SCREEN | (1 << READ_ONLY),
        State::CanBeUsed => ON_THE_SCREEN | (1 << SENSITIVE) | (1 << FOCUSABLE),
        State::OnOrOff => ON_THE_SCREEN | (1 << SENSITIVE) | (1 << FOCUSABLE) | (1 << CHECKABLE),
    };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "each half of a 64-bit set of states is one 32-bit word, by construction"
    )]
    let words = [set as u32, (set >> 32) as u32];
    words
}

/// The same states, off the screen.
///
/// **A surface that is not up is described and not shown.** A reader is still
/// told it exists — a person can be told what this machine has — but it does
/// not read as something in front of them, which is what `SHOWING` means and
/// why it is not the same bit as `VISIBLE`.
pub(crate) const fn off_the_screen(words: [u32; 2]) -> [u32; 2] {
    [words[0] & !(1 << SHOWING), words[1]]
}

/// Whether a state is one a reader announces the moment it changes.
pub(crate) const fn is_announced(state: State) -> bool {
    matches!(state, State::AnnouncedWhenItChanges)
}

#[cfg(test)]
#[path = "access_roles_tests.rs"]
mod tests;
