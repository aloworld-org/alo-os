//! **The order surfaces are painted in, read off the one painter.**
//!
//! A text guard, for the reason `every_windows_edge_tests` gives for its own:
//! the property is the order of the calls in the function both roads end in,
//! and a picture test would need two surfaces built to overlap on one pixel for
//! every pair. Until 2026-10-28 alo's own terminal was painted after Settings
//! and the record, so a terminal covered Settings opened behind it — invisible
//! to every test, because no frame on a real display carried a terminal before
//! #974.

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
    found[0]
}

/// **A terminal is a window among windows**: above the desktop it sits on,
/// and under the record, Settings and the question, which no window covers.
#[test]
fn the_terminal_is_painted_above_the_desktop_and_under_every_surface_of_alos() {
    let desktop = painted_at("desktop.paint(&mut frame, &washed)");
    let terminal = painted_at("terminal.paint(&mut frame)");
    let record = painted_at("record.paint(&mut frame)");
    let settings = painted_at("settings.paint(&mut frame)");
    let approval = painted_at("approval.paint(&mut frame)");

    assert!(
        desktop < terminal,
        "the terminal is painted under the desktop"
    );
    assert!(
        terminal < record,
        "the terminal is painted over the record, so a terminal covers what the machine did"
    );
    assert!(
        terminal < settings,
        "the terminal is painted over Settings, so a setting being changed is covered"
    );
    assert!(
        terminal < approval,
        "the terminal is painted over the question a person is asked to approve"
    );
}
