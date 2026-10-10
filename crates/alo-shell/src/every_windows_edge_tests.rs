//! That the production draw path is what builds the edges, and that it is read
//! from a list rather than a constant.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a file this crate ships is the failure being reported"
)]

/// `direct_desktop.rs`, as text.
fn the_direct_desktop() -> String {
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/direct_desktop.rs");
    std::fs::read_to_string(&at).expect("direct_desktop.rs is beside this file")
}

/// **The real display backend builds an edge for every window, on every
/// display.**
///
/// This is a text guard for the reason `the_recheck_has_a_caller`'s is: the
/// property is *that the production path is the caller*, and a draw that built
/// edges and handed `&[]` to the layers would pass every behavioural test in
/// this crate while no window on a machine had an edge. That was the state
/// until 2026-10-10, with every part of the edge built and tested.
#[test]
fn the_direct_backend_builds_an_edge_for_every_window() {
    let text = the_direct_desktop();

    // The count first, as ever: a file that stopped calling it would otherwise
    // satisfy the two assertions below by having no call sites to disagree with.
    let asks = text
        .matches("every_windows_edge::every_windows_edge(")
        .count();
    assert_eq!(
        asks, 2,
        "the direct backend draws the display the loop holds and every other \
         one, so it asks twice; it asked {asks} times"
    );

    // **Read from a binding, never an empty literal.** `edges: &[]` is what
    // `NativeLayers::nothing()` already means, and writing it at a call site
    // would be a display that draws no edge while looking wired.
    assert!(
        !text.contains("edges: &[]"),
        "a display is handed an empty edge list explicitly, which is a window \
         with no edge written as though it were wiring"
    );
    assert!(
        text.contains("edges: &edges") && text.contains("edges: its_edges"),
        "both displays take their edges from the list that was built for them"
    );
}

/// **Each display is asked at its own scale**, not at the held display's.
///
/// The same window on two displays of different scales is two different
/// pictures, because `EdgePicture::of` converts logical units to physical ones
/// once against one output. A single list would be right for one display and
/// silently wrong for the other, and nothing about the drawn result would say
/// so on a machine with one display — which is every machine today.
#[test]
fn every_display_is_asked_at_its_own_scale() {
    let text = the_direct_desktop();
    assert!(
        text.contains("u32::from(this_display.0)") && text.contains("u32::from(its_scale)"),
        "a display's edges are built at a scale that is not its own"
    );
    // And both scales are read back from the frame they were set on rather
    // than asked of the server a second time, which would be two answers to
    // one question that could differ for a frame after a person changes it.
    assert!(
        text.contains("let this_display = (frame.display_scale,")
            && text.contains("let its_scale = frame.display_scale;"),
        "a scale is asked of the server again instead of read from the frame"
    );
}
