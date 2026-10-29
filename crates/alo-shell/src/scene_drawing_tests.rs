//! **The order surfaces are painted in, read off the one painter.**
//!
//! A text guard, for the reason `every_windows_edge_tests` gives for its own:
//! the property is the order of the calls in the function both roads end in,
//! and a picture test would need two surfaces built to overlap on one pixel for
//! every pair. Until 2026-10-28 alo's own terminal was painted after Settings
//! and the record, so a terminal covered Settings opened behind it, and the
//! card a person approves a change on — invisible to every test, because no
//! frame on a real display carried a terminal before #974.

/// `scene_drawing.rs`, as text.
const SOURCE: &str = include_str!("scene_drawing.rs");

/// Where a surface is painted, by its one call.
fn painted_at(call: &str) -> usize {
    let found: Vec<usize> = SOURCE.match_indices(call).map(|(at, _)| at).collect();
    assert_eq!(
        found.len(),
        1,
        "`{call}` should be painted exactly once, and is painted {} times",
        found.len()
    );
    found.first().copied().unwrap_or_default()
}

/// **A terminal is a window among windows**: painted where a client's window
/// is, under the Dock and the task windows, and under the record, Settings and
/// the question, which no window covers.
#[test]
fn the_terminal_is_painted_where_a_window_is_under_every_surface_of_alos() {
    let terminal = painted_at("terminal.paint(&mut frame, terminal_washed.as_ref())");

    for (surface, call) in [
        (
            "the Dock and the task windows, so it covers a change waiting for approval",
            "desktop.paint(&mut frame, &washed)",
        ),
        (
            "the record, so it covers what the machine did",
            "record.paint(&mut frame)",
        ),
        (
            "Settings, so it covers a setting being changed",
            "settings.paint(&mut frame)",
        ),
        (
            "the question a person is asked to approve",
            "approval.paint(&mut frame)",
        ),
    ] {
        assert!(
            terminal < painted_at(call),
            "the terminal is painted over {surface}"
        );
    }
}
