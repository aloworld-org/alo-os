//! Every `pub fn` of this workspace, held to having a caller or a test.
//!
//! **The fault this exists to catch has been found seven times**, in seven sets
//! of clothes, and `docs/autonomy/what-nothing-calls.md` carries them all: a
//! mechanism built, tested, gated green, and reachable by nobody. The sweep that
//! found them ran **once, by hand, on 2026-10-20**, and its own page proposed
//! this gate and then declined to write it. The reason it gave was that an
//! accepted list is only honest if the test refuses *growth* of the list rather
//! than merely tolerating its contents.
//!
//! So that is what this does. The list is a committed snapshot, and the test
//! fails two ways: a `pub fn` nothing reaches that is **not** in the list, and a
//! name in the list that **is** now reached. The second is what makes the list
//! shrink and never grow by accident.
//!
//! # What it cannot see, said here rather than discovered
//!
//! **A caller count of one and a caller count of nine are the same answer to
//! *is this reached*.** The fault that shipped a blank install screen on
//! 2026-10-21 was a mechanism with one correct caller and a second surface that
//! should have used it, and **no reference count will ever find that** — the
//! test passes on the first caller. `docs/autonomy/what-nothing-calls.md`'s last
//! section says so at length. This gate catches *nothing calls it*. It does not
//! catch *not everything that should, does*, and nothing here should be read as
//! though it did.
//!
//! # The holes, carried from the page rather than rediscovered
//!
//! Each of these was paid for once already:
//!
//! | | |
//! |---|---|
//! | a reference from inside dead code is not a caller | so reachability is a **fixed point**: drop what nothing reaches, recount, repeat |
//! | a `pub use` is a declaration, not a call | `use` lines are stripped |
//! | a doc comment naming a function is prose | comments are stripped |
//! | a test file in `src/` is a test | `*_tests.rs` counts as test usage, not as a caller |
//! | a `#[cfg(test)]` block **inside** a product file is a test | its text is moved to the test side, **not discarded** |
//! | the slow version was `O(functions × files)` and did not finish | one tokenising pass, a count per file, built once |
//!
//! **The fifth of those is this crate's own**, added 2026-10-21 after a rewrite
//! of the hand sweep falsely accused three functions in `alo-access`: it
//! stripped `#[cfg(test)]` blocks out of the product text, which is right, and
//! then threw the text away instead of counting it as a test. All seven of their
//! callers were in their own files' test blocks, so they were counted nowhere
//! and came out as *no product caller **and** no test* — the exact conjunction
//! that defines an orphan. Hole four's fix applied once too often.
//!
//! # Why the answer to *does a test count* is **yes**
//!
//! A `pub fn` reached only by its own tests is not a mechanism a person can use,
//! and on a stricter reading it should be reported. It is not, for one reason: a
//! gate that **falsely accuses** gets switched off, and an `impl` block whose
//! methods are exercised by the module's own tests is how most of this
//! workspace is written. A check nobody trusts protects nothing.

#![forbid(unsafe_code)]

mod accepted;
mod finding;
mod reaching;
mod the_code;

pub use accepted::{Accepted, THE_LIST};
pub use finding::{Finding, the_line_for};
pub use reaching::{APubFn, what_nothing_reaches};
pub use the_code::{THE_WORKSPACE, a_crates_files};
