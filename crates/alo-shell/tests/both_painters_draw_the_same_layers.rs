//! **Every layer one painting road draws, the other draws too.**
//!
//! This shell has two roads to pixels, and they are not interchangeable in
//! what they cost — only in what they must show:
//!
//! | | |
//! |---|---|
//! | the card | `direct_target` → `offscreen::render_native_scanout` → `scene_drawing::paint` |
//! | the processor | `software_scanout::paint` → `paint_layers` |
//!
//! A layer added to one and not the other is a machine that shows different
//! things depending on what graphics hardware it has, and the failure is
//! silent on whichever road nobody is looking at that day.
//!
//! # Why this exists
//!
//! **Measured on 2026-10-11.** `NativeLayers` grew an `edges` field in #613 -
//! *every window's edge is a layer, so every window gets one* - and only
//! `software_scanout` was taught to paint it. `scene_drawing` named `edges`
//! zero times, so a machine with a graphics card drew **no window edge at
//! all**, while the pixman path drew them correctly, every unit test passed
//! and the gate was green. It was found by reading the two files beside each
//! other, not by anything failing.
//!
//! # Read as text, which is the only way one file can ask this of another
//!
//! Neither road runs here: one wants a card, the other wants pixman and a
//! frame. What can be checked without either is that both files mention every
//! field of the layer set and paint them in the same order - weaker than
//! *both draw it correctly*, and far stronger than *somebody remembered*. The
//! technique is `alo-access`'s, which holds this crate's exports to its own
//! list by reading them, and `alo-saying`'s for a rented name.
//!
//! **No `panic!`, no `expect`, no `unwrap`**, which this crate forbids even in
//! a test: every answer below is an assertion on a value.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// A file of this crate's own source, and empty if it could not be read.
fn source(called: &str) -> String {
    let at: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(called);
    let text = std::fs::read_to_string(&at).unwrap_or_default();
    assert!(
        !text.is_empty(),
        "{} is empty or unreadable, so nothing below was checked",
        at.display()
    );
    text
}

/// Every field of `NativeLayers`, read from its own declaration.
///
/// Read rather than listed, so a field added tomorrow is in this test the day
/// it is added. A hand-written list would be one more place to forget, which
/// is the shape of the fault this file is about.
fn every_layer() -> BTreeSet<String> {
    let text = source("scene_native.rs");
    let after = text.split("pub(crate) struct NativeLayers").nth(1);
    assert!(
        after.is_some(),
        "NativeLayers is not declared where this reads for it"
    );
    let body: String = after
        .and_then(|rest| rest.split_once('{'))
        .map(|(_, rest)| rest)
        .and_then(|rest| rest.split_once("\n}"))
        .map(|(body, _)| body.to_owned())
        .unwrap_or_default();
    assert!(!body.is_empty(), "the struct body could not be read");

    let mut fields = BTreeSet::new();
    for line in body.lines() {
        let line = line.trim();
        if line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        if let Some((named, _)) = line.split_once(": ") {
            let named = named.trim_start_matches("pub(crate) ").trim();
            if !named.is_empty() && named.chars().all(|it| it.is_ascii_lowercase() || it == '_') {
                fields.insert(named.to_owned());
            }
        }
    }
    fields
}

/// **Both roads to pixels name every layer there is.**
#[test]
fn every_layer_one_painter_draws_the_other_draws_too() {
    let layers = every_layer();
    assert!(
        layers.len() >= 5,
        "only {} layers were read from NativeLayers, which is a broken parse rather than a \
         shell with that few: {layers:?}",
        layers.len()
    );

    let roads = [
        ("the card", source("scene_drawing.rs")),
        ("the processor", source("software_scanout.rs")),
    ];

    let mut missing = Vec::new();
    for (road, text) in &roads {
        for layer in &layers {
            // `native.edges` on one road and `layers.edges` on the other: the
            // binding differs and the field does not, so the field is what is
            // asked about.
            if !text.contains(&format!(".{layer}")) {
                missing.push(format!("{road} never names `{layer}`"));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "a layer is drawn on one road and not the other, so what a person sees depends on \
         their graphics hardware:\n  {}",
        missing.join("\n  ")
    );
}

/// **And the two paint them in the same order**, for the pair whose order a
/// person can see.
///
/// An edge belongs above the window it frames and below the Dock, because the
/// Dock is furniture no window's chrome covers. Two roads disagreeing about
/// that is one machine where the Dock sits under a window's edge and another
/// where it sits over it.
#[test]
fn the_edges_sit_between_the_windows_and_the_dock_on_both_roads() {
    for called in ["scene_drawing.rs", "software_scanout.rs"] {
        let text = source(called);
        // **The painting call, not the first mention.** Written as
        // `find(".desktop")` first, and it failed against a correct file:
        // both roads read `native.desktop` long before they paint it - once
        // for the colour behind everything, once to validate it - so the
        // first mention is nowhere near where anything is drawn.
        let edges = text.find("edge.paint(");
        let desktop = text.find("desktop.paint(");
        assert!(edges.is_some(), "{called} never paints an edge");
        assert!(desktop.is_some(), "{called} never paints the desktop");
        assert!(
            edges < desktop,
            "{called} paints the desktop before the edges, so the Dock would sit under a \
             window's chrome"
        );
    }
}
