//! What a person has turned on, and the only road to changing one.

use std::collections::BTreeSet;
use std::time::Duration;

use alo_appearance::TextScale;
use serde::{Deserialize, Serialize};

use crate::key_filter::KeyFilter;
use crate::setting::Setting;

/// How much larger text is when [`Setting::LargerText`] is on.
///
/// `alo-appearance` decides what sizes exist and a person may set any of them
/// for their own reasons; this is what *larger text* means for somebody who
/// turned it on for access and was not asked for a number.
const LARGER: u16 = 150;

/// **What a person has turned on.**
///
/// An untouched machine has none of it — [`TurnedOn::nothing`] — and that is a
/// person who has not been asked, never a person who does not need any of this.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TurnedOn {
    /// Which settings are on, in the order [`Setting::ALL`] lists them.
    on: BTreeSet<String>,
    /// How much larger the magnifier draws, where it is on.
    magnification: Option<Magnification>,
    /// How long a key is held before it counts, where slow keys are on.
    held_before_it_counts: Option<Duration>,
    /// How long the same key is ignored for, where bounce keys are on.
    ignored_after_the_same_key: Option<Duration>,
}

impl TurnedOn {
    /// A machine nobody has changed.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// Whether this setting is on.
    #[must_use]
    pub fn has(&self, setting: Setting) -> bool {
        self.on.contains(named(setting))
    }

    /// Turn one on.
    pub fn turn_on(&mut self, setting: Setting) {
        self.on.insert(named(setting).to_owned());
    }

    /// Turn one off. Its value — a magnification, a delay — is kept, so that
    /// turning it on again does not ask a person the same question twice.
    pub fn turn_off(&mut self, setting: Setting) {
        self.on.remove(named(setting));
    }

    /// Every setting that is on.
    pub fn all_on(&self) -> impl Iterator<Item = Setting> + '_ {
        Setting::ALL
            .into_iter()
            .filter(|setting| self.has(*setting))
    }

    /// How much larger the magnifier draws — [`Magnification::twice`] until
    /// somebody says otherwise.
    #[must_use]
    pub fn magnification(&self) -> Magnification {
        self.magnification.unwrap_or_else(Magnification::twice)
    }

    /// Say how much larger the magnifier draws.
    pub fn magnify_by(&mut self, magnification: Magnification) {
        self.magnification = Some(magnification);
    }

    /// The text scale [`Setting::LargerText`] means.
    #[must_use]
    pub fn larger_text(&self) -> TextScale {
        TextScale::percent(LARGER).unwrap_or_else(|_| TextScale::ordinary())
    }

    /// **The three key settings as one filter**, which is what a keyboard reads.
    #[must_use]
    pub fn key_filter(&self) -> KeyFilter {
        KeyFilter {
            sticky: self.has(Setting::StickyKeys),
            held_before_it_counts: self
                .has(Setting::SlowKeys)
                .then(|| self.held_before_it_counts.unwrap_or(A_MOMENT)),
            ignored_after_the_same_key: self
                .has(Setting::BounceKeys)
                .then(|| self.ignored_after_the_same_key.unwrap_or(A_MOMENT)),
        }
    }

    /// Say how long a key is held before it counts.
    pub fn hold_keys_for(&mut self, how_long: Duration) {
        self.held_before_it_counts = Some(how_long);
    }

    /// Say how long the same key is ignored for.
    pub fn ignore_the_same_key_for(&mut self, how_long: Duration) {
        self.ignored_after_the_same_key = Some(how_long);
    }
}

/// What *a moment* is, where a person turned a key setting on and named no time.
const A_MOMENT: Duration = Duration::from_millis(300);

/// The name a setting is kept under, which is the name it serialises as.
fn named(setting: Setting) -> &'static str {
    match setting {
        Setting::ScreenReader => "screen-reader",
        Setting::Magnifier => "magnifier",
        Setting::HighContrast => "high-contrast",
        Setting::LargerText => "larger-text",
        Setting::ReducedMotion => "reduced-motion",
        Setting::StickyKeys => "sticky-keys",
        Setting::SlowKeys => "slow-keys",
        Setting::BounceKeys => "bounce-keys",
        Setting::FocusAlwaysVisible => "focus-always-visible",
    }
}

