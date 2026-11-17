//! **Whether a picture of an application's windows may be taken**, decided
//! from what its accessibility tree shows — the screen road's last-resort rule
//! ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md)
//! §1) and the *Pictures and keys* switch's promise.
//!
//! - **A window that describes itself is read, never pictured.** alo's
//!   strength is reading an application by what it says
//!   (`accessible.read_window`); a picture is for the windows nothing better
//!   reaches. A thing describes itself when it shows text, or names itself and
//!   is something a person reads or uses — not the frame, the layout around it,
//!   a picture, or the one area an application draws itself, which SDL and most
//!   game engines name after the game.
//! - **A window with a password field is never pictured**, at any setting: the
//!   switch promises *it never takes a picture of a window with a password
//!   field*. This outranks the first, because no setting changes it.
//! - **No window on the screen, nothing to picture.**
//!
//! **What this can know, and what it cannot.** It can know a password field the
//! application describes. A window that describes nothing could draw one the
//! tree never mentions, and nothing on this machine can see that before a
//! picture is taken — so the record of every picture says it was checked by the
//! accessibility tree, and the person's switch says the same before it is on.

use crate::role::Role;
use crate::shown::{Contents, Shown};

/// Why no picture is taken of an application's windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoPicture {
    /// Its tree describes what its windows show, so alo reads it instead.
    ItDescribesItself,
    /// One of its windows has a password field.
    APasswordField,
    /// None of its windows is on the screen.
    NoWindow,
}

/// Whether a picture may be taken of the windows `shown`.
///
/// # Errors
/// [`NoPicture`]: no window first, then a password field before describing
/// itself, because no setting changes the password rule.
pub fn a_picture_may_be_taken(shown: &Shown) -> Result<(), NoPicture> {
    if shown.windows().is_empty() {
        return Err(NoPicture::NoWindow);
    }
    let seen = || shown.windows().iter().flat_map(|window| window.seen());
    if seen().any(|one| one.role() == Role::PasswordField) {
        return Err(NoPicture::APasswordField);
    }
    let describes = seen().any(|one| {
        matches!(one.contents(), Contents::Text(_))
            || (!one.name().is_empty() && !says_nothing_by_its_name(one.role()))
    });
    if describes {
        Err(NoPicture::ItDescribesItself)
    } else {
        Ok(())
    }
}

/// Roles whose name says nothing about what is in a window.
const fn says_nothing_by_its_name(role: Role) -> bool {
    matches!(role, Role::Window | Role::Part | Role::Canvas | Role::Image)
}

#[cfg(test)]
#[path = "a_picture_may_be_taken_tests.rs"]
mod tests;
