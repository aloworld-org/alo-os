//! The portals this machine offers applications, as a closed list.
//!
//! `docs/features.md`, v0.5: *Portals: file chooser and documents, open-with
//! and default applications, notifications, print, screenshot, screen capture,
//! camera, microphone, clipboard, trash, wallpaper, settings, inhibit, network
//! and power-profile monitors.* Fifteen — and a sixteenth on a line of its own,
//! *secret storage — one keyring behind the Secret portal*. [`Portal`] is those
//! sixteen.
//!
//! # Closed, and the v1 portals are absent
//!
//! USB devices, global shortcuts an application registers, dynamic launchers
//! and remote desktop are v1. They are **not** variants here that answer no —
//! a portal this machine does not offer is not a portal it lists, and a
//! variant that exists only to be refused is a variant somebody one day makes
//! answer yes.
//!
//! # What each is over
//!
//! Four are about a file a person granted: the file chooser, open-with,
//! print and trash. The other twelve are about something this machine has that
//! is not a path, and each is exactly one [`Facility`] — ADR 0040's table and
//! its amendment for the Secret portal, written as a `match` the compiler holds
//! to every portal.

use alo_capability::Facility;
use alo_strings::{Filling, Said, Strings};

use crate::words;

/// One portal this machine offers an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Portal {
    /// The file chooser, and the documents it hands over.
    FileChooser,
    /// Open-with, and which application opens a kind of file.
    OpenWith,
    /// Sending the person notifications.
    Notifications,
    /// Printing a document.
    Print,
    /// One picture of the screen.
    Screenshot,
    /// Recording or sharing the screen.
    ScreenCapture,
    /// The camera.
    Camera,
    /// The microphone.
    Microphone,
    /// The clipboard.
    Clipboard,
    /// Moving a file to the trash.
    Trash,
    /// Setting the desktop background.
    Wallpaper,
    /// Reading the person's appearance settings.
    Settings,
    /// Keeping the machine awake.
    Inhibit,
    /// Whether the machine is connected to a network.
    NetworkMonitor,
    /// The power profile.
    PowerProfileMonitor,
    /// The application's own passwords, in the person's one keyring.
    Secret,
}

/// What a request to a portal is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Over {
    /// A file a person granted, named by the request.
    APath,
    /// Something this machine has that is not a path, fixed by the portal.
    Facility(Facility),
}

impl Portal {
    /// Every portal, in the order `docs/features.md` promises them: the
    /// fifteen of its portal line, then the Secret portal from the line after.
    pub const EVERY: [Self; 16] = [
        Self::FileChooser,
        Self::OpenWith,
        Self::Notifications,
        Self::Print,
        Self::Screenshot,
        Self::ScreenCapture,
        Self::Camera,
        Self::Microphone,
        Self::Clipboard,
        Self::Trash,
        Self::Wallpaper,
        Self::Settings,
        Self::Inhibit,
        Self::NetworkMonitor,
        Self::PowerProfileMonitor,
        Self::Secret,
    ];

    /// What a request to this portal is about.
    ///
    /// The whole of ADR 0040's table. A path portal names its file in the
    /// request; a facility portal is about one facility and names nothing,
    /// so a request cannot ask the camera portal for the microphone.
    #[must_use]
    pub const fn over(self) -> Over {
        match self {
            Self::FileChooser | Self::OpenWith | Self::Print | Self::Trash => Over::APath,
            Self::Notifications => Over::Facility(Facility::Notifications),
            Self::Screenshot => Over::Facility(Facility::ScreenOnce),
            Self::ScreenCapture => Over::Facility(Facility::ScreenContinuously),
            Self::Camera => Over::Facility(Facility::Camera),
            Self::Microphone => Over::Facility(Facility::Microphone),
            Self::Clipboard => Over::Facility(Facility::Clipboard),
            Self::Wallpaper => Over::Facility(Facility::DesktopBackground),
            Self::Settings => Over::Facility(Facility::AppearanceSettings),
            Self::Inhibit => Over::Facility(Facility::Sleep),
            Self::NetworkMonitor => Over::Facility(Facility::NetworkState),
            Self::PowerProfileMonitor => Over::Facility(Facility::PowerProfile),
            Self::Secret => Over::Facility(Facility::Secrets),
        }
    }

