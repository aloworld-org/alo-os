//! **Where `try` goes when the cgroup it is in is not the person's.**
//!
//! A box is a child of the cgroup `try` is already in
//! ([`crate::where_the_box_goes`]), which works where that cgroup was delegated
//! to the person. Until 2026-10-30 it always was: the person's shell descended
//! from `alo-desktop.service`, which said `Delegate=yes`.
//!
//! [ADR 0141](../../../docs/decisions/0141-the-desktop-is-put-inside-the-persons-session-by-what-already-has-the-privilege.md)
//! moved the desktop into the person's login session, so that restart, power
//! off and the time zone are theirs. **A session's scope is `logind`'s, and not
//! the person's.** Measured inside the image that day, as the person:
//!
//! ```text
//! in: /sys/fs/cgroup/user.slice/user-4711.slice/session-c1.scope
//! owner: root:root
//! mkdir: cannot create directory '.../session-c1.scope/alo-try': Permission denied
//! ```
//!
//! — and the same with a `Delegate=yes` drop-in for every session scope, which
//! `logind`'s scopes do not take. **The person's own manager can give them
//! one**, measured the same day:
//!
//! ```text
//! systemd-run --user --scope -p Delegate=yes …
//! in: /sys/fs/cgroup/user.slice/user-4711.slice/user@4711.service/app.slice/run-p543-i232117.scope
//! owner: aloprobe:aloprobe
//! mkdir: ok, type domain
//! move: ok
//! ```
//!
//! So when the box cannot be made where `try` is, `try` runs itself again
//! inside a scope of its own, once. Only the `try` process and what it runs
//! move; the terminal it was typed in stays in the person's session.
//!
//! **This is the person's own command, not a capability.** The scope is asked
//! of the person's own manager, as the person, with nothing a person could not
//! type themselves.

use std::ffi::OsString;
use std::path::Path;

/// What carries the fact that this `try` has already asked for a scope, so
/// that it asks once and never loops.
pub const IN_A_SCOPE_OF_ITS_OWN: &str = "ALO_TRY_IN_A_SCOPE_OF_ITS_OWN";

/// The tool that asks the person's own manager for a scope.
///
/// A fixed path rather than a `PATH` lookup, because what runs here must not
/// depend on a folder somebody put first in their `PATH`.
pub const SYSTEMD_RUN: &str = "/usr/bin/systemd-run";

/// What to do when the folder a box goes in could not be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhenTheBoxCannotBeMade {
    /// Run this `try` again inside a delegated scope from the person's own
    /// manager.
    AskForAScopeOfItsOwn,
    /// Say so and stop.
    Refuse,
}

/// Decide, from why the folder was refused and whether this `try` has asked
/// already.
///
/// **Only a refusal of permission is a cgroup that is not the person's**, and
/// only the first time: a `try` already in a scope of its own that is still
/// refused is a machine whose manager does not delegate, and asking again would
/// loop for ever.
#[must_use]
pub fn when_the_box_cannot_be_made(
    why: std::io::ErrorKind,
    asked_already: bool,
) -> WhenTheBoxCannotBeMade {
    if why == std::io::ErrorKind::PermissionDenied && !asked_already {
        WhenTheBoxCannotBeMade::AskForAScopeOfItsOwn
    } else {
        WhenTheBoxCannotBeMade::Refuse
    }
}

/// What [`SYSTEMD_RUN`] is told: a delegated scope from the person's own
/// manager, and this program with exactly what the person typed.
///
/// `--` before the program, so nothing the person typed is read as an option
/// of `systemd-run`'s.
#[must_use]
pub fn the_scope_to_ask_for(this_program: &Path, typed: &[OsString]) -> Vec<OsString> {
    let mut asked: Vec<OsString> = [
        "--user",
        "--scope",
        "--quiet",
        "--collect",
        "--property=Delegate=yes",
        "--",
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    asked.push(this_program.as_os_str().to_owned());
    asked.extend(typed.iter().cloned());
    asked
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    /// **A cgroup that is not the person's asks for a scope, once.**
    #[test]
    fn a_refused_cgroup_asks_for_a_scope_of_its_own_once() {
        assert_eq!(
            when_the_box_cannot_be_made(ErrorKind::PermissionDenied, false),
            WhenTheBoxCannotBeMade::AskForAScopeOfItsOwn
        );
        assert_eq!(
            when_the_box_cannot_be_made(ErrorKind::PermissionDenied, true),
            WhenTheBoxCannotBeMade::Refuse,
            "a try already in a scope of its own asked again, which is a loop"
        );
    }

    /// **Anything else is refused**: a missing cgroup filesystem or a full one
    /// is not fixed by a scope, and running again would hide what was wrong.
    #[test]
    fn a_folder_refused_for_any_other_reason_is_not_asked_again() {
        for other in [
            ErrorKind::NotFound,
            ErrorKind::ReadOnlyFilesystem,
            ErrorKind::Other,
        ] {
            assert_eq!(
                when_the_box_cannot_be_made(other, false),
                WhenTheBoxCannotBeMade::Refuse,
                "{other:?} asked for a scope"
            );
        }
    }

    /// **The scope is delegated and the person's own, and what they typed
    /// arrives untouched after `--`.**
    #[test]
    fn the_scope_is_the_persons_own_and_delegated_and_carries_what_was_typed() {
        let typed: Vec<OsString> = ["--only-here", "make", "--version"]
            .into_iter()
            .map(OsString::from)
            .collect();
        let asked = the_scope_to_ask_for(Path::new("/usr/bin/alo-try"), &typed);
        let asked: Vec<&str> = asked.iter().filter_map(|it| it.to_str()).collect();

        assert_eq!(
            asked,
            [
                "--user",
                "--scope",
                "--quiet",
                "--collect",
                "--property=Delegate=yes",
                "--",
                "/usr/bin/alo-try",
                "--only-here",
                "make",
                "--version",
            ]
        );
        assert!(
            !asked.contains(&"--system"),
            "a scope from the machine's manager is root's"
        );
    }
}
