//! The boundary a turn runs inside, imposed by the kernel rather than promised
//! by us.
//!
//! [ADR 0013](../../../docs/decisions/0013-the-grant-is-enforced-by-the-kernel.md)
//! says what is wrong today: `alo-record` records what `alo-agentd` reports, so
//! a verb with a bug writes down a lie in the person's own language with a
//! grant id beside it, looking exactly like the truth.
//! [ADR 0015](../../../docs/decisions/0015-the-kernel-learns-what-a-turn-is.md)
//! is the mechanism that fixes it: teach the kernel the one noun it does not
//! have — the agent turn — by attaching our own program to its security hooks.
//!
//! This crate is the smallest thing that can carry that idea or disprove it.
//! One program, on one hook, holding one grant.
//!
//! | | |
//! |---|---|
//! | [`Imposed`] | The programme loaded into the kernel and pinned, which is `alo-boundaryd`'s and needs `CAP_BPF` |
//! | [`Pinned`] | Where it is pinned, and the modes that decide who may reach it there |
//! | [`Boundary`] | The one map a person's own daemon writes, and the entry that tells the kernel what a turn may reach |
//! | [`Cgroup`] | The control group a turn runs in, which is how the kernel tells one turn from another |
//! | [`Turns`] | Where this service's turns are made, and the one door into and out of a boundary |
//! | [`place_of`] | A folder, as the two numbers the kernel knows it by |
//! | [`places_of`] | The paths one execution named, as the bound it runs inside |
//! | [`NotBounded`] | Why a boundary could not be imposed, which is always a refusal |
//!
//! # The crate is in two halves, and they run as two different users
//!
//! [ADR 0018](../../../docs/decisions/0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)
//! divided it. Loading a BPF LSM programme needs `CAP_BPF` and `CAP_SYS_ADMIN`,
//! and `alo-agentd` runs as the signed-in person — *never with capabilities the
//! person does not have* (ADR 0001 §2). So [`Imposed`] is `alo-boundaryd`'s, it
//! runs once at boot as root, and it pins what it loaded; [`Boundary`] is the
//! per-person daemon's, it opens one of those pins by path, and it holds no
//! capability at all. **The interface between them is a file with a group and a
//! mode on it**, not an API, and `pinned.rs` is where that is decided.
//!
//! # What actually happens
//!
//! ```text
//! at boot       alo-boundaryd loads the programme, attaches it to file_open,
//!               and pins the link and both maps under /sys/fs/bpf/alo
//!
//! turn begins   a cgroup is made, and the work runs in it
//!               each path this execution named is resolved to a filesystem
//!               and an inode
//!               { cgroup id -> those places } goes into a map the kernel reads
//!
//! every open    a BPF LSM program on file_open looks up the cgroup
//!               not a turn  -> allowed, and nothing is remembered
//!               a turn      -> walk up from the file; a granted place, or EACCES
//!
//! turn ends     the entry is removed, and the authority is gone
//! ```
//!
//! # What runs in the cgroup, and why nothing is started
//!
//! *Runs the verb's work inside that cgroup* is ADR 0015's line, and taken
//! literally it says **start something**. Law 2 is why it does not: every shape
//! that spawns needs a program to spawn, and a program alo OS starts on an
//! agent's behalf is one review away from a program an agent named.
//!
//! So nothing is started. What a turn's work already is on this machine is one
//! thread of `alo-agentd` calling `alo-files`, which is one of six verbs on a
//! closed list — and **that thread is what goes into the cgroup**, by writing one
//! byte into `cgroup.threads`. There is no `fork`, no `exec` and no `Command` in
//! this crate, and `tests/a_turn_is_this_thread.rs` reads the crate's own source
//! and says so.
//!
//! It is the narrower answer as well as the lawful one. A whole process in the
//! cgroup would put the record, the socket and the person's own door inside the
//! agent's boundary; one thread puts the verb inside it and leaves the service
//! outside. [`Turns`] is the arrangement that makes it possible and
//! [`Turns::doing`] is the only door.
//!
//! # This crate is Linux, and on any other host it is nothing
//!
//! Every module is `#[cfg(target_os = "linux")]`. A BPF program, a cgroup and
//! an LSM hook have no meaning anywhere else, and a `Boundary` that could be
//! constructed on a machine unable to impose one would be a type whose
//! existence means nothing — which is `alo-agentd`'s argument, and this crate
//! is the second to make it.
//!
//! **So the test count is part of the result.** On Windows this compiles to
//! almost nothing, runs no tests and exits `0`, which is the same green as a
//! full pass. `docs/autonomy/LOOP.md` has the rule that came out of that
//! happening once already: for a crate like this, the run under Linux is *the*
//! gate rather than a supplement to one, and the number of tests it ran is
//! reported rather than the colour.
//!
//! # This crate is a mechanism, and it says nothing to anybody
//!
//! It is wired into a turn as of item 26d, and the wiring is somewhere else on
//! purpose: `alo-agentd`'s `bounding.rs` implements `alo_turn::Bounding` out of
//! [`Turns`], [`Boundary`] and [`places_of`], because the daemon is the one
//! thing that holds both a turn and a kernel. Nothing here reaches `alo-turn`
//! and nothing here knows what a verb is.
//!
//! It also holds **no words**. It had one — a single sentence for all fifteen
//! reasons in [`NotBounded`] — and item 26d moved it to `alo-turn`, where the
//! crate that tells a person lives and where a machine's vocabulary can actually
//! find it; `failing.rs` has the argument. What is left of that division is the
//! administrator's half: every reason keeps its English and its
//! [`Display`](std::fmt::Display), for the service log.
//!
//! So what is here is the whole of the mechanism and none of the policy: the
//! kernel refusing (item 26), the thread that goes into the boundary and comes
//! back out of it (item 26a), **which** places a turn is bound to (item 26b) —
//! the ones this execution named, which [`places_of`] makes and says why — and
//! the door a service makes its subtree through ([`Turns::of_this_service`]).
//! Which paths a real call names is `alo_files::Reaching`'s (item 26c), and the
//! order they are asked in is `alo-turn`'s `carrying.rs`.
//!
//! # The dangerous property, said out loud
//!
//! A BPF LSM sits on the security hooks, so it sees every open on the machine
//! by construction. The mechanism that enforces a grant would also record a
//! person's whole day, and only the discipline differs. So **the program
//! decides and forgets**: it has no ring buffer, no counter and no log line,
//! and an open outside a turn is one hash lookup that misses and changes
//! nothing. `crates/alo-bounding-kernel/src/kernel.rs` is where that absence
//! is, and `tests/the_boundary_decides_and_forgets.rs` is what holds it there —
//! ordinary programs spend a day opening files under the loaded program, and
//! afterwards the program still has two maps, the map of turns is empty, the
//! spare slots of the other are still zero, and this kernel's trace buffer has
//! not been written a line. [`Imposed::every_map_the_kernel_holds`],
//! [`Boundary::every_turn_the_kernel_is_holding`] and
//! [`Imposed::every_field_the_kernel_was_given`] are what that is counted
//! through, and each of them is read out of the kernel rather than remembered
//! here. Two of the three are the loader's since ADR 0018 and one is the
//! daemon's, which is the division rather than an accident: what a person's own
//! service can reach is the map it writes, and the map of fields is one it
//! cannot open at all.
//!
//! # What this boundary watches on a filesystem, and what it does not
//!
//! Four hooks decide about files — `file_open`, `inode_rename`, `inode_unlink`
//! and `inode_link` — and a fifth, `socket_connect`, about the network. A
//! filesystem has more verbs than four, and somebody auditing this crate is owed
//! the list of the ones nothing here decides about rather than the count of the
//! ones it does.
//!
//! **The promise these four keep is narrower than *a turn cannot change
//! anything outside its bound*, and reading the second where the first is
//! written is the mistake this section exists to prevent.** What they keep is
//! this: **no mutation left unwatched moves a byte of somebody's file past a
//! grant.** A turn can still make a symbolic link (`inode_symlink`) in a folder
//! somebody granted that leads to a file nobody did — and reading through it is
//! an open of the file it leads to, refused. It can make a file
//! (`inode_create`, or `inode_mknod` without an open at all) in a folder nobody
//! granted — and putting anything in it is an open, refused, so what it leaves
//! is an empty file. It can make and remove **empty** directories
//! (`inode_mkdir`, `inode_rmdir`) — and emptying one that is not needs
//! `inode_unlink`, which is watched. It can change a file's mode, owner, times
//! and extended attributes (`inode_setattr`, `inode_setxattr`) — and is no
//! better off, because what decides here is where a file is and not what its
//! mode says.
//!
//! **One of those goes further than the promise and is not hidden inside it.**
//! `truncate(2)` reaches `inode_setattr` without opening anything, so a bound
//! turn can empty a file nobody granted it. Nothing is read and nothing is
//! copied, so no contents leave a grant; contents are destroyed where they sit,
//! which is a different harm and a real one. `docs/quirks.md` has the
//! measurement and says why it is the one item not in the committed suite.
//!
//! Two things sit beside the list rather than in it. **A descriptor opened
//! before a turn began** stays usable inside it, which is its own section below.
//! And **starting a program is not a way round any of this**: `execve` opens the
//! file it runs, so a turn asking for a program outside its bound is refused like
//! any other file.
//!
//! Every one of these is reproduced against the real loaded programme in
//! `tests/what_a_bound_turn_can_still_change.rs`, each with a refused open
//! beside it proving the boundary was in force and a legitimate write inside the
//! grant proving it was not simply refusing everything. `docs/quirks.md` carries
//! the same list with the release that owns closing each — all of them v0.5,
//! with ADR 0013's other primitives — and
//! `tests/the_unwatched_mutations_are_written_down.rs` fails the day one of them
//! lands while the documents still call it unwatched.
//!
//! # What a turn inherits, and why it is not on that list
//!
//! Every hook above decides at the moment something is *done to a name*. None of
//! them decides about a descriptor that already exists, and there is no hook here
//! on a read, on a write, or on a descriptor arriving from somewhere else. **So
//! anything open when a turn begins stays fully usable inside it**, and the
//! boundary is never asked.
//!
//! That is not a corner of the design, it is most of the daemon. [`Turns::doing`]
//! puts **one thread** of `alo-agentd` into a control group — the argument is in
//! `turns.rs` and it is law 2's — and a thread shares its process's whole
//! descriptor table. What is in that table on this machine today is the record
//! `alo_keeping::Writing` holds open for appending, the socket the daemon is
//! listening on and the caller it is answering, standard output and error, and
//! [`Turns::doing`]'s own way out of a turn.
//!
//! **A descriptor opened before a turn began is the one gap in this crate that
//! moves contents past a grant.** The list above was measured against exactly
//! that promise and every item keeps it; this does not. A turn reads a file
//! nobody granted through an inherited descriptor and writes what it read into
//! the folder somebody did, where a `move_file` or an `archive_folder` carries it
//! onwards and where the record names only a granted path. It is why this is its
//! own piece of work rather than a seventh row.
//!
//! What it does **not** permit is measured beside it and is the floor under it: a
//! descriptor cannot be reopened by name, `/proc/self/fd/<n>` does not turn one
//! back into an open — the walk starts at the file the open really reached — and
//! `openat` relative to an inherited folder is an open like any other, so a
//! directory handle is not a key to what is under it.
//!
//! **Closing it is a decision rather than a patch, and this crate has not taken
//! it.** The kernel's answer would be `file_permission` — a hook on every read
//! and write on the machine, which is the opposite direction from *decides and
//! forgets* — and it would refuse a turn its own way out, because leaving one is
//! a write to a descriptor opened before it began. The other answer is to make a
//! turn a process of its own, which is a change to what a turn *is* and belongs
//! in an ADR. `docs/quirks.md` carries the account, every row of it is reproduced
//! in `tests/what_a_turn_inherits.rs` against the real loaded programme, and
//! `tests/what_a_turn_inherits_is_written_down.rs` fails the day either hook
//! lands while the documents still say neither has.
//!
//! # What this boundary can decide about the network, and what it cannot
//!
//! ADR 0013 gives this crate a second job it has not started: *which sockets
//! may be opened, and attribution of every one to the turn that caused it.* The
//! promise it answers is the egress indicator's own remaining half — **the
//! enforcement at the network boundary, without which all of this describes
//! only the code that asked.** This section is the policy, written before a
//! hook is chosen, because the obvious reading does not survive contact with
//! the kernel.
//!
//! **`alo-egress`'s policy cannot be enforced here, and pretending otherwise
//! would be the failure.** That policy decides by *provider* and by *region*: a
//! question may be answered in the building, on this machine, or in a named
//! part of the world. A programme on a socket sees a control group, a protocol
//! and an address. A provider is a name somebody resolves through DNS and a
//! region is a fact about a company — neither is visible where the enforcement
//! would sit, and a kernel-side approximation of them (an address list, a guess
//! from a hostname seen earlier) would be a second policy disagreeing with the
//! first in ways nobody could predict. **The kernel is not the policy engine
//! and this crate will not make it one.**
//!
//! What *is* enforceable is narrower and stronger, and it is what ADR 0013
//! actually names:
//!
//! > **A turn opens no socket unless the person has been shown that it is
//! > about to.**
//!
//! `alo-egress` already makes that a thing with a type: an
//! `alo_egress::Departing` cannot be obtained without `Indicator::beginning`
//! having asked the policy and shown the person, and nothing may open a
//! connection without one. Today that is a promise the daemon keeps. The
//! enforcement is the same promise with the kernel behind it — the daemon
//! writes a turn's permission to leave where a programme can read it, exactly
//! as it writes the places a turn may reach, and a turn with nothing written
//! for it is refused by the machine rather than by our own code.
//!
//! Three things follow, and they are the whole of the policy:
//!
//! - **Default deny, and for turns only.** A bound turn with no departure
//!   written opens no socket. Every other process on this machine — a person's
//!   browser, their mail client, this service's own errands, which are not
//!   turns — is unaffected, exactly as it is for files.
//! - **Attribution is the same question as permission.** What the kernel is
//!   asked is *which turn is this*, and it already knows: the control group. No
//!   second mechanism, and nothing new for the daemon to keep in step with.
//! - **Nothing is written down by the programme.** ADR 0015's *the LSM decides
//!   and forgets* is not relaxed for sockets. What a person is shown is what
//!   `alo-egress` shows them; the kernel's part is refusing, not recording.
//!
//! **This is not a widening.** No grant covers more, no agent gains a
//! capability, and a turn that could open a socket before can still open
//! exactly the ones the person was shown. What changes is that a verb with a
//! bug in it can no longer open one the person was not shown — the same floor
//! the file hooks are, under the other half of law 1.

#![cfg(target_os = "linux")]

mod bounding;
mod btf;
mod cgroup;
mod failing;
mod fields;
mod imposing;
mod inside;
mod pinned;
mod place;
mod places;
mod turns;
mod waiting;

#[cfg(test)]
mod testing;

pub use bounding::Boundary;
pub use btf::{Member, Types};
pub use cgroup::Cgroup;
pub use failing::NotBounded;
pub use fields::Offsets;
pub use imposing::Imposed;
pub use pinned::{Pinned, THE_ROOT};
pub use place::{as_the_kernel_keeps_it, place_of};
pub use places::places_of;
pub use turns::Turns;
pub use waiting::{A_NOTE, AT_MOST, NotWaited, ON_THIS_KERNEL, Waited};

pub use alo_bounding_map::{
    Bounds, DEPTH, DESTINATIONS, Departure, Departures, Family, Field, PLACES, Place, WORDS,
};
