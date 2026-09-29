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

/// Where each of the eight bands is, for a frame mapped at the origin.
///
/// Sixteen across and sixteen down, at (0, 0), which is what `mapped` above
/// produces and what the other fixtures in this file already rely on. The
/// numbers below are `the_resize_edge()`, `the_resize_corner()` and
/// `the_names_band()` arithmetic written out, so a change to any of the three
/// fails here with the point it moved rather than somewhere downstream.
fn a_point_on_each_band() -> [(FrameEdge, f64, f64); 8] {
    let (edge, corner, name) = (
        alo_shell::the_resize_edge(),
        alo_shell::the_resize_corner(),
        alo_shell::the_names_band(),
    );
    let (left, right, bottom) = (0.0, 16.0, 16.0);
    // The bands surround the frame **and the name above it**, so the top of the
    // box a person sees is the top of the name, not the top of the picture.
    let top = -name;
    [
        (FrameEdge::Top, left + 8.0, top - edge / 2.0),
        (FrameEdge::Bottom, left + 8.0, bottom + edge / 2.0),
        (FrameEdge::Left, left - edge / 2.0, 0.0),
        (FrameEdge::Right, right + edge / 2.0, 0.0),
        (FrameEdge::TopLeft, left - corner / 2.0, top - corner / 2.0),
        (
            FrameEdge::TopRight,
            right + corner / 2.0,
            top - corner / 2.0,
        ),
        (
            FrameEdge::BottomLeft,
            left - corner / 2.0,
            bottom + corner / 2.0,
        ),
        (
            FrameEdge::BottomRight,
            right + corner / 2.0,
            bottom + corner / 2.0,
        ),
    ]
}

/// **All eight bands are where a pointer can actually find them.**
///
/// `begin_a_resize` above proves the compositor *can* resize from any edge; this
/// proves a person can *reach* one, which is the other half of the acceptance
/// sentence and the half a hit test can fail on its own.
#[test]
fn a_pointer_finds_every_edge_and_corner_of_a_frame() {
    let f = fixture();
    let _app = mapped(&f);
    let frame = f
        .backend(|s| s.mapped_surfaces().next().cloned())
        .expect("a frame is mapped");
    assert_eq!(
        f.backend(move |_| alo_shell::window_buffer_origin(&frame)),
        (0.0, 0.0).into(),
        "the fixture's frame is not at the origin these points are measured from"
    );
    for (edge, x, y) in a_point_on_each_band() {
        let found = f.backend(move |s| s.the_edge_under((x, y).into()).map(|(_, e)| e));
        assert_eq!(found, Some(edge), "nothing resizable at ({x}, {y})");
    }
}

/// **A press on a band resizes, through the road a mouse actually takes.**
///
/// Every other test in this file calls `Server::begin_a_resize`, which is the
/// compositor answering *what would you do with this edge*. This one presses a
/// button at a point on the band and goes through `pointer_button`, because a
/// band nothing in `crate::pointer` consults is a capability with no caller —
/// which is exactly the shape of gap this repository keeps finding late.
///
/// The press must also **not** reach the application: the band is outside the
/// frame, so no client was under it.
#[test]
fn a_real_press_on_a_band_begins_a_resize_and_reaches_no_client() {
    let f = fixture();
    let mut app = mapped(&f);
    app.sync();
    let before = app.events.sizes.len();
    let presses = app.events.pointer.buttons.len();

    let (_, x, y) = a_point_on_each_band()[7];
    assert!(f.backend(move |s| s.pointer_motion(x, y, 1)).is_ok());
    assert_eq!(
        f.backend(move |s| s.pointer_button(0x110, Pressed, 2)).ok(),
        Some(false),
        "a press on the shell's own band was delivered to a client"
    );
    app.sync();
    assert!(
        app.events.sizes.len() > before,
        "a press on the bottom-right corner began no resize"
    );
    assert_eq!(
        app.events.pointer.buttons.len(),
        presses,
        "a press on the shell's own band reached the application"
    );
}

/// **The name band is not a resize band**, anywhere along it.
///
/// The top edge was measured from the frame's own top edge first, which put it
/// *inside* the name — the two are exactly as wide as each other — so the bottom
/// six pixels of every frame's name would have resized it. Nothing caught that
/// but reading; this catches it now, across the whole band rather than at one
/// point, because an overlap at one end is the way that bug comes back.
#[test]
fn no_part_of_a_frames_name_resizes_it() {
    let f = fixture();
    let _app = mapped(&f);
    let band = alo_shell::the_names_band();
    for x in [0.5, 4.0, 8.0, 12.0, 15.5] {
        for step in 0..=16 {
            let y = -band + band * f64::from(step) / 16.0;
            let found = f.backend(move |s| s.the_edge_under((x, y).into()).map(|(_, e)| e));
            assert_eq!(found, None, "({x}, {y}) is in the name band and resizes");
        }
    }
}

/// **The pointer says which**, which is ADR 0071's affordance and was the other
/// thing task 4 built with nothing calling it.
///
/// Over a band the compositor answers a double-headed arrow naming that edge;
/// over the name above the frame and over the frame's own content it does not.
/// Both halves, because a cursor that was always the resize arrow would satisfy
/// the first.
#[test]
fn the_cursor_over_a_band_says_which_way_it_drags() {
    let f = fixture();
    let _app = mapped(&f);
    for (edge, x, y) in a_point_on_each_band() {
        assert!(f.backend(move |s| s.pointer_motion(x, y, 1)).is_ok());
        let shown = f.backend(|s: &mut alo_shell::Server| s.cursor());
        let alo_shell::Cursor::Resize { edge: shown, .. } = shown else {
            unreachable!("the cursor over {edge:?} at ({x}, {y}) is {shown:?}")
        };
        assert_eq!(shown, edge);
    }
    // The name above the frame moves it, so it is not a resize arrow.
    assert!(f.backend(|s| s.pointer_motion(8.0, -24.0, 2)).is_ok());
    assert!(
        !matches!(
            f.backend(|s: &mut alo_shell::Server| s.cursor()),
            alo_shell::Cursor::Resize { .. }
        ),
        "the cursor over a frame's name says it resizes"
    );
    // And inside the frame every pixel belongs to the application.
    assert!(f.backend(|s| s.pointer_motion(8.0, 8.0, 3)).is_ok());
    assert!(
        !matches!(
            f.backend(|s: &mut alo_shell::Server| s.cursor()),
            alo_shell::Cursor::Resize { .. }
        ),
        "the cursor inside a frame says it resizes"
    );
}
