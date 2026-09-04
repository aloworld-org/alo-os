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
//! | [`place_of`] | A folder, as the two numbers the kernel knows it by |
//! | [`NotBounded`] | Why a boundary could not be imposed, which is always a refusal |
//!
//! # What actually happens
//!
//! ```text
//! turn begins   a cgroup is made, and the work runs in it
//!               the granted folder is resolved to a filesystem and an inode
//!               { cgroup id -> place } goes into a map the kernel reads
//!
//! every open    a BPF LSM program on file_open looks up the cgroup
//!               not a turn  -> allowed, and nothing is remembered
//!               a turn      -> walk up from the file; the granted place, or EACCES
//!
//! turn ends     the entry is removed, and the authority is gone
//! ```
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
//! approval, an execution and the record; making a cgroup, imposing the
//! boundary and running the work inside it is a step in front of all of that,
//! and it is a queue item of its own. What is here is the mechanism and the
//! proof that a kernel really refuses — which is what ADR 0015 asked for first,
//! because everything else in it depends on the answer being yes.
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
mod place;

#[cfg(test)]
mod testing;

pub use bounding::Boundary;
pub use btf::{Member, Types};
pub use cgroup::Cgroup;
pub use failing::NotBounded;
pub use fields::Offsets;
pub use place::{as_the_kernel_keeps_it, place_of};

pub use alo_bounding_map::{DEPTH, Field, Place};
