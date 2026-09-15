//! What a machine has that an application may be allowed, and that is not a
//! path.
//!
//! [ADR 0040](../../../docs/decisions/0040-what-an-applications-grant-is-over.md),
//! part 1. A [`crate::Reach`] was a folder, a file or an application, and eleven
//! of the fifteen v0.5 portals ask about none of those: the camera, the
//! microphone, the screen, the person's notifications, and so on. This is the
//! closed list of them — and of the twelfth, [`Facility::Secrets`], which the
//! Secret portal asks for and which was added under the ADR's own rule that the
//! list grows additively (ADR 0040, amendment of 2026-09-15).
//!
//! # A camera grant is a grant to *the camera*
//!
//! Not to `/dev/video0`, which the kernel numbers in the order devices are found
//! and which can mean another camera after a replug, and not to a link under
//! `/dev/v4l/by-id`, which [`crate::path`] would never match an ask against.
//! A facility is matched exactly, like an application's identifier: a grant to
//! the camera never covers the microphone, and the screen once never covers
//! the screen continuously.
//!
//! # Closed, and additive
//!
//! The v1 portals — USB devices, global shortcuts, launchers, remote desktop,
//! location — are **absent** rather than present and refused. A facility this
//! machine does not offer is not one it lists. The grants file names these by
//! [`Facility::named`], so the list is a public surface: adding to it is
//! additive, and renaming anything on it needs a new grants-file format.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words;

/// Something this machine has that is not a path, and that a grant may be over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Facility {
    /// The camera — whichever camera the person has, not a device number.
    Camera,
    /// The microphone.
    Microphone,
    /// The screen, once: one picture of it.
    ScreenOnce,
    /// The screen, continuously: a recording or a shared screen.
    ScreenContinuously,
    /// Showing the person notifications.
    Notifications,
    /// What the person copied.
    Clipboard,
    /// The desktop background.
    DesktopBackground,
    /// The person's appearance settings — light or dark, accent, contrast.
    AppearanceSettings,
    /// Whether the machine sleeps.
    Sleep,
    /// The state of the network.
    NetworkState,
    /// The power profile.
    PowerProfile,
    /// The application's own secrets in the person's one keyring, and no one
    /// else's — the Secret portal's reach, added after ADR 0040's first eleven.
    Secrets,
}

impl Facility {
    /// Every facility, in the order this file declares them.
    pub const EVERY: [Self; 12] = [
        Self::Camera,
        Self::Microphone,
        Self::ScreenOnce,
        Self::ScreenContinuously,
        Self::Notifications,
        Self::Clipboard,
        Self::DesktopBackground,
        Self::AppearanceSettings,
        Self::Sleep,
        Self::NetworkState,
        Self::PowerProfile,
        Self::Secrets,
    ];

    /// The name this facility is written down by, in the grants file and
    /// anywhere else it is kept.
    ///
    /// The same spelling `serde` gives it, which a test holds, so a file
    /// written by hand and one written by this crate cannot disagree.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::Camera => "camera",
            Self::Microphone => "microphone",
            Self::ScreenOnce => "screen-once",
            Self::ScreenContinuously => "screen-continuously",
            Self::Notifications => "notifications",
            Self::Clipboard => "clipboard",
            Self::DesktopBackground => "desktop-background",
            Self::AppearanceSettings => "appearance-settings",
            Self::Sleep => "sleep",
            Self::NetworkState => "network-state",
            Self::PowerProfile => "power-profile",
            Self::Secrets => "secrets",
        }
    }

    /// The facility written down by this name, or [`None`].
    ///
    /// Exactly, with no case folding: a name that is almost one of these is
    /// not one of these.
    #[must_use]
    pub fn by_name(name: &str) -> Option<Self> {
        Self::EVERY.into_iter().find(|one| one.named() == name)
    }

    /// The string this crate declares for this facility.
    #[must_use]
    pub fn word(self) -> words::Word {
        match self {
            Self::Camera => words::THE_CAMERA,
            Self::Microphone => words::THE_MICROPHONE,
            Self::ScreenOnce => words::THE_SCREEN_ONCE,
            Self::ScreenContinuously => words::THE_SCREEN_CONTINUOUSLY,
            Self::Notifications => words::NOTIFICATIONS,
            Self::Clipboard => words::THE_CLIPBOARD,
            Self::DesktopBackground => words::THE_DESKTOP_BACKGROUND,
            Self::AppearanceSettings => words::APPEARANCE_SETTINGS,
            Self::Sleep => words::SLEEP,
            Self::NetworkState => words::THE_NETWORK_STATE,
            Self::PowerProfile => words::THE_POWER_PROFILE,
            Self::Secrets => words::ITS_OWN_SECRETS,
        }
    }

    /// This facility as a clause, in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use std::collections::BTreeSet;

    /// **The name it is written down by is the name serde gives it**, so the
    /// grants file and anything that serialises a grant spell it one way.
    #[test]
    fn the_name_it_is_kept_by_is_the_one_serde_writes() {
        for facility in Facility::EVERY {
            assert_eq!(
                serde_json::to_string(&facility).unwrap(),
                format!("\"{}\"", facility.named())
            );
            assert_eq!(Facility::by_name(facility.named()), Some(facility));
        }
    }

    /// **A name that is almost one is not one.** Matching loosely would reach
    /// more than the person allowed.
    #[test]
    fn a_name_is_matched_exactly() {
        assert_eq!(Facility::by_name("Camera"), None);
        assert_eq!(Facility::by_name(" camera"), None);
        assert_eq!(Facility::by_name("screen"), None);
        assert_eq!(Facility::by_name("/dev/video0"), None);
        assert_eq!(Facility::by_name("usb"), None);
    }

    /// Twelve, each with its own name and its own words.
    #[test]
    fn every_facility_is_named_and_said_once() {
        let names: BTreeSet<&str> = Facility::EVERY.iter().map(|one| one.named()).collect();
        assert_eq!(names.len(), Facility::EVERY.len());
        let words: BTreeSet<&str> = Facility::EVERY
            .iter()
            .map(|one| one.word().named())
            .collect();
        assert_eq!(words.len(), Facility::EVERY.len());

        let strings = in_english();
        for facility in Facility::EVERY {
            let said = facility.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
        }
        assert_eq!(Facility::Camera.said(&strings).text(), "the camera");
        assert_eq!(
            Facility::Secrets.said(&strings).text(),
            "a place for its own passwords in your keyring"
        );
    }

    /// And translated like everything else here.
    #[test]
    fn a_facility_is_said_in_the_readers_language() {
        let strings = translated(&[(words::THE_CAMERA, "die Kamera")]);
        let said = Facility::Camera.said(&strings);
        assert!(said.is_translated());
        assert_eq!(said.text(), "die Kamera");
    }
}
