//! What a person changed about notifications, which is the only part written
//! down — and the settings a session runs by, which are what alo OS ships with
//! those changes over them.
//!
//! The shape `alo-dock`, `alo-appearance` and `alo-sleeping` keep, for the same
//! reason: the defaults live in the running release and the file holds the
//! difference, so a release that moves a default reaches every machine that
//! never touched it. An untouched machine has no `notifying.toml` at all
//! ([`crate::keeping`]).
//!
//! **Two settings, and no more.** Whether notifications are held, and the
//! stretch of the clock during which they are held anyway. What happens while
//! the screen is being read is **not** a setting and never becomes one — it is
//! [`crate::Quiet::now`]'s first question, decided before this file is
//! consulted, and there is nothing here to switch it off with.
//!
//! **And no notification is ever written into this file.** What is kept is two
//! values about how notifications behave; the notifications themselves live in
//! [`crate::Missed`], in this session's own memory, and reach no disk at all.

use serde::{Deserialize, Serialize};

use crate::quiet_hours::QuietHours;

/// One thing a person can change about notifications, for a settings panel
/// that offers *put it back*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Setting {
    /// Whether notifications are held.
    DoNotDisturb,
    /// The stretch of the clock during which they are held.
    QuietHours,
}

/// Everything a person has changed about notifications.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Written", into = "Written")]
pub struct Changes {
    /// Whether they turned notifications off.
    do_not_disturb: Option<bool>,
    /// The hours they set aside.
    quiet_hours: Option<QuietHours>,
}

impl Changes {
    /// Nothing changed yet.
    #[must_use]
    pub fn untouched() -> Self {
        Self::default()
    }

    /// Whether nothing has been changed at all.
    #[must_use]
    pub const fn is_untouched(&self) -> bool {
        self.do_not_disturb.is_none() && self.quiet_hours.is_none()
    }

    /// Turn notifications off, or on.
    pub fn set_do_not_disturb(&mut self, on: bool) {
        self.do_not_disturb = Some(on);
    }

    /// Set aside a stretch of the clock.
    pub fn set_quiet_hours(&mut self, hours: QuietHours) {
        self.quiet_hours = Some(hours);
    }

    /// Forget that this was ever changed. Says whether there was anything to
    /// forget.
    pub fn forget(&mut self, setting: Setting) -> bool {
        match setting {
            Setting::DoNotDisturb => self.do_not_disturb.take().is_some(),
            Setting::QuietHours => self.quiet_hours.take().is_some(),
        }
    }
}

/// The settings a session runs by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// Whether notifications are held.
    pub do_not_disturb: bool,
    /// The stretch of the clock during which they are held, if the person set
    /// one.
    pub quiet_hours: Option<QuietHours>,
}

impl Settings {
    /// What alo OS ships: notifications are shown, and no hours are set aside.
    ///
    /// Shown rather than held, because a machine that arrives silent is a
    /// machine whose owner finds out a week later that nothing was ever
    /// reaching them. What holds without being asked for is the screen being
    /// read, and that is not a setting.
    #[must_use]
    pub const fn shipped() -> Self {
        Self {
            do_not_disturb: false,
            quiet_hours: None,
        }
    }

    /// These settings with a person's changes over them.
    #[must_use]
    pub fn with(self, changes: &Changes) -> Self {
        Self {
            do_not_disturb: changes.do_not_disturb.unwrap_or(self.do_not_disturb),
            quiet_hours: changes.quiet_hours.or(self.quiet_hours),
        }
    }
}

/// Changes as a settings file holds them: anything untouched is absent.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Written {
    /// Whether notifications are held, if that was chosen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    do_not_disturb: Option<bool>,
    /// The hours set aside, if any were.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    quiet_hours: Option<QuietHours>,
}

impl From<Written> for Changes {
    fn from(written: Written) -> Self {
        Self {
            do_not_disturb: written.do_not_disturb,
            quiet_hours: written.quiet_hours,
        }
    }
}

impl From<Changes> for Written {
    fn from(changes: Changes) -> Self {
        Self {
            do_not_disturb: changes.do_not_disturb,
            quiet_hours: changes.quiet_hours,
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
    use crate::testing::at;

    /// **What alo OS ships shows notifications and sets no hours aside**, and
    /// a person's changes land over it one at a time.
    #[test]
    fn changes_land_over_what_ships() {
        assert_eq!(
            Settings::shipped().with(&Changes::untouched()),
            Settings::shipped()
        );

        let mut changes = Changes::untouched();
        changes.set_do_not_disturb(true);
        let settings = Settings::shipped().with(&changes);
        assert!(settings.do_not_disturb);
        assert_eq!(settings.quiet_hours, None);

        let night = QuietHours::these_two_times(at(23, 0), at(7, 0)).unwrap();
        changes.set_quiet_hours(night);
        assert_eq!(Settings::shipped().with(&changes).quiet_hours, Some(night));

        assert!(changes.forget(Setting::DoNotDisturb));
        assert!(!changes.forget(Setting::DoNotDisturb));
        assert!(changes.forget(Setting::QuietHours));
        assert!(changes.is_untouched());
    }

    /// **A machine nobody configured is not a silent machine.** The shipped
    /// settings show notifications: arriving quiet is how somebody discovers a
    /// week later that nothing was ever reaching them.
    #[test]
    fn a_machine_nobody_configured_is_not_a_silent_one() {
        assert!(!Settings::shipped().do_not_disturb);
        assert_eq!(Settings::shipped().quiet_hours, None);
    }
}
