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

/// **What alo's own copy of an application starts with**: the person's
/// session as their systemd manager holds it (`session`, `NAME=value` each),
/// with what the desktop hands every start it makes (`handed`) laid over it.
///
/// The session first, because an application started by a press is started by
/// that manager with exactly that environment: `HOME`, `PATH`,
/// `XDG_RUNTIME_DIR` and the session bus among it. A Flatpak application
/// started without them has no runtime folder and no bus to start in, and
/// alo's copy is then a copy nobody can see (the trace of 2026-11-14 found it
/// started with almost nothing). **What is handed is laid over it and wins**,
/// because where alo draws, and whether applications describe themselves, are
/// this desktop's to say for the copy it starts, not an older value the
/// manager may hold.
///
/// An entry with no `=` is not a variable and is left out; a name given twice
/// takes its last value. An empty `session` — the manager could not be asked —
/// is what the copy was started with before, and the caller says so on its log.
#[must_use]
pub fn alos_copy_starts_with(session: &[String], handed: &[String]) -> Vec<(String, String)> {
    let mut by_name = std::collections::BTreeMap::new();
    for pair in session.iter().chain(handed) {
        if let Some((name, value)) = pair.split_once('=')
            && !name.is_empty()
        {
            by_name.insert(name.to_owned(), value.to_owned());
        }
    }
    by_name.into_iter().collect()
}

#[cfg(test)]
#[path = "the_environment_tests.rs"]
mod tests;
