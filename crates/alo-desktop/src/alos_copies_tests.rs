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
