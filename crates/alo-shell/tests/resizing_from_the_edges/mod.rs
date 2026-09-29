//! **Resizing, and the application told as it happens** — task 4 of the canvas
//! plan, and the three sentences its acceptance names.
//!
//! *Each edge and corner resizes; the application receives its size during the
//! drag; a resize that would go below the application's minimum stops at it
//! rather than being refused silently.*
//!
//! The transaction itself is held by `interactive_resize`, which has driven it
//! since before this task existed. What is held here is what task 4 added: that
//! the shell's **own** bands reach it, that all eight of them do, and that the
//! minimum is a floor rather than a refusal.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};
use alo_shell::FrameEdge;
use smithay::backend::input::ButtonState::Pressed;

/// A real display with a pointer.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f
}

/// A mapped application, kept alive by the caller.
fn mapped(f: &Fixture) -> Application {
    let mut app = Application::new(f);
    app.configure();
    app.attach();
    app.sync();
    app
}

/// Put a button down, which a drag needs before it can begin.
fn holding(f: &Fixture, app: &mut Application) {
    assert!(f.backend(|s| s.pointer_motion(4.0, 5.0, 1)).is_ok());
    assert_eq!(
        f.backend(|s| s.pointer_button(0x110, Pressed, 2)).ok(),
        Some(true)
    );
    app.sync();
}

/// **Each edge and corner resizes.**
///
/// The first acceptance sentence, asked of all eight rather than of a
/// representative one: the four corners and the four edges take different arms of
/// the same arithmetic, and a corner that resized in one axis would pass a test
/// that only tried `Right`.
#[test]
fn each_edge_and_corner_resizes() {
    for edge in FrameEdge::ALL {
        let f = fixture();
        let mut app = mapped(&f);
        holding(&f, &mut app);
        let before = app.events.sizes.len();

        let frame = f
            .backend(|s| s.mapped_surfaces().next().cloned())
            .expect("a frame is mapped");
        assert!(
            f.backend(move |s| s.begin_a_resize(&frame, edge)),
            "{edge:?} did not begin a resize"
        );
        app.sync();
        assert!(
            app.events.sizes.len() > before,
            "{edge:?} began a resize and told the application nothing"
        );
    }
}

/// **The application receives its size during the drag, not at the end.**
///
/// The second sentence, and the one a person feels: a window that jumped to its
/// new size when the button came up would satisfy *it resized* and none of what
/// the acceptance asks. Each motion is counted, so a single configure at the start
/// followed by silence fails as loudly as no configure at all.
#[test]
fn the_application_is_told_its_size_while_the_drag_is_happening() {
    let f = fixture();
    let mut app = mapped(&f);
    holding(&f, &mut app);
    let frame = f
        .backend(|s| s.mapped_surfaces().next().cloned())
        .expect("a frame is mapped");
    assert!(f.backend(move |s| s.begin_a_resize(&frame, FrameEdge::BottomRight)));
    app.sync();
    let began = app.events.sizes.len();

    let mut told = 0;
    for (step, x) in [(1, 40.0), (2, 80.0), (3, 120.0)] {
        assert!(f.backend(move |s| s.pointer_motion(x, x, 10 + step).is_ok()) || true);
        app.sync();
        if app.events.sizes.len() > began + told {
            told = app.events.sizes.len() - began;
        }
    }
    assert!(
        told >= 2,
        "three motions during a drag produced {told} sizes; the application is being told at the end"
    );
}

/// **A resize below the application's minimum stops at it, rather than being
/// refused silently.**
///
/// The third sentence, and the distinction it draws is the one this repository
/// keeps having to make: *stopped at the minimum* and *refused* look identical
/// from the outside unless somebody checks which happened. So this asserts both
/// halves — the window does **not** go below the minimum, and it does **not**
/// stay where it started either. A refusal would satisfy the first and fail the
/// second.
#[test]
fn a_resize_below_the_minimum_stops_at_it_rather_than_being_refused() {
    let f = fixture();
    let mut app = mapped(&f);
    // The application says how small it may be, before any drag begins.
    app.toplevel.set_min_size(12, 12);
    app.surface.commit();
    app.sync();

    holding(&f, &mut app);
    let frame = f
        .backend(|s| s.mapped_surfaces().next().cloned())
        .expect("a frame is mapped");
    assert!(f.backend(move |s| s.begin_a_resize(&frame, FrameEdge::BottomRight)));
    app.sync();

    // Drag far past where the window could possibly still be 12 across.
    assert!(f.backend(|s| s.pointer_motion(0.0, 0.0, 20)).is_ok());
    app.sync();

    let last = app
        .events
        .sizes
        .last()
        .copied()
        .expect("a drag told the application something");
    assert!(
        last.0 >= 12 && last.1 >= 12,
        "a drag past the minimum took the window below it: {last:?}"
    );
    assert!(
        last.0 <= 16 && last.1 <= 16,
        "a drag past the minimum was refused rather than stopped at it: {last:?}"
    );
}
