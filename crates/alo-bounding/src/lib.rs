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
//! | [`Boundary`] | The program in the kernel, and the entry that tells it what a turn may reach |
//! | [`Cgroup`] | The control group a turn runs in, which is how the kernel tells one turn from another |
//! | [`Turns`] | Where this service's turns are made, and the one door into and out of a boundary |
//! | [`place_of`] | A folder, as the two numbers the kernel knows it by |
//! | [`places_of`] | The paths one execution named, as the bound it runs inside |
//! | [`NotBounded`] | Why a boundary could not be imposed, which is always a refusal |
//!
//! # What actually happens
//!
//! ```text
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
//! # What this crate is not, yet
//!
//! It is not wired into a turn. `alo-turn` joins an invocation, a call, an
//! approval, an execution and the record, and `alo-agentd` is what holds one; a
//! boundary around the execution and *not* around the entry written afterwards
//! needs those two doors separated, and that is queue item 26c.
//!
//! What is here is the whole of the mechanism a turn stands on: the kernel
//! refusing (item 26), the thread that goes into the boundary and comes back out
//! of it (item 26a), the sentence a person reads when it could not be imposed,
//! and **which** places a turn is bound to (item 26b) — the ones this execution
//! named, which [`places_of`] makes and says why.
//!
//! What is not here is the caller that names them. Nothing on this machine calls
//! [`places_of`] with a real call's paths yet, because the crate that would is
//! `alo-turn` and its doors do the disk work and write the record together; a
//! thread bounded across both would be refused the record. Separating them, and
//! `alo-agentd` making its subtree when it starts, is item 26c.
//!
//! # The dangerous property, said out loud
//!
//! A BPF LSM sits on the security hooks, so it sees every open on the machine
//! by construction. The mechanism that enforces a grant would also record a
//! person's whole day, and only the discipline differs. So **the program
//! decides and forgets**: it has no ring buffer, no counter and no log line,
//! and an open outside a turn is one hash lookup that misses and changes
//! nothing. `crates/alo-bounding-kernel/src/kernel.rs` is where that absence
//! is, and queue item 27 is the test that holds it there.

#![cfg(target_os = "linux")]

mod bounding;
mod btf;
mod cgroup;
mod failing;
mod fields;
mod inside;
mod place;
mod places;
mod turns;
pub mod words;

#[cfg(test)]
mod testing;

pub use bounding::Boundary;
pub use btf::{Member, Types};
pub use cgroup::Cgroup;
pub use failing::NotBounded;
pub use fields::Offsets;
pub use place::{as_the_kernel_keeps_it, place_of};
pub use places::places_of;
pub use turns::Turns;
pub use words::{EVERY_WORD, Word, WordsError, bounding_words, declare_into};

pub use alo_bounding_map::{Bounds, DEPTH, Field, PLACES, Place, WORDS};
