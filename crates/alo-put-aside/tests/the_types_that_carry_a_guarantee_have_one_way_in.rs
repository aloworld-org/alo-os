//! Three types here hold a promise in their shape rather than in a rule somebody remembers.
//! This is the check that the shape survives the next person editing the file.
//!
//! - [`alo_put_aside::WhereItGoesBack`] carries a patch **and** a zoom, so a caller cannot
//!   restore a window to its saved rectangle at whatever zoom the person has since wandered
//!   to.
//! - [`alo_put_aside::Shown`] is evidence that a placement was put in front of somebody, so
//!   a proposal cannot be accepted without having been shown.
//! - [`alo_put_aside::Placed`] carries where a window went **and** where it was, because the
//!   original position is what History is owed and neither road may lose it.
//!
//! # Why a check and not a comment
//!
//! Because the guarantee is held by there being **one constructor, taking everything**. Two
//! edits inside any of those files would break it silently and pass every other test in this
//! crate:
//!
//! 1. `#[derive(Default)]`, which compiles today. `Zoom`'s own `Default` is `LIFE_SIZE`
//!    (`alo-canvas/src/camera.rs`), so a defaulted `WhereItGoesBack` is not an obviously
//!    broken value — it is a plausible one, silently wrong, exactly the number every fixture
//!    in this crate avoids on purpose.
//! 2. A second constructor taking less: `of_just_a_patch`, `at`, `new`. Convenient, and it
//!    makes the omission reachable.
//!
//! The question *does anything stop a second constructor being added later* was asked by
//! another lane about `WhereItGoesBack`, and the honest answer was: for the rest of the crate
//! yes, because the fields are private to the module; inside that one file, nothing. A
//! comment there would say so and be deleted by somebody tidying.
//!
//! # Which of these three tests actually does work
//!
//! Each was mutated and watched, and they did not all pass the same way.
//!
//! - **The constructor count fails** when a second way in is added. Watched: a
//!   `Shown::somehow(TheView)` appended to `shown.rs` produced *shown.rs has 2 ways to build
//!   Shown*.
//! - **The `Copy` check fails** when `Shown` derives `Copy`. Watched.
//! - **The `Default` check cannot fail today**, because `Patch` does not implement `Default`
//!   and the derive is a compile error. It is a tripwire for the day `alo-dock` gives `Patch`
//!   one — `Camera` already has a `Default`, so that is an ordinary change to make. Its own
//!   doc says this, because a check that cannot fail lends unearned credit to the checks
//!   beside it.
//!
//! Two of the first three mutations attempted here **did not compile**, so they proved nothing
//! and reported nothing, and the run looked like three passes. That is how the `Default`
//! finding surfaced at all: a mutation that produces no output is not a mutation that passed.
//!
//! # What it deliberately does not do
//!
//! It does not count constructors by parsing Rust, and it does not forbid methods that return
//! `Self` in general — a future `WhereItGoesBack::with_place` taking every field plus a Place
//! is exactly what task 1 and 2 are waiting for and must not be blocked by a harness. What it
//! forbids is the two specific silent edits above, named, so that a new constructor is a
//! deliberate act somebody writes a reason for.
#![expect(
    clippy::expect_used,
    reason = "reading this crate's own source; a missing file is this test's own mistake"
)]

use std::path::Path;

/// A file in this crate's `src`, read whole.
fn source(file: &str) -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file);
    std::fs::read_to_string(&at).expect("this crate's own source")
}

/// The derive list on the line above a named struct, as written.
fn derives_above(text: &str, declaration: &str) -> String {
    let mut derives = String::new();
    for line in text.lines() {
        if line.contains(declaration) {
            return derives;
        }
        if line.trim_start().starts_with("#[derive(") {
            derives = line.trim().to_owned();
        } else if !line.trim_start().starts_with("///") && !line.trim().is_empty() {
            derives.clear();
        }
    }
    String::new()
}

