//! Direct pointer translation observed through real client wire events.

use super::support::{Application, Fixture};
use alo_shell::{DirectPointerEvent, InputError};
use smithay::{
    backend::input::{Axis, AxisSource, ButtonState},
    input::pointer::AxisFrame,
};
use wayland_client::protocol::wl_pointer;

/// Route using the production direct bridge on the server's thread.
fn route(f: &Fixture, event: DirectPointerEvent) -> Result<(), InputError> {
    f.backend(move |s| s.direct_pointer(true, (32, 32), Some(event)))
}

/// A real mapped client with pointer capability.
fn setup() -> (Fixture, Application) {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    let mut app = Application::new(&f);
    app.configure();
    app.attach();
    app.sync();
    (f, app)
}

#[test]
fn direct_pointer_refusal_preserves_wire_position_and_button_state() -> Result<(), InputError> {
    let (f, mut app) = setup();
    route(
        &f,
        DirectPointerEvent::Relative {
            dx: 2.5,
            dy: 3.25,
            time: 1,
        },
    )?;
    route(
        &f,
        DirectPointerEvent::Button {
            code: 0x110,
            state: ButtonState::Pressed,
            time: 2,
        },
    )?;
    app.sync();
    let frames = app.events.pointer.frames;
    for event in [
        DirectPointerEvent::Relative {
            dx: 10.0,
            dy: f64::NAN,
            time: 3,
        },
        DirectPointerEvent::Absolute {
            x: 0.5,
            y: 1.1,
            time: 3,
        },
        DirectPointerEvent::Button {
            code: 30,
            state: ButtonState::Pressed,
            time: 3,
        },
        DirectPointerEvent::Axis(AxisFrame::new(3).value(Axis::Vertical, f64::NAN)),
    ] {
        assert!(matches!(route(&f, event), Err(InputError::InvalidPointer)));
    }
    assert!(
        f.backend(|s| s.direct_pointer(
            true,
            (0, 32),
            Some(DirectPointerEvent::Relative {
                dx: 1.0,
                dy: 1.0,
                time: 3
            })
        ))
        .is_err()
    );
    app.sync();
    assert_eq!(app.events.pointer.frames, frames);
    route(
        &f,
        DirectPointerEvent::Relative {
            dx: 1.0,
            dy: 1.0,
            time: 4,
        },
    )?;
    route(
        &f,
        DirectPointerEvent::Button {
            code: 0x110,
            state: ButtonState::Released,
            time: 5,
        },
    )?;
    app.sync();
    assert_eq!(app.events.pointer.motion.last(), Some(&(3.5, 4.25)));
    assert_eq!(app.events.pointer.enters.len(), 1);
    assert_eq!(
        app.events.pointer.buttons,
        [
            (0x110, wl_pointer::ButtonState::Pressed),
            (0x110, wl_pointer::ButtonState::Released),
        ]
    );
    assert!(app.events.pointer.axes.is_empty());
    Ok(())
}

