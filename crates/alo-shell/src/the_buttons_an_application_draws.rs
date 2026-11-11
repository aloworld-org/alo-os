//! Which window buttons an application that draws its own header shows,
//! as each one was **measured**.
//!
//! # Why a table, and why a reviewed one
//!
//! The owner, 2026-11-12: *No need to add controls if the app has, we can only
//! add the ones that are not there.* To do that, alo has to know which buttons
//! an application draws, and nothing on the protocol says. `xdg_decoration`
//! tells a compositor *who* draws a frame, never *what* is on it. Looking at
//! the window to find out, through its pixels or its accessibility tree, would
//! be watching it, which `CLAUDE.md` forbids.
//!
//! So this is **a short list, written by a person, of what each application
//! was seen to draw**. Every row names its `app_id` (which the application
//! declares itself) and points to the measurement. A row changes only when
//! somebody measures again. An application with no row is
//! [`ItsOwnButtons::NOT_KNOWN`]: alo adds Window actions and nothing else, and
//! `docs/quirks/an-application-that-draws-its-own-header-is-not-asked-which-buttons.md`
//! says what that costs.
//!
//! # What the rows were measured on
//!
//! Once, by hand, on 2026-11-12. Each application came from Flathub and ran
//! under a headless sway on Ubuntu 24.04, with windows floating, and with the
//! Settings portal refusing every key. **That last part is what a fresh alo
//! machine does**: no application holds the `appearance-settings` grant until
//! the person gives it. The buttons were read once off the application's
//! accessibility tree and a screenshot. The method is in that quirk. **Nothing
//! here runs it again.** A Flathub update that moves an application to a new
//! runtime can change its buttons, and then its row is wrong until it is
//! measured again.

use crate::window_edge::ItsOwnButtons;

/// Minimise, Maximise and Close: GTK 4 and libadwaita on GNOME's runtime 50
/// and 51, and VSCodium's own title bar.
const ALL_THREE: ItsOwnButtons = ItsOwnButtons {
    minimise: true,
    close: true,
};

/// Close alone: Firefox's own header.
const CLOSE_ONLY: ItsOwnButtons = ItsOwnButtons {
    minimise: false,
    close: true,
};

/// Every application measured, by the `app_id` it declares.
///
/// Sorted by `app_id`, so a reviewer finds a row where they expect it.
pub(crate) const MEASURED: [(&str, ItsOwnButtons); 6] = [
    // Ptyxis, the terminal a machine ships with (`alo-software/shipped.toml`).
    ("app.devsuite.Ptyxis", ALL_THREE),
    // VSCodium with its default `window.titleBarStyle`, which is `custom`: its
    // own Minimise, Maximise or Restore, and Close.
    ("com.vscodium.codium", ALL_THREE),
    ("org.gnome.Loupe", ALL_THREE),
    ("org.gnome.Papers", ALL_THREE),
    ("org.gnome.TextEditor", ALL_THREE),
    // Firefox draws Close at the end of its tab strip and no Minimise.
    ("org.mozilla.firefox", CLOSE_ONLY),
];

/// What this application was measured to draw, or [`None`] where it never was.
#[must_use]
pub(crate) fn measured(application: &str) -> Option<ItsOwnButtons> {
    MEASURED
        .iter()
        .find(|(named, _)| *named == application)
        .map(|(_, its_own)| *its_own)
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic naming the unread file is the failure being reported"
)]
#[path = "the_buttons_an_application_draws_tests.rs"]
mod tests;
