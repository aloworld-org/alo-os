//! **The agent opens the screen road from what a read answered**: once a read
//! in the turn came back with windows that describe nothing, and not before or
//! otherwise — so a turn going on after an approval keeps it.

use alo_adapters::{Role, Seen, Shown, ShownWindow};
use alo_agent::the_screen_road_is_open;
use alo_instructing::AStep;
use alo_protocol::{Done, ToAnAgent};

fn a_read_answered(seen: Vec<Seen>) -> AStep {
    let done = Done::of_a_window_read(
        &Shown::of(
            "net.supertuxkart.SuperTuxKart".to_owned(),
            vec![ShownWindow::titled("SuperTuxKart".to_owned(), seen)],
            false,
        ),
        &[],
    );
    let came_back = ToAnAgent::did(done).written().unwrap_or_default();
    AStep::of("a read", &came_back)
}

#[test]
fn only_a_read_of_windows_that_describe_nothing_opens_it() {
    assert!(!the_screen_road_is_open(&[]), "open before any read");
    let describing = a_read_answered(vec![Seen::of(
        Role::Button,
        "Start race".to_owned(),
        None,
        1,
    )]);
    assert!(
        !the_screen_road_is_open(std::slice::from_ref(&describing)),
        "open after a read of a window that names its controls"
    );
    let nothing = a_read_answered(vec![Seen::of(Role::Canvas, String::new(), None, 1)]);
    assert!(
        the_screen_road_is_open(&[describing, nothing]),
        "not open after a read of a window that describes nothing"
    );
}
