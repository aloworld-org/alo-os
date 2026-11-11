//! **What an application is told so that it describes itself** to alo and to
//! screen readers — or, where the person turned that off, so that it does not.
//!
//! The owner's decision of 2026-11-11: *on by default, switch in Settings*.
//! Every toolkit has its own switch, and none of them is the same one. What
//! each one reads was taken from its own source and measured once, on
//! 2026-11-11, on a real accessibility bus — in Ubuntu 24.04 under a headless
//! Wayland compositor, with the Flathub builds of VSCodium 1.143, KCalc 26.08
//! (Qt 6.11) and Firefox 160, and GTK 3 and GTK 4 windows of their own
//! (`docs/quirks/chromium-describes-only-its-window-frame-until-asked.md`
//! holds the table). **Not on an alo machine**: a boot walk is where that
//! is measured, and the tests here hold the decision, not the toolkits.
//!
//! | Toolkit | Describes itself | Turned on by | Turned off by |
//! |---|---|---|---|
//! | GTK 3, LibreOffice | always | — | `NO_AT_BRIDGE=1` |
//! | GTK 4 | always | — | `GTK_A11Y=none` |
//! | Qt 5, and Qt 6 outside Flatpak | only if asked | `QT_LINUX_ACCESSIBILITY_ALWAYS_ON` being set at all | its absence |
//! | Qt 6 inside Flatpak | always: Flatpak sets `AT_SPI_BUS_ADDRESS`, which Qt 6 obeys | — | **nothing alo can set** |
//! | Firefox | only if asked | `GNOME_ACCESSIBILITY=1` | `GNOME_ACCESSIBILITY=0` |
//! | Chromium, Electron: the window | only if asked | `ACCESSIBILITY_ENABLED=1` | `ACCESSIBILITY_ENABLED=0` |
//! | Chromium, Electron: the page inside it | only if asked | the command-line switch [`THE_PAGE_TOO`] | its absence |
//!
//! # Two roads, because one cannot carry the last row
//!
//! **The environment** carries the first five rows. The image names the *on*
//! values as the default for every person's systemd manager
//! (`/usr/lib/environment.d/60-alo-apps-describe-themselves.conf`), so an
//! application started by anything — a press, the bus, the terminal — is told
//! the same thing. Where the person turned it off the desktop tells their
//! session [`what_the_session_is_told`] instead.
//!
//! **A command-line switch** carries the last: Chromium reads no variable for
//! it. So the desktop adds it to the command of an application it starts
//! whose identifier is in [`DRAWN_BY_CHROMIUM`] — a closed list, reviewed, for
//! the reason `crate::alos_copy` gives: a switch handed to a program that does
//! not know it is a file name, or worse.
//!
//! **One variable cannot be turned off by a value.** Qt asks only whether
//! `QT_LINUX_ACCESSIBILITY_ALWAYS_ON` is *set* — an empty one counts — so off
//! is the variable **removed** from the person's manager, and never pushed to
//! the session bus, which can be told a value and never told to forget one.
//!
//! # What this does not do
//!
//! **Nothing here reads a tree, and nothing listens to one.** An application
//! describing itself is an application *able* to answer; alo still asks only
//! during a turn the person started (`CLAUDE.md`: *context is offered, never
//! watched*), and a screen reader asks only when the person runs one.
//!
//! **`org.a11y.Status` is left alone.** Its `IsEnabled` would reach Qt,
//! Firefox and Chromium's window too, but writing it writes the person's
//! `toolkit-accessibility` setting, and from at-spi2-core 2.61 it is the bus's
//! own answer to *is anybody listening*, which is not alo's to overwrite. Its
//! `ScreenReaderEnabled` is the GSettings key that **starts the Orca screen
//! reader** on desktops that watch it, which is a different request from this
//! one.

/// The switch that makes a Chromium-drawn application describe the page
/// inside its window, not only the window around it.
///
/// **`=basic`, and not the bare switch.** The bare switch also tells the
/// application a screen reader is running: VSCodium then turns on its
/// *screen reader optimized* mode, which changes how its editor behaves for a
/// person who did not ask for that. `basic` is the page's controls by name
/// and role, which is what alo reads, and says nothing about a screen reader.
/// A person running one is detected by Chromium itself.
pub const THE_PAGE_TOO: &str = "--force-renderer-accessibility=basic";

/// The applications drawn by Chromium that the desktop hands
/// [`THE_PAGE_TOO`], by identifier.
///
/// Each takes Chromium's switches after its own command: VSCodium's and VS
/// Code's own argument lists name it (`src/vs/platform/environment/node/argv.ts`),
/// and Chromium is the program that defines it.
pub const DRAWN_BY_CHROMIUM: [&str; 3] = [
    "com.vscodium.codium",
    "com.visualstudio.code",
    "org.chromium.Chromium",
];

/// Qt's switch, which is read as *set or not* and never by its value.
pub const QT_ALWAYS_ON: &str = "QT_LINUX_ACCESSIBILITY_ALWAYS_ON";

/// What the person's session is told: values to set, and names to remove.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Told {
    /// `NAME=value`, for the person's manager and the session bus alike.
    pub everywhere: Vec<String>,
    /// `NAME=value`, for the person's manager only, because the bus could never
    /// be told to forget it again.
    pub the_manager_only: Vec<String>,
    /// Names the person's manager is to forget.
    pub forgotten: Vec<&'static str>,
}

/// The variables that have a value either way, and that value when on and
/// when off, in the order the image's file lists them.
const EITHER_WAY: [(&str, &str, &str); 4] = [
    ("NO_AT_BRIDGE", "0", "1"),
    ("GTK_A11Y", "0", "none"),
    ("GNOME_ACCESSIBILITY", "1", "0"),
    ("ACCESSIBILITY_ENABLED", "1", "0"),
];

/// **What the person's session is told** when applications are to describe
/// themselves (`describe`), or not.
#[must_use]
pub fn what_the_session_is_told(describe: bool) -> Told {
    let everywhere = EITHER_WAY
        .iter()
        .map(|(name, on, off)| format!("{name}={}", if describe { on } else { off }))
        .collect();
    if describe {
        Told {
            everywhere,
            the_manager_only: vec![format!("{QT_ALWAYS_ON}=1")],
            forgotten: Vec::new(),
        }
    } else {
        Told {
            everywhere,
            the_manager_only: Vec::new(),
            forgotten: vec![QT_ALWAYS_ON],
        }
    }
}

/// **What one application started by the desktop is handed**, as `NAME=value`
/// beside its display.
///
/// Every variable that has a value, and Qt's only when on: a unit can be
/// handed a value and cannot be handed an absence, so where it is off the
/// manager has already forgotten it ([`what_the_session_is_told`]), and a
/// process the desktop starts with an empty environment never had it.
#[must_use]
pub fn what_a_start_is_handed(describe: bool) -> Vec<String> {
    let told = what_the_session_is_told(describe);
    told.everywhere
        .into_iter()
        .chain(told.the_manager_only)
        .collect()
}

/// **What follows an application's own command** when the desktop starts it:
/// [`THE_PAGE_TOO`] for an application drawn by Chromium where applications
/// describe themselves, and nothing otherwise.
#[must_use]
pub fn what_its_command_gains(identifier: &str, describe: bool) -> &'static [&'static str] {
    if describe && DRAWN_BY_CHROMIUM.contains(&identifier) {
        &[THE_PAGE_TOO]
    } else {
        &[]
    }
}

#[cfg(test)]
#[path = "describing_themselves_tests.rs"]
mod tests;