/// **How much larger the magnifier draws** — between twice and twenty times.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct Magnification {
    /// Tenths, so that 1.5× is a whole number here and not a float in a file.
    tenths: u16,
}

impl Magnification {
    /// The smallest magnification that is one at all.
    pub const SMALLEST: u16 = 12;
    /// The largest anything on a screen is worth drawing.
    pub const LARGEST: u16 = 200;

    /// Twice, which is what a magnifier is set to until somebody changes it.
    #[must_use]
    pub const fn twice() -> Self {
        Self { tenths: 20 }
    }

    /// A magnification in tenths — 15 is one and a half times.
    ///
    /// # Errors
    /// [`NotAMagnification`] outside the range, because a magnifier at 1.0×
    /// magnifies nothing and one at 30× shows four letters.
    pub const fn of_tenths(tenths: u16) -> Result<Self, NotAMagnification> {
        if tenths < Self::SMALLEST || tenths > Self::LARGEST {
            return Err(NotAMagnification { asked_for: tenths });
        }
        Ok(Self { tenths })
    }

    /// How many tenths this is.
    #[must_use]
    pub const fn tenths(self) -> u16 {
        self.tenths
    }
}

impl From<Magnification> for u16 {
    fn from(magnification: Magnification) -> Self {
        magnification.tenths
    }
}

impl TryFrom<u16> for Magnification {
    type Error = NotAMagnification;

    fn try_from(tenths: u16) -> Result<Self, Self::Error> {
        Self::of_tenths(tenths)
    }
}

/// A magnification nobody could have meant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error(
    "a magnifier draws between {}× and {}×, not {asked_for} tenths",
    Magnification::SMALLEST / 10,
    Magnification::LARGEST / 10
)]
pub struct NotAMagnification {
    /// What was asked for, in tenths.
    pub asked_for: u16,
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **An untouched machine has nothing turned on**, and says so rather than
    /// guessing what somebody needs.
    #[test]
    fn a_machine_nobody_has_changed_has_nothing_on() {
        let nothing = TurnedOn::nothing();
        assert_eq!(nothing.all_on().count(), 0);
        for setting in Setting::ALL {
            assert!(!nothing.has(setting), "{setting:?}");
            assert_eq!(
                setting.what_it_changes(&nothing, alo_appearance::Scheme::Light),
                None,
                "{setting:?}"
            );
        }
        assert!(!nothing.key_filter().changes_anything());
    }

    /// **Turning one on changes one thing**, and turning it off leaves the rest
    /// as they were.
    #[test]
    fn one_setting_on_is_one_setting_on() {
        let mut turned_on = TurnedOn::nothing();
        turned_on.turn_on(Setting::HighContrast);
        assert!(turned_on.has(Setting::HighContrast));
        assert_eq!(turned_on.all_on().count(), 1);
        assert!(!turned_on.has(Setting::LargerText));
        assert!(!turned_on.key_filter().changes_anything());

        turned_on.turn_on(Setting::SlowKeys);
        assert_eq!(
            turned_on.key_filter().held_before_it_counts,
            Some(A_MOMENT),
            "a key setting with no time said is a moment, not nothing"
        );
        turned_on.turn_off(Setting::HighContrast);
        assert!(turned_on.has(Setting::SlowKeys));
        assert!(!turned_on.has(Setting::HighContrast));
    }

    /// **A value a person set is kept when they turn the setting off**, so that
    /// turning it back on does not ask them again.
    #[test]
    fn a_value_outlives_the_setting_being_turned_off() {
        let mut turned_on = TurnedOn::nothing();
        turned_on.turn_on(Setting::Magnifier);
        turned_on.magnify_by(Magnification::of_tenths(40).unwrap());
        turned_on.turn_off(Setting::Magnifier);
        turned_on.turn_on(Setting::Magnifier);
        assert_eq!(turned_on.magnification().tenths(), 40);
    }

    /// **A magnifier magnifies**, and there is no setting for one that does not.
    #[test]
    fn a_magnification_nobody_could_mean_is_refused() {
        assert!(Magnification::of_tenths(10).is_err());
        assert!(Magnification::of_tenths(0).is_err());
        assert!(Magnification::of_tenths(400).is_err());
        assert_eq!(Magnification::of_tenths(12).unwrap().tenths(), 12);
        assert_eq!(Magnification::twice().tenths(), 20);
    }
}
