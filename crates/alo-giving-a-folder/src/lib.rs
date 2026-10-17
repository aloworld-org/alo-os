//! The one thing on this machine that may give a directory to a person.
//!
//! [ADR 0119](../../../docs/decisions/0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)
//! is why this crate exists at all, and why it is a crate rather than a few
//! lines inside something that is already root.
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) term 8
//! is what it does: `/var/lib/alo-undo` is `0711 root:root`, each person's
//! directory inside it is `0700` and theirs, and **a privileged step makes
//! both**. That folder is where the agent's undo snapshots go, so without it
//! [ADR 0064](../../../docs/decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)'s
//! *a snapshot before each run, so it can be undone* cannot be kept.
//!
//! | | |
//! |---|---|
//! | [`what_to_make`] | What should exist, in the order it has to be made — two numbers and a path, decided without touching a disk |
//! | [`made_for`] | The part that touches the disk, and the one place that may `chown` |
//! | [`NotMade`] | Why it did not happen, which is always the machine being unable |
//!
//! # Why this is its own component, measured rather than preferred
//!
//! Handing a directory to somebody needs `CAP_CHOWN`, and no unit in `image/`
//! granted it. The two components already running as root are each
//! `CapabilityBoundingSet=` with nothing after it, so neither could do this
//! without being given a power it has gone out of its way not to have.
//!
//! ADR 0119 measured the three candidates:
//!
//! ```text
//! alo-compositor (from alo-shell)   113,587 lines   51 dependencies
//! alo-sessiond                        2,160 lines    5 dependencies
//! this crate                            ~100 lines    1 dependency
//! ```
//!
//! **A capability belongs to the smallest thing that can hold it.** The
//! tempting candidate was the worst one — the account is already written inside
//! `alo-compositor`, which is also the screen a person types a password into —
//! and `alo-sessiond`'s own test had already answered the question: *adding the
//! capability is a decision and not a fix.*
//!
//! # One capability, and that is a measurement
//!
//! `CAP_CHOWN` is **necessary**: with every capability except it, root cannot
//! give a directory away. And it is **sufficient for the whole job**: with only
//! that one, making the directory, setting its mode and giving it away all
//! succeed, because this process owns what it has just made.
//!
//! So the unit grants one capability and there is no second to argue about.
//!
//! # What it will not do
//!
//! **It never removes anything.** Removing a read-only snapshot needs
//! `CAP_SYS_ADMIN`, it is `alo-letting-go`'s, and it is a different grant to a
//! different unit. A component that could both give and take would be two
//! powers in the place ADR 0119 chose for holding one.
//!
//! **It reads nothing but its two arguments.** No configuration file, nothing
//! from a network, nothing from a person. What a reader has to check is two
//! numbers and a path.

#![cfg(target_os = "linux")]

mod failing;
mod giving;

pub use failing::NotMade;
pub use giving::{A_PERSONS_MODE, THE_FOLDER, THE_FOLDERS_MODE, ToMake, made_for, what_to_make};
