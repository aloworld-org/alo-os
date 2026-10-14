//! **A developer who opens the terminal can clone and build** — the five
//! commands that claim is about, and the check that each one starts.
//!
//! It reads `image/Containerfile` and nothing else, so it runs on any machine
//! and needs no network — which is why it can be in the gate at all.
//!
//! # What this proves and what it cannot
//!
//! It proves **the recipe still asks for them, and still starts each one.** It
//! cannot tell you a developer really cloned anything: that needs a booted
//! machine, and `docs/booting.md` is about what a virtual machine can never
//! show. The division is the one
//! `the_image_can_play_what_the_decision_says.rs` draws and for the same
//! reason — the build proves the binaries, this proves the build still asks.
//!
//! # Why the commands and not the packages
//!
//! `gcc-c++` and `pkgconf-pkg-config` are what `dnf` is told; `g++` and
//! `pkg-config` are what a person types. **Only the second pair is a promise
//! to anybody**, and before 2026-10-14 neither appeared in the check at all —
//! the block stat-ed three binaries with `test -x` and started one. A package
//! renamed upstream, or split, would have taken the command with it and
//! nothing here would have noticed.
//!
//! So this file asserts the **commands** are started, and asserts the packages
//! only as the line that installs them. If a future recipe gets `g++` from
//! somewhere else, this test keeps passing — which is right, because what is
//! promised is the command.
//!
//! # Why `--version` rather than `test -x`
//!
//! `CLAUDE.md`'s first named fault: *`test -x` on a converter binary stood in
//! for the converter **starting**. Three releases shipped one that died at
//! launch on eleven missing libraries.* An executable bit says a file is there.
//! `--version` loads the binary and its shared libraries and makes it answer,
//! which is the cheapest check that is about the thing rather than about a
//! stand-in for it.

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

/// The commands a developer types, every one of which is a promise.
///
/// **`g++` and `pkg-config` are here although `dnf` is told `gcc-c++` and
/// `pkgconf-pkg-config`**, which is the point of the list: what is promised is
/// the command.
const WHAT_A_DEVELOPER_TYPES: [&str; 5] = ["git", "gcc", "g++", "make", "pkg-config"];

#[test]
fn the_recipe_installs_a_toolchain_and_version_control() {
    let recipe = the_recipe();
    for package in ["git", "gcc", "gcc-c++", "make", "pkgconf-pkg-config"] {
        assert!(
            recipe.contains(package),
            "the recipe no longer installs `{package}`, so a developer who opens the terminal \
             cannot clone and build"
        );
    }
}

/// **Each command is started, so a binary that cannot load is caught.**
///
/// The assertion is on `"<command> --version"` rather than on the command alone,
/// because the command alone appears in this file's own prose and in the
/// recipe's comments. A check that matched a comment would pass after somebody
/// deleted the line it is about — which is this repository's own recurring
/// fault: *a check that stands in for the thing is not the thing.*
#[test]
fn every_command_it_promises_is_started_rather_than_stat_ed() {
    let recipe = the_recipe();
    for command in WHAT_A_DEVELOPER_TYPES {
        let started = format!("{command} --version");
        assert!(
            recipe.contains(&started),
            "the recipe never runs `{started}`, so nothing checks that `{command}` starts — an \
             executable that dies on a missing library would ship"
        );
    }
}

/// **`test -x` is gone from this block and must not come back.**
///
/// Asserted by name because the weaker check is the one somebody reaches for:
/// it is shorter, it looks equivalent, and it passes on a binary that cannot
/// run. Scoped to the toolchain's own `RUN` block rather than the whole recipe,
/// so other blocks that legitimately stat a file are untouched.
#[test]
fn the_toolchain_block_does_not_stand_in_an_executable_bit_for_a_run() {
    let recipe = the_recipe();
    let block = recipe
        .split("RUN dnf install")
        .find(|block| block.contains("a developer who opens the terminal can clone and build"))
        .expect("the recipe still has the block that installs a developer's toolchain");
    assert!(
        !block.contains("test -x"),
        "the toolchain block stats a binary again instead of starting it: {block}"
    );
}

/// **The claim the build echoes is the claim this file is named after**, so the
/// two cannot drift apart silently.
#[test]
fn the_build_says_what_it_proved() {
    assert!(
        the_recipe().contains("a developer who opens the terminal can clone and build"),
        "the recipe no longer says what this block proves, so a reader of the build log has \
         nothing to match against this test"
    );
}
