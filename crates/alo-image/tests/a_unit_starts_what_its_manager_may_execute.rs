//! A unit's manager must be allowed to execute what the unit names.
//!
//! **This is for a machine on which the keyring never started.** Measured on a
//! booted image on 2026-10-23: `alo-keyring.service` failed **364 times in
//! three hours**, two seconds apart, every one of them
//!
//! ```text
//! Unable to locate executable '/usr/bin/gnome-keyring-daemon': Permission denied
//! avc: denied { execute } scontext=system_u:system_r:init_t:s0
//!                         tcontext=system_u:object_r:gkeyringd_exec_t:s0
//! ```
//!
//! The unit was right, the binary was present, and it ran when asked by hand.
//! What it could not survive is **who asked**: a system unit is executed by
//! PID 1, and Fedora's policy lets only a person's own session execute
//! `gkeyringd_exec_t`.
//!
//! # The rule this holds, and what it cannot hold
//!
//! Our own binaries live under `/usr/libexec` and carry a generic label PID 1
//! may execute — `alo-portald` is one, same shape, and works. A unit naming a
//! **distribution's** binary is the case to look at, and the keyring is the
//! only one today.
//!
//! **This does not read SELinux policy**, and could not: the labels exist on a
//! booted machine, not in this repository. What it does is far narrower and
//! still worth having — it pins the keyring to the *user* directory, so moving
//! it back to `system/` fails here rather than on somebody's machine three
//! hours into a session. The reason lives in the unit; this makes the reason
//! load-bearing.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_image::THE_IMAGE;

/// Units that name a binary this distribution packages, and must therefore be
/// started by the person's own manager rather than by PID 1.
const THE_DISTRIBUTIONS_OWN: [(&str, &str); 1] =
    [("alo-keyring.service", "/usr/bin/gnome-keyring-daemon")];

/// **A unit naming a packaged binary is a user unit**, because PID 1 may not
/// execute what the distribution labels for a session.
#[test]
fn a_unit_naming_the_distributions_own_binary_is_a_user_unit() {
    for (unit, binary) in THE_DISTRIBUTIONS_OWN {
        let as_a_user_unit = Path::new(THE_IMAGE).join("usr/lib/systemd/user").join(unit);
        let as_a_system_unit = Path::new(THE_IMAGE)
            .join("usr/lib/systemd/system")
            .join(unit);

        assert!(
            as_a_user_unit.is_file(),
            "{unit} names {binary}, which this distribution labels for a \
             person's session. It belongs in usr/lib/systemd/user so the \
             person's own manager starts it. As a system unit it is executed \
             by PID 1, and SELinux refused that 364 times on a booted machine \
             without one line of it being the unit's fault."
        );
        assert!(
            !as_a_system_unit.exists(),
            "{unit} is in both places, so which one a machine uses is whichever \
             systemd looked at first"
        );

        // The unit must still actually name that binary, or this test is
        // pinning a file that has nothing to do with the rule.
        let said = std::fs::read_to_string(&as_a_user_unit)
            .unwrap_or_else(|why| panic!("{unit} did not read: {why}"));
        assert!(
            said.contains(binary),
            "{unit} no longer names {binary}, so this test is guarding a rule \
             that has moved"
        );
    }
}

/// **And it does not name what only a system unit can mean.**
///
/// `User=`, `Group=` and `BindsTo=user@1000.service` are how a *system* unit
/// reaches into a person's session. Inside their own manager each is either
/// redundant or wrong, and leaving one behind is how a file half-moves.
#[test]
fn the_user_unit_does_not_keep_what_only_a_system_unit_needs() {
    for (unit, _) in THE_DISTRIBUTIONS_OWN {
        let at = Path::new(THE_IMAGE).join("usr/lib/systemd/user").join(unit);
        let said =
            std::fs::read_to_string(&at).unwrap_or_else(|why| panic!("{unit} did not read: {why}"));
        for line in said.lines().map(str::trim) {
            for forbidden in ["User=", "Group=", "BindsTo=user@", "WantedBy=user@"] {
                assert!(
                    !line.starts_with(forbidden),
                    "{unit} is a user unit and still says `{line}` — that is a \
                     system unit reaching into a session it is already inside"
                );
            }
        }
    }
}
