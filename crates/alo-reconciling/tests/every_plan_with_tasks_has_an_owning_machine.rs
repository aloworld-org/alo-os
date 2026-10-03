//! **Every plan with tasks has an owning machine, checked rather than read.**
//!
//! Three lanes found the same gap independently on 2026-10-03: twenty-five
//! documents under `docs/autonomy` have a `## Tasks` section, fifteen had a row
//! in `a-new-machine-becomes-a-lane.md`, and **ten did not** — including every
//! plan all three lanes were working in. A plan with tasks and no owning machine
//! is work nobody is accountable for, and it was found by three separate lanes
//! noticing rather than by anything failing.
//!
//! This is the thing that fails instead. It would have gone red at the tenth
//! unowned plan rather than at three lanes comparing notes.
//!
//! # Presence, not structure
//!
//! This test checks that a plan's **filename appears in the charter**. It does
//! not parse the table, extract the cell, or read the machine out of it. Three
//! reasons, each of them something the repository already knew:
//!
//! 1. `tools/kernel-loop/src/who_owns.rs` deliberately does not parse this
//!    table, and its header says why: the table is written for people, and a
//!    parser of it breaks on the next rewording.
//! 2. That prediction has already come true. The charter now holds a **second**
//!    table whose first column is a crate rather than a plan, so a
//!    row-indiscriminate parser reads crate rows where it expects plans.
//! 3. `the-executable-plan.md` **deliberately names no single machine.** Its
//!    tasks carry their own owners in their own lines, so any one machine in
//!    that cell would be wrong for most of its tasks. A test that demanded a
//!    machine per plan would force a false answer into the one plan whose
//!    correct answer is *per task*.
//!
//! So the plain assertion is the correct one: presence is what was missing and
//! presence is what is held. The section below shows that the alternative which
//! looks stronger — reading the plan's own ownership field — is measurably
//! weaker, so this is not a weaker check chosen for convenience.
//!
//! # Why not the field check, which looks stronger and is strictly weaker
//!
//! `who_owns.rs` reads `**Crates this plan owns`, a machine-readable field in
//! the plan itself, and a check that every plan declares it would need no prose
//! at all. Counted on 2026-10-03, against the ten plans that had no row:
//!
//! | | |
//! |---|---|
//! | plans with `## Tasks` | 25 |
//! | had no charter row | 10 |
//! | lack the ownership field | 8 |
//! | in both | 8 |
//! | lack the field but had a row | 0 |
//!
//! **The field gap is a strict subset of the row gap.** Not a different gap and
//! not an overlapping one — eight, all of them inside the ten, nothing outside.
//! So the field check is the weaker of the two, and it is also not assertable
//! today: turned on now it fails at eight.
//!
//! **And the two it misses are the two hardest cases.**
//! `the-machine-measured-plan.md` and `the-models-measured-plan.md` both carry
//! the field and had no row, so a field check would have passed them — and those
//! are exactly the two rows that had to be decided from outside the plan, one
//! from another document naming which plan a Mac lane runs and one written as
//! thin because nothing in the plan states ownership at all. A check that is
//! silent precisely where the evidence is weakest is worse than its pass rate
//! suggests.
//!
//! The field is still worth requiring, as its own work and for its own reason —
//! it says which crates, which no charter row does. This check neither blocks it
//! nor needs removing when it arrives.
//!
//! # Two cautions for whoever writes the field gate, both of them mistakes made
//! here first
//!
//! **The label is a prefix, not a literal.** `who_owns.rs` matches
//! `**Crates this plan owns` and the line above the constant says why: seven
//! plans write `**Crates this plan owns, all new:**`. Counting the full `:**`
//! form gives fifteen rather than eight, and the fifteen is what this section
//! shipped in its first version.
//!
//! **A crate name is written two ways, and a gate must take both.** Counted on
//! 2026-10-03, each number labelled with what it counted, because they answer
//! different questions:
//!
//! | | bare `alo-…` | `crates/alo-…` |
//! |---|---|---|
//! | inside the paragraphs `owned()` reads | 163 | 78 |
//! | across every document in `docs/autonomy` | 2465 | 538 |
//!
//! The first is what a field gate needs. The second says the ratio holds at
//! corpus scale, so the two spellings are **house style rather than a few
//! authors' habits** — a gate allowing one would be wrong about most of the
//! repository rather than a corner of it. `owned()` already handles both, with
//! `name.strip_prefix("crates/")`.
//!
//! Both of these are the same fault in the same direction: **matching the form
//! I had just seen instead of asking which forms exist.** In each case the
//! variation was written down adjacent to what I matched on — one line above the
//! constant, eleven lines below it, and in the file this test declined to copy
//! its filter from. The query is cheap exactly when you are already reading the
//! definition.
//!
//! # Its own discriminator
//!
//! `## Tasks`, and not a filename pattern. `the_gate_is_held_to_itself.rs`
//! selects plans with `name.contains("plan")`, which misses four of the
//! twenty-five — `putting-a-window-aside.md`, `the-canvas-and-its-places.md`,
//! `the-smallest-canvas-worth-showing.md` and
//! `applications-people-already-use.md`. Those four are exactly the plans the
//! canvas and panel lanes have been working in, so the convenient filter would
//! have excluded the plans most in need of an owner. A filter is a claim about a
//! category; this one asks the document what it contains.

