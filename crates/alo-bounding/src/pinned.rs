//! Where the boundary is kept once it is loaded, and who may reach it there.
//!
//! [ADR 0018](../../../docs/decisions/0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)
//! divides this crate in two. `alo-boundaryd` runs at boot as root, loads the
//! programme and pins it here; `alo-agentd` runs as the signed-in person, holds
//! no capability at all, and opens one of these pins by path. **The interface
//! between the two is this directory, not an API**, and everything that decides
//! who may do what is a mode and a group on three files.
//!
//! ```text
//! /sys/fs/bpf/alo            0750 root:<the agent's group>  made by the loader
//!   ├─ bounds                0660 root:<the agent's group>  the daemon writes it
//!   ├─ fields                0600 root:root                 nobody else reads it
//!   └─ file_open             0600 root:root                 what holds the attach
//! ```
//!
//! # The two maps are not given away on the same terms, and that is the point
//!
//! `bounds` is `{cgroup id → the places a turn may reach}`. Writing it is how a
//! turn is bounded, it is the daemon's whole business with the kernel, and it
//! needs **permission on a file** rather than `CAP_BPF` — which is the sentence
//! ADR 0018 exists to make true.
//!
//! `fields` is where this kernel keeps `f_path`, `d_parent`, `i_ino` and the
//! rest, filled once by the loader from the kernel's own type information. It is
//! `0600` and the daemon never opens it, because a process that could write it
//! could make the programme read the front of a `struct file` as though it were
//! a directory entry — which is `fields.rs`' *the width check is the point of
//! this file*, arriving as a permission rather than as a check. **The daemon can
//! bind a turn and cannot change how the kernel reads a file.**
//!
//! `file_open` is the pinned link, and it is what keeps the programme attached
//! after the loader has exited. Removing it detaches; nothing else does.
//!
//! # A root the caller names, for the reason `alo-agentd`'s `place.rs` has one
//!
//! [`Pinned::on_this_machine`] is the real machine's and [`Pinned::beneath`]
//! takes the root as an argument. A rule about a directory is only a rule with a
//! test if a test can be run against a directory it may write in, and
//! `/sys/fs/bpf/alo` is not one — a test that used it would impose the machine's
//! real boundary on whoever ran it.
//!
//! # It has to be a BPF filesystem, and this file does not mount one
//!
//! Pinning is a `bpffs` operation: the parent directory has to exist and has to
//! be on a mounted `bpf` filesystem. On a machine where it is not, [`Pinned`]
//! refuses in the machine's own words rather than mounting anything — a loader
//! that mounted filesystems would be doing something a boot has already decided,
//! and `docs/hardware.md` asks the question instead.

use std::fs::{DirBuilder, Permissions};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::failing::NotBounded;

/// Where a machine's boundary is pinned, which both halves of ADR 0018 know.
pub const THE_ROOT: &str = "/sys/fs/bpf/alo";

/// The map of turns, which the daemon writes.
const THE_BOUNDS: &str = "bounds";

/// The map of where this kernel keeps its fields, which nobody but the loader
/// ever opens.
const THE_FIELDS: &str = "fields";

/// The pinned link, which is what holds the programme on the hook.
const THE_HOOK: &str = "file_open";

/// Root owns it, the agent's group may enter it, nobody else exists.
const THE_DIRECTORY_MODE: u32 = 0o750;

/// The loader writes it, the daemon writes it, nobody else may.
const THE_BOUNDS_MODE: u32 = 0o660;

/// The loader's, and nobody's else — including the daemon's.
const THE_LOADERS_OWN_MODE: u32 = 0o600;

/// Where this machine's boundary is pinned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pinned {
    /// The directory the three pins go in.
    root: PathBuf,

    /// The map of turns.
    bounds: PathBuf,

    /// The map of this kernel's field offsets.
    fields: PathBuf,

    /// The link that holds the programme on `file_open`.
    hook: PathBuf,
}

