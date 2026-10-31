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
/// is, under the Dock, and under the record, Settings and the question, which
/// no window covers. Whether it is under the task windows is the person's
/// (`crate::which_window_is_in_front`); the next test holds that order.
#[test]
fn the_terminal_is_painted_where_a_window_is_under_every_surface_of_alos() {
    let terminal = painted_at("terminal.paint(&mut frame, terminal_washed.as_ref())");

    for (surface, call) in [
        (
            "the Dock, so it covers the furniture no window covers",
            "desktop.paint(&mut frame, &washed, the_tasks_are_in_front)",
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

/// **The three kinds of window are painted in the person's order**, and in
/// one loop: the applications, the terminal and the task windows are each
/// painted inside the loop over `bottom_to_top`, and nowhere else, so no kind
/// can be drawn outside the order a press and the keyboard read.
#[test]
fn the_windows_are_painted_in_the_order_the_person_put_them_in() {
    let the_loop = painted_at("for kind in native.in_front.bottom_to_top()");
    let the_desktop = painted_at("desktop.paint(&mut frame, &washed, the_tasks_are_in_front)");
    for call in [
        "draw_render_elements(&mut frame, zoom, &drawing.elements, &[damage])",
        "terminal.paint(&mut frame, terminal_washed.as_ref())",
        "desktop.paint_the_tasks(&mut frame, &washed)",
        "task_edge.paint(&mut frame)",
    ] {
        let at = painted_at(call);
        assert!(
            the_loop < at && at < the_desktop,
            "`{call}` is painted outside the loop over the person's order"
        );
    }
}
