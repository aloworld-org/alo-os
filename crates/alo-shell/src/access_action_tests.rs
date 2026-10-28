//! That a reader's press is handed over once, and only for the action there is.

use std::sync::mpsc;

use super::{Pressable, Pressed};
use crate::the_tasks_a_person_opened::WhichTask;

/// A pressable Close on the first task window, and where its presses land.
fn close() -> (Pressable, mpsc::Receiver<Pressed>) {
    let (to, from) = mpsc::channel();
    let pressable = Pressable {
        pressed: Pressed::CloseTheTask(WhichTask::numbered(0)),
        said: "Close".to_owned(),
        to,
    };
    (pressable, from)
}

/// **One press is one press**: handed over once, and nothing more.
#[test]
fn a_press_is_handed_over_exactly_once() {
    let (pressable, from) = close();
    assert!(pressable.do_action(0), "the press was not accepted");
    assert_eq!(
        from.try_iter().collect::<Vec<_>>(),
        [Pressed::CloseTheTask(WhichTask::numbered(0))]
    );
}

/// **An action that is not there presses nothing**, and says so.
#[test]
fn an_action_that_is_not_there_presses_nothing() {
    let (pressable, from) = close();
    for index in [1, -1, i32::MAX] {
        assert!(!pressable.do_action(index), "action {index} was accepted");
    }
    assert_eq!(
        from.try_iter().count(),
        0,
        "a refused press reached the shell"
    );
}

/// **A press with nobody to take it is refused**, not reported as done.
#[test]
fn a_press_with_no_desktop_left_is_refused() {
    let (pressable, from) = close();
    drop(from);
    assert!(!pressable.do_action(0));
}

/// **What a person hears is the control's own name**; the wire name is not
/// a word anybody is told.
#[test]
fn the_action_is_announced_by_the_controls_own_name() {
    let (pressable, _from) = close();
    assert_eq!(pressable.get_localized_name(0), "Close");
    assert_eq!(pressable.get_name(0), "click");
    assert_eq!(
        pressable.get_actions(),
        [("Close".to_owned(), String::new(), String::new())]
    );
    assert_eq!(pressable.get_localized_name(1), "");
}
