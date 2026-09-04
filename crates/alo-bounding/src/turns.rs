//! Where this service's turns are made, and the way back out of one.
//!
//! Item 26 proved a kernel really refuses: a cgroup, a bound, a **child
//! process** inside it, and an open the kernel said no to. A child process is
//! the right shape for a test and the wrong shape for the daemon, and this file
//! is the difference.
//!
//! # Nothing is started, and that is law 2
//!
//! *Run the verb's work inside that cgroup* reads like *start something in the
//! cgroup*, and starting something is where law 2 dies. Every shape that spawns
//! needs a program to spawn, and a program alo OS starts on an agent's behalf is
//! one review away from a program an agent named — which is the escape hatch
//! `CLAUDE.md` says makes every other control decorative.
//!
//! So **nothing is started**. What a turn's work already is on this machine is
//! one thread of `alo-agentd` calling `alo-files`, which is one of six verbs on
//! a closed list. That thread is what goes into the cgroup, and it goes in by
//! writing one byte. There is no `fork`, no `exec` and no `Command` anywhere in
//! this crate, and `tests/a_turn_is_this_thread.rs` reads the crate's own source
//! and says so rather than leaving it to somebody's memory.
//!
//! It is also the narrower answer. A whole process in the cgroup would put the
//! record, the socket and the person's own door inside the agent's boundary; one
//! thread puts the verb inside it and leaves the service outside.
//!
//! # The way out is a descriptor opened before the way in was taken
//!
//! A thread leaves a cgroup by writing into another cgroup's `cgroup.threads`,
//! and *opening* that file while the boundary is in force is an open outside the
//! grant — which our own program in the kernel refuses, with `EACCES`, correctly.
//! A turn that could be entered and not left would be a service that stops
//! working the first time it worked.
//!
//! So [`Turns::under`] opens `home/cgroup.threads` **before** this service is
//! ever in a turn and holds it for the life of the daemon. Leaving is a write to
//! a descriptor that already exists, and a write is not an open.
//!
//! # The shape on the machine
//!
//! ```text
//! <the service's own cgroup>          made by systemd, or by whoever runs it
//!   ├─ home                           threaded; every thread of alo-agentd
//!   └─ turn-<n>                       threaded; one thread, for one execution
//! ```
//!
//! The subtree is made **inside** the cgroup this service is already in rather
//! than at the top of `/sys/fs/cgroup`, because a daemon that arranges the top
//! of the hierarchy around itself is a daemon that has taken the machine over.
//! Under `systemd` that is `system.slice/alo-agentd.service`; in a test it is
//! whatever the test made. [`Turns::under`] takes it rather than deciding it.

use std::{
    fs,
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};

use crate::{cgroup::Cgroup, failing::NotBounded};

/// Where this service's own threads are when no turn is under way.
const HOME: &str = "home";

/// Where the unified control group hierarchy is mounted.
const WHERE_CGROUPS_ARE: &str = "/sys/fs/cgroup";

/// What the kernel answers when asked which cgroup a task is in.
const WHERE_WE_ARE: &str = "/proc/self/cgroup";

/// What every line of that file starts with in the unified hierarchy.
const THE_ONLY_HIERARCHY: &str = "0::";

/// The place this service makes a turn's boundary in, and the way back out.
///
/// One per service. Making it moves this process into a cgroup of its own, so
/// it is a thing that happens once, when the daemon starts, and is given back
/// when it stops.
#[derive(Debug)]
pub struct Turns {
    /// The cgroup `home` and every turn are made in.
    root: PathBuf,

    /// Where this service's threads are while no turn is under way.
    home: Cgroup,

    /// `home/cgroup.threads`, open since before the first turn began.
    ///
    /// The only reason this is a field rather than a path: leaving a turn must
    /// not open anything. See this file's own documentation.
    back: File,

    /// The cgroup this process was in before it moved into `home`.
    came_from: PathBuf,
}

impl Turns {
    /// Makes the subtree under `root` and moves this service's threads into it.
    ///
    /// `root` is the cgroup this service is already in. It must hold no
    /// processes of its own once this returns, which is what moving into `home`
    /// arranges, and it becomes the resource domain for every turn.
    ///
    /// # Errors
    /// [`NotBounded`], and ADR 0015's rule applies to all of them: a service
    /// that cannot make this cannot bound a turn, and a turn that cannot be
    /// bounded does not run.
    pub fn under(root: &Path) -> Result<Self, NotBounded> {
        let came_from = where_this_process_is()?;
        let home = Cgroup::made_under(root, HOME)?;
        if let Err(why) = home.holding_threads() {
            drop(home.removed());
            return Err(why);
        }

        // Before anything is in it, and before any turn exists. This descriptor
        // is the only way back out of a boundary, and opening it later would be
        // opening a file the boundary refuses.
        let back = match OpenOptions::new().write(true).open(home.threads()) {
            Ok(back) => back,
            Err(why) => {
                let path = home.threads().display().to_string();
                drop(home.removed());
                return Err(NotBounded::Cgroup {
                    what: "cannot hold open the way out of a turn at",
                    path,
                    why,
                });
            }
        };

        if let Err(why) = home.admit(std::process::id()) {
            drop(home.removed());
            return Err(why);
        }

        Ok(Self {
            root: root.to_path_buf(),
            home,
            back,
            came_from,
        })
    }

