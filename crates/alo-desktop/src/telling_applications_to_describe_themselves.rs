//! **Telling applications whether to describe themselves** to alo and to
//! screen readers — the person's switch in Settings → Accessibility, on as a
//! machine comes (the owner, 2026-11-11).
//!
//! `alo_launching::describing_themselves` decides what each toolkit is told;
//! this tells it. Three moments:
//!
//! - **at sign-in**, the person's session is told only if they answered in
//!   Settings — otherwise the image's default stands
//!   (`/usr/lib/environment.d/60-alo-apps-describe-themselves.conf`), and so
//!   does anything they set themselves in their own `~/.config/environment.d`,
//!   for the reason `crate::telling_applications_about_the_pointer` gives;
//! - **when they turn the switch over**, the session is told at once, so the
//!   next application started by anything is told the new answer;
//! - **at each start the desktop makes**, the answer is read again from their
//!   `access.toml` and handed to that application, with Chromium's switch for
//!   the page where it is one drawn by Chromium.
//!
//! **What it does not reach: an application already running** keeps what it
//! was told when it started, which is what Settings says under the switch.
//!
//! **Nothing here reads anybody's tree**; it decides whether an application
//! is able to answer when alo, in a turn the person started, or a screen
//! reader they run, asks it (`CLAUDE.md`: *context is offered, never watched*).

use std::collections::HashMap;

/// The person's answer, read from their `access.toml` now: whether
/// applications describe themselves, and whether the person said so
/// themselves.
///
/// **A file that does not read is the default** — applications describe
/// themselves, and nobody chose — because a broken settings file is not a
/// person who turned something off. Settings says why it did not read.
#[must_use]
pub fn the_persons_answer() -> (bool, bool) {
    let Some(folder) = alo_choosing::where_the_folder_is(
        std::env::var_os("XDG_CONFIG_HOME").as_deref(),
        std::env::var_os("HOME").as_deref(),
    ) else {
        return (true, false);
    };
    let (turned_on, _) = alo_access::keeping::at_sign_in(&folder);
    (
        turned_on.apps_describe_themselves(),
        turned_on.chose_whether_apps_describe_themselves(),
    )
}

/// Tell the person's manager and the session bus whether applications started
/// from now on describe themselves, over a connection of its own, and say on
/// the log what could not be told.
pub fn tell(describe: bool) {
    let told = alo_launching::describing_themselves::what_the_session_is_told(describe);
    let session = match zbus::blocking::Connection::session() {
        Ok(session) => session,
        Err(why) => {
            eprintln!(
                "alo-desktop: the session bus could not be reached, so applications started \
                 from now on are told what they were told before about describing themselves: \
                 {why}"
            );
            return;
        }
    };
    let connection = session.inner();
    let cost = "whether applications describe themselves, so one started from now on may do \
                what the person turned off";
    let to_the_manager: Vec<String> = told
        .everywhere
        .iter()
        .chain(&told.the_manager_only)
        .cloned()
        .collect();
    let asked = |method: &str, names: &[String]| {
        zbus::block_on(connection.call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            method,
            &(names.to_vec(),),
        ))
    };
    if let Err(why) = asked("SetEnvironment", &to_the_manager) {
        eprintln!("alo-desktop: the person's systemd manager was not told {cost}: {why}");
    }
    let forgotten: Vec<String> = told.forgotten.iter().map(|it| (*it).to_owned()).collect();
    if !forgotten.is_empty()
        && let Err(why) = asked("UnsetEnvironment", &forgotten)
    {
        eprintln!("alo-desktop: the person's systemd manager did not forget {cost}: {why}");
    }
    // **The bus is told only the values it can be told again**: it has no
    // call that forgets a variable (`alo_launching::describing_themselves`).
    let pairs: HashMap<&str, &str> = told
        .everywhere
        .iter()
        .filter_map(|it| it.split_once('='))
        .collect();
    let to_the_bus = zbus::block_on(connection.call_method(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        Some("org.freedesktop.DBus"),
        "UpdateActivationEnvironment",
        &(pairs,),
    ));
    if let Err(why) = to_the_bus {
        eprintln!("alo-desktop: the session bus was not told {cost}: {why}");
    }
}
