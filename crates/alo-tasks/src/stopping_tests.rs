//! That Stop has a waiting state, reports a real ending, and says what changed.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::Stopping;
use crate::changed::WhatChanged;

/// Two changes, as a task would have recorded them.
fn two_changes() -> WhatChanged {
    let mut changed = WhatChanged::nothing();
    changed.and_then("Renamed 3 files in Documents/invoices");
    changed.and_then("Installed Blender");
    changed
}

/// **Pressing Stop does not stop anything**, and the type says so.
///
/// The owner's wording is *requests cancellation* and then *reports when work
/// has stopped* — two moments. A `bool` has one, and the only way to answer it
/// at the press is to answer `true`, which tells a person the work is over
/// while it is running.
#[test]
fn pressing_stop_is_a_request_and_not_an_ending() {
    let asked = Stopping::nobody_asked().asked();

    assert!(
        !asked.has_stopped(),
        "pressing Stop reported that the work had stopped, which it has not"
    );
    assert!(
        asked.is_waiting(),
        "a person who pressed Stop is shown neither stopping nor stopped"
    );
    assert_eq!(
        asked.what_changed(),
        None,
        "what changed was answered before the work ended, so the account is of a task still \
         running"
    );
}

/// **The ending is its own moment, and carries what was done.**
#[test]
fn stopping_reports_the_ending_and_what_it_changed() {
    let stopped = Stopping::stopped(two_changes());

    assert!(stopped.has_stopped());
    assert!(!stopped.is_waiting());
    let changed = stopped
        .what_changed()
        .expect("a stopped task says what it changed");
    assert_eq!(changed.how_many(), 2);
    assert!(!changed.nothing_happened());
}

/// **A task that stopped having changed nothing says so**, rather than showing
/// a person an empty area they have to interpret.
///
/// *It changed nothing* and *nobody has told us* are opposite readings of an
/// empty list, and this is the one that distinguishes them.
#[test]
fn a_task_that_changed_nothing_still_says_so() {
    let stopped = Stopping::stopped(WhatChanged::nothing());

    let changed = stopped
        .what_changed()
        .expect("a stopped task says what it changed, including nothing");
    assert!(changed.nothing_happened());
    assert_eq!(changed.how_many(), 0);
}

/// **Pressing Stop twice is pressing it once.**
///
/// There is no harder stop to escalate to, and a second press that looked like
/// a stronger request would be a control lying about what it does.
#[test]
fn pressing_stop_twice_asks_once() {
    let once = Stopping::nobody_asked().asked();
    let twice = once.clone().asked();

    assert_eq!(
        twice, once,
        "a second press changed the state of the request"
    );
}

/// **A late press does not discard the account of what changed.**
///
/// Somebody presses Stop as the work ends. The press arrives after the ending,
/// and what they must not lose is the list of what was done to their machine.
#[test]
fn pressing_stop_after_it_ended_keeps_what_changed() {
    let ended = Stopping::stopped(two_changes());

    let late = ended.asked();

    assert!(late.has_stopped(), "a late press un-ended a finished task");
    assert_eq!(
        late.what_changed().map(WhatChanged::how_many),
        Some(2),
        "a press arriving after the ending threw away the account of what the task changed"
    );
}

/// **Nobody asked is not waiting and not stopped**, which is every task for
/// most of its life.
#[test]
fn a_task_nobody_stopped_is_neither() {
    let running = Stopping::nobody_asked();

    assert!(!running.has_stopped());
    assert!(!running.is_waiting());
    assert_eq!(running.what_changed(), None);
}
