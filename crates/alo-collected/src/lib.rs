//! Every crate of this workspace that declares words, held to being collected
//! into the one vocabulary — and the one that is not, named with its reason.
//!
//! `alo-saying` is the single vocabulary every word a person reads is collected
//! into, and it exists because a translation is **checked against the vocabulary
//! it is loaded into**: a process holding only its own strings would read a
//! translator's correct line for another part of the system as a mistake. What
//! it collects, though, is a **hand-written list** — a constant and one
//! `declare` call per crate, kept beside the crates rather than derived from
//! them. A crate that declares words and is not on that list compiles, tests,
//! ships, and says nothing to anybody in any language.
//!
//! That is not hypothetical. `crates/alo-overlay` declared nine strings and
//! `alo-saying` collected nothing of them, so every sentence the overlay had been
//! given would have reached a real shell as a missing key. It was found by a
//! person reading — which is how `docs/features.md`'s missing promises were found
//! six times over before `alo-reconciling` made it arithmetic, and how a verb
//! with no plain way would have arrived before `alo-by-hand` did the same.
//!
//! This crate is the same answer for words.
//!
//! # The crates are read out of the workspace, never out of a list kept here
//!
//! [`whoever_declares_words`] walks this workspace's own member list and reads
//! each member's `src/words.rs`, so a crate added anywhere is a crate this check
//! sees, with nothing to update. **Nothing here holds a crate's name** — not the
//! twenty-four that declare words and not the one that stands apart. Comparing
//! two lists written by hand would only ever prove that two lists agree.
//!
//! # Three directions, because a list rots in more than one
//!
//! [`held`] answers all three:
//!
//! | | |
//! |---|---|
//! | A crate declares words and nothing collects them | [`Finding::ACrateNothingCollects`] |
//! | A crate is collected and declares nothing any more | [`Finding::ACollectedCrateThatSaysNothing`] |
//! | A crate stands apart with no reason, or no longer needs to | [`Finding::AnExceptionWithNoReason`], [`Finding::AnExceptionNobodyNeeds`] |
//!
//! The second is not tidiness. A list with a dead name in it is a list nobody
//! reads as a check, and the day it is wrong about a crate that *does* declare
//! words, nobody believes it either way.
//!
//! # The one that is not collected, and why an exception has to argue
//!
//! `alo-agentd` declares three strings and is deliberately outside the one
//! vocabulary: it is Linux, every module in it is compiled out anywhere else, and
//! a vocabulary assembled from it would hold three fewer strings on a host with
//! no daemon than on a machine with one — so one host would refuse a translation
//! file the other accepted, which is the exact failure `alo-saying` exists to
//! prevent, in its platform-shaped form. It declares its own three on top of the
//! machine's.
//!
//! So this check takes exceptions, and holds each of them to a sentence
//! ([`is_a_reason`]). A name on that list with a shrug beside it is a crate that
//! says nothing to anybody in any language — the failure this crate is for, with
//! permission — and the difference between the two is entirely the argument.
//!
//! # It says nothing to a person
//!
//! Nothing here reaches a screen. The reader of a [`Finding`] is whoever is
//! adding a crate, in the repository, with `alo-saying` open — so the findings
//! keep their English and their `Display`, exactly as `alo_saying::NotCollected`,
//! `alo_reconciling::Finding` and `alo_by_hand::Finding` do for the same reason.
//! There are no strings to externalise here, and **`alo-saying` does not collect
//! this crate** — which this check itself is what proves, since a crate with no
//! `src/words.rs` is not a crate that declares words.
//!
//! # It reads no disk
//!
//! [`held`] is handed the two lists, the manifest's text and a way to read a file
//! by its repository-relative path. The test in `tests/` is what puts the real
//! repository behind it. That is what lets every refusal below be shown
//! happening against a fixture: **a check that has never been seen to refuse
//! anything passes on the day it stops looking, in exactly the same colour.**

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod apart;
pub mod declaring;
pub mod finding;
pub mod holding;

pub use apart::{A_REASON, is_a_reason};
pub use declaring::{THE_WORKSPACE, whoever_declares_words};
pub use finding::Finding;
pub use holding::{Held, THE_EXCEPTIONS, THE_VOCABULARY, held};