    /// The same, in the control group this service was started in.
    ///
    /// What a daemon uses, and the reason [`Turns::under`] takes a path at all
    /// is the reason this does not: where a service is put is whoever started
    /// it's decision — `systemd` puts it in a slice, a test puts it wherever the
    /// test made — so the kernel is asked rather than a place assumed, and a
    /// test that wants a subtree of its own says so by naming one.
    ///
    /// # Errors
    /// [`NotBounded::NotAPlace`] or [`NotBounded::NotInAHierarchy`] if the
    /// kernel will not say where this process is, and everything
    /// [`Turns::under`] answers with. ADR 0015's rule applies to all of them: a
    /// service that cannot make this cannot bound a turn.
    pub fn of_this_service() -> Result<Self, NotBounded> {
        Self::under(&where_this_process_is()?)
    }

    /// A control group for one turn's work, made beside `home`.
    ///
    /// Threaded, because what goes into it is a thread. The name is one
    /// component and nothing else, held to that by [`Cgroup::made_under`].
    ///
    /// # Errors
    /// [`NotBounded::NotAName`] for a name that is a path, and
    /// [`NotBounded::Cgroup`] for anything the hierarchy would not do.
    pub fn beginning(&self, named: &str) -> Result<Cgroup, NotBounded> {
        let turn = Cgroup::made_under(&self.root, named)?;
        if let Err(why) = turn.holding_threads() {
            drop(turn.removed());
            return Err(why);
        }
        Ok(turn)
    }

    /// Where this service's threads are when no turn is under way.
    #[must_use]
    pub const fn home(&self) -> &Cgroup {
        &self.home
    }

    /// The descriptor a thread leaves a turn through.
    pub(crate) const fn the_way_back(&self) -> &File {
        &self.back
    }

    /// Puts this service back where it was and takes the subtree away.
    ///
    /// Separate from dropping the value for the reason [`Cgroup::removed`] is:
    /// moving a process between control groups can fail, and a `Drop` that
    /// swallowed it would leave a machine filling with the remains of daemons
    /// with nothing saying so.
    ///
    /// # Errors
    /// [`NotBounded::Cgroup`] if the process could not be put back or the
    /// cgroup could not be removed. The subtree is left exactly as it was.
    pub fn given_back(self) -> Result<(), NotBounded> {
        this_process_into(&self.came_from)?;
        self.home.removed()
    }
}

/// The cgroup this process is in, as a path under the mounted hierarchy.
///
/// Read from the kernel rather than assumed, because where a service is put is
/// whoever started it's decision — `systemd` puts it in a slice, a test puts it
/// wherever the test made, and a daemon that assumed either would be wrong on
/// the other.
fn where_this_process_is() -> Result<PathBuf, NotBounded> {
    let said = fs::read_to_string(WHERE_WE_ARE).map_err(|why| NotBounded::NotAPlace {
        path: WHERE_WE_ARE.to_owned(),
        why,
    })?;
    let under = said
        .lines()
        .find_map(|line| line.strip_prefix(THE_ONLY_HIERARCHY))
        .ok_or(NotBounded::NotInAHierarchy {
            what: "there is no unified control group hierarchy on this machine",
        })?;
    Ok(PathBuf::from(WHERE_CGROUPS_ARE).join(under.trim().trim_start_matches('/')))
}

/// Moves this whole process into the cgroup at `at`.
///
/// A zero rather than this process's number, which the kernel reads as *the one
/// asking*. Nothing here has to find out its own identity to move itself, and a
/// number written by hand is a number that can be somebody else's.
fn this_process_into(at: &Path) -> Result<(), NotBounded> {
    let file = at.join(crate::cgroup::THE_PROCESSES);
    fs::write(&file, "0").map_err(|why| NotBounded::Cgroup {
        what: "cannot put this service back into the control group at",
        path: file.display().to_string(),
        why,
    })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The kernel's own answer, read the way this file reads it. A machine with
    /// a unified hierarchy says `0::` and a path; anything else is refused
    /// rather than guessed at, because guessing wrong means making a turn's
    /// boundary somewhere nobody meant.
    #[test]
    fn this_process_is_somewhere_in_the_hierarchy() {
        let where_we_are = where_this_process_is().expect("this machine has a cgroup hierarchy");
        assert!(
            where_we_are.starts_with(WHERE_CGROUPS_ARE),
            "{}",
            where_we_are.display()
        );
    }

    /// A turn's cgroup is one component under the service's own, so a name that
    /// is a path is refused before anything is made — [`Cgroup::made_under`]'s
    /// rule, asked here because this is the door a caller reaches it by.
    #[test]
    fn a_turn_cannot_be_named_a_path() {
        assert!(matches!(
            Cgroup::made_under(Path::new(WHERE_CGROUPS_ARE), "../elsewhere"),
            Err(NotBounded::NotAName { name }) if name == "../elsewhere"
        ));
    }
}
