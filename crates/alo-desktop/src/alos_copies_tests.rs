//! Reading where alo's copy runs, and when its process began.

use std::path::{Path, PathBuf};

use super::*;

const SLICE: &str = "/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice";

/// **Where Flatpak moved alo's copy is read off the copy's own process**, and
/// accepted only inside the person's application slice.
#[test]
fn the_scope_flatpak_moved_alos_copy_into_is_read_off_its_process() {
    let said = "0::/user.slice/user-1000.slice/user@1000.service/app.slice/\
                app-flatpak-org.mozilla.firefox-2716289303.scope\n";
    assert_eq!(
        the_group_in(said, Path::new(SLICE)),
        Some(PathBuf::from(format!(
            "{SLICE}/app-flatpak-org.mozilla.firefox-2716289303.scope"
        )))
    );
}

/// A process somewhere else — the desktop's own group, the slice itself, a
/// system service — is not an application's, so nothing of it is counted.
#[test]
fn a_group_outside_the_persons_applications_is_never_alos_copy() {
    for said in [
        "0::/user.slice/user-1000.slice/user@1000.service/session.slice/alo-desktop.service\n",
        "0::/user.slice/user-1000.slice/user@1000.service/app.slice\n",
        "0::/system.slice/alo-agentd.service\n",
        "0::/user.slice/user-1001.slice/user@1001.service/app.slice/app-x.scope\n",
        "1:name=systemd:/app.slice/app-x.scope\n",
        "",
    ] {
        assert_eq!(the_group_in(said, Path::new(SLICE)), None, "{said}");
    }
}

/// Field 22, after a name that holds spaces and a closing bracket of its own.
#[test]
fn when_a_process_began_is_read_after_its_name() {
    let stat = "4242 (Web Content (x)) S 1 4242 4242 0 -1 4194560 100 0 0 0 1 2 0 0 20 0 \
                30 0 987654 123456 789";
    assert_eq!(began_in(stat), Some(987_654));
}

/// A process that has ended and not been reaped is not a copy still running.
#[test]
fn an_ended_process_has_not_begun() {
    let stat = "4242 (firefox) Z 1 4242 4242 0 -1 4194560 100 0 0 0 1 2 0 0 20 0 30 0 987654";
    assert_eq!(began_in(stat), None);
    assert_eq!(began_in("4242 (firefox"), None);
}

/// And this test's own process has begun, read the real way.
#[test]
fn this_process_has_a_beginning() {
    assert!(began_at(std::process::id()).is_some());
}

/// **A group holds a process when its own list, or one beneath it, names
/// one** — how alo's copy is still known after the process alo started has
/// ended and VSCodium runs on in Flatpak's scope (the walk of 2026-11-15, run
/// 43726518930).
///
/// What would have to be true for this to pass while broken: the kernel
/// listing a scope's processes somewhere other than `cgroup.procs`, which is
/// cgroup v2's own interface and what this machine runs.
#[test]
fn a_group_holds_a_process_when_it_or_one_beneath_lists_one() {
    let made = tempfile::tempdir();
    assert!(made.is_ok(), "no folder to stand in for the groups");
    let Ok(at) = made else {
        return;
    };
    let scope = at
        .path()
        .join("app-flatpak-com.vscodium.codium-2042859277.scope");
    let beneath = scope.join("inner");
    assert!(std::fs::create_dir_all(&beneath).is_ok());
    assert!(std::fs::write(scope.join("cgroup.procs"), "").is_ok());
    assert!(std::fs::write(beneath.join("cgroup.procs"), "").is_ok());
    assert!(
        !holds_a_process(&scope, 0),
        "an empty scope was read as running"
    );

    assert!(std::fs::write(beneath.join("cgroup.procs"), "7175\n").is_ok());
    assert!(holds_a_process(&scope, 0), "a process beneath was missed");

    assert!(std::fs::write(beneath.join("cgroup.procs"), "").is_ok());
    assert!(std::fs::write(scope.join("cgroup.procs"), "7175\n7176\n").is_ok());
    assert!(holds_a_process(&scope, 0));

    assert!(
        !holds_a_process(&at.path().join("a-scope-that-went.scope"), 0),
        "a group that has gone was read as running"
    );
}
