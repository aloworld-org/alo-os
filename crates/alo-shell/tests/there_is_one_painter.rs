//! **There is one painter, and this is what says so.**
//!
//! This shell has two roads to pixels — a graphics card through
//! `direct_target` and the processor through `software_scanout` — and they
//! differ in what they cost, not in what they show. The way to keep them
//! showing the same thing is not to check that two painters agree. It is for
//! there to be one painter.
//!
//! # What this file used to be, and why that was not enough
//!
//! It read `both_painters_draw_the_same_layers` and held each source file to
//! naming every field of `NativeLayers`. It was written on 2026-10-11, after
//! the `edges` field was added in #613 and only `software_scanout` was taught
//! to paint it — so a machine with a graphics card drew **no window edge at
//! all**, every unit test passed, and the gate was green.
//!
//! That test would have caught the next such omission. It could not stop one
//! happening, because two painters is what makes the omission possible: a
//! layer added to the set is two places that have to learn about it, and the
//! test only notices after one of them has not.
//!
//! So `software_scanout::paint` now calls `crate::scene_drawing::paint`, the
//! same function the card's road ends in, and the refusals it kept for the
//! client surfaces it could not draw are gone with it — 119 lines, named by
//! clippy as unused the moment the call changed. **A second opinion about
//! what a frame contains cannot drift from the first when there is no second
//! opinion.**
//!
//! # Read as text, which is the only way one file can ask this of another
//!
//! Neither road runs here: one wants a card, the other pixman and a frame.
//! What can be checked without either is that one file calls the other and
//! that the one it calls knows every layer. The technique is `alo-access`'s,
//! which holds this crate's exports to its own list by reading them.
//!
//! **No `panic!`, no `expect`, no `unwrap`**, which this crate forbids even in
//! a test.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// A file of this crate's own source, and a failure if it is not there.
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
/// it is added. A hand-written list would be a second declaration, which is
/// the same fault one level up from the one this file is about.
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

/// **The processor's road paints by calling the card's road's painter.**
#[test]
fn the_software_painter_calls_the_one_painter() {
    let text = source("software_scanout.rs");

    assert!(
        text.contains("crate::scene_drawing::paint("),
        "software_scanout no longer calls the shared painter, so this shell has two painters \
         again and what a person sees depends on their graphics hardware"
    );
}

/// **And it has grown no second opinion of its own.**
///
/// These four are the painter it used to hold. Each is named rather than the
/// absence being inferred, because a check that says *there is nothing like
/// this here* is only as good as its idea of what *like this* means.
#[test]
fn the_software_painter_decides_nothing_about_what_a_frame_contains() {
    let text = source("software_scanout.rs");

    for grown_back in [
        "struct ToImport",
        "fn refused_layer",
        "fn validate_layers",
        "fn paint_layers",
    ] {
        assert!(
            !text.contains(grown_back),
            "`{grown_back}` is back in software_scanout: a second place deciding what a frame \
             contains, which is how the edges layer came to be drawn on one road and not the \
             other"
        );
    }
}

/// **The one painter knows every layer there is.**
///
/// With one painter this is the whole of the old two-road check: a layer the
/// painter never names is a layer nothing draws, on any hardware.
#[test]
fn the_one_painter_names_every_layer() {
    let layers = every_layer();
    assert!(
        layers.len() >= 5,
        "only {} layers were read from NativeLayers, which is a broken parse rather than a \
         shell with that few: {layers:?}",
        layers.len()
    );

    let painter = source("scene_drawing.rs");
    let missing: Vec<&String> = layers
        .iter()
        .filter(|layer| !painter.contains(&format!(".{layer}")))
        .collect();

    assert!(
        missing.is_empty(),
        "the painter never names {missing:?}, so nothing draws them on any machine"
    );
}

/// **An edge is painted above the window it frames and below the Dock.**
///
/// The Dock is furniture no window's chrome covers — and the reason is
/// correctness rather than taste: the Dock's own hit-testing would disagree
/// with what a person sees if a window's edge were drawn over it.
#[test]
fn the_edges_sit_between_the_windows_and_the_dock() {
    let painter = source("scene_drawing.rs");

    // **The painting call, not the first mention.** Written as
    // `find(".desktop")` first, and it failed against a correct file: the
    // painter reads `native.desktop` long before it paints it, once to take
    // the colour behind everything.
    let clients = painter.find("draw_render_elements(");
    let edges = painter.find("edge.paint(");
    let desktop = painter.find("desktop.paint(");

    assert!(clients.is_some(), "the painter never draws a client");
    assert!(edges.is_some(), "the painter never paints an edge");
    assert!(desktop.is_some(), "the painter never paints the desktop");
    assert!(
        clients < edges,
        "an edge is painted before the client it frames, so it would be drawn underneath it"
    );
    assert!(
        edges < desktop,
        "the desktop is painted before the edges, so the Dock would sit under a window's chrome"
    );
}