impl Pinned {
    /// Where the boundary is pinned on a real machine, beneath [`THE_ROOT`].
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::beneath(Path::new(THE_ROOT))
    }

    /// The same shape beneath a root somebody names.
    ///
    /// Nothing is made or looked at: this is four paths joined, and every other
    /// method here is what touches a filesystem.
    #[must_use]
    pub fn beneath(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            bounds: root.join(THE_BOUNDS),
            fields: root.join(THE_FIELDS),
            hook: root.join(THE_HOOK),
        }
    }

    /// The directory the three pins go in.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The map of turns, which is the one thing the daemon opens.
    #[must_use]
    pub fn bounds(&self) -> &Path {
        &self.bounds
    }

    /// The map of this kernel's field offsets.
    #[must_use]
    pub fn fields(&self) -> &Path {
        &self.fields
    }

    /// The link that holds the programme on the hook.
    #[must_use]
    pub fn hook(&self) -> &Path {
        &self.hook
    }

    /// Refuse a machine that already has a boundary pinned here.
    ///
    /// A second programme on `file_open` is a second boundary: both are asked
    /// about every open, either can refuse one, and which grant a turn is really
    /// running under stops being a question with one answer. A loader run twice
    /// is a machine somebody is fixing, so it says so and changes nothing.
    ///
    /// # Errors
    /// [`NotBounded::AlreadyThere`], naming the pin that is in the way.
    pub fn nothing_is_there(&self) -> Result<(), NotBounded> {
        for pin in [&self.bounds, &self.fields, &self.hook] {
            if pin.exists() {
                return Err(NotBounded::AlreadyThere {
                    path: pin.display().to_string(),
                });
            }
        }
        Ok(())
    }

    /// Make the directory the pins go in, `0750` from the moment it exists.
    ///
    /// The mode is given to the call that creates it rather than set
    /// afterwards, so there is no moment at which the directory exists and
    /// anybody can write it — `alo-agentd`'s `place.rs` makes the person's
    /// directory the same way and for the same reason.
    ///
    /// A directory that is already there is not an error: the pins inside it are
    /// what [`Pinned::nothing_is_there`] refuses over, and a `bpffs` that
    /// survived a daemon being restarted still has the directory.
    ///
    /// # Errors
    /// [`NotBounded::NoPinDirectory`], which on an ordinary machine means
    /// `/sys/fs/bpf` is not a mounted BPF filesystem.
    pub fn made(&self) -> Result<(), NotBounded> {
        match DirBuilder::new()
            .mode(THE_DIRECTORY_MODE)
            .create(&self.root)
        {
            Ok(()) => Ok(()),
            Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            Err(why) => Err(NotBounded::NoPinDirectory {
                path: self.root.display().to_string(),
                why,
            }),
        }
    }

    /// Let the agent's group in, and nobody else at all.
    ///
    /// Three files and three different answers, which this file's own
    /// documentation argues: the directory is entered by the group, the map of
    /// turns is written by it, and the map of fields and the link are the
    /// loader's alone. Set every time rather than checked, because a mode that
    /// was right yesterday is not an argument about today.
    ///
    /// # Errors
    /// [`NotBounded::NotOurGroup`] if the group could not be given the map, and
    /// [`NotBounded::NotShutTo`] if a mode would not be set. In both, the
    /// boundary is loaded and is reachable by root only.
    pub fn given_to_group(&self, group: u32) -> Result<(), NotBounded> {
        for path in [&self.root, &self.bounds] {
            std::os::unix::fs::chown(path, None, Some(group)).map_err(|why| {
                NotBounded::NotOurGroup {
                    path: path.display().to_string(),
                    group,
                    why,
                }
            })?;
        }
        shut(&self.root, THE_DIRECTORY_MODE)?;
        shut(&self.bounds, THE_BOUNDS_MODE)?;
        shut(&self.fields, THE_LOADERS_OWN_MODE)?;
        shut(&self.hook, THE_LOADERS_OWN_MODE)
    }

    /// Take the boundary off this machine: the three pins, then the directory.
    ///
    /// Removing the link's pin is what detaches the programme, so this is the
    /// one thing in alo OS that ends a boundary — and it is deliberately not a
    /// [`Drop`], because a value going out of scope is not somebody deciding a
    /// machine should stop enforcing its grants.
    ///
    /// Nothing is reported. It runs where a load has already failed and on a
    /// test's way out, and in both there is either nothing to say or nobody to
    /// say it to; what it costs when it fails is a pin that
    /// [`Pinned::nothing_is_there`] will refuse over next time, which is the
    /// answer that file argues for anyway.
    pub fn taken_away(&self) {
        for pin in [&self.hook, &self.bounds, &self.fields] {
            drop(std::fs::remove_file(pin));
        }
        drop(std::fs::remove_dir(&self.root));
    }
}

