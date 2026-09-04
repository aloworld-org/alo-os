//! The few bytes the daemon and the kernel must read the same way.
//!
//! [ADR 0015](../../../docs/decisions/0015-the-kernel-learns-what-a-turn-is.md)
//! puts a BPF map between two programs: `alo-bounding` writes into it from this
//! machine, and a program inside the kernel reads out of it on every
//! `file_open`. The two are compiled separately, for different machines, by
//! different invocations of the compiler. **Nothing makes them agree except
//! this crate**, and that is the whole of why it exists.
//!
//! | | |
//! |---|---|
//! | [`Place`] | One thing on a disk, named the way the kernel names it |
//! | [`Bounds`] | Everywhere one turn may reach, which is the value of one entry |
//! | [`reaches`] | Whether an opened file is a granted place or lies under one |
//! | [`Field`] | Where in the kernel's own structures the program has to look |
//!
//! # Why a crate rather than a struct copied twice
//!
//! Because a copy is a promise a reviewer has to check by reading two files and
//! believing their eyes, and a byte order that drifts by one field does not
//! fail loudly — it refuses the wrong files, quietly, on somebody's machine. So
//! the map's key, its value and the order of the two numbers inside it are
//! written down once, in [`Place::words`] and [`Place::of_words`], and both
//! halves go through them.
//!
//! It is `alo-strings`' argument about a declaration and its translation, one
//! layer down: the guarantee holds because there is one thing, not two that
//! currently agree.
//!
//! [`Bounds`] is the same argument about a *width*. A turn is bound to several
//! places, so the entry has a count and a fixed number of slots in it — and a
//! half that read one more slot than the other wrote would compare a folder
//! against whatever the previous turn left in memory. [`PLACES`] is that number,
//! it is here, and changing it either moves both halves or compiles in neither.
//!
//! # Why the containment rule is here too
//!
//! [`reaches`] is the decision itself — the one this repository is asking the
//! kernel to make — and it is written here rather than in the kernel half for a
//! reason that has nothing to do with sharing: **on the BPF target there are no
//! tests.** A program compiled for the kernel cannot be run by `cargo test`, so
//! logic that lives only there is logic nobody can hold to a case. Here it is
//! ordinary Rust with ordinary tests, and the kernel half's job shrinks to
//! fetching the numbers this function asks for.
//!
//! # This crate is `no_std`
//!
//! Not as a preference. Half of its compilations target a virtual machine with
//! no allocator, no threads and no operating system beneath it. Anything that
//! allocates cannot be here, and neither can anything that panics: a panic in a
//! BPF program is not an unwind, it is a program the kernel's verifier refuses
//! to load at all.

#![no_std]

mod bound;
mod bounds;
mod field;
mod reaching;

pub use bound::Place;
pub use bounds::{Bounds, PLACES, WORDS};
pub use field::Field;
pub use reaching::{DEPTH, reaches};