#![expect(
    clippy::expect_used,
    reason = "a document this test cannot read is a broken checkout, and saying so \
              by name is the failure being reported"
)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Where the charter lives, relative to the repository.
const THE_CHARTER: &str = "docs/autonomy/a-new-machine-becomes-a-lane.md";

/// What makes a document a plan with work in it.
const TASKS: &str = "\n## Tasks\n";

/// This repository.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is two directories above this crate")
}

/// Every document at the top of `docs/autonomy` that has a `## Tasks` section.
///
/// The top level only, because that is where plans live; `updates/` below it
/// holds reports, which are a record of work rather than a claim on it.
fn the_plans_with_tasks() -> BTreeMap<String, String> {
    let at = the_repository().join("docs/autonomy");
    let mut plans = BTreeMap::new();
    for entry in fs::read_dir(&at)
        .expect("docs/autonomy is readable")
        .flatten()
    {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.ends_with(".md") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if text.contains(TASKS) {
            plans.insert(name.to_owned(), text);
        }
    }
    plans
}

/// **Every plan with tasks is named in the charter.**
///
/// The failure names the plans, because the repair is per plan: somebody decides
/// which machine owns it and writes the row. A count would send the next reader
/// back to the measurement this test exists to replace.
#[test]
fn every_plan_with_tasks_is_named_in_the_charter() {
    let charter =
        fs::read_to_string(the_repository().join(THE_CHARTER)).expect("the charter is readable");
    let plans = the_plans_with_tasks();

    let unowned: Vec<&str> = plans
        .keys()
        .filter(|name| !charter.contains(name.as_str()))
        .map(String::as_str)
        .collect();

    assert!(
        unowned.is_empty(),
        "these have tasks and no row in {THE_CHARTER}, so nobody owns the work \
         in them ({} of {}):{}",
        unowned.len(),
        plans.len(),
        unowned
            .iter()
            .map(|name| format!("\n  {name}"))
            .collect::<String>()
    );
}

/// **The discriminator still finds the plans.**
///
/// Without this, the check above passes by finding nothing: a renamed heading, a
/// moved directory or a changed filter would make every plan invisible and the
/// charter trivially complete. Twenty-five had tasks when this was written and
/// the floor is deliberately well below that, so the test reports a discriminator
/// that broke rather than a repository that grew.
#[test]
fn the_discriminator_still_finds_the_plans() {
    let found = the_plans_with_tasks().len();
    assert!(
        found > 20,
        "only {found} documents under docs/autonomy have a `{}` section, which \
         is not this repository — the discriminator broke, not the charter",
        TASKS.trim()
    );
}
