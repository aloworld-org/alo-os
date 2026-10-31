//! An application installed after the desktop started is noticed.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::time::Duration;

use alo_software::rented::TheRentedTool;
use alo_software::the_applications_arrived::TheApplicationsArrived;

use super::{kept_watching, what_is_there};
use crate::what_this_machine_offers::{WhereToLook, what_is_offered};

/// Places of a test's own, with nothing installed in them yet — and no
/// flatpak export directory at all, which is a fresh machine before its first
/// application.
fn a_fresh_machine(named: &str) -> (std::path::PathBuf, WhereToLook) {
    let root =
        std::env::temp_dir().join(format!("alo-desktop-watch-{}-{named}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("usr/share/applications")).expect("a directory");
    let at = WhereToLook::beneath(
        &root.join("var/lib/flatpak"),
        Some(&root.join("home/someone")),
        None,
        Some(std::ffi::OsStr::new(
            &root.join("usr/share").display().to_string(),
        )),
        TheApplicationsArrived::beneath(&root),
    );
    (root, at)
}

/// Install one application the way flatpak does: its export directory is made
/// if it is not there, and an entry appears in it.
fn flatpak_installs(root: &std::path::Path, identifier: &str, name: &str) {
    let exports = root.join("var/lib/flatpak/exports/share/applications");
    std::fs::create_dir_all(&exports).expect("the export directory");
    std::fs::write(
        exports.join(format!("{identifier}.desktop")),
        format!("[Desktop Entry]\nType=Application\nName={name}\n"),
    )
    .expect("an entry");
}

/// **An application installed after start is picked up**, from a directory
/// that did not exist when the watching began — the export directory before
/// the first flatpak.
///
/// The tool is one that is not there, so the roles are `Failed` and the test
/// is about the entries alone: a reading arrives, and it holds the application.
#[test]
fn an_application_added_after_start_is_picked_up() {
    let (root, at) = a_fresh_machine("added");
    let before = what_is_there(&at);
    let (tell, told) = std::sync::mpsc::channel();
    let watching = at.clone();
    let _thread = std::thread::spawn(move || {
        kept_watching(
            &watching,
            before,
            Duration::from_millis(20),
            |at| what_is_offered(at, &TheRentedTool::at("/nonexistent/alo-test-tool")),
            &tell,
        );
    });

    assert!(
        told.recv_timeout(Duration::from_millis(300)).is_err(),
        "the control: nothing changed, and a reading was taken anyway"
    );
    flatpak_installs(&root, "org.mozilla.firefox", "Firefox");
    let offered = told
        .recv_timeout(Duration::from_secs(5))
        .expect("an installed application was never noticed");
    assert!(
        offered
            .installed
            .iter()
            .any(|it| it.identifier == "org.mozilla.firefox"),
        "a reading arrived and did not hold what was installed: {:?}",
        offered.installed
    );
    let _ = std::fs::remove_dir_all(root);
}

/// **The installing finishing is a change of its own**, though no entry moved:
/// it is what turns the Dock's answer from *so far* into *final*.
#[test]
fn the_installing_finishing_is_noticed() {
    let (root, at) = a_fresh_machine("finished");
    let before = what_is_there(&at);
    at.arrived.record().expect("the mark");
    assert_ne!(before, what_is_there(&at));
    let _ = std::fs::remove_dir_all(root);
}

/// **Removing one is a change too**, so an application uninstalled from the
/// market leaves the launcher.
#[test]
fn an_application_removed_is_a_change() {
    let (root, at) = a_fresh_machine("removed");
    flatpak_installs(&root, "org.kde.dolphin", "Dolphin");
    let before = what_is_there(&at);
    std::fs::remove_file(
        root.join("var/lib/flatpak/exports/share/applications/org.kde.dolphin.desktop"),
    )
    .expect("removed");
    assert_ne!(before, what_is_there(&at));
    let _ = std::fs::remove_dir_all(root);
}