    /// The name `docs/features.md` promises this portal by.
    ///
    /// Not shown to anybody — [`Portal::said`] is — and kept so that a test can
    /// hold this list to the promise it was built from.
    #[must_use]
    pub const fn promised_as(self) -> &'static str {
        match self {
            Self::FileChooser => "file chooser and documents",
            Self::OpenWith => "open-with and default applications",
            Self::Notifications => "notifications",
            Self::Print => "print",
            Self::Screenshot => "screenshot",
            Self::ScreenCapture => "screen capture",
            Self::Camera => "camera",
            Self::Microphone => "microphone",
            Self::Clipboard => "clipboard",
            Self::Trash => "trash",
            Self::Wallpaper => "wallpaper",
            Self::Settings => "settings",
            Self::Inhibit => "inhibit",
            Self::NetworkMonitor => "network monitor",
            Self::PowerProfileMonitor => "power-profile monitor",
            Self::Secret => "secret storage",
        }
    }

    /// The `org.freedesktop.portal.*` interface this machine answers this
    /// portal on, or [`None`] while it answers it on none.
    ///
    /// Three, for now: the Secret portal, open-with and the appearance settings,
    /// the portals whose every answer is decided without a dialog (tasks 3, 4
    /// and 6 of the applications plan). Every other portal is **not
    /// registered** on the bus, so an
    /// application asking one is told by the bus that nothing answers it,
    /// rather than told yes by a backend with nothing to show.
    /// `docs/contracts/portals.md` is this list for people building against it.
    #[must_use]
    pub const fn answered_on_the_bus(self) -> Option<&'static str> {
        match self {
            Self::Secret => Some("org.freedesktop.portal.Secret"),
            Self::OpenWith => Some("org.freedesktop.portal.OpenURI"),
            Self::Settings => Some("org.freedesktop.portal.Settings"),
            Self::FileChooser
            | Self::Notifications
            | Self::Print
            | Self::Screenshot
            | Self::ScreenCapture
            | Self::Camera
            | Self::Microphone
            | Self::Clipboard
            | Self::Trash
            | Self::Wallpaper
            | Self::Inhibit
            | Self::NetworkMonitor
            | Self::PowerProfileMonitor => None,
        }
    }

    /// The string this crate declares for this portal.
    #[must_use]
    pub const fn word(self) -> words::Word {
        match self {
            Self::FileChooser => words::FILE_CHOOSER,
            Self::OpenWith => words::OPEN_WITH,
            Self::Notifications => words::NOTIFICATIONS,
            Self::Print => words::PRINT,
            Self::Screenshot => words::SCREENSHOT,
            Self::ScreenCapture => words::SCREEN_CAPTURE,
            Self::Camera => words::CAMERA,
            Self::Microphone => words::MICROPHONE,
            Self::Clipboard => words::CLIPBOARD,
            Self::Trash => words::TRASH,
            Self::Wallpaper => words::WALLPAPER,
            Self::Settings => words::SETTINGS,
            Self::Inhibit => words::INHIBIT,
            Self::NetworkMonitor => words::NETWORK_MONITOR,
            Self::PowerProfileMonitor => words::POWER_PROFILE_MONITOR,
            Self::Secret => words::SECRET,
        }
    }

    /// What this portal lets an application do, in the language the person
    /// reads.
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
    use std::collections::BTreeSet;

    /// **Every facility is one portal's, and no two portals share one**, so
    /// the camera portal is the one road to a camera grant.
    #[test]
    fn every_facility_belongs_to_exactly_one_portal() {
        let over: Vec<Facility> = Portal::EVERY
            .iter()
            .filter_map(|portal| match portal.over() {
                Over::Facility(facility) => Some(facility),
                Over::APath => None,
            })
            .collect();
        let distinct: BTreeSet<Facility> = over.iter().copied().collect();
        assert_eq!(over.len(), distinct.len());
        assert_eq!(distinct, Facility::EVERY.into_iter().collect());
    }

    /// Sixteen portals, each with its own sentence and its own promised name.
    #[test]
    fn every_portal_has_its_own_words() {
        let words: BTreeSet<&str> = Portal::EVERY.iter().map(|p| p.word().named()).collect();
        let names: BTreeSet<&str> = Portal::EVERY.iter().map(|p| p.promised_as()).collect();
        assert_eq!(words.len(), 16);
        assert_eq!(names.len(), 16);

        let strings = Strings::of(crate::portal_words().unwrap());
        for portal in Portal::EVERY {
            let said = portal.said(&strings);
            assert!(!said.is_a_bug(), "{portal:?}: {said}");
            assert!(said.text().starts_with("Lets an application"), "{said}");
        }
    }
}
