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

pub use alo_bounding_map::{Bounds, DEPTH, Field, PLACES, Place, WORDS};
