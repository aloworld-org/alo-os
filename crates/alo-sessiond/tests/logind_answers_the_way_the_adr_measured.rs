//! The measurement ADR 0024 said it owed, as a test rather than as a paragraph.
//!
//! The ADR was accepted on one finding: `systemd-logind` will open a session
//! for a caller that is not `pam_systemd`, provided that caller is privileged.
//! `CreateSession` answers root *Leader PID is not valid* — the call
//! authorised, and only its contents refused — and an unprivileged caller
//! *Access denied*. `docs/quirks.md` records it on both systemds it has been
//! taken on, including the one thing that differs between them.
//!
//! What this file adds is that **`alo-sessiond`'s own call is the call that was
//! measured**. A measurement taken with `busctl` proves something about
//! `logind`; it proves nothing about the arguments this crate assembles, the
//! interface it names, or the reply it expects. So the unprivileged half is
//! asked here through `TheMachinesLogind`, on whatever bus the machine running
//! the tests has.
//!
//! # The privileged half is deliberately not run
//!
//! A test running as root would not be refused — it would **open a session on
//! the developer's machine**, with this test process as its leader. That is a
//! shared-system change of exactly the kind `docs/autonomy/SHARED_MAIN.md`
//! says to coordinate rather than make, and it is a strange thing for
//! `cargo test` to do. So the privileged answer stays a by-hand measurement,
//! written into `docs/quirks.md` with the machine and the systemd version
//! beside it, and this test **skips itself** when it happens to be root rather
//! than pretending to have checked.
//!
//! # A machine with no system bus is not a failure either
//!
//! The workspace gate runs on machines with no `logind` — a container, a build
//! host, a Windows checkout. There is nothing to measure there, and a test that
//! failed would be reporting the absence of a bus as a defect in this crate. It
//! says what it skipped and why, which is the honest floor: a skip nobody can
//! see is the same colour as a pass.

#![cfg(target_os = "linux")]
#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_sessiond::{Logind, NotOpened, TheMachinesLogind};

/// The D-Bus error name a refused caller is turned away under.
///
/// The **name**, not the sentence. The sentence differs between a bus policy
/// that refuses the call before `logind` sees it and a `logind` that refuses it
/// itself — `docs/quirks.md` records both wordings — and the name is the same
/// in either case.
const ACCESS_DENIED: &str = "org.freedesktop.DBus.Error.AccessDenied";

/// **An unprivileged caller is refused, through this crate's own call.**
///
/// The one half of ADR 0024's measurement that can be taken inside a test
/// without changing the machine it runs on. It is the security half: if this
/// stopped being true, anything on the machine could open a session for
/// anybody, and the privilege this component is trusted with would have stopped
/// being the boundary.
#[test]
fn an_unprivileged_caller_is_refused_by_logind() {
    if rustix::process::geteuid().is_root() {
        eprintln!(
            "skipped: this process is root, and the privileged half of ADR 0024's measurement \
             would open a real session on this machine — it is taken by hand and recorded in \
             docs/quirks.md"
        );
        return;
    }

    let mut logind = TheMachinesLogind::default();
    match logind.open_a_session_for(1000) {
        Ok(()) => panic!(
            "systemd-logind opened a session for an unprivileged caller — ADR 0024's boundary is \
             privilege, and on this machine it is not"
        ),
        Err(NotOpened::NoBus { why }) => {
            eprintln!("skipped: this machine has no system bus to ask ({why})");
        }
        Err(NotOpened::Refused { person, named, why }) => {
            assert_eq!(person, 1000);
            assert_eq!(
                named, ACCESS_DENIED,
                "logind refused an unprivileged caller with something other than access denied, \
                 which is not the answer ADR 0024 was accepted on: {why}"
            );
        }
        Err(why @ NotOpened::NotUnderstood { .. }) => panic!(
            "systemd-logind answered an unprivileged caller with a reply this crate could not \
             read, which means it was not refused: {why}"
        ),
    }
}