/// Set one path's mode, whatever it was before.
fn shut(path: &Path, mode: u32) -> Result<(), NotBounded> {
    std::fs::set_permissions(path, Permissions::from_mode(mode)).map_err(|why| {
        NotBounded::NotShutTo {
            path: path.display().to_string(),
            mode,
            why,
        }
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::os::unix::fs::MetadataExt as _;

    /// A root of this process's own, on an ordinary filesystem.
    ///
    /// Every test here is about paths, refusals and modes rather than about
    /// `bpffs`, so an ordinary directory answers all of them — and the ones that
    /// really need a BPF filesystem are the integration tests, which say so and
    /// fail loudly on a machine without one.
    fn a_root_of_our_own(what: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!("alo-pinned-{}-{what}", std::process::id()));
        drop(std::fs::remove_dir_all(&at));
        at
    }

    /// The shape is decided here, and both halves of ADR 0018 look for it: a
    /// daemon that opened something else would find no boundary on a machine
    /// that has one.
    #[test]
    fn the_boundary_is_pinned_where_the_decision_says_it_is() {
        let pinned = Pinned::on_this_machine();
        assert_eq!(pinned.root(), Path::new("/sys/fs/bpf/alo"));
        assert_eq!(pinned.bounds(), Path::new("/sys/fs/bpf/alo/bounds"));
        assert_eq!(pinned.fields(), Path::new("/sys/fs/bpf/alo/fields"));
        assert_eq!(pinned.hook(), Path::new("/sys/fs/bpf/alo/file_open"));
    }

    /// The directory is made shut: root owns it, the agent's group may enter
    /// it, and nobody else can so much as see what is in it.
    #[test]
    fn the_directory_is_made_shut() {
        let root = a_root_of_our_own("made-shut");
        let pinned = Pinned::beneath(&root);

        pinned.made().unwrap();

        let mode = std::fs::metadata(&root).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o750);
        pinned.taken_away();
    }

    /// Making it twice is making it once: a boot that ran the loader after a
    /// `bpffs` survived something is the ordinary case, not an error.
    #[test]
    fn making_the_directory_twice_is_making_it_once() {
        let root = a_root_of_our_own("twice");
        let pinned = Pinned::beneath(&root);

        pinned.made().unwrap();
        pinned.made().unwrap();

        pinned.taken_away();
    }

    /// **A machine with no BPF filesystem is told so**, rather than having a
    /// directory made for it somewhere a pin can never go.
    #[test]
    fn a_root_that_cannot_be_made_names_the_directory() {
        let root = a_root_of_our_own("no-parent").join("nothing").join("here");
        let pinned = Pinned::beneath(&root);

        let refused = pinned.made().unwrap_err();

        assert!(
            matches!(&refused, NotBounded::NoPinDirectory { path, .. } if path == &root.display().to_string()),
            "{refused}"
        );
    }

    /// **A boundary already pinned here is refused**, and the refusal names the
    /// pin that is in the way. Two programmes on `file_open` are two boundaries,
    /// and which grant a turn is running under stops having one answer.
    #[test]
    fn a_boundary_already_here_is_refused_and_named() {
        let root = a_root_of_our_own("already");
        let pinned = Pinned::beneath(&root);
        pinned.made().unwrap();
        pinned.nothing_is_there().unwrap();
        std::fs::write(pinned.bounds(), b"a map that is already here").unwrap();

        let refused = pinned.nothing_is_there().unwrap_err();

        assert!(
            matches!(&refused, NotBounded::AlreadyThere { path } if path.ends_with("bounds")),
            "{refused}"
        );
        pinned.taken_away();
    }

    /// **The map of turns is the group's and the map of fields is not.** A
    /// daemon that could write the offsets could make the kernel read the front
    /// of a `struct file` as a directory entry, so what it is given is the one
    /// map its job is, at the one mode that job needs.
    #[test]
    fn only_the_map_of_turns_is_given_away() {
        let root = a_root_of_our_own("given-away");
        let pinned = Pinned::beneath(&root);
        pinned.made().unwrap();
        for pin in [pinned.bounds(), pinned.fields(), pinned.hook()] {
            std::fs::write(pin, b"").unwrap();
        }

        // This process's own group, because chown to a group nobody is in is
        // refused to anybody who is not root — and what is being tested is the
        // modes, which are the same either way.
        let ours = std::fs::metadata(&root).unwrap().gid();
        pinned.given_to_group(ours).unwrap();

        let mode_of = |path: &Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            mode_of(pinned.bounds()),
            0o660,
            "the daemon writes this one"
        );
        assert_eq!(mode_of(pinned.fields()), 0o600, "and never this one");
        assert_eq!(mode_of(pinned.hook()), 0o600);
        assert_eq!(mode_of(&root), 0o750);
        pinned.taken_away();
    }

    /// **Taking it away takes the link's pin with it**, which is what detaches
    /// the programme — and leaves nothing behind for the next loader to refuse
    /// over.
    #[test]
    fn taking_it_away_leaves_nothing_to_refuse_over() {
        let root = a_root_of_our_own("taken-away");
        let pinned = Pinned::beneath(&root);
        pinned.made().unwrap();
        for pin in [pinned.bounds(), pinned.fields(), pinned.hook()] {
            std::fs::write(pin, b"").unwrap();
        }

        pinned.taken_away();

        assert!(!root.exists());
        pinned.nothing_is_there().unwrap();
    }
}
