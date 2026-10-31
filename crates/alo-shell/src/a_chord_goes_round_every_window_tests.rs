//! The arithmetic of the walk: where *next* lands, wrapping both ways, and
//! nowhere to go when there is one stop and the keyboard is on it.

use super::{AStop, the_next_stop};
use crate::the_tasks_a_person_opened::WhichTask;
use crate::window_switch::WindowSwitchDirection::{Backward, Forward};

/// The terminal and two task windows.
fn three() -> Vec<AStop> {
    vec![
        AStop::TheTerminal,
        AStop::ATask(WhichTask::numbered(0)),
        AStop::ATask(WhichTask::numbered(1)),
    ]
}

#[test]
fn next_goes_on_and_wraps_round() {
    let stops = three();
    assert_eq!(
        the_next_stop(&stops, Some(&AStop::TheTerminal), Forward),
        Some(AStop::ATask(WhichTask::numbered(0)))
    );
    assert_eq!(
        the_next_stop(&stops, Some(&AStop::ATask(WhichTask::numbered(1))), Forward),
        Some(AStop::TheTerminal),
        "the last did not wrap to the first"
    );
}

#[test]
fn back_goes_back_and_wraps_round() {
    let stops = three();
    assert_eq!(
        the_next_stop(&stops, Some(&AStop::TheTerminal), Backward),
        Some(AStop::ATask(WhichTask::numbered(1))),
        "the first did not wrap to the last"
    );
}

#[test]
fn from_nowhere_it_starts_at_the_end_it_goes_from() {
    let stops = three();
    assert_eq!(the_next_stop(&stops, None, Forward), Some(AStop::TheTerminal));
    assert_eq!(
        the_next_stop(&stops, None, Backward),
        Some(AStop::ATask(WhichTask::numbered(1)))
    );
}

#[test]
fn one_stop_with_the_keyboard_on_it_is_nowhere_to_go() {
    let one = vec![AStop::ATask(WhichTask::numbered(0))];
    assert_eq!(the_next_stop(&one, one.first(), Forward), None);
    assert_eq!(the_next_stop(&[], None, Forward), None);
    assert_eq!(
        the_next_stop(&one, None, Forward),
        one.first().cloned(),
        "one stop the keyboard is not on is somewhere to go"
    );
}
