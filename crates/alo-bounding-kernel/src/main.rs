//! The half of the boundary that runs inside the kernel.
//!
//! One BPF LSM program, attached to `file_open`, which is
//! [ADR 0015](../../../docs/decisions/0015-the-kernel-learns-what-a-turn-is.md)
//! made small enough to be either proved or disproved: a turn granted a folder
//! opens a file inside it, and the same turn reaching for a private key gets
//! `EACCES` from the kernel rather than a sentence from us.
//!
//! | | |
//! |---|---|
//! | [`kernel`] | The only file here that talks to the kernel, and the only one with `unsafe` in it |
//! | [`deciding`] | What happens on an open, in ordinary Rust |
//!
//! # Why this is its own package
//!
//! It compiles for `bpfel-unknown-none` — a virtual machine inside the kernel
//! with no allocator, no threads and an instruction budget — using the nightly
//! compiler and `-Z build-std=core`, because there is no prebuilt `core` for
//! that target. None of that is true of anything else in this repository, and a
//! workspace has one lockfile and one toolchain. `alo-bounding`'s `build.rs`
//! builds this package and embeds the result, so a person building alo OS runs
//! `cargo build` and nothing else.
//!
//! # There is `unsafe` here, and this is the honest account of it
//!
//! `CLAUDE.md` forbids `unsafe`, and the workspace above enforces it with
//! `unsafe_code = "forbid"`. A package outside that workspace does not inherit
//! the lint, and being quiet about that would be the loop working around a rule
//! rather than meeting it. So:
//!
//! - The crate root **denies** `unsafe_code`, and exactly one module — [`kernel`]
//!   — allows it, which is the same shape as `alo-agentd`'s `unix.rs` and
//!   `signalling.rs`: what the kernel will only be asked for through `unsafe` is
//!   asked in one file, and every other file is held to the rule.
//! - What is `unsafe` here is not a choice between a safe spelling and an unsafe
//!   one. Reading a `struct file` handed over as a raw pointer *is* the program;
//!   there is no crate to rent that makes it safe, because the safety comes from
//!   the kernel's verifier refusing to load a program that would read out of
//!   bounds — a check stricter than the compiler's and made after ours.
//! - Four things need it: the exported symbol the kernel attaches to, the
//!   argument the hook is called with, a read of kernel memory, and a lookup in
//!   a map. They are the whole of it, and they are all in one file.
//!
//! # Nothing here is written down
//!
//! ADR 0015's dangerous property is that a program on the security hooks sees
//! everything by construction, and the discipline against it is that **the LSM
//! decides and forgets**. There is no map of observations here, no counter, no
//! ring buffer and no `bpf_printk`: an open outside a turn is looked up, missed,
//! and allowed, and nothing anywhere is different afterwards. Queue item 27 is
//! the test that holds this to it.

#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![warn(missing_docs)]

mod deciding;
mod kernel;

/// What a BPF program does when something has gone wrong, which is nothing.
///
/// There is nothing to unwind into and nowhere to report to, and a program the
/// verifier can reach a panic in is a program it refuses to load. Everything
/// here is written to be unable to panic — no indexing, no arithmetic that can
/// overflow, no allocation — so this exists to satisfy `core` rather than to be
/// reached.
#[cfg(target_arch = "bpf")]
#[panic_handler]
fn nothing_to_be_done(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
