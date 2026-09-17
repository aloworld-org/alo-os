//! What a person changed about sleep, which is the only part written down —
//! and the settings a session runs by, which are what alo OS ships with those
//! changes over them.
//!
//! The shape `alo-dock` and `alo-appearance` keep, for the same reason: the
//! defaults live in the running release and the file holds the difference, so a
//! release that moves a default reaches every machine that never touched it. An
//! untouched machine has no `sleeping.toml` at all (`crate::keeping`).
//!
//! **Two settings, and no more.** What closing the lid does, and whether the
//! machine stays awake when nobody is using it. How long *nobody using it* is,
//! battery thresholds and power profiles are the devices plan's, not this
//! crate's.

use serde::{Deserialize, Serialize};

use crate::lid::Lid;

/// One thing a person can change about sleep, for a settings panel that offers
/// *put it back*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Setting {
    /// What closing the lid does.
    Lid,
    /// Whether the machine stays awake when nobody is using it.
    KeepAwake,
}

/// Everything a person has changed about sleep.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Written", into = "Written")]
pub struct Changes {
    /// What they chose closing the lid does.
    lid: Option<Lid>,
    /// Whether they keep the machine awake.
    keep_awake: Option<bool>,
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
        self.lid.is_none() && self.keep_awake.is_none()
    }

    /// Choose what closing the lid does.
    pub fn set_lid(&mut self, lid: Lid) {
        self.lid = Some(lid);
    }

    /// Keep the machine awake, or not.
    pub fn set_keep_awake(&mut self, on: bool) {
        self.keep_awake = Some(on);
    }

    /// Forget that this was ever changed. Says whether there was anything to
    /// forget.
    pub fn forget(&mut self, setting: Setting) -> bool {
        match setting {
            Setting::Lid => self.lid.take().is_some(),
            Setting::KeepAwake => self.keep_awake.take().is_some(),
        }
    }
}

/// The settings a session runs by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// What closing the lid does.
    pub lid: Lid,
    /// Whether the machine stays awake when nobody is using it.
    pub keep_awake: bool,
}

impl Settings {
    /// What alo OS ships: closing the lid sleeps, and nothing keeps the machine
    /// awake.
    #[must_use]
    pub const fn shipped() -> Self {
        Self {
            lid: Lid::Sleeps,
            keep_awake: false,
        }
    }

    /// These settings with a person's changes over them.
    #[must_use]
    pub fn with(self, changes: &Changes) -> Self {
        Self {
            lid: changes.lid.unwrap_or(self.lid),
            keep_awake: changes.keep_awake.unwrap_or(self.keep_awake),
        }
    }
}

/// Changes as a settings file holds them: anything untouched is absent.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Written {
    /// Whether the machine is kept awake, if that was chosen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    keep_awake: Option<bool>,
    /// What closing the lid does, if it was chosen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lid: Option<Lid>,
}

impl From<Written> for Changes {
    fn from(written: Written) -> Self {
        Self {
            lid: written.lid,
            keep_awake: written.keep_awake,
        }
    }
}

impl From<Changes> for Written {
    fn from(changes: Changes) -> Self {
        Self {
            lid: changes.lid,
            keep_awake: changes.keep_awake,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **What alo OS ships sleeps on a closed lid and keeps nothing awake**, and
    /// a person's changes land over it one at a time.
    #[test]
    fn changes_land_over_what_ships() {
        assert_eq!(
            Settings::shipped().with(&Changes::untouched()),
            Settings::shipped()
        );

        let mut changes = Changes::untouched();
        changes.set_keep_awake(true);
        let settings = Settings::shipped().with(&changes);
        assert!(settings.keep_awake);
        assert_eq!(settings.lid, Lid::Sleeps);

        changes.set_lid(Lid::StaysAwakeWithADisplay);
        assert_eq!(
            Settings::shipped().with(&changes).lid,
            Lid::StaysAwakeWithADisplay
        );

        assert!(changes.forget(Setting::KeepAwake));
        assert!(!changes.forget(Setting::KeepAwake));
        assert!(changes.forget(Setting::Lid));
        assert!(changes.is_untouched());
    }
}
