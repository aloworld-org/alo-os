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
//! Every one of these says something about the *kernel underneath the daemon*,
//! or about a machine this alo OS cannot bound a turn on — that the kernel
//! publishes no type information, that a structure has moved, that the verifier
//! refused a program, that a call named more places than an entry holds. Nobody
//! signs in to a machine and reads them.
//!
//! # The sentence a person reads is not this crate's, and item 26d moved it
//!
//! There was a [`Display`](std::fmt::Display)-free half here — one word, in a
//! `words.rs` of this crate's own, rendering one sentence for all fifteen
//! reasons. It is now `alo_turn::words::NOT_BOUNDED`, and the move is the
//! 9-series' own rule rather than a tidying:
//!
//! - **Nothing could look it up.** This crate is Linux, so its list is not in
//!   the vocabulary `alo-saying` collects, and the sentence would have reached a
//!   person only on a machine whose process had been told to declare it. A
//!   string that arrives as a key on a forgetful machine is not a string.
//! - **This crate says nothing to anybody.** It is a mechanism: a programme, a
//!   control group and a map. What tells a person their agent did nothing is the
//!   crate holding the turn, and that crate is portable, so its refusal is
//!   sayable on every host it compiles for.
//!
//! What is left here is the administrator's half, whole: [`Display`] on every
//! variant, in English, for the service log. `alo_turn::NoBoundary` carries it
//! there as text and never shows it.
//!
//! [`Display`]: std::fmt::Display

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

    /// There is no boundary pinned on this machine.
    ///
    /// The daemon's refusal since ADR 0018, and it replaced *cannot load*: the
    /// daemon no longer loads anything, so what it can find out is whether
    /// something else did. On a machine that boots properly this cannot happen —
    /// `alo-boundaryd` runs before `alo-agentd` — so it names the service rather
    /// than the syscall, because whoever reads it is looking at a boot order.
    #[error(
        "there is no boundary at {path}: alo-boundaryd loads one at boot, and until it has, this \
         machine cannot bound a turn"
    )]
    NoBoundaryHere {
        /// Where it was looked for.
        path: String,
    },

    /// The directory the boundary is pinned in could not be made.
    ///
    /// Almost always a machine with no BPF filesystem mounted at
    /// `/sys/fs/bpf` — pinning is a `bpffs` operation and there is nowhere to
    /// pin to without one. Nothing here mounts it: a boot has already decided
    /// what is mounted, and `docs/hardware.md` asks the question.
    #[error("the boundary has nowhere to be pinned at {path}: {why}")]
    NoPinDirectory {
        /// Where it was to be made.
        path: String,

        /// What the machine said.
        #[source]
        why: std::io::Error,
    },

    /// The kernel would not pin one of the three.
    #[error("the kernel would not pin {what} at {path}")]
    WillNotPin {
        /// Which of the three it was.
        what: &'static str,

        /// Where it was to go.
        path: String,

        /// What the kernel said.
        #[source]
        why: aya::pin::PinError,
    },

    /// A boundary is already pinned where this one was to go.
    ///
    /// Refused rather than replaced. A second programme on `file_open` is a
    /// second boundary — both are asked about every open, either can refuse
    /// one — so which grant a turn is really running under would stop being a
    /// question with one answer.
    #[error("a boundary is already pinned at {path}, and a machine has one")]
    AlreadyThere {
        /// The pin that is in the way.
        path: String,
    },

    /// The agent's group could not be given the map it writes.
    ///
    /// Which leaves a machine whose boundary is loaded and whose daemon cannot
    /// reach it, so it is a refusal at boot rather than a surprise at the first
    /// turn.
    #[error("the boundary at {path} could not be given to group {group}: {why}")]
    NotOurGroup {
        /// Which pin.
        path: String,

        /// The group it was to be given to.
        group: u32,

        /// What the machine said.
        #[source]
        why: std::io::Error,
    },

    /// A pin could not be shut to everybody else.
    #[error("the boundary at {path} could not be shut to {mode:o}: {why}")]
    NotShutTo {
        /// Which pin.
        path: String,

        /// The mode it was to be set to.
        mode: u32,

        /// What the machine said.
        #[source]
        why: std::io::Error,
    },

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

    /// A turn's boundary was to be drawn around no places at all.
    ///
    /// Not a narrower grant: a turn bound to nowhere is refused every open it
    /// makes, including the ones it needs in order to report that it failed. So
    /// it is refused at the door, where the reason can still be said.
    #[error("a turn's boundary is drawn around at least one place, and none was named")]
    NothingToBound,

    /// More places than one entry in the kernel's map holds.
    ///
    /// Refused rather than cut down to the first few. A turn bounded to some of
    /// what it named would do part of what it was asked and be refused the rest
    /// by the kernel, which reads to whoever is watching as a broken machine —
    /// and ADR 0015's rule is that a boundary that cannot be applied means the
    /// turn does not run.
    #[error("a turn named {asked} places to be bounded to, and one entry holds {most}")]
    TooManyPlaces {
        /// How many were named.
        asked: usize,

        /// How many one entry holds.
        most: usize,
    },

    /// This machine has no unified control group hierarchy to make a turn in.
    ///
    /// Which means `/proc/self/cgroup` said something this cannot read, on a
    /// machine still running cgroup v1 or with the hierarchy unmounted. There is
    /// nowhere to put a turn, so there is no turn.
    #[error("{what}")]
    NotInAHierarchy {
        /// Which way it stopped making sense.
        what: &'static str,
    },

    /// A thread went into a turn and could not be brought back out.
    ///
    /// The worst thing in this enum, and the one the service stops over. The
    /// thread is still inside the boundary, so it is refused everything outside
    /// a grant that is over — which fails closed, and is why the entry is left
    /// in the kernel rather than taken away underneath it.
    #[error("a thread went into a turn and could not be brought back out: {why}")]
    NotBroughtBack {
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
    /// Whether a thread went into a boundary and is still in there.
    ///
    /// The one question a caller has to be able to ask without reading English:
    /// every other reason means nothing was attempted, and this one means a
    /// thread of the service is inside a turn that is over, refused everything
    /// outside a grant that no longer exists. What `alo-agentd` does about it is
    /// stop, the way it stops when nothing can be written down, and a service
    /// cannot decide that from a sentence.
    #[must_use]
    pub const fn a_thread_is_still_inside(&self) -> bool {
        matches!(self, Self::NotBroughtBack { .. })
    }

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
