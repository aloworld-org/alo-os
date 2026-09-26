//! **There is exactly one thing in this compositor that decides where a window
//! goes, and it is not in this compositor.**
//!
//! `docs/autonomy/v0-5-the-shell-plan.md` task 16: *`crate::window_tiling`'s
//! half is gone, with a test that there is exactly one layout decider in this
//! compositor — read from the crates rather than from a list kept beside them,
//! so a second one added anywhere is a second one this check sees*.
//!
//! So this reads `src/`. A list of known deciders kept in a test file would go
//! stale the moment somebody added a decider without reading the test, which is
//! exactly the day it needs to fail.
//!
//! # What it reads, and why that is the whole road
//!
//! A window is placed in this crate by exactly one mechanism: a `Mode` is set
//! on it, `window_mode_plan` turns that `Mode` into a size and an anchor, and
//! the anchor becomes the origin its buffer is drawn at. `Mode::Normal` returns
//! a window to the geometry it already had and `Mode::Maximized` gives it the
//! whole output — neither is a layout, because neither looks at another window.
//! A **layout** decision is the one that puts this window somewhere because of
//! where another window is, and that arrives as a rectangle carried in a `Mode`
//! variant.
//!
//! Hence the four things asserted below: how many `Mode` variants carry a
//! rectangle, how many places build one, where that place got it, and that the
//! half that used to compute one is not still sitting in the crate unused. A
//! second decider has to show up in one of them — a new variant, a second
//! builder, the one builder computing the rectangle itself instead of asking,
//! or the old half coming back.
//!
//! # What it does not see, named rather than assumed
//!
//! A future road that configured a size straight onto a client without going
//! through a `Mode` would place a window without this test noticing. There is
//! no such road today — `window_mode::set_window_mode` is the only caller of
//! the configure that carries a size — and if one is ever added, this test
//! needs a fourth clause rather than a note.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// This crate's own source, wherever it is checked out.
fn source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every Rust file under `src/` that is not itself a test.
///
/// The `_tests.rs` files are how this crate keeps a module's tests beside it;
/// what a test constructs is a fixture, not a decision the compositor makes.
fn every_source_file() -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut to_walk = vec![source()];
    while let Some(directory) = to_walk.pop() {
        for entry in std::fs::read_dir(&directory).expect("this crate has a src directory") {
            let path = entry.expect("a directory entry can be read").path();
            if path.is_dir() {
                to_walk.push(path);
            } else if path.extension().is_some_and(|it| it == "rs")
                && !path
                    .file_name()
                    .is_some_and(|it| it.to_string_lossy().ends_with("_tests.rs"))
            {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// The body of `enum Mode`, as it is written.
fn the_mode_enum() -> String {
    let written = std::fs::read_to_string(source().join("window_mode.rs"))
        .expect("the window mode lives in window_mode.rs");
    let at = written
        .find("enum Mode {")
        .expect("window_mode.rs declares an enum named Mode");
    let body = &written[at..];
    let ends = body.find("\n}").expect("the Mode enum is closed");
    body[..ends].to_owned()
}

/// **Exactly one way for a window's place to be decided elsewhere.**
///
/// A `Mode` variant with no payload cannot carry a layout: it names a state
/// this crate already knows how to compute on its own. A variant that carries
/// something is carrying an answer, and there is room for exactly one answer.
#[test]
fn exactly_one_window_mode_carries_a_decided_rectangle() {
    let carrying: Vec<String> = the_mode_enum()
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with("//") && line.ends_with(','))
        .filter(|line| line.contains('('))
        .map(str::to_owned)
        .collect();

    assert_eq!(
        carrying.len(),
        1,
        "a window's place is decided in {} ways, not one: {carrying:?}",
        carrying.len()
    );
    let carried = carrying.first().expect("just counted one");
    assert!(
        carried.starts_with("InAShare(Share)"),
        "the one carried answer is no longer a share: {carried}"
    );
}

/// **Exactly one place in this crate builds that rectangle.**
///
/// Not one place *we remember* — every file under `src/` is read, so a second
/// builder added in a module nobody thought to check is a second one this sees.
///
/// What is counted is the **construction** of a share and not a match on one:
/// `window_mode` and `window_mode_plan` both take a share apart to turn it into
/// a configure, which is carrying out the decision rather than making it. The
/// declaration is not a construction either, so the file that declares the type
/// is not counted for declaring it.
#[test]
fn exactly_one_file_puts_a_window_in_a_share() {
    let builders: Vec<PathBuf> = every_source_file()
        .into_iter()
        .filter(|path| {
            std::fs::read_to_string(path)
                .expect("a source file this crate compiles can be read")
                .lines()
                .map(str::trim)
                .any(|line| line.contains("Share {") && !line.contains("struct Share {"))
        })
        .collect();

    assert_eq!(
        builders.len(),
        1,
        "{} files decide a window's share, and the plan's constraint allows one: {builders:?}",
        builders.len()
    );
    let builder = builders.first().expect("just counted one");
    assert!(
        builder.ends_with("window_dividing.rs"),
        "the share is now decided somewhere else: {builder:?}"
    );
}

/// **That one place asks rather than computes.**
///
/// This is what the removal was for. `window_tiling` took a side and worked out
/// half an output; what stands there now takes the rectangle `alo-dividing`
/// already decided and converts its units. A builder that did arithmetic on an
/// output's width would be the half coming back under another name, so what is
/// asserted is that the rectangle's numbers come out of a division's shares.
#[test]
fn the_share_comes_from_the_division_rather_than_from_arithmetic() {
    let written = std::fs::read_to_string(source().join("window_dividing.rs"))
        .expect("the one builder is window_dividing.rs");

    assert!(
        written.contains("division\n            .shares()") || written.contains(".shares()"),
        "the share is no longer read from alo_dividing::Division::shares"
    );
    assert!(
        written.contains("divide_with_next"),
        "the split is no longer alo_dividing::Division::divide_with_next's answer"
    );
    for arithmetic in ["/ 2", "/2", "* 2", ">> 1"] {
        assert!(
            !written.contains(arithmetic),
            "the one builder divides an output itself ({arithmetic}), which is the half returning"
        );
    }
}

/// **And the half is gone from the crate, not merely unused.**
///
/// A module left behind compiles, and a module that compiles is one a later
/// change can call. Read from `src/` rather than from the exports, because a
/// private second decider is the same fault as a public one.
#[test]
fn nothing_under_src_is_still_a_half_of_an_output() {
    let left: Vec<PathBuf> = every_source_file()
        .into_iter()
        .filter(|path| {
            let written = std::fs::read_to_string(path)
                .expect("a source file this crate compiles can be read");
            // In code, not in the sentences that record where the half went:
            // every line of prose in this crate begins a comment.
            written.lines().map(str::trim).any(|line| {
                !line.starts_with("//")
                    && (line.contains("TileSide")
                        || line.contains("window_tiling")
                        || line.contains("Mode::Tiled"))
            })
        })
        .collect();

    assert!(left.is_empty(), "the half is still reachable from {left:?}");
}
