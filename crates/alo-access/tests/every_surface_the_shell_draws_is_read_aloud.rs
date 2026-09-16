//! **A surface the shell draws with nothing to say about it fails here.**
//!
//! Task 2 of `docs/autonomy/v0-5-access-and-language-plan.md` asks that the
//! roles be *read from the shell's own list of surfaces rather than retyped*.
//! There is no such list in that crate today — what it has is one frame or
//! screen per surface, exported from its `lib.rs` — so this reads those exports
//! as text and holds [`alo_access::Surface`] to them.
//!
//! **Read as text, and never as a dependency.** `alo-access` decides what a
//! reader is told and the shell draws it; a crate that decided the roles *and*
//! imported the drawing would have the arrow pointing both ways, and the access
//! plan says nothing of this lane's belongs in `crates/alo-shell`. So this test
//! opens that crate's source and reads it, the way `alo-instructing` reads its
//! own manifest to prove what it carries.
//!
//! A frame added there and not here fails **in this lane**, which is the point:
//! the person who adds a surface is told that somebody who cannot see it has no
//! way to be told it exists.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_access::Surface;

/// The frames and screens the shell exports that are surfaces a person meets.
///
/// Two of its exports are not: `DirectFrame` and `XrgbFrame` are how a frame
/// reaches a screen, and `NestedReaderFrame`, `WindowControlLabelFrame` and
/// `WindowControlReaderFrame` are the keyboard's own labels — the *reader* in
/// those names is the label reader for keyboard navigation, not a screen
/// reader, which is worth saying because the name invites the other reading.
const NOT_A_SURFACE_A_PERSON_MEETS: [&str; 5] = [
    "DirectFrame",
    "XrgbFrame",
    "NestedReaderFrame",
    "WindowControlLabelFrame",
    "WindowControlReaderFrame",
];

#[test]
fn every_frame_the_shell_exports_is_a_surface_something_can_be_said_about() {
    let shell = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../alo-shell/src/lib.rs")
        .canonicalize()
        .expect("the shell's own source, which this reads and never edits");
    let source = std::fs::read_to_string(&shell).unwrap();

    let mut drawn: Vec<String> = Vec::new();
    for line in source.lines().filter(|line| line.starts_with("pub use ")) {
        for name in line
            .split(|letter: char| !letter.is_alphanumeric())
            .filter(|name| name.ends_with("Frame") || name.ends_with("Screen"))
        {
            if !NOT_A_SURFACE_A_PERSON_MEETS.contains(&name) && !drawn.contains(&name.to_owned()) {
                drawn.push(name.to_owned());
            }
        }
    }
    assert!(
        drawn.len() >= 6,
        "only {} surfaces were read from the shell; the export list moved",
        drawn.len()
    );

    let said: Vec<&str> = Surface::ALL
        .into_iter()
        .flat_map(|surface| surface.drawn_by().iter().copied())
        .collect();
    let unsaid: Vec<&String> = drawn
        .iter()
        .filter(|frame| !said.contains(&frame.as_str()))
        .collect();
    assert!(
        unsaid.is_empty(),
        "the shell draws {unsaid:?} and a screen reader would be told nothing about it — add it \
         to alo_access::Surface with its role, name and state"
    );

    // And nothing here claims a frame the shell does not export.
    for surface in Surface::ALL {
        for frame in surface.drawn_by() {
            assert!(
                drawn.iter().any(|exported| exported == frame),
                "{surface:?} names {frame}, which the shell does not export"
            );
        }
    }
}

/// **Every control a reader is told about has a role, a name and a state**, and
/// the approval surface reads as ADR 0001 requires.
#[test]
fn every_control_is_named_and_the_approval_reads_as_the_sentence_it_asks() {
    for surface in Surface::ALL {
        let controls = surface.read_aloud();
        assert!(!controls.is_empty(), "{surface:?}");
        for control in controls {
            assert!(
                !control.name.says().is_empty(),
                "{surface:?} has a control with no name"
            );
        }
    }
    let approval: Vec<&str> = alo_access::the_approval_in_reading_order()
        .iter()
        .map(|control| control.name.says())
        .collect();
    assert_eq!(
        approval,
        vec![
            "the machine is asking you something",
            "what will happen if you approve",
            "no",
            "approve",
        ],
        "the approval surface is not read as the sentence and its two answers"
    );
}
