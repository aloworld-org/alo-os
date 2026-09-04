//! The only file here that talks to the kernel, and the only one with `unsafe`
//! in it.
//!
//! `alo-agentd`'s `unix.rs` is the shape this follows: what the kernel will
//! only be asked for through `unsafe` is asked in one file, named there, and
//! every other file in the crate is held to the rule. There are four such
//! things and no others.
//!
//! **The symbol the kernel attaches to.** `#[lsm(hook = "file_open")]` puts an
//! exported name in a link section the loader looks for. Exporting a symbol is
//! `unsafe` in this edition because the compiler can no longer prove nothing
//! else claims the name.
//!
//! **The argument.** An LSM hook is called with the arguments of the kernel
//! function it stands in front of. `security_file_open(struct file *file)` has
//! one, and a second the kernel adds: whatever the previous LSM on this path
//! decided.
//!
//! **A read of kernel memory.** `bpf_probe_read_kernel` is how a BPF program
//! dereferences anything, and it is unsafe for the ordinary reason: it is given
//! an address. What makes it *safe* here is not a proof by the compiler — it is
//! that the read is a helper call which fails rather than faults, and that the
//! kernel's verifier has already refused to load this program if it could read
//! somewhere it should not.
//!
//! **A lookup in a map.** `HashMap::get` is unsafe in `aya-ebpf` because a map
//! this program only reads and the daemon only writes could, in general, have
//! an entry removed underneath the reference. Here the value is copied out
//! before anything else happens, so the reference does not outlive the lookup.
//!
//! # What is not here
//!
//! No writing. Nothing in this file puts anything into a map, a ring buffer, a
//! counter or a log line, and there is no `bpf_printk`. That is ADR 0015's
//! *the LSM decides and forgets*, and the absence is the whole of it: a program
//! with nowhere to write cannot become a record of somebody's day.

// The one module in this crate the rule is lifted for. Everything the four
// paragraphs above describe is below; nothing else in the package may.
#![allow(
    unsafe_code,
    reason = "a BPF program is a raw pointer from the kernel and helper calls on it; \
              the crate root has the whole argument"
)]

use aya_ebpf::{
    helpers::{bpf_get_current_cgroup_id, bpf_probe_read_kernel},
    macros::{lsm, map},
    maps::{Array, HashMap},
    programs::LsmContext,
};

use alo_bounding_map::{Field, Place};

use crate::deciding;

/// Which turn may reach where: the cgroup a turn runs in, against the one place
/// it was granted.
///
/// The daemon writes an entry when a turn begins and takes it out when the turn
/// ends, and a cgroup with no entry is a program that is not an agent turn —
/// which is every other process on the machine.
#[map(name = "BOUNDS")]
static BOUNDS: HashMap<u64, [u64; 2]> = HashMap::with_max_entries(1024, 0);

/// Where the fields this program reads sit in this kernel's own structures.
///
/// Filled by the daemon out of `/sys/kernel/btf/vmlinux` before the program is
/// attached, so nothing here is compiled against a kernel version. `Field` is
/// the agreement about which slot is which.
#[map(name = "FIELDS")]
static FIELDS: Array<u32> = Array::with_max_entries(8, 0);

/// Every open of every file, on this machine, from now until the program is
/// detached.
///
/// It is worth reading that sentence twice, because it is the reason ADR 0015
/// calls this the most dangerous thing in the repository. What the function
/// does with that reach is look up one cgroup and, almost always, find nothing
/// and return.
#[lsm(hook = "file_open")]
pub fn file_open(ctx: LsmContext) -> i32 {
    // `security_file_open(struct file *file)`, and after the hook's own
    // arguments the kernel appends what the previous LSM on this path decided.
    let file: u64 = ctx.arg(0);
    let already: i32 = ctx.arg(1);
    if already != 0 {
        // Somebody else has already refused. Ours is not the decision that
        // matters, and turning a refusal into an allow is not something an
        // additional security module may do.
        return already;
    }
    deciding::decide(file)
}

/// Which turn this open belongs to, or the cgroup of whoever is not in one.
pub fn turn() -> u64 {
    unsafe { bpf_get_current_cgroup_id() }
}

/// The place a turn was granted, or [`None`] if this cgroup is not a turn.
///
/// [`None`] is the answer for every ordinary program on the machine, and it is
/// the answer that costs nothing: one hash lookup, a miss, and the open goes
/// ahead as though this program were not loaded.
pub fn granted(turn: u64) -> Option<Place> {
    // Copied out immediately, so the borrow of the map's value does not outlive
    // the lookup — which is the whole of what `get`'s safety note asks for.
    let words = *unsafe { BOUNDS.get(turn) }?;
    Some(Place::of_words(words))
}

/// Where a field sits in this kernel, as the daemon found it.
pub fn offset(field: Field) -> Option<u64> {
    FIELDS.get(field.index()).map(|found| u64::from(*found))
}

/// Eight bytes of kernel memory, or [`None`] if the kernel would not give them.
pub fn word_at(address: u64) -> Option<u64> {
    if address == 0 {
        return None;
    }
    unsafe { bpf_probe_read_kernel(address as *const u64) }.ok()
}

/// Four bytes of kernel memory, or [`None`] if the kernel would not give them.
pub fn half_word_at(address: u64) -> Option<u32> {
    if address == 0 {
        return None;
    }
    unsafe { bpf_probe_read_kernel(address as *const u32) }.ok()
}
