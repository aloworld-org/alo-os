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
