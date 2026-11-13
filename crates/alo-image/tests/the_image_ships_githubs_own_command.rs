//! **GitHub's own command, `gh`, is on the image and is started by the build**
//! — the floor under Demo 2's second road, where a person signs in to GitHub
//! once and alo publishes a folder to their GitHub Pages site (owner,
//! 2026-11-13: *ship GitHub's own `gh` on the image*).
//!
//! It reads `image/Containerfile` and nothing else, so it runs on every build
//! and needs no network — the division `the_image_has_what_a_developer_needs`
//! draws: the build proves the binary starts, this proves the build still asks.
//!
//! # What this cannot prove
//!
//! That a person can sign in, that the sign-in is kept in the keyring, or that
//! a push reaches GitHub. Each needs a booted machine and a network; none of it
//! is checked here and nothing here pretends to be.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "an unreadable recipe is the failure this test reports, and a formatted panic names \
              the path it looked at"
)]

/// The recipe, as one string.
fn the_recipe() -> String {
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("image/Containerfile");
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("the recipe is not readable at {}: {why}", at.display()))
}

/// A recipe's lines with its comments taken out, so a command named in prose
/// is never read as a command run.
fn what_a_shell_would_run(recipe: &str) -> String {
    recipe
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// What the build echoes once `gh` has started, which names the block.
const THE_CLAIM: &str = "a person can sign in to GitHub and publish with GitHub's own command";

/// The `RUN` block that installs `gh`, comments removed.
fn the_block() -> String {
    let recipe = the_recipe();
    let block = recipe
        .split("RUN dnf install")
        .find(|block| block.contains(THE_CLAIM))
        .expect("the recipe still has the block that installs GitHub's own command");
    // **Up to its own claim and no further**, so what the next blocks run is
    // not read as this one's.
    let ends = block
        .find(THE_CLAIM)
        .map_or(block.len(), |at| at + THE_CLAIM.len());
    what_a_shell_would_run(block.get(..ends).unwrap_or(block))
}

/// **It is installed from the base's own repository**, which the base image's
/// digest pins, rather than fetched from somewhere this recipe holds no hash
/// for.
#[test]
fn the_recipe_installs_gh_from_the_pinned_base() {
    let block = the_block();
    assert!(
        block
            .lines()
            .next()
            .is_some_and(|first| first.split_whitespace().any(|word| word == "gh")),
        "the block no longer asks dnf for `gh`: {block}"
    );
    assert!(
        !block.contains("curl") && !block.contains("http"),
        "`gh` is fetched from the network by this block, with no hash the recipe holds: {block}"
    );
    assert!(
        the_recipe().lines().any(|line| line
            .strip_prefix("ARG THE_BASE=quay.io/fedora/fedora-bootc:")
            .is_some_and(|pinned| pinned.contains("@sha256:"))),
        "the base is no longer pinned by digest, so the `gh` it resolves is not pinned either"
    );
}

/// **Started, not stat-ed**, so a `gh` that cannot load fails the build.
#[test]
fn gh_is_started_by_the_build_rather_than_stat_ed() {
    let block = the_block();
    assert!(
        block.contains("gh --version"),
        "the build never runs `gh --version`, so a `gh` that dies on a missing library would \
         ship: {block}"
    );
    assert!(
        !block.contains("test -x"),
        "the block stats `gh` instead of starting it: {block}"
    );
}

/// **The comment-stripping does not hide a real command**, so the two checks
/// above can fail.
#[test]
fn a_command_named_only_in_a_comment_is_not_read_as_run() {
    let only_prose = "# gh --version is what this comment is about\nRUN dnf install gh\n";
    assert!(!what_a_shell_would_run(only_prose).contains("gh --version"));
    let run = "# about it\nRUN dnf install gh \\\n && gh --version\n";
    assert!(what_a_shell_would_run(run).contains("gh --version"));
}