#[test]
fn direct_pointer_drag_pause_reactivation_and_disconnect_follow_wire_lifetimes()
-> Result<(), InputError> {
    let (f, mut app) = setup();
    route(
        &f,
        DirectPointerEvent::Absolute {
            x: 0.0,
            y: 0.0,
            time: 1,
        },
    )?;
    route(
        &f,
        DirectPointerEvent::Button {
            code: 0x110,
            state: ButtonState::Pressed,
            time: 2,
        },
    )?;
    route(
        &f,
        DirectPointerEvent::Relative {
            dx: 1000.0,
            dy: 1000.0,
            time: 3,
        },
    )?;
    app.sync();
    assert_eq!(app.events.pointer.motion.last(), Some(&(31.0, 31.0)));
    // Pause ignores even malformed pending input, releasing the implicit drag.
    f.backend(|s| {
        s.direct_pointer(
            false,
            (0, 0),
            Some(DirectPointerEvent::Relative {
                dx: f64::NAN,
                dy: 0.0,
                time: 4,
            }),
        )
    })?;
    f.backend(|s| s.direct_pointer(true, (32, 32), None))?;
    route(
        &f,
        DirectPointerEvent::Button {
            code: 0x110,
            state: ButtonState::Pressed,
            time: 5,
        },
    )?;
    route(
        &f,
        DirectPointerEvent::Axis(AxisFrame::new(5).value(Axis::Vertical, 15.0)),
    )?;
    // **And let go of it.** The pointer was carried to (31, 31) by the relative
    // motion above, which the implicit grab delivered to the client but which is
    // outside the frame — it is on the frame's own bottom-right corner band, so
    // since ADR 0071 that press begins a resize rather than reaching anybody. The
    // press staying undelivered is what this test already asserted; the drag it
    // now starts would hold the pointer for the rest of the test, and this test is
    // about wire lifetimes rather than about resizing.
    route(
        &f,
        DirectPointerEvent::Button {
            code: 0x110,
            state: ButtonState::Released,
            time: 5,
        },
    )?;
    app.sync();
    assert_eq!(app.events.pointer.enters.len(), 1);
    assert_eq!(app.events.pointer.leaves, 1);
    assert_eq!(app.events.pointer.buttons.len(), 2);
    assert!(app.events.pointer.axes.is_empty());
    // That scroll landed on empty canvas, so it panned the plane — the canvas
    // plan's task 5, held by `tests/panning_the_plane`. This test is about the
    // pointer's wire lifetimes on the direct backend and every assertion in it is
    // about a surface-local coordinate, so the camera is put back rather than the
    // coordinates below being rewritten around a pan nobody here is testing.
    assert!(f.backend(|s| s.look_at_the_canvas(alo_canvas::At::origin()).is_some()));
    route(
        &f,
        DirectPointerEvent::Absolute {
            x: 0.25,
            y: 0.25,
            time: 6,
        },
    )?;
    route(
        &f,
        DirectPointerEvent::Axis(
            AxisFrame::new(7)
                .source(AxisSource::Wheel)
                .value(Axis::Vertical, 15.0)
                .v120(Axis::Vertical, 120),
        ),
    )?;
    app.sync();
    assert_eq!(
        app.events.pointer.enters.last().map(|(_, x, y)| (*x, *y)),
        Some((7.75, 7.75))
    );
    assert_eq!(
        app.events.pointer.axes,
        [(wl_pointer::Axis::VerticalScroll, 15.0)]
    );
    drop(app);
    f.wait_for((0, 0));
    route(
        &f,
        DirectPointerEvent::Relative {
            dx: 1.0,
            dy: 1.0,
            time: 8,
        },
    )?;
    route(
        &f,
        DirectPointerEvent::Button {
            code: 0x110,
            state: ButtonState::Pressed,
            time: 9,
        },
    )?;
    let mut replacement = Application::new(&f);
    replacement.configure();
    replacement.attach();
    replacement.sync();
    route(
        &f,
        DirectPointerEvent::Button {
            code: 0x110,
            state: ButtonState::Released,
            time: 10,
        },
    )?;
    replacement.sync();
    assert!(replacement.events.pointer.enters.is_empty());
    assert!(replacement.events.pointer.buttons.is_empty());
    Ok(())
}

#[test]
fn direct_pointer_requires_an_enabled_pointer() {
    let f = Fixture::keyboard();
    assert!(matches!(
        f.backend(|s| s.direct_pointer(true, (32, 32), None)),
        Err(InputError::PointerUnavailable)
    ));
    assert!(matches!(
        f.backend(|s| s.direct_pointer(false, (32, 32), None)),
        Err(InputError::PointerUnavailable)
    ));
}

/// **A jump far past the corner leaves the pointer free to move again.**
///
/// On 2026-11-10 a walk on a booted image moved the pointer by -8000, -8000 and
/// saw it stay drawn at the top-left corner through every later move, which
/// read as the shell's position being poisoned by one huge delta. It was not:
/// qemu's PS/2 mouse carries what it cannot send yet and pays it out 508 units
/// per input sync, so the moves that followed reached the guest as more
/// movement into the corner (`docs/quirks/qemu-ps2-carries-a-huge-move-forward.md`).
/// This holds the half the shell owns: a delta of any size is clamped to the
/// output and the next ordinary one moves from the edge.
#[test]
fn a_huge_relative_jump_is_clamped_and_the_next_move_still_moves() -> Result<(), InputError> {
    let (f, mut app) = setup();
    for (dx, dy, time) in [(-8000.0, -8000.0, 1), (8.0, 6.0, 2), (2.5, 1.0, 3)] {
        route(&f, DirectPointerEvent::Relative { dx, dy, time })?;
    }
    app.sync();
    // The jump puts the pointer on the corner, which is on the surface, and the
    // next two moves reach the client as motion from there.
    assert_eq!(
        app.events.pointer.enters.last().map(|(_, x, y)| (*x, *y)),
        Some((0.0, 0.0))
    );
    assert_eq!(app.events.pointer.motion, [(8.0, 6.0), (10.5, 7.0)]);
    // And past the far corner: the output is 32 wide, so a jump of +8000 stops at
    // 31, off the client's 16 by 16 surface at the origin, so it leaves; coming
    // back by 20 lands at 11, which enters it again there. Measured, not
    // assumed: an earlier version of this test asserted a motion here and failed.
    route(
        &f,
        DirectPointerEvent::Relative {
            dx: 8000.0,
            dy: 8000.0,
            time: 4,
        },
    )?;
    route(
        &f,
        DirectPointerEvent::Relative {
            dx: -20.0,
            dy: -20.0,
            time: 5,
        },
    )?;
    app.sync();
    assert_eq!(app.events.pointer.leaves, 1);
    assert_eq!(
        app.events
            .pointer
            .enters
            .iter()
            .map(|(_, x, y)| (*x, *y))
            .collect::<Vec<_>>(),
        [(0.0, 0.0), (11.0, 11.0)]
    );
    Ok(())
}
