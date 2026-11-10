//! **The process polkit is asked about**: one process in the person's desktop
//! session, named so a recycled number cannot stand in for it.
//!
//! polkit 126 aborts when `CheckAuthorization` is asked about a `unix-session`
//! subject: its rules engine (`polkitbackendduktapeauthority.c`, `push_subject`)
//! accepts only a process or a bus name and reaches `g_assert_not_reached ()`
//! for anything else. Measured once on 2026-11-09 against `polkit-126-3.fc42.2`
//! in the dev image run as a container — the session subject killed polkitd
//! with `SIGABRT`, and the same question about the session's leader process was
//! answered — and seen the same day on a booted machine, where it took the
//! password sheet down with it (`docs/quirks/polkit-aborts-on-a-session-subject.md`).
//!
//! So the runner asks about the session's **leader** — the process logind
//! started the desktop's session with — which puts polkit's password sheet in
//! that session exactly as the session itself would, because polkit finds an
//! agent by the session its subject is in. The leader is named by its number
//! *and* its start time, which is what polkit compares to tell a process from
//! a later one that was handed the same number.

use std::collections::HashMap;

use zbus::zvariant::Value;

/// One process, as polkit's `unix-process` subject names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AProcess {
    /// Its number.
    pub pid: u32,
    /// When it started, in clock ticks since the machine booted: the 22nd
    /// field of `/proc/<pid>/stat`.
    pub start_time: u64,
    /// The login it runs as.
    pub uid: u32,
}

impl AProcess {
    /// polkit's `(sa{sv})` for this process. [`None`] for a login number
    /// polkit's signed field cannot hold, which no person's can be.
    #[must_use]
    pub fn as_polkits_subject(&self) -> Option<(&'static str, HashMap<&'static str, Value<'_>>)> {
        let uid = i32::try_from(self.uid).ok()?;
        Some((
            "unix-process",
            HashMap::from([
                ("pid", Value::from(self.pid)),
                ("start-time", Value::from(self.start_time)),
                ("uid", Value::from(uid)),
            ]),
        ))
    }
}

/// **When a process started**, read from its `/proc/<pid>/stat`: the 22nd
/// field. The second field is the program's name in brackets and may itself
/// hold spaces and brackets, so the count starts after the *last* `)`.
/// [`None`] when the line is not one the kernel writes.
#[must_use]
pub fn its_start_time(stat: &str) -> Option<u64> {
    let (_, after_the_name) = stat.rsplit_once(')')?;
    // Field 3 (the state) is the first after the name, so field 22 is the
    // twentieth from there.
    after_the_name.split_whitespace().nth(19)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line as the kernel writes it, with the start time at field 22.
    const A_LINE: &str = "562 (sleep) S 1 562 562 0 -1 4194560 105 0 0 0 0 0 0 0 20 0 1 0 \
                          23100667 2408448 256 18446744073709551615 1 1 0 0 0 0 0 0 0 0 0 0 17 \
                          1 0 0 0 0 0";

    /// **The start time is field 22**, as the kernel writes it.
    #[test]
    fn the_start_time_is_the_twenty_second_field() {
        assert_eq!(its_start_time(A_LINE), Some(23_100_667));
    }

    /// **A name with spaces and brackets does not move the count**: the name
    /// is a program's to choose, and a person's desktop must not be misnamed
    /// by one.
    #[test]
    fn a_name_with_brackets_and_spaces_does_not_move_the_count() {
        let line = A_LINE.replace("(sleep)", "(a) b (c) d)");
        assert_eq!(its_start_time(&line), Some(23_100_667));
    }

    /// **A line that is not the kernel's is refused**, rather than read as a
    /// start time of something else.
    #[test]
    fn a_line_that_is_not_the_kernels_is_refused() {
        assert_eq!(its_start_time(""), None);
        assert_eq!(its_start_time("562 sleep S 1"), None);
        assert_eq!(its_start_time("562 (sleep) S 1 562"), None);
        let not_a_number = A_LINE.replace("23100667", "soon");
        assert_eq!(its_start_time(&not_a_number), None);
    }

    /// **The subject is polkit's `unix-process`**, with the three keys polkit
    /// reads, and a number polkit cannot hold is refused.
    #[test]
    fn the_subject_is_a_process_with_its_start_time_and_login() {
        let process = AProcess {
            pid: 562,
            start_time: 23_100_667,
            uid: 1000,
        };
        let (kind, details) = process.as_polkits_subject().unwrap_or_else(|| {
            unreachable!("1000 fits");
        });
        assert_eq!(kind, "unix-process");
        assert_eq!(details.get("pid"), Some(&Value::from(562_u32)));
        assert_eq!(
            details.get("start-time"),
            Some(&Value::from(23_100_667_u64))
        );
        assert_eq!(details.get("uid"), Some(&Value::from(1000_i32)));
        assert_eq!(details.len(), 3);
        let too_big = AProcess {
            uid: u32::MAX,
            ..process
        };
        assert!(too_big.as_polkits_subject().is_none());
    }
}
