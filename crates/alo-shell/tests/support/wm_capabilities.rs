//! Exact capability assertions without imposing a wire ordering.
use wayland_protocols::xdg::shell::client::xdg_toplevel::WmCapabilities;

pub fn assert_window_capabilities(events: &[Vec<u32>], count: usize) {
    assert_eq!(events.len(), count, "capability event count");
    let mut expected = vec![
        WmCapabilities::Maximize as u32,
        WmCapabilities::Minimize as u32,
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
    assert_window_capabilities(&[vec![maximize, minimize], vec![minimize, maximize]], 2);
}

#[test]
#[should_panic(expected = "advertised window capabilities")]
fn duplicate_capabilities_are_not_discarded() {
    let maximize = WmCapabilities::Maximize as u32;
    let minimize = WmCapabilities::Minimize as u32;
    assert_window_capabilities(&[vec![maximize, minimize, minimize]], 1);
}

#[test]
#[should_panic(expected = "advertised window capabilities")]
fn a_missing_capability_is_not_accepted() {
    assert_window_capabilities(&[vec![WmCapabilities::Maximize as u32]], 1);
}

#[test]
#[should_panic(expected = "advertised window capabilities")]
fn an_extra_capability_is_not_accepted() {
    assert_window_capabilities(
        &[vec![
            WmCapabilities::Maximize as u32,
            WmCapabilities::Minimize as u32,
            WmCapabilities::Fullscreen as u32,
        ]],
        1,
    );
}