/// **None of the three may derive `Default`.**
///
/// A defaulted value is a value nobody supplied, and each of these types exists to make
/// supplying everything unavoidable.
///
/// # This check cannot fail today, and saying so is the point
///
/// Measured rather than assumed: `alo_dock::on_the_canvas::Patch` does **not** implement
/// `Default`, so adding the derive to any of these three is a compile error —
/// *the trait bound `Patch: Default` is not satisfied*. **The compiler holds this rule, not
/// this test.**
///
/// That was found by trying the edit rather than by trusting the test, and it matters because
/// the shape of the mistake has a name here. A check that cannot fail is worse than no check:
/// no check is an absence somebody might notice, and this one sits beside two that do real
/// work and **raises a reader's confidence in them**. One lane drafted a forbidden-names list
/// with an invented type in it a few hours before this file existed, for the same reason —
/// three entries read as more thorough than two.
///
/// It stays for one specific reason, and not for thoroughness. `Camera` already derives
/// `Default`, so `Patch` gaining one is an ordinary thing for the lane that owns it to do, and
/// on the day it does, every type here becomes silently defaultable — with `Zoom::default()`
/// being `LIFE_SIZE`, the wrong value would look entirely reasonable. **This is the guard for
/// that day**, and until then it is a tripwire rather than a test. Read it as one.
#[test]
fn no_type_that_carries_a_guarantee_can_be_defaulted() {
    for (file, declaration) in [
        ("where_it_goes_back.rs", "pub struct WhereItGoesBack {"),
        ("shown.rs", "pub struct Shown {"),
        ("restoring_into_a_taken_place.rs", "pub struct Placed {"),
    ] {
        let derives = derives_above(&source(file), declaration);
        assert!(
            !derives.contains("Default"),
            "{file} derives Default on {declaration}\n\n\
             That type carries a promise in its shape: it can only be built by supplying \
             everything it holds. Default makes a value nobody supplied, and the value is \
             plausible rather than obviously broken — Zoom's own Default is LIFE_SIZE, which \
             is the one zoom every fixture in this crate avoids because saving it is \
             indistinguishable from saving nothing.\n\n\
             Derives found: {derives}"
        );
    }
}

/// **Each of the three has exactly one function returning `Self`.**
///
/// Counted on the whole file rather than inside an `impl` block, which is coarse and is the
/// point: it fires on a second way in wherever somebody puts it. A new constructor is then a
/// deliberate act with a reason written beside it, which is all this check is asking for.
#[test]
fn each_of_them_has_one_constructor_and_not_two() {
    for (file, what, allowed) in [
        ("where_it_goes_back.rs", "WhereItGoesBack", 1),
        ("shown.rs", "Shown", 1),
        ("restoring_into_a_taken_place.rs", "Placed", 1),
    ] {
        let text = source(file);
        let building: Vec<&str> = text
            .lines()
            .filter(|line| {
                let code = only_the_code(line);
                code.contains("fn ") && code.contains("-> Self")
            })
            .map(str::trim)
            .collect();
        assert_eq!(
            building.len(),
            allowed,
            "{file} has {} ways to build {what}, and the guarantee holds because there is \
             one that takes everything.\n\n\
             A second constructor taking less makes the omission reachable: a \
             WhereItGoesBack without a zoom, a Shown without a view, a Placed that forgot \
             where the window used to be. None of those fail a test — they are all \
             plausible values.\n\n\
             If the new one also takes everything, say so in its doc and raise the number \
             here in the same change. Do not raise the number on its own.\n\n\
             Found: {building:#?}",
            building.len()
        );
    }
}

/// **`Shown` is not `Copy`, so one showing authorises one placement.**
///
/// It is taken by value and spent. A `Copy` token would let a caller hold a single showing
/// and place a window repeatedly, which is the same fault as an approval that is really a
/// session — and this repository has a standing rule that an approval is never a session.
#[test]
fn a_showing_cannot_be_spent_twice() {
    let derives = derives_above(&source("shown.rs"), "pub struct Shown {");
    assert!(
        !derives.contains("Copy"),
        "Shown derives Copy, so a caller can show a proposal once and place windows with \
         that evidence for ever.\n\n\
         It is consumed by value on purpose: one showing, one placement. The repository's \
         own rule is that one approval causes exactly one execution, and an approval is \
         never a session.\n\n\
         Derives found: {derives}"
    );
}

/// A line with its documentation and string literals removed, so that prose naming a
/// constructor is not counted as one.
///
/// Every file checked here explains its own single constructor in a doc comment, and two of
/// them quote the forbidden shapes by name. Without this, the check would fail on its
/// explanation and the fix somebody reached for would be to delete the explanation.
fn only_the_code(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return String::new();
    }
    let mut code = String::new();
    let mut inside_a_string = false;
    let mut letters = line.chars().peekable();
    while let Some(letter) = letters.next() {
        match letter {
            '\\' if inside_a_string => {
                letters.next();
            }
            '"' => inside_a_string = !inside_a_string,
            _ if !inside_a_string => code.push(letter),
            _ => {}
        }
    }
    code
}
