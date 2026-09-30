//! Exact capability assertions without imposing a wire ordering.
use wayland_protocols::xdg::shell::client::xdg_toplevel::WmCapabilities;

pub fn assert_window_capabilities(events: &[Vec<u32>], count: usize) {
    assert_eq!(events.len(), count, "capability event count");
    let mut expected = vec![
        WmCapabilities::Maximize as u32,
        WmCapabilities::Minimize as u32,
        // Joined on 2026-09-30, when the compositor gained a
        // `fullscreen_request` to answer it with. Before that its absence was
        // the assertion: a capability advertised without a handler is a promise
        // a client acts on and is met with silence.
        WmCapabilities::Fullscreen as u32,
    ];
    expected.sort_unstable();
    for event in events {
        let mut actual = event.clone();
        // Sorting preserves duplicates: a duplicate or an extra capability fails.
        actual.sort_unstable();
        assert_eq!(actual, expected, "advertised window capabilities");
    }
}

#[test]
fn capability_order_is_not_part_of_the_assertion() {
    let maximize = WmCapabilities::Maximize as u32;
    let minimize = WmCapabilities::Minimize as u32;
    let filling = WmCapabilities::Fullscreen as u32;
    assert_window_capabilities(
        &[
            vec![maximize, minimize, filling],
            vec![filling, minimize, maximize],
        ],
        2,
    );
}

#[test]
#[should_panic(expected = "advertised window capabilities")]
fn duplicate_capabilities_are_not_discarded() {
    let maximize = WmCapabilities::Maximize as u32;
    let minimize = WmCapabilities::Minimize as u32;
    assert_window_capabilities(
        &[vec![
            maximize,
            minimize,
            minimize,
            WmCapabilities::Fullscreen as u32,
        ]],
        1,
    );
}

#[test]
#[should_panic(expected = "advertised window capabilities")]
fn a_missing_capability_is_not_accepted() {
    assert_window_capabilities(&[vec![WmCapabilities::Maximize as u32]], 1);
}

/// **And a set missing only the new one is refused too.** Without this, an
/// advertisement that dropped `Fullscreen` while keeping the other two would
/// pass every other test here, which is the regression this change makes
/// possible and therefore the one it owes a test.
#[test]
#[should_panic(expected = "advertised window capabilities")]
fn dropping_full_screen_alone_is_not_accepted() {
    assert_window_capabilities(
        &[vec![
            WmCapabilities::Maximize as u32,
            WmCapabilities::Minimize as u32,
        ]],
        1,
    );
}

/// **A fourth capability is still refused**, which is what keeps this an
/// assertion about the whole set rather than about one member of it.
///
/// It used `Fullscreen` as its example until 2026-09-30, when that became a
/// capability this compositor really has. Left alone, this test would have
/// stopped panicking and its `should_panic` would have failed — telling a
/// reader the set is exact through a test that had quietly become one asserting
/// it is not. `WindowMenu` replaces it: nothing here implements a window menu.
#[test]
#[should_panic(expected = "advertised window capabilities")]
fn an_extra_capability_is_not_accepted() {
    assert_window_capabilities(
        &[vec![
            WmCapabilities::Maximize as u32,
            WmCapabilities::Minimize as u32,
            WmCapabilities::Fullscreen as u32,
            WmCapabilities::WindowMenu as u32,
        ]],
        1,
    );
}
