//! What an application is handed, so that it draws on alo's display.
//!
//! A Wayland client finds its compositor through `WAYLAND_DISPLAY`. alo's
//! socket is not the conventional `$XDG_RUNTIME_DIR/wayland-0`: it is in the
//! desktop unit's own runtime directory, so the variable holds its **absolute
//! path**, which the Wayland library accepts as written. Without it a GTK or Qt
//! application looks for an X server, finds none, and exits without a window —
//! *listed, pressed, and nothing drew*.
//!
//! `XDG_SESSION_TYPE=wayland` is what Qt, Firefox and LibreOffice read to pick
//! their Wayland backend rather than trying X first. `XDG_DATA_DIRS` is the
//! list the Dock found the entry in, flatpak's exports included, so the
//! application finds its own icons and data the way the Dock found it.
//!
//! # Two audiences, two lists
//!
//! **A unit this desktop starts** is handed all three ([`what_an_application_is_handed`]).
//! **The session itself** — the bus's activation environment and the person's
//! systemd manager — is told the display ([`what_the_session_is_told`]), so an
//! application the *bus* starts draws too. Every desktop does this at sign-in
//! (`dbus-update-activation-environment --systemd` is the usual spelling);
//! until this crate nothing on alo did, so even the applications a press could
//! already reach were started with no display to draw on. `XDG_DATA_DIRS` is
//! not pushed into the session: the person's manager already has its own, from
//! the system's generators, and replacing it for every service they run is not
//! this desktop's to decide.

use std::path::Path;

/// The variables a unit started by a press is given.
///
/// [`None`] when the socket's path or the data directories are not UTF-8,
/// which a systemd `Environment` entry must be — the desktop then says so
/// rather than handing over a mangled path.
#[must_use]
pub fn what_an_application_is_handed(
    wayland: &Path,
    data_dirs: &std::ffi::OsStr,
) -> Option<Vec<String>> {
    let mut handed = what_the_session_is_told(wayland)?;
    handed.push(format!("XDG_DATA_DIRS={}", data_dirs.to_str()?));
    Some(handed)
}

/// The variables the session's bus and the person's manager are told, so an
/// application either of them starts finds alo's display.
#[must_use]
pub fn what_the_session_is_told(wayland: &Path) -> Option<Vec<String>> {
    // `starts_with('/')` rather than `Path::is_absolute`: this is a Linux
    // path whichever host compiles the test, and Windows would call it
    // relative.
    let socket = wayland
        .to_str()
        .filter(|it| it.starts_with('/') && !it.contains('\n'))?;
    Some(vec![
        format!("WAYLAND_DISPLAY={socket}"),
        "XDG_SESSION_TYPE=wayland".to_owned(),
    ])
}

#[cfg(test)]
#[path = "the_environment_tests.rs"]
mod tests;
