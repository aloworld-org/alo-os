//! What to withdraw and what to add, decided from two lists of names.
//!
//! The function around this one needs a device descriptor, a real DRM
//! discovery and a painter — so it needs a machine with monitors plugged into
//! it, and on this lane's hardware it cannot be tested at all. The decision
//! needs none of that and is where the mistakes live, so it is tested here.

use super::{DeskChange, what_the_desk_now_holds};

/// The display the frame loop itself holds, in every case below.
const THE_LOOPS_OWN: &str = "alo-drm-31";

/// What this machine calls the connector with that number.
fn named(connector: u32) -> String {
    format!("alo-drm-{connector}")
}

/// A desk drawing to these, in this order.
fn drawing_to(connectors: &[u32]) -> Vec<Option<String>> {
    connectors.iter().map(|it| Some(named(*it))).collect()
}

#[test]
fn a_desk_nobody_touched_withdraws_nothing_and_adds_nothing() {
    // **Task 13's constraint.** A session that never sees a change must behave
    // exactly as it does now, and this is the frame it is on all day.
    let change =
        what_the_desk_now_holds(&[named(31), named(42)], &drawing_to(&[42]), THE_LOOPS_OWN);
    assert_eq!(change, DeskChange::default());
}

#[test]
fn a_display_the_machine_has_started_reporting_is_added() {
    let change = what_the_desk_now_holds(
        &[named(31), named(42), named(55)],
        &drawing_to(&[42]),
        THE_LOOPS_OWN,
    );
    assert_eq!(change.present, vec![named(55)]);
    assert!(change.retire.is_empty(), "nothing left, so nothing retires");
}

#[test]
fn a_display_the_machine_has_stopped_reporting_is_withdrawn() {
    let change = what_the_desk_now_holds(&[named(31)], &drawing_to(&[42]), THE_LOOPS_OWN);
    assert_eq!(change.retire, vec![0]);
    assert!(change.present.is_empty());
}

#[test]
fn the_loops_own_display_is_never_added_as_an_extra() {
    // **The case the exclusion exists for, and it fires on every frame.** The
    // loop's own display is reported by the machine and is deliberately absent
    // from the list this `Desk` draws to — `run_with_input` holds that one
    // target. Without the exclusion every frame would decide the primary
    // screen was a new arrival and present a second copy of it.
    let change = what_the_desk_now_holds(&[named(31)], &[], THE_LOOPS_OWN);
    assert_eq!(change, DeskChange::default());
}

#[test]
fn the_loops_own_display_is_never_withdrawn_either() {
    // It is not in the drawn list, so there is no position to withdraw. Held
    // as a test because the honest limitation depends on it: a primary display
    // being unplugged is **not** handled here, and this is what *not handled*
    // has to look like — nothing, rather than something wrong.
    let change = what_the_desk_now_holds(&[], &drawing_to(&[42]), THE_LOOPS_OWN);
    assert_eq!(change.retire, vec![0], "the extra display goes");
    assert!(
        change.present.is_empty(),
        "and the loop's own is not resurrected"
    );
}

#[test]
fn one_cable_moved_withdraws_the_old_and_adds_the_new() {
    let change =
        what_the_desk_now_holds(&[named(31), named(55)], &drawing_to(&[42]), THE_LOOPS_OWN);
    assert_eq!(change.retire, vec![0]);
    assert_eq!(change.present, vec![named(55)]);
}

#[test]
fn a_display_that_will_not_name_itself_is_left_alone() {
    // It cannot be matched against what the machine reports, so withdrawing it
    // would be acting on a guess about which display it is — and the guess
    // that is wrong turns off a screen somebody is using.
    // `LoopInput::retire_the_rest` takes it when the session ends.
    let change = what_the_desk_now_holds(&[named(31)], &[None], THE_LOOPS_OWN);
    assert!(
        change.retire.is_empty(),
        "an unnameable display is kept, not guessed at"
    );
}

#[test]
fn the_positions_withdrawn_point_at_the_right_displays() {
    // **The test that catches an off-by-one in the direction that does
    // damage.** The caller removes by position, so a position that is one out
    // turns off a monitor the person is still using and keeps one that is
    // gone. The middle display leaves; the first and last stay.
    let change = what_the_desk_now_holds(
        &[named(31), named(40), named(60)],
        &drawing_to(&[40, 50, 60]),
        THE_LOOPS_OWN,
    );
    assert_eq!(change.retire, vec![1], "only the middle one");
    assert!(change.present.is_empty());
}

#[test]
fn several_displays_leaving_at_once_are_withdrawn_in_ascending_order() {
    // The caller removes in reverse, so the order this returns is part of the
    // contract rather than an accident: descending removal is only safe if
    // these ascend.
    let change = what_the_desk_now_holds(
        &[named(31), named(50)],
        &drawing_to(&[40, 50, 60]),
        THE_LOOPS_OWN,
    );
    assert_eq!(change.retire, vec![0, 2]);
}

#[test]
fn a_display_already_drawn_to_is_not_added_a_second_time() {
    // The cheap question is allowed to be over-eager, so this runs on a desk
    // where nothing actually changed. Adding again would mean two targets and
    // two painters for one monitor.
    let change =
        what_the_desk_now_holds(&[named(31), named(42)], &drawing_to(&[42]), THE_LOOPS_OWN);
    assert!(change.present.is_empty());
}

#[test]
fn a_machine_reporting_nothing_at_all_withdraws_everything_it_drew() {
    // Not a refusal — a refusal never reaches this function, because the
    // caller returns early and keeps the desk it had. This is the machine
    // answering honestly that there is nothing there.
    let change = what_the_desk_now_holds(&[], &drawing_to(&[40, 50]), THE_LOOPS_OWN);
    assert_eq!(change.retire, vec![0, 1]);
    assert!(change.present.is_empty());
}
