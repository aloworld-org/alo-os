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

/// **A program this desktop starts can write its instance on the pipe, by the
/// number it is told, and this desktop hears it** — both sides real: a real
/// child process inherits the write end under that number and writes to it,
/// as `flatpak run --instance-id-fd` does (`flatpak-run.c:3219-3221`, read
/// not run), and the read end hears nothing before and the number after.
///
/// What would have to be true for this to pass while broken: Flatpak not
/// writing on the descriptor it is told, or writing something that is not a
/// number — which the walk is where it is seen.
#[test]
fn a_started_program_can_say_its_instance_on_the_pipe_and_it_is_heard() {
    use std::os::fd::AsRawFd as _;

    let made = an_instance_pipe();
    assert!(made.is_some(), "the kernel made no pipe");
    let Some((read, write)) = made else {
        return;
    };
    assert_eq!(the_instance_heard(&read), Heard::Nothing);
    let fd = write.as_raw_fd();
    let ran = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("printf 2042859277 >&{fd}"))
        .status();
    drop(write);
    assert!(
        ran.is_ok_and(|status| status.success()),
        "the child could not write on {fd}"
    );
    assert_eq!(
        the_instance_heard(&read),
        Heard::It("2042859277".to_owned())
    );
    // *Never* once it closes is not asserted here: a test running beside this
    // one may have started a program that inherited the write end too, and
    // holds it open. Hearing the number is what the desktop waits for.
}

/// **The read end is never handed to a program this desktop starts**: only
/// the write end crosses, so no started program can read what another said.
#[test]
fn the_read_end_is_not_inherited() {
    use std::os::fd::AsRawFd as _;

    let made = an_instance_pipe();
    assert!(made.is_some(), "the kernel made no pipe");
    let Some((read, _write)) = made else {
        return;
    };
    let fd = read.as_raw_fd();
    let ran = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("[ -e /proc/self/fd/{fd} ]"))
        .status();
    assert!(
        ran.is_ok_and(|status| !status.success()),
        "the read end {fd} was inherited"
    );
}

/// **A group is kept once, and no more than there is room for** — so a copy
/// that moves between scopes is remembered in each, and a copy that keeps
/// moving cannot grow the list for ever.
#[test]
fn a_group_is_kept_once_and_only_while_there_is_room() {
    let mut seen = Vec::new();
    kept(&mut seen, PathBuf::from("/g/a"));
    kept(&mut seen, PathBuf::from("/g/a"));
    assert_eq!(seen, [PathBuf::from("/g/a")], "one group kept twice");
    for n in 1..THE_MOST_SEEN {
        kept(&mut seen, PathBuf::from(format!("/g/{n}")));
    }
    assert_eq!(
        seen.len(),
        THE_MOST_SEEN,
        "room was refused before it ran out"
    );
    kept(&mut seen, PathBuf::from("/g/one-too-many"));
    assert_eq!(seen.len(), THE_MOST_SEEN, "a group was kept past the room");
}

/// **A copy runs while the process alo started does, or while anything runs
/// in a group it was seen in — and not otherwise** — which is what keeps the
/// copy known after `flatpak run` ends (the walk of 2026-11-15, run
/// 43726518930) and lets it go when VSCodium closes.
#[test]
fn a_copy_runs_while_its_launcher_or_anything_in_its_groups_does() {
    let made = tempfile::tempdir();
    assert!(made.is_ok(), "no folder to stand in for the groups");
    let Ok(at) = made else {
        return;
    };
    let scope = at.path().join("app-flatpak-com.vscodium.codium-1.scope");
    assert!(std::fs::create_dir_all(&scope).is_ok());
    assert!(std::fs::write(scope.join("cgroup.procs"), "").is_ok());
    let seen = vec![scope.clone()];

    assert!(still_alive(true, &[]), "the launcher alone was not enough");
    assert!(!still_alive(false, &[]), "nothing was read as running");
    assert!(
        !still_alive(false, &seen),
        "an empty scope was read as running"
    );
    assert!(std::fs::write(scope.join("cgroup.procs"), "7175\n").is_ok());
    assert!(
        still_alive(false, &seen),
        "the copy was forgotten while it ran"
    );
}

/// **A read that fails for any reason but *not yet* is *never***, so a pipe
/// that cannot be read is let go rather than looked at for ever.
#[test]
fn a_pipe_that_cannot_be_read_is_never() {
    let made = an_instance_pipe();
    assert!(made.is_some(), "the kernel made no pipe");
    let Some((_read, write)) = made else {
        return;
    };
    // The write end cannot be read: the kernel answers that it is not open
    // for reading, which is neither *not yet* nor an interruption.
    assert_eq!(the_instance_heard(&write), Heard::Never);
}

/// This test process's own control group and the folder it is in, as the
/// kernel says — or [`None`] where the machine has no cgroup v2 line for it.
fn this_process_group() -> Option<(PathBuf, PathBuf)> {
    let said = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    let path = said.lines().find_map(|line| line.strip_prefix("0::"))?;
    let group = Path::new(CGROUPS).join(path.trim_start_matches('/'));
    let slice = group.parent()?.to_path_buf();
    (group != slice && slice.starts_with(CGROUPS)).then_some((group, slice))
}

/// **A copy this desktop started runs while its process does, and stops when
/// it ends** — with a real process, and a slice nothing of it is in.
#[test]
fn a_copy_started_here_runs_while_its_process_does() {
    let started = std::process::Command::new("/bin/sleep").arg("30").spawn();
    assert!(started.is_ok(), "no process to stand in for the copy");
    let Ok(child) = started else {
        return;
    };
    let pid = child.id();
    let mut copy = Running {
        pid,
        began: began_at(pid).unwrap_or(0),
        child: Some(child),
        unit: None,
        seen: Vec::new(),
        instance: None,
    };
    let nowhere = Path::new("/sys/fs/cgroup/no-such.slice");
    assert!(looked_at(&mut copy, "com.vscodium.codium", nowhere));
    if let Some(child) = copy.child.as_mut() {
        let _ = child.kill();
        let _ = child.wait();
    }
    assert!(
        !looked_at(&mut copy, "com.vscodium.codium", nowhere),
        "a copy whose process ended, seen nowhere, still ran"
    );
}

/// **A copy found again after a restart keeps its group only while its
/// process is the one written down** — the process asked when it began, and
/// a different answer keeps nothing.
///
/// What would have to be true for this to pass while broken: this machine
/// running tests outside cgroup v2, where there is no group to keep and the
/// test says so and stops.
#[test]
fn a_copy_found_again_keeps_its_group_only_while_it_is_the_same_process() {
    let Some((group, slice)) = this_process_group() else {
        eprintln!("no cgroup v2 group for this test; nothing to keep");
        return;
    };
    let pid = std::process::id();
    let began = began_at(pid).unwrap_or(0);
    let mut the_same = Running {
        pid,
        began,
        child: None,
        unit: None,
        seen: Vec::new(),
        instance: None,
    };
    assert!(looked_at(&mut the_same, "com.vscodium.codium", &slice));
    assert_eq!(
        the_same.seen,
        [group],
        "the same process's group was not kept"
    );

    let mut another = Running {
        pid,
        began: began.wrapping_add(1),
        child: None,
        unit: None,
        seen: Vec::new(),
        instance: None,
    };
    assert!(
        !looked_at(&mut another, "com.vscodium.codium", &slice),
        "a process that began at another moment was read as the copy"
    );
    assert!(another.seen.is_empty(), "another process's group was kept");
}
