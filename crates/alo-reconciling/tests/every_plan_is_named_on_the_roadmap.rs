//! **Every plan with tasks is named on the roadmap, checked rather than hoped.**
//!
//! Before 2026-10-03 the whole of `ROADMAP.md`'s current release cited **one**
//! plan. Lanes are assigned to plans by the charter and the roadmap is organised
//! by phases, so a reader asking *where is the canvas* found nothing: the canvas
//! is a plan, and plans were not on the page. Each phase now carries a table of
//! the plans that close it, and this is what keeps that true when a plan is
//! added.
//!
//! # Presence, not phase
//!
//! This checks that a plan's filename appears on the page. It does not check
//! **which** phase it was placed under, and deliberately so: the phase
//! assignment is a judgement with no recorded source — `docs/autonomy/DELIVERY.md`
//! names no plan, and only two plans name a phase — so a test asserting it would
//! turn one person's reading into something the build enforces. The owner can
//! move any row in one line and nothing here objects.
//!
//! The same shape as `every_plan_with_tasks_has_an_owning_machine.rs`, which
//! checks a plan has a charter row and never reads the machine out of the cell,
//! and for the same reason: hold what was actually missing, not what would be
//! satisfying to assert.
//!
//! # Its own discriminator
//!
//! `## Tasks`, read from each document, rather than a filename pattern. The
//! helper in `the_gate_is_held_to_itself.rs` selects on `name.contains("plan")`
//! and misses four of the twenty-five, all four of them the canvas and panel
//! plans — which are precisely the ones this page was failing to name.

#![expect(
    clippy::expect_used,
    reason = "a document this test cannot read is a broken checkout, and saying so \
              by name is the failure being reported"
)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

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

/// **Every plan with tasks is named somewhere on the roadmap.**
#[test]
fn every_plan_with_tasks_is_named_on_the_roadmap() {
    let roadmap =
        fs::read_to_string(the_repository().join("ROADMAP.md")).expect("ROADMAP.md is readable");
    let plans = the_plans_with_tasks();

    let unnamed: Vec<&str> = plans
        .keys()
        .filter(|name| !roadmap.contains(name.as_str()))
        .map(String::as_str)
        .collect();

    assert!(
        unnamed.is_empty(),
        "these plans carry tasks and are named nowhere in ROADMAP.md, so nobody \
         reading it can find where the work happens ({} of {}):{}",
        unnamed.len(),
        plans.len(),
        unnamed
            .iter()
            .map(|name| format!("\n  {name}"))
            .collect::<String>()
    );
}

/// **The discriminator still finds the plans.**
///
/// Without this the check above passes by finding nothing: a renamed heading or
/// a moved directory would make every plan invisible and the roadmap trivially
/// complete. Twenty-five had tasks when this was written.
#[test]
fn the_discriminator_still_finds_the_plans() {
    let found = the_plans_with_tasks().len();
    assert!(
        found > 20,
        "only {found} documents under docs/autonomy have a `{}` section, which \
         is not this repository — the discriminator broke, not the roadmap",
        TASKS.trim()
    );
}
