//! **The scope `try` asks the person's own manager for can hold a box**, two
//! levels down, and a process can join it.
//!
//! A booted machine on 2026-10-31 refused every `try` with *did not start
//! inside …/run-….scope/alo-try/alo-try: Permission denied*. The cgroup half
//! was the first suspect, so this asks exactly that half: the same
//! `systemd-run` arguments `try` uses (`the_scope_to_ask_for`), the same two
//! folders `try` and `alo-running` make, and a child writing itself into the
//! inner one, as `alo-enter-the-box` does.
//!
//! **Measured once by hand inside the image under podman, the same day, and it
//! held** — which is why the next suspect is the command starting inside the
//! box on a composefs `/`, not the box itself.
//!
//! # Where it runs, said rather than assumed
//!
//! It needs the person's own systemd manager, so it runs wherever that is
//! reachable — a machine somebody is signed in to, or a session opened for
//! it — and **says it skipped** where it is not, which includes the gate's
//! runners. A skip prints its reason so it cannot be read as a pass.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use alo_try::a_scope_of_its_own::{SYSTEMD_RUN, the_scope_to_ask_for};

/// Inside the scope: make the box's two folders, join the inner one from a
/// child, and say where the child ended up.
const INSIDE: &str = r#"
cg=/sys/fs/cgroup$(sed -n 's/^0:://p' /proc/self/cgroup)
mkdir "$cg/alo-try" "$cg/alo-try/alo-try" || exit 3
sh -c 'echo 0 > "$1/alo-try/alo-try/cgroup.procs" && sed -n "s/^0:://p" /proc/self/cgroup' _ "$cg"
joined=$?
rmdir "$cg/alo-try/alo-try" "$cg/alo-try" 2>/dev/null
exit $joined
"#;

#[test]
fn a_box_two_levels_down_in_the_scope_try_asks_for_can_be_joined() {
    if !Path::new(SYSTEMD_RUN).exists() {
        eprintln!("skipped: {SYSTEMD_RUN} is not on this machine");
        return;
    }
    let typed: Vec<OsString> = ["-c", INSIDE].into_iter().map(OsString::from).collect();
    let ran = Command::new(SYSTEMD_RUN)
        .args(the_scope_to_ask_for(Path::new("/bin/sh"), &typed))
        .output()
        .expect("systemd-run to start");
    let said = String::from_utf8_lossy(&ran.stdout);
    let complained = String::from_utf8_lossy(&ran.stderr);

    // No manager of the person's own to ask: nothing here can be measured.
    if !ran.status.success()
        && said.is_empty()
        && (complained.contains("Failed to connect")
            || complained.contains("No medium found")
            || complained.contains("XDG_RUNTIME_DIR"))
    {
        eprintln!("skipped: no user manager to ask for a scope here: {complained}");
        return;
    }

    assert!(
        ran.status.success(),
        "a box two levels down in the scope try asks for could not be made or joined \
         (exit {:?}): {complained}",
        ran.status.code()
    );
    assert!(
        said.trim_end().ends_with("/alo-try/alo-try") && said.contains(".scope/"),
        "the child did not end up in the box inside a scope of its own: {said:?}"
    );
}
