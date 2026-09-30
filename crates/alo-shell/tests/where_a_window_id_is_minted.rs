//! Where a window's model identity is minted, and that it is minted once.
//!
//! **The Mac lane's rule, and this is the half that lives here.** A crate that
//! mints ids can give two different windows the same one, or one window two;
//! a crate that can only hold an id it was handed cannot. So `alo-put-aside`
//! never calls `WindowId::numbered` outside its own tests — the Panel lane
//! holds that where it can break it — and `alo-shell`, which is the only place
//! a compositor surface becomes a model window, mints in **exactly one named
//! conversion**.
//!
//! That conversion is `putting_a_window_aside::as_a_model_window`, and the
//! identity it uses is `window_number`'s: a `u64` assigned the first time a
//! surface is seen, unique by construction and never reused. `alo_dividing`
//! already bridges to its own id the same way, so **a window put aside and a
//! window a division moved are the same window** rather than two ids that agree
//! by luck.
//!
//! # Why a source check rather than a type
//!
//! A private constructor would be better and is not available: `WindowId` is
//! another crate's public type and `numbered` is its public constructor,
//! reachable from anywhere that depends on `alo-dock`. What can be held is that
//! this crate reaches for it once, in a place with a name, and a second reach
//! is a failing build rather than a thing somebody notices in review.
//!
//! **It is true today by accident and this makes it true on purpose**, which is
//! the whole difference — the conversion exists because the minimise gesture
//! needed it, not because anybody had decided there should be one.
#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// This crate's source directory.
fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every non-test source file in this crate: its name and its lines.
///
/// `*_tests.rs` and `*_testing.rs` are tests, and every `#[cfg(test)]` module
/// in this crate is the last thing in its file — which is what lets a line
/// count stand in for a parse.
fn the_shipped_files() -> Vec<(String, String)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(src()).unwrap() {
        let at = entry.unwrap().path();
        if at.is_dir() {
            continue;
        }
        let named = at.file_name().unwrap().to_string_lossy().into_owned();
        if !named.ends_with(".rs") || named.ends_with("_tests.rs") || named.ends_with("_testing.rs")
        {
            continue;
        }
        let whole = std::fs::read_to_string(&at).unwrap();
        let shipped = whole
            .split_once("#[cfg(test)]")
            .map_or(whole.clone(), |(above, _)| above.to_owned());
        read.push((named, shipped));
    }
    read.sort();
    read
}

/// **This crate mints a window's model identity in exactly one place.**
///
/// Counted over shipped code only, with comments left in on purpose: a mention
/// inside a comment is a mention somebody can copy, and the point is that there
/// is one place to copy from.
#[test]
fn a_window_id_is_minted_in_exactly_one_named_conversion() {
    let files = the_shipped_files();
    assert!(
        files.len() > 100,
        "only {} shipped files were read, so this test is checking almost nothing",
        files.len()
    );

    let mut minting: Vec<(String, usize)> = Vec::new();
    for (named, shipped) in &files {
        let times = shipped.matches("WindowId::numbered").count();
        if times > 0 {
            minting.push((named.clone(), times));
        }
    }

    assert_eq!(
        minting,
        vec![("putting_a_window_aside.rs".to_owned(), 2)],
        "a window id is minted somewhere other than the one named conversion, \
         or that conversion has changed shape: {minting:?}"
    );
}

/// **And the conversion is where it says it is**, so the count above cannot be
/// satisfied by two mentions in a file that no longer converts anything.
///
/// Without this, deleting `as_a_model_window` and leaving two mentions in a
/// comment would pass — which is the shape of check that keeps passing after
/// the thing it was about has gone.
#[test]
fn the_one_place_is_the_conversion_it_is_supposed_to_be() {
    let at = src().join("putting_a_window_aside.rs");
    let shipped = std::fs::read_to_string(&at).unwrap();
    let above = shipped
        .split_once("#[cfg(test)]")
        .map_or(shipped.clone(), |(above, _)| above.to_owned());

    assert!(
        above.contains("fn as_a_model_window"),
        "the named conversion is gone, so the rule above is about nothing"
    );
    let inside = above
        .split_once("fn as_a_model_window")
        .map(|(_, after)| after)
        .unwrap_or_default();
    assert!(
        inside.contains("WindowId::numbered"),
        "the conversion no longer mints an id, so the mint moved somewhere \
         the count above did not look"
    );
}

/// **The identity is the compositor's own number, not a fresh count.**
///
/// A `WindowId` minted from anything else — a counter of this file's own, an
/// index into a list, a hash of a title — would be an identity that agrees with
/// `alo_dividing`'s only by luck, and the two crates would disagree about which
/// window was which the first time a list was reordered.
#[test]
fn the_number_it_mints_from_is_the_compositors_own() {
    let at = src().join("putting_a_window_aside.rs");
    let shipped = std::fs::read_to_string(&at).unwrap();
    for line in shipped.lines() {
        if line.contains("WindowId::numbered") && !line.trim_start().starts_with("//") {
            assert!(
                line.contains("window_number::Numbers::of"),
                "an id is minted from something other than the surface's own \
                 number: {}",
                line.trim()
            );
        }
    }
}
