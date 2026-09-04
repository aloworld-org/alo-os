//! The order the one privileged thing alo OS does happens in.
//!
//! Six steps, and five of them are refusals. That ratio is the crate: nothing
//! here decides *what* to load — there is one programme, compiled into
//! `alo-bounding` and reachable no other way — so all this file can contribute
//! is the order, and stopping on a machine where the order would produce
//! something worse than nothing.
//!
//! ```text
//! 1. running as root?              no  -> NotAsRoot, and nothing was touched
//! 2. in the agent's group?         no  -> TheRootsGroup, and nothing was touched
//! 3. is a boundary already here?   yes -> AlreadyThere, and nothing was touched
//! 4. make /sys/fs/bpf/alo
//! 5. load, attach, pin
//! 6. give the map of turns to the group    fails -> everything is unpinned again
//! ```
//!
//! # The three questions come before anything is loaded, and that is the order
//!
//! Every one of them is arithmetic or a `stat`, and every one of them describes
//! a machine where imposing the boundary would leave something worse than a
//! machine without one — a second programme on `file_open`, or a map no daemon
//! can write. Asking them first means the refusal costs nothing and leaves
//! nothing, which is `alo-bounding`'s own *everything that can be wrong with the
//! machine is found here rather than at the first turn*, one process out.
//!
//! # A failure at step 6 unpins what step 5 made
//!
//! A boundary loaded and not given away is a machine that enforces grants for a
//! daemon that can never write one — every turn refused, forever, by a component
//! nobody can see. So it is taken off again and the machine is left the way it
//! was found: with no boundary, which is a state `alo-agentd` refuses to run in
//! and says so.

use alo_bounding::{Imposed, Pinned};

use crate::refusing::NotLoaded;

/// The one user that can load a BPF LSM programme.
const ROOT: u32 = 0;

/// The group nobody can be let in through.
const THE_ROOTS_GROUP: u32 = 0;

/// Impose the boundary on this machine, once.
///
/// `us` and `group` are who the unit file started this process as, and `pinned`
/// is where the boundary goes; all three are taken rather than found, for the
/// reason `alo-agentd`'s `place.rs` takes a root — a rule about a machine is
/// only a rule with a test if a test can be run against something it may write
/// in, and `/sys/fs/bpf/alo` is not one.
///
/// What comes back is the loader's own handles on the programme. They are worth
/// nothing to anybody but a test: the pins are what hold the boundary on the
/// machine, so dropping this closes some descriptors and changes nothing.
///
/// # Errors
///
/// [`NotLoaded::NotAsRoot`] and [`NotLoaded::TheRootsGroup`] are the two
/// mistakes a unit file can make, and in both nothing has been loaded, made or
/// pinned. [`NotLoaded::NotImposed`] is everything the kernel and the filesystem
/// can refuse, including a machine that already has a boundary — and on every
/// one of those roads nothing is left pinned either.
pub fn imposed(us: u32, group: u32, pinned: &Pinned) -> Result<Imposed, NotLoaded> {
    if us != ROOT {
        return Err(NotLoaded::NotAsRoot { uid: us });
    }
    if group == THE_ROOTS_GROUP {
        return Err(NotLoaded::TheRootsGroup);
    }
    pinned.nothing_is_there()?;
    pinned.made()?;

    let imposed = Imposed::once(pinned)?;
    if let Err(why) = pinned.given_to_group(group) {
        pinned.taken_away();
        return Err(why.into());
    }
    Ok(imposed)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    /// The group an agent is in on an ordinary machine, which is not root's.
    const THE_AGENTS_GROUP: u32 = 989;

    /// A pin root of this process's own, which no test here ever loads into.
    ///
    /// Every test below is one of the refusals that happen *before* anything is
    /// loaded, so an ordinary directory answers all of them. What needs a real
    /// BPF filesystem and a real kernel is
    /// `tests/the_boundary_outlives_the_loader.rs`, which says so and fails
    /// loudly on a machine without one.
    fn a_root_of_our_own(what: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!("alo-loading-{}-{what}", std::process::id()));
        drop(std::fs::remove_dir_all(&at));
        at
    }

    /// **A loader that is not root is told so**, rather than being handed the
    /// verifier's permission error on a syscall — what is wrong is a unit file,
    /// and the sentence says which line of it.
    #[test]
    fn a_loader_that_is_not_root_does_not_load() {
        let root = a_root_of_our_own("not-root");
        let pinned = Pinned::beneath(&root);

        let refused = imposed(1000, THE_AGENTS_GROUP, &pinned).unwrap_err();

        assert!(
            matches!(refused, NotLoaded::NotAsRoot { uid: 1000 }),
            "{refused}"
        );
        assert!(!root.exists(), "and nothing was made on the way out");
    }

    /// **A loader in root's group does not load either.** The pinned map of
    /// turns is handed to this process's own group, so root's group is a map
    /// only root could write — a machine whose boundary is loaded and whose
    /// daemon can never bound a turn.
    #[test]
    fn a_loader_in_roots_group_would_pin_a_map_nobody_can_write() {
        let root = a_root_of_our_own("roots-group");
        let pinned = Pinned::beneath(&root);

        let refused = imposed(ROOT, THE_ROOTS_GROUP, &pinned).unwrap_err();

        assert!(matches!(refused, NotLoaded::TheRootsGroup), "{refused}");
        assert!(!root.exists(), "and nothing was made on the way out");
    }

    /// **Not being root is asked before the group is**, because it is the one
    /// that makes every other question moot: a machine where both are wrong
    /// reads the sentence about the harder problem first.
    #[test]
    fn the_first_question_is_the_one_that_makes_the_others_moot() {
        let pinned = Pinned::beneath(Path::new("/nowhere-this-test-will-ever-write"));

        let refused = imposed(1000, THE_ROOTS_GROUP, &pinned).unwrap_err();

        assert!(matches!(refused, NotLoaded::NotAsRoot { .. }), "{refused}");
    }

    /// **A machine that already has a boundary is left with the one it has.**
    /// Two programmes on `file_open` are two boundaries, and which grant a turn
    /// is running under would stop being a question with one answer — so the
    /// second loader says so and changes nothing.
    #[test]
    fn a_machine_that_already_has_a_boundary_keeps_it() {
        let root = a_root_of_our_own("already-here");
        let pinned = Pinned::beneath(&root);
        pinned.made().unwrap();
        std::fs::write(pinned.hook(), b"a link that is already here").unwrap();

        let refused = imposed(ROOT, THE_AGENTS_GROUP, &pinned).unwrap_err();

        assert!(
            matches!(
                refused,
                NotLoaded::NotImposed(alo_bounding::NotBounded::AlreadyThere { .. })
            ),
            "{refused}"
        );
        assert_eq!(
            std::fs::read(pinned.hook()).unwrap(),
            b"a link that is already here",
            "and what was there is still there"
        );
        pinned.taken_away();
    }

    /// **A machine with no BPF filesystem is told where the boundary had
    /// nowhere to go.** Pinning is a `bpffs` operation; this crate does not
    /// mount one, because a boot has already decided what is mounted.
    #[test]
    fn a_machine_with_nowhere_to_pin_says_where_it_looked() {
        let root = a_root_of_our_own("no-bpffs").join("nothing").join("here");
        let pinned = Pinned::beneath(&root);

        let refused = imposed(ROOT, THE_AGENTS_GROUP, &pinned).unwrap_err();

        assert!(
            matches!(
                &refused,
                NotLoaded::NotImposed(alo_bounding::NotBounded::NoPinDirectory { path, .. })
                    if path == &root.display().to_string()
            ),
            "{refused}"
        );
    }
}
