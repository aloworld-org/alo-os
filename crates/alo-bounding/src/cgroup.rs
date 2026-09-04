//! The cgroup a turn runs in, which is how the kernel knows one turn from
//! another.
//!
//! ADR 0015's mechanism begins *`alo-agentd` creates a cgroup for this turn*,
//! and the reason is that a BPF program has no other way to ask the question.
//! Inside `file_open` there is a task, and a task's process id is reused; what
//! is stable for exactly as long as the turn lasts is the control group it was
//! put in, and `bpf_get_current_cgroup_id` is one instruction away.
//!
//! # A cgroup is a directory, so a name is a place
//!
//! Making one is `mkdir` under `/sys/fs/cgroup` and putting a process in one is
//! writing its number into a file. That makes a cgroup's *name* a path, and a
//! name with a separator in it a caller choosing where under `/sys/fs/cgroup`
//! to write. [`Cgroup::made`] refuses those rather than cleaning them up, which
//! is `alo-capability`'s `path.rs` rule arriving here: normalising means this
//! crate and the kernel disagreeing about what a name meant.
//!
//! # The identifier is the directory's inode number
//!
//! `bpf_get_current_cgroup_id` answers with the kernel's identifier for a
//! cgroup, and on a sixty-four-bit machine that identifier **is** the inode
//! number of the directory in `/sys/fs/cgroup`. So there is no syscall to ask —
//! the number is what `stat` already says.
//!
//! # A cgroup holds processes, and it can be told to hold threads instead
//!
//! [`Cgroup::admit`] writes into `cgroup.procs` and moves a whole process,
//! threads and all. [`Cgroup::holding_threads`] is the other shape: a cgroup
//! told to hold threads takes one task at a time through `cgroup.threads`, and
//! the cgroup above it becomes the resource domain for the subtree.
//!
//! Both exist because a turn is not a process. What a turn's work really is on
//! this machine is one thread of `alo-agentd` carrying out one enumerated verb,
//! so the boundary has to be something a thread steps into — see
//! [`crate::Turns`], which is where that order lives.
//!
//! That is a fact about kernfs rather than a documented promise, so it is not
//! taken on trust anywhere it matters: if it were wrong, the map would be keyed
//! by a number no open ever presents, every lookup would miss, and every open
//! would be allowed. `tests/the_kernel_refuses.rs` is what would notice, and it
//! is written so that the refusal is the assertion rather than the allow.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::failing::NotBounded;

/// Where the unified control group hierarchy is mounted.
const WHERE_CGROUPS_ARE: &str = "/sys/fs/cgroup";

/// The file a process's number is written into to put it in a cgroup.
pub(crate) const THE_PROCESSES: &str = "cgroup.procs";

/// The file a *thread's* number is written into to put it in a cgroup.
///
/// The other half of `cgroup.procs`, and the difference is the whole of
/// [`crate::Turns`]: writing here moves one task, so a service can put the
/// thread carrying out a verb inside a turn's boundary and leave the rest of
/// itself outside it.
pub(crate) const THE_THREADS: &str = "cgroup.threads";

/// The file that says whether a cgroup holds processes or threads.
const THE_TYPE: &str = "cgroup.type";

/// What is written into it to make a cgroup hold threads.
const HOLDS_THREADS: &str = "threaded";

/// One control group: a turn, as the kernel is able to recognise it.
#[derive(Debug)]
pub struct Cgroup {
    /// The directory under `/sys/fs/cgroup`.
    at: PathBuf,
}

impl Cgroup {
    /// A new cgroup of that name, made under the unified hierarchy.
    ///
    /// The name is one component — letters, digits, a dash or an underscore —
    /// and anything else is refused. A cgroup that already exists is refused
    /// too, because two turns sharing one cgroup would share one boundary, and
    /// the second turn to end would take away the first turn's.
    pub fn made(name: &str) -> Result<Self, NotBounded> {
        Self::made_under(Path::new(WHERE_CGROUPS_ARE), name)
    }

    /// The same, somewhere other than the top of the hierarchy.
    ///
    /// A service under `systemd` is put in a control group of its own, and the
    /// turns it holds belong inside that rather than beside it — a subtree at
    /// the top of `/sys/fs/cgroup` would be a daemon arranging the machine
    /// around itself. [`crate::Turns`] is what uses this; the name is held to
    /// the same one component, for the same reason.
    pub fn made_under(parent: &Path, name: &str) -> Result<Self, NotBounded> {
        if !is_a_name(name) {
            return Err(NotBounded::NotAName {
                name: name.to_owned(),
            });
        }
        let at = parent.join(name);
        fs::create_dir(&at).map_err(|why| NotBounded::Cgroup {
            what: "cannot make a control group at",
            path: at.display().to_string(),
            why,
        })?;
        Ok(Self { at })
    }

