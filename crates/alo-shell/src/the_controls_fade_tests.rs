//! The fade, held to the owner's lengths and to reduced motion.

use super::*;

/// A moment to measure from.
fn then() -> Instant {
    Instant::now()
}

/// **In over 120 ms**: nothing at the start, half at 60, all of it at 120.
#[test]
fn they_come_in_over_a_hundred_and_twenty_milliseconds() {
    let t = then();
    let fade = TheFade::settled(false, t).heading(true, t, false);
    assert_eq!(fade.how_shown(t, false), 0, "no jump at the start");
    assert_eq!(fade.how_shown(t + Duration::from_millis(60), false), 500);
    assert_eq!(fade.how_shown(t + THEY_COME_IN, false), ALL_THE_WAY);
}

/// **Out over 160 ms**, from the moment it truly leaves — no wait first.
#[test]
fn they_go_over_a_hundred_and_sixty_milliseconds_with_no_wait() {
    let t = then();
    let fade = TheFade::settled(true, t).heading(false, t, false);
    assert!(
        fade.how_shown(t + Duration::from_millis(1), false) < ALL_THE_WAY,
        "the fade begins at once: there is no hide delay"
    );
    assert_eq!(fade.how_shown(t + Duration::from_millis(80), false), 500);
    assert_eq!(fade.how_shown(t + THEY_GO_IN, false), 0);
}

/// **Under reduced motion they are where they are heading, at once.**
#[test]
fn under_reduced_motion_the_change_is_immediate() {
    let t = then();
    let coming = TheFade::settled(false, t).heading(true, t, true);
    assert_eq!(coming.how_shown(t, true), ALL_THE_WAY);
    let going = TheFade::settled(true, t).heading(false, t, true);
    assert_eq!(going.how_shown(t, true), 0);
}

/// **Turned round part-way, it carries on from where it got to** — no jump,
/// and only the share of the length that is left.
#[test]
fn turned_round_part_way_it_carries_on_from_where_it_was() {
    let t = then();
    let coming = TheFade::settled(false, t).heading(true, t, false);
    let halfway = t + Duration::from_millis(60);
    let going = coming.heading(false, halfway, false);
    assert_eq!(going.how_shown(halfway, false), 500, "no jump at the turn");
    assert_eq!(
        going.how_shown(halfway + Duration::from_millis(80), false),
        0,
        "half the way out takes half the out length"
    );
}

/// **Heading where it already is changes nothing**, so a frame that asks
/// again does not restart the fade.
#[test]
fn heading_the_same_way_again_does_not_restart_it() {
    let t = then();
    let coming = TheFade::settled(false, t).heading(true, t, false);
    let later = t + Duration::from_millis(60);
    assert_eq!(coming.heading(true, later, false), coming);
}
