//! The recipes are built on a base alo keeps a copy of, and the workflow that
//! makes the copy does only that.
//!
//! [ADR 0154](../../../docs/decisions/0154-the-base-is-a-copy-alo-keeps.md),
//! the owner's decision of 2026-11-13: *"alo's GitHub registry"*. On that day
//! the digest ADR 0152 pinned on Quay was deleted by Quay within hours, and
//! every image build from main failed at its first line. So:
//!
//! - both Containerfiles name a base `image/bases.toml` records, with where it
//!   came from;
//! - `copying-a-base.yml` copies only a recorded base, keeps its digest, reads
//!   it back and makes sure anybody can pull it, and never builds, pushes an
//!   image or signs;
//! - every workflow that builds a recipe asks whether its base can be pulled
//!   before it builds, and says so in a sentence when it cannot.
//!
//! # What this cannot see
//!
//! **Whether the copy exists.** That is a question for the registry, and this
//! test does not reach the network; the build workflows' own first step asks
//! it, every time they run. A green run here says the recipes name a recorded
//! base, not that anybody copied it.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_image::{THE_BASES, THE_IMAGE, TheBases, TheWorkflow};

/// The two recipes that are built on the base.
const THE_RECIPES: [&str; 2] = ["Containerfile", "installing/Containerfile"];

/// The workflow that copies a base.
const THE_COPYING: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../.github/workflows/copying-a-base.yml"
);

/// Every workflow that builds one of the recipes.
const THE_BUILDS: [&str; 3] = [
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../.github/workflows/image.yml"
    ),
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../.github/workflows/installer-candidate.yml"
    ),
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../.github/workflows/release.yml"
    ),
];

/// What a recipe calls its base.
const THE_ARGUMENT: &str = "ARG THE_BASE=";

/// A file this test reads, as text.
fn text(at: &Path) -> String {
    std::fs::read_to_string(at).unwrap_or_else(|why| panic!("{} did not read: {why}", at.display()))
}

/// The record, read.
fn the_bases() -> TheBases {
    let at = Path::new(THE_IMAGE).join(THE_BASES);
    TheBases::read(&text(&at)).unwrap_or_else(|why| panic!("{}: {why}", at.display()))
}

/// The base a recipe names.
fn the_base_of(recipe: &str) -> String {
    let at = Path::new(THE_IMAGE).join(recipe);
    let named: Vec<String> = text(&at)
        .lines()
        .filter_map(|line| line.trim().strip_prefix(THE_ARGUMENT))
        .map(|base| base.trim().to_owned())
        .collect();
    let [base] = named.as_slice() else {
        panic!("{recipe} names {} bases; a recipe names one", named.len());
    };
    base.clone()
}

/// Whether the record answers for this base: alo's copy, by digest, and
/// nothing else — not the maker's original, which can vanish.
fn is_recorded(bases: &TheBases, base: &str) -> bool {
    bases.copied_as(base).is_some()
}

/// Where a line first appears among what a workflow runs.
fn first(workflow: &TheWorkflow, what: &str) -> Option<usize> {
    workflow.live().iter().position(|line| line.contains(what))
}

/// **Both recipes are built on a base the record answers for.**
#[test]
fn every_recipe_is_built_on_a_recorded_base() {
    let bases = the_bases();
    for recipe in THE_RECIPES {
        let base = the_base_of(recipe);
        assert!(
            is_recorded(&bases, &base),
            "{recipe} is built on {base}, which image/bases.toml does not record. ADR 0154: \
             record the base there, run the Copying a base workflow with its digest, and name \
             the copy"
        );
    }
}

/// **A base the record does not answer for is caught**: another digest, the
/// copy named by its tag, which somebody can move, or the maker's original
/// even at the recorded digest, which is what stopped every build on
/// 2026-11-13.
#[test]
fn a_base_nobody_recorded_is_caught() {
    let bases = the_bases();
    let Some(base) = bases.bases().first() else {
        panic!("image/bases.toml records no base");
    };
    let moved = base.reference().replacen("sha256:", "sha256:0", 1);
    let by_tag = format!("{}:{}", base.copy(), base.tag());
    let unpinned = base.upstream().to_owned();
    let the_original = base.origin();
    for wrong in [moved, by_tag, unpinned, the_original] {
        assert!(!is_recorded(&bases, &wrong), "{wrong} read as recorded");
    }
}

/// **The copying workflow copies only a recorded base, keeps its digest and
/// checks it twice**, in that order.
#[test]
fn the_copy_keeps_the_makers_digest() {
    let workflow = TheWorkflow::read(&text(Path::new(THE_COPYING)));
    let found = first(&workflow, "image/bases.toml");
    let read = first(&workflow, "docker://$UPSTREAM@$DIGEST\" > upstream.json");
    let copied = first(&workflow, "skopeo copy --all --preserve-digests");
    let read_back = first(&workflow, "docker://$COPY@$DIGEST\" > copied.json");
    let as_anybody = first(&workflow, "--no-creds");
    assert!(
        matches!((found, read, copied, read_back, as_anybody),
            (Some(a), Some(b), Some(c), Some(d), Some(e)) if a < b && b < c && c < d && d < e),
        "copying-a-base.yml must find the base in the record, read the original by digest, copy \
         every platform keeping the digest, read the copy back, then read it as anybody would: \
         {found:?} {read:?} {copied:?} {read_back:?} {as_anybody:?}"
    );
    assert!(
        workflow.runs_something_naming("sha256sum"),
        "copying-a-base.yml must hash what it read, not trust what a registry says it is"
    );
    assert!(
        workflow.runs_something_naming("ghcr.io/aloworld-org/?*)"),
        "copying-a-base.yml must refuse a copy outside ghcr.io/aloworld-org"
    );
}

/// **The copying workflow runs only when asked, only from main, and never
/// builds, pushes an image or signs.**
#[test]
fn the_copying_workflow_does_nothing_else() {
    let workflow = TheWorkflow::read(&text(Path::new(THE_COPYING)));
    assert!(workflow.runs_only_when_asked(), "{:?}", workflow.triggers());
    assert!(workflow.builds_only_from_main());
    assert!(!workflow.signs(), "copying a base is not signing it");
    for never in ["podman build", "podman push", "gh release"] {
        assert!(
            !workflow.runs_something_naming(never),
            "copying-a-base.yml runs `{never}`, and it only copies"
        );
    }
}

/// **Every build asks whether its base can be pulled, before it builds**, and
/// asks without credentials, as a lane machine would.
#[test]
fn every_build_asks_for_its_base_first() {
    for at in THE_BUILDS {
        let workflow = TheWorkflow::read(&text(Path::new(at)));
        let asked = first(
            &workflow,
            "skopeo inspect --raw --no-creds \"docker://$base\"",
        );
        let read = first(&workflow, THE_ARGUMENT);
        let built = first(&workflow, "podman build");
        assert!(
            matches!((read, asked, built), (Some(r), Some(a), Some(b)) if r < a && a < b),
            "{at} must read the recipe's base and ask whether it can be pulled before it builds: \
             {read:?} {asked:?} {built:?}"
        );
    }
}
