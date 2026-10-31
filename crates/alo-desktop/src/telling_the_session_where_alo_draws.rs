//! **Telling the person's session where alo draws**, once, at sign-in.
//!
//! An application the *bus* starts — Text Editor, Loupe, Papers, Ptyxis, every
//! entry that says `DBusActivatable=true` — is started by the session bus or by
//! the person's systemd manager, with **their** environment, not this
//! desktop's. Neither had ever been told `WAYLAND_DISPLAY`, so such an
//! application was activated, looked for a display, found none, and exited
//! without a window. Every desktop tells them at sign-in; GNOME, KDE and the
//! tiling compositors' documentation all spell it
//! `dbus-update-activation-environment --systemd WAYLAND_DISPLAY …`.
//!
//! This is those two calls, over the connection the desktop already holds:
//! `SetEnvironment` on the person's systemd manager, and
//! `UpdateActivationEnvironment` on the bus itself — both, because which one
//! starts a bus-activated application depends on how the bus was built, and
//! a machine is not obliged to be either.
//!
//! **What changes for the person's other services**: anything their manager
//! starts after this sees alo's display, which is what a session is. Nothing is
//! taken from the environment, and `XDG_DATA_DIRS` is left as the manager has
//! it (`alo_launching::the_environment` says why).

use std::collections::HashMap;
use std::path::Path;

/// Tell the session's bus and the person's manager where alo draws, and say
/// on the log what could not be told.
pub fn tell(connection: &zbus::Connection, wayland: &Path) {
    let Some(told) = alo_launching::what_the_session_is_told(wayland) else {
        eprintln!(
            "alo-desktop: the display at {} is not a path the session can be told, so an \
             application the bus starts will find no display",
            wayland.display()
        );
        return;
    };
    let to_the_manager = zbus::block_on(connection.call_method(
        Some("org.freedesktop.systemd1"),
        "/org/freedesktop/systemd1",
        Some("org.freedesktop.systemd1.Manager"),
        "SetEnvironment",
        &(told.clone(),),
    ));
    if let Err(why) = to_the_manager {
        eprintln!(
            "alo-desktop: the person's systemd manager was not told where alo draws, so an \
             application it starts may find no display: {why}"
        );
    }
    let pairs: HashMap<&str, &str> = told.iter().filter_map(|it| it.split_once('=')).collect();
    let to_the_bus = zbus::block_on(connection.call_method(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        Some("org.freedesktop.DBus"),
        "UpdateActivationEnvironment",
        &(pairs,),
    ));
    if let Err(why) = to_the_bus {
        eprintln!(
            "alo-desktop: the session bus was not told where alo draws, so an application it \
             starts may find no display: {why}"
        );
    }
}