    /// Makes this cgroup one that holds threads rather than processes.
    ///
    /// A cgroup holds whole processes until it is told otherwise, and *told
    /// otherwise* is one word written into `cgroup.type`. It also changes the
    /// cgroup **above** this one, which is the part worth knowing: the parent
    /// becomes the resource domain for a threaded subtree, so every threaded
    /// cgroup made under it is a place one task of a process can be while its
    /// siblings are elsewhere in the same subtree.
    ///
    /// That is what makes a boundary the daemon's own thread can step into and
    /// out of, and it is why the parent is made first and emptied of processes
    /// — [`crate::Turns::under`] is the order.
    pub fn holding_threads(&self) -> Result<(), NotBounded> {
        let file = self.at.join(THE_TYPE);
        fs::write(&file, HOLDS_THREADS).map_err(|why| NotBounded::Cgroup {
            what: "cannot make a control group hold threads at",
            path: file.display().to_string(),
            why,
        })
    }

    /// The file one thread is put into this cgroup by.
    ///
    /// Handed out as a path rather than written to here, because the *order* a
    /// thread goes in and comes back out in is [`crate::Turns`]'s and is the
    /// security property: the way out is a descriptor opened before the way in
    /// was taken.
    pub(crate) fn threads(&self) -> PathBuf {
        self.at.join(THE_THREADS)
    }

    /// What `bpf_get_current_cgroup_id` will answer for anything inside it.
    pub fn id(&self) -> Result<u64, NotBounded> {
        use std::os::linux::fs::MetadataExt;
        let known = self.at.metadata().map_err(|why| NotBounded::NotAPlace {
            path: self.at.display().to_string(),
            why,
        })?;
        Ok(known.st_ino())
    }

    /// Puts a process into this cgroup, and everything it does afterwards
    /// inside the turn.
    ///
    /// Writing a process's number here moves the whole process, threads and
    /// all — which is what a turn wants, and is why the daemon puts a *child*
    /// in rather than itself.
    pub fn admit(&self, process: u32) -> Result<(), NotBounded> {
        let file = self.at.join(THE_PROCESSES);
        fs::write(&file, process.to_string()).map_err(|why| NotBounded::Cgroup {
            what: "cannot put a process into the control group at",
            path: file.display().to_string(),
            why,
        })
    }

    /// Where this cgroup is, for whoever has to put a process in it from
    /// somewhere this type cannot reach.
    #[must_use]
    pub fn at(&self) -> &std::path::Path {
        &self.at
    }

    /// Takes the cgroup away, which the kernel only allows once it is empty.
    ///
    /// Separate from dropping the value on purpose: removing a cgroup can fail,
    /// and a `Drop` that swallowed the failure would leave a machine slowly
    /// filling with the remains of turns and nothing saying so.
    pub fn removed(self) -> Result<(), NotBounded> {
        fs::remove_dir(&self.at).map_err(|why| NotBounded::Cgroup {
            what: "cannot take away the control group at",
            path: self.at.display().to_string(),
            why,
        })
    }
}

/// Whether a name is one component and nothing else.
///
/// Letters, digits, a dash and an underscore. Everything else — a separator, a
/// step upwards, a space, a dot — is a caller choosing a directory rather than
/// naming a turn.
fn is_a_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|letter| letter.is_ascii_alphanumeric() || letter == '-' || letter == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A name with a separator in it is a caller picking a directory anywhere
    /// under `/sys/fs/cgroup`, and a name that steps upwards is one picking a
    /// directory outside it. Both are refused before anything is created.
    #[test]
    fn a_name_that_is_a_path_is_not_a_name() {
        for asked in [
            "../elsewhere",
            "one/two",
            "/absolute",
            "",
            "with space",
            ".",
        ] {
            assert!(!is_a_name(asked), "{asked} is not a name for a turn");
        }
    }

    /// And an ordinary name is one, which is what keeps the test above from
    /// passing because everything is refused.
    #[test]
    fn an_ordinary_name_is_a_name() {
        assert!(is_a_name("alo-turn_1"));
        assert!(is_a_name("t"));
    }

    /// The refusal reaches the caller as a value with the name in it, rather
    /// than as something the filesystem said.
    #[test]
    fn a_name_that_is_a_path_is_refused_before_anything_is_made() {
        assert!(matches!(
            Cgroup::made("../elsewhere"),
            Err(NotBounded::NotAName { name }) if name == "../elsewhere"
        ));
    }
}
