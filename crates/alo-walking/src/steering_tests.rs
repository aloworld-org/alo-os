//! Steering a pointer by looking, on a screen whose pointer is drawn by the real
//! artwork and lands a little away from where it was put.

use alo_cursors::CursorIcon;

use super::{CLOSE, Steered, steer};
use crate::a_screen_with_a_pointer::AScreenWithAPointer;
use crate::frame::Frame;
use crate::the_pointer_seen::{Drawn, Look, Reference, found_in};

/// A finder for the shipped kite, around where the pointer should be.
fn the_kite() -> impl Fn(&Frame, (usize, usize)) -> Option<(usize, usize)> {
    let reference = Reference::of(Drawn {
        icon: CursorIcon::Default,
        look: Look::SHIPPED,
        scale: 100,
    })
    .ok();
    move |frame, _| {
        let reference = reference.as_ref()?;
        found_in(frame, reference, None).first().copied()
    }
}

/// **It gets there whether the pointer lands short of or past where it was
/// put**, and says how many rounds it took.
#[test]
fn it_gets_there_however_the_moves_are_multiplied() {
    for lands_at in [70, 100, 130] {
        let mut machine = AScreenWithAPointer {
            lands_at,
            ..AScreenWithAPointer::grey()
        };
        let steered = steer(&mut machine, (100, 100), (300, 200), the_kite());
        let Ok(Steered::There { at, rounds }) = steered else {
            unreachable!("landing at {lands_at} per cent: {steered:?}");
        };
        assert!(
            at.0.abs_diff(300) <= CLOSE && at.1.abs_diff(200) <= CLOSE,
            "landing at {lands_at} per cent it stopped at {at:?}"
        );
        assert!(rounds >= 1, "landing at {lands_at} per cent");
    }
}

/// **Already there is no move at all.**
#[test]
fn already_there_moves_nothing() {
    let mut machine = AScreenWithAPointer::grey();
    let steered = steer(&mut machine, (100, 100), (102, 99), the_kite());
    assert_eq!(
        steered,
        Ok(Steered::There {
            at: (100, 100),
            rounds: 0
        })
    );
    assert!(machine.pointed.is_empty());
}

/// **The refusal: a pointer the screen does not show is lost**, after the
/// first move, rather than assumed to be where it was asked to go.
#[test]
fn a_pointer_not_on_the_screen_is_lost() {
    let mut machine = AScreenWithAPointer {
        draws_the_pointer: false,
        ..AScreenWithAPointer::grey()
    };
    let steered = steer(&mut machine, (100, 100), (400, 300), the_kite());
    assert_eq!(
        steered,
        Ok(Steered::Lost {
            last: (100, 100),
            rounds: 1
        })
    );
}

/// **A place the pointer cannot reach is not there**, said with where it got
/// to, or lost where the screen's edge cuts the pointer off.
#[test]
fn a_place_off_the_screen_is_never_reached() {
    let mut machine = AScreenWithAPointer::grey();
    let steered = steer(&mut machine, (100, 100), (100, 2000), the_kite());
    assert!(
        matches!(steered, Ok(Steered::NotThere { .. } | Steered::Lost { .. })),
        "{steered:?}"
    );
}
