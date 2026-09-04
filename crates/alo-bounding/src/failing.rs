//! Why a boundary could not be imposed.
//!
//! ADR 0013 and ADR 0015 both end on the same rule: **a turn whose boundary
//! cannot be applied does not run.** Every value here is the reason for one of
//! those refusals, and none of them is a warning to carry on past.
//!
//! # These keep their English, and that is a decision rather than an omission
//!
//! `CLAUDE.md` calls hardcoded English a bug, and the 9-series moved every
//! sentence in this workspace onto `alo-strings`. This type is deliberately in
//! the same category as `alo-shortcuts`' `DefaultsError` and `alo-capability`'s
//! `VerbError`: it is read by whoever is standing a machine up, not by the
//! person using one.
//!
//! Every one of these says something about the *kernel underneath the daemon* —
//! that it publishes no type information, that a structure has moved, that the
//! verifier refused a program. Nobody signs in to a machine and reads them. The
//! sentence a person reads is *this turn cannot run*, it is said by whatever
//! holds the turn, and it is the wiring item's to write with the rest of that
//! sentence's context around it.
//!
//! If that ever stops being true — if one of these reaches a screen — it moves
//! onto `alo-strings` the way `alo-files`' `Failed` did, and the crate gains a
//! `words.rs`.

use std::io;

use alo_bounding_map::Field;

/// Why this turn cannot be given a boundary.
#[derive(Debug, thiserror::Error)]
pub enum NotBounded {
    /// The kernel publishes no description of its own structures.
    ///
    /// Which almost always means `CONFIG_DEBUG_INFO_BTF` was off when it was
    /// built. The path is in the message because that is what somebody needs in
    /// order to find out.
    #[error("this kernel does not publish its own type information at {path}: {why}")]
    NoTypeInformation {
        /// Where it was looked for.
        path: String,

        /// What the machine said.
        #[source]
        why: io::Error,
    },

    /// There is a file there and it is not type information this can read.
    #[error("this kernel's type information cannot be read: {what}")]
    TypesAreNotReadable {
        /// Which way it stopped making sense.
        what: &'static str,
    },

    /// This kernel has no such structure, or no such member of it.
    ///
    /// A kernel far enough from the ones alo OS certifies that the walk has
    /// nothing to walk. Refused rather than guessed at: an offset of zero is a
    /// real offset, and a boundary using it would read the front of a
    /// `struct file` as though it were a directory entry.
    #[error("this kernel has no {structure}.{member}, so the boundary has nowhere to look")]
    FieldIsMissing {
        /// The structure that was searched.
        structure: &'static str,

        /// The member that was not in it.
        member: &'static str,
    },

    /// The member is there and is not the width the program reads.
    ///
    /// The dangerous one, and the reason the width is checked at all: reading
    /// eight bytes where the kernel keeps four compares a number against half
    /// of itself and part of whatever is beside it, and it does that without
    /// failing.
    #[error("this kernel's {structure}.{member} is {found} bytes and the boundary reads {wanted}")]
    FieldIsNotTheWidth {
        /// The structure the member is in.
        structure: &'static str,

        /// The member.
        member: &'static str,

        /// What this kernel makes it.
        found: u32,

        /// What the program in the kernel reads.
        wanted: u32,
    },

    /// The kernel would not load the program.
    ///
    /// Usually the verifier, and usually with a great deal to say; the source
    /// carries it.
    #[error("the kernel would not load the boundary")]
    WillNotLoad(#[source] aya::EbpfError),

    /// The program loaded and would not attach.
    ///
    /// On a kernel with `CONFIG_BPF_LSM=y` but no `bpf` in the list of security
    /// modules it actually started, this is where it stops — which is a
    /// distinction `docs/hardware.md` exists to make, because the first is what
    /// everybody checks and the second is what decides.
    #[error("the kernel would not attach the boundary to file_open")]
    WillNotAttach(#[source] aya::programs::ProgramError),

    /// The compiled program has no map or no program of that name.
    ///
    /// Not something a machine can be in the wrong state for: it means the two
    /// halves of this crate were built from different sources.
    #[error("the compiled boundary has nothing called {what} in it")]
    NothingCalled {
        /// What was looked for.
        what: &'static str,
    },

    /// The map would not take what was put in it.
    #[error("the kernel would not take an entry for the boundary")]
    WillNotHold(#[source] aya::maps::MapError),

    /// The name given for a turn's cgroup is not one this will make.
    ///
    /// A cgroup is made by creating a directory, so a name with a separator or
    /// a step upwards in it is a caller choosing where in `/sys/fs/cgroup` to
    /// write. Refused rather than cleaned up, which is `path.rs`'s rule about
    /// normalising, one crate along.
    #[error("{name} is not a name a turn's cgroup can be given")]
    NotAName {
        /// What was asked for.
        name: String,
    },

    /// A path a boundary was to be drawn around could not be looked at.
    ///
    /// Refused rather than answered with zeroes, because a place made of
    /// nothing is a real place — whichever filesystem is numbered zero — and a
    /// turn bound to it would be a turn bound to somewhere nobody granted.
    #[error("cannot find out what the kernel calls {path}: {why}")]
    NotAPlace {
        /// What was named.
        path: String,

        /// What the machine said.
        #[source]
        why: io::Error,
    },

    /// The cgroup filesystem would not do what was asked.
    #[error("{what} {path}: {why}")]
    Cgroup {
        /// What was being done.
        what: &'static str,

        /// To what.
        path: String,

        /// What the machine said.
        #[source]
        why: io::Error,
    },
}

impl NotBounded {
    /// A field this kernel does not have.
    #[must_use]
    pub fn missing(field: Field) -> Self {
        Self::FieldIsMissing {
            structure: field.structure(),
            member: field.member(),
        }
    }

    /// A field this kernel has, at a width the program does not read.
    #[must_use]
    pub fn wrong_width(field: Field, found: u32) -> Self {
        Self::FieldIsNotTheWidth {
            structure: field.structure(),
            member: field.member(),
            found,
            wanted: field.width(),
        }
    }
}
