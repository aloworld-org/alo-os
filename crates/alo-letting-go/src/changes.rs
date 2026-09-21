//! How far back a person asked their machine to keep what an agent changed.
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s first
//! accepted term: *undo reaches back a bounded window — the last seven days or
//! the last fifty changing turns, whichever ends first — and what falls outside
//! it is removed by the machine. **The window is one value in `alo-keeping-up`,
//! read from the person's settings where they change it.***
//!
//! That sentence has two halves in two crates, deliberately.
//! [`alo_keeping_up::HowFarBack`] is the value and all of the arithmetic — what
//! a window is, what it still reaches, and that a window reaching nothing is
//! refused. This is the **file** it is read from, which `alo-keeping-up` may not
//! have: nothing in that crate reads a clock, opens a socket or touches a disk,
//! and a test there reads its dependency list to keep it so.
//!
//! # One setting, and it is the only way to keep a snapshot longer
//!
//! There is no second lever. No *keep this one for ever*, no *never expire*, no
//! per-folder exception — ADR 0045's seventh term is explicit that *the only way
//! to keep a snapshot longer is to widen the window, which is the person's
//! setting and always was.* A machine with two ways to reprieve a snapshot is a
//! machine where nobody can answer *when will this be gone*.
//!
//! # What a file that is not there means
//!
//! The person has changed nothing, and the machine keeps what the release ships
//! — seven days or fifty changing turns. That is [`Changes::untouched`] and
//! [`Settings::shipped`], and it is `alo-kept`'s second clause rather than
//! anything decided here.

use alo_keeping_up::HowFarBack;
use serde::{Deserialize, Serialize};

/// What the person changed about how far back an undo reaches.
///
/// Only the difference: a machine nobody has configured writes no file at all,
/// and this value is what such a file reads as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Changes {
    /// How far back they asked it to reach, when they asked for something other
    /// than what alo OS ships.
    #[serde(skip_serializing_if = "Option::is_none")]
    window: Option<HowFarBack>,
}

/// How far back this machine keeps what an agent changed, for one person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// The window itself.
    pub window: HowFarBack,
}

impl Changes {
    /// A person who has changed nothing.
    #[must_use]
    pub const fn untouched() -> Self {
        Self { window: None }
    }

    /// This person asked for `window` instead of what alo OS ships.
    #[must_use]
    pub const fn with_window(window: HowFarBack) -> Self {
        Self {
            window: Some(window),
        }
    }

    /// What they changed it to, or [`None`] when they have not.
    #[must_use]
    pub const fn window(&self) -> Option<HowFarBack> {
        self.window
    }
}

impl Settings {
    /// What the release ships: [`alo_keeping_up::HowFarBack::AS_SHIPPED`].
    #[must_use]
    pub const fn shipped() -> Self {
        Self {
            window: HowFarBack::AS_SHIPPED,
        }
    }

    /// The release's settings with this person's change over them.
    #[must_use]
    pub const fn with(self, changes: &Changes) -> Self {
        match changes.window() {
            Some(window) => Self { window },
            None => self,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **What the machine keeps for somebody who has changed nothing is what
    /// alo OS ships**, and that is `alo-keeping-up`'s value rather than a
    /// second copy of it here.
    #[test]
    fn a_person_who_changed_nothing_gets_what_alo_os_ships() {
        let settings = Settings::shipped().with(&Changes::untouched());
        assert_eq!(settings.window, HowFarBack::AS_SHIPPED);
        assert_eq!(settings.window.days(), 7);
        assert_eq!(settings.window.turns(), 50);
    }

    /// **A person's own window is what the machine uses**, wider or narrower.
    #[test]
    fn a_persons_own_window_is_what_the_machine_uses() {
        for window in [
            HowFarBack::of(30, 200).unwrap(),
            HowFarBack::of(1, 1).unwrap(),
        ] {
            let settings = Settings::shipped().with(&Changes::with_window(window));
            assert_eq!(settings.window, window);
        }
    }

    /// **A window that reaches nothing never becomes a setting**, because
    /// `HowFarBack` refuses one on the way in and again when it is read back
    /// off a disk — undo switched off by arithmetic, quietly, is what that
    /// refusal exists for.
    #[test]
    fn a_window_that_reaches_nothing_is_not_a_setting() {
        assert!(HowFarBack::of(0, 50).is_err());
        assert!(HowFarBack::of(7, 0).is_err());
        assert!(
            toml::from_str::<Changes>("[window]\ndays = 0\nturns = 50\n").is_err(),
            "a window of no days was read as a setting"
        );
    }

    /// **Nothing but the window is on this file**, so a person cannot switch
    /// expiry off or reprieve one snapshot by hand: a key nobody declared is
    /// refused whole.
    #[test]
    fn nothing_but_the_window_is_on_this_file() {
        assert!(toml::from_str::<Changes>("never-expire = true\n").is_err());
        assert_eq!(
            toml::from_str::<Changes>("[window]\ndays = 30\nturns = 200\n").unwrap(),
            Changes::with_window(HowFarBack::of(30, 200).unwrap())
        );
    }
}
