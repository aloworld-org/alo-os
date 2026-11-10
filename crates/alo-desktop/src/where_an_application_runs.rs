//! **Which control groups are an application's**, read off the person's own
//! service manager's tree (ADR 0148 §3).
//!
//! An application a person starts runs in a unit of the person's manager,
//! under `app.slice`: `app-alo-<id>@<n>.service` when this desktop started it,
//! `app-flatpak-<id>-<n>.scope` once Flatpak has moved it. Which names count is
//! `alo_launching::is_a_unit_of`, beside the rule that makes them. This file
//! walks the directories and answers each one's number — **a control group's
//! number is its directory's inode**, which is what the kernel's
//! `bpf_skb_cgroup_id` answers for a socket in it.
//!
//! Read on every look and kept nowhere, so a unit that appears during a command
//! is found on the next look and one that has gone is not.

use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

use alo_shell::counting_during_their_commands::{FindsWhereItRuns, THE_MOST_GROUPS};

/// How deep beneath a unit this looks. A unit with control groups of its own
/// inside it is unusual; one nested deeper than this is not an application's.
const DEEPEST: usize = 4;

/// The person's manager's `app.slice`, walked for an application's units.
pub struct UnderTheAppSlice {
    /// The directory of `app.slice`.
    slice: PathBuf,
}

impl UnderTheAppSlice {
    /// The slice of the person this process runs as, where systemd puts it:
    /// `user.slice/user-<uid>.slice/user@<uid>.service/app.slice`.
    #[must_use]
    pub fn for_this_person() -> Self {
        let person = rustix::process::geteuid().as_raw();
        Self::beneath(
            &Path::new("/sys/fs/cgroup")
                .join("user.slice")
                .join(format!("user-{person}.slice"))
                .join(format!("user@{person}.service"))
                .join("app.slice"),
        )
    }

    /// The same, at a directory somebody names — a test's road.
    #[must_use]
    pub fn beneath(slice: &Path) -> Self {
        Self {
            slice: slice.to_path_buf(),
        }
    }
}

impl FindsWhereItRuns for UnderTheAppSlice {
    fn groups_of(&self, application: &str) -> Vec<u64> {
        let Ok(units) = std::fs::read_dir(&self.slice) else {
            return Vec::new();
        };
        let mut found = Vec::new();
        for unit in units.flatten() {
            let name = unit.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if alo_launching::is_a_unit_of(name, application) {
                with_everything_beneath(&unit.path(), 0, &mut found);
            }
        }
        // More than can be counted is still answered whole: the counting says
        // the count missed part of the application, rather than this file
        // choosing which part to leave out.
        found.truncate(THE_MOST_GROUPS + 1);
        found
    }
}

/// This control group's number, and every one beneath it.
pub(crate) fn with_everything_beneath(group: &Path, depth: usize, found: &mut Vec<u64>) {
    let Ok(about) = std::fs::symlink_metadata(group) else {
        return;
    };
    if !about.is_dir() || found.len() > THE_MOST_GROUPS {
        return;
    }
    found.push(about.ino());
    if depth >= DEEPEST {
        return;
    }
    let Ok(beneath) = std::fs::read_dir(group) else {
        return;
    };
    for one in beneath.flatten() {
        if one.file_type().is_ok_and(|kind| kind.is_dir()) {
            with_everything_beneath(&one.path(), depth + 1, found);
        }
    }
}

#[cfg(test)]
#[path = "where_an_application_runs_tests.rs"]
mod tests;
