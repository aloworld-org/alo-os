//! What an application is handed, as the desktop unit lays it out.

use std::ffi::OsStr;
use std::path::Path;

use super::*;

/// Where `alo-desktop@.service`'s `RuntimeDirectory=alo-desktop` and the
/// shell's socket put the display.
const THE_SOCKET: &str = "/run/alo-desktop/wayland-alo/wayland";

#[test]
fn a_started_application_is_handed_the_display_the_session_type_and_the_data_dirs() {
    assert_eq!(
        what_an_application_is_handed(
            Path::new(THE_SOCKET),
            OsStr::new("/var/lib/flatpak/exports/share:/usr/local/share:/usr/share")
        ),
        Some(vec![
            format!("WAYLAND_DISPLAY={THE_SOCKET}"),
            "XDG_SESSION_TYPE=wayland".to_owned(),
            "XDG_DATA_DIRS=/var/lib/flatpak/exports/share:/usr/local/share:/usr/share".to_owned(),
        ])
    );
}

#[test]
fn the_session_is_told_the_display_and_not_the_data_dirs() {
    assert_eq!(
        what_the_session_is_told(Path::new(THE_SOCKET)),
        Some(vec![
            format!("WAYLAND_DISPLAY={THE_SOCKET}"),
            "XDG_SESSION_TYPE=wayland".to_owned(),
        ])
    );
}

/// A relative socket would be read against `XDG_RUNTIME_DIR`, which is not
/// where alo's is, so it is refused rather than handed over wrong.
#[test]
fn a_socket_that_is_not_an_absolute_path_is_not_handed_over() {
    assert_eq!(what_the_session_is_told(Path::new("wayland-alo")), None);
    assert_eq!(
        what_the_session_is_told(Path::new("/run/a\nWAYLAND_DISPLAY=elsewhere")),
        None,
        "a line break would be a second variable"
    );
}

/// **alo's copy starts with the person's session, and what the desktop hands
/// it wins**: the runtime folder, the bus, `PATH` and `HOME` are the
/// session's, and where alo draws is the desktop's even where the session holds
/// an older one.
///
/// What would have to be true for this to pass while broken: the desktop
/// starting the copy without calling this — `alo-desktop`'s `alos_copies`
/// names it in both of the places a copy is started.
#[test]
fn alos_copy_starts_with_the_persons_session_and_the_desktops_display_on_top() {
    let session: Vec<String> = [
        "HOME=/var/home/anna",
        "PATH=/usr/local/bin:/usr/bin",
        "XDG_RUNTIME_DIR=/run/user/1000",
        "DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus",
        "WAYLAND_DISPLAY=/run/an-older-display",
        "not a variable",
    ]
    .map(str::to_owned)
    .to_vec();
    let handed = what_an_application_is_handed(Path::new(THE_SOCKET), OsStr::new("/usr/share"))
        .unwrap_or_default();
    let started = alos_copy_starts_with(&session, &handed);
    let value = |name: &str| {
        started
            .iter()
            .find(|(named, _)| named == name)
            .map(|(_, value)| value.as_str())
    };
    assert_eq!(value("HOME"), Some("/var/home/anna"));
    assert_eq!(value("XDG_RUNTIME_DIR"), Some("/run/user/1000"));
    assert_eq!(
        value("DBUS_SESSION_BUS_ADDRESS"),
        Some("unix:path=/run/user/1000/bus")
    );
    assert_eq!(value("PATH"), Some("/usr/local/bin:/usr/bin"));
    assert_eq!(value("WAYLAND_DISPLAY"), Some(THE_SOCKET));
    assert_eq!(value("XDG_DATA_DIRS"), Some("/usr/share"));
    assert!(started.iter().all(|(name, _)| !name.contains(' ')));
}

/// **A session that could not be read leaves what the desktop hands**, which
/// is what a copy was started with before, and nothing invented.
#[test]
fn with_no_session_alos_copy_has_only_what_it_is_handed() {
    let handed = vec!["WAYLAND_DISPLAY=/run/x".to_owned()];
    assert_eq!(
        alos_copy_starts_with(&[], &handed),
        vec![("WAYLAND_DISPLAY".to_owned(), "/run/x".to_owned())]
    );
}
