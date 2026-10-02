//! **`wp_viewporter`: a surface says how big it is drawn** — one of the protocols
//! `alo-shell` did not speak.
//!
//! A client gives a buffer and a destination size, and the compositor draws the
//! buffer at that size. Without it a surface is exactly as big as its buffer, so
//! a client that wants to draw a 16×16 buffer across 200 logical pixels has to
//! allocate 200×200 — which is what *scaling* means for everything from a video
//! player to a browser on a fractional-scale display.
//!
//! # Why wiring the global is the whole of it here, and why that is not a general rule
//!
//! The other lane's warning on these six protocols is right and is the reason
//! this file exists: **a global is not a protocol.** `wl_data_device` can be
//! bound and still never hold a selection, because something has to follow the
//! keyboard focus; a protocol advertised and unreachable is the same shape as a
//! rule with no caller.
//!
//! So the question was asked here too, and the answer is different — **measured
//! in smithay's source rather than assumed.** `on_commit_buffer_handler` reads
//! `ViewportCachedState` itself, validates it with `ensure_viewport_valid`, and
//! writes the result into `SurfaceView { src, dst, offset }`. `crate::scene`'s
//! `tree_bounds` already reads `view.dst`. **The effect was already plumbed; only
//! the global was missing**, which is the opposite of the data-device case and is
//! exactly why it had to be checked rather than reasoned about.
//!
//! These tests assert the effect rather than the advertisement, because
//! *advertised* is what was already provable and is not what a person gets.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture};
use wayland_client::protocol::wl_surface;

/// The viewport this test renders to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// A real display with an output.
fn fixture() -> Fixture {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    f
}

/// The frame this compositor sees, if one is mapped.
fn the_only_frame(f: &Fixture) -> Option<alo_canvas::Frame> {
    f.backend(|s| s.the_frames_on_the_plane().into_iter().next())
}

/// **The protocol is advertised**, which is necessary and is not the point.
///
/// Asserted by binding it rather than by reading a list: a client that can get a
/// viewport is the only evidence that matters, and `set_a_viewport_destination`
/// panics if the global is absent.
#[test]
fn the_compositor_offers_a_viewporter() {
    let f = fixture();
    let mut app = Application::new(&f);
    app.configure();
    app.attach();
    app.sync();
    assert!(
        app.set_a_viewport_destination(32, 32),
        "wp_viewporter is not advertised, so no client can scale a surface"
    );
    app.sync();
    assert!(
        app.connection.protocol_error().is_none(),
        "binding wp_viewporter and asking for a viewport was refused"
    );
}

/// **A surface with no viewport is as big as its buffer**, which is the state
/// this protocol exists to change and the baseline the next test is read against.
#[test]
fn without_a_viewport_a_surface_is_the_size_of_its_buffer() {
    let f = fixture();
    let mut app = Application::new(&f);
    app.configure();
    app.attach();
    app.sync();

    let frame = the_only_frame(&f).expect("one frame is mapped");
    assert_eq!(
        (frame.size().width(), frame.size().height()),
        (16, 16),
        "the fixture's 16x16 buffer was not drawn at its own size"
    );
}

/// **A destination size changes how big the surface is drawn, without changing
/// the buffer.**
///
/// The acceptance of this protocol in one assertion: the same 16×16 buffer, drawn
/// across 200×120, because the client said so. Before `delegate_viewporter!` the
/// client could not even bind the global to ask.
#[test]
fn a_destination_size_is_what_the_surface_is_drawn_at() {
    let f = fixture();
    let mut app = Application::new(&f);
    app.configure();
    app.attach();
    app.sync();
    assert_eq!(the_only_frame(&f).expect("mapped").size().width(), 16);

    assert!(app.set_a_viewport_destination(200, 120), "no viewporter");
    app.sync();

    let frame = the_only_frame(&f).expect("one frame is still mapped");
    assert_eq!(
        (frame.size().width(), frame.size().height()),
        (200, 120),
        "the destination size was not what the surface was drawn at"
    );
}

/// **Taking the viewport away puts the surface back to its buffer's size.**
///
/// A destination is a thing a client sets and unsets while it runs — a video
/// leaving full screen is exactly this — so a viewport that could only be
/// applied once would be a surface stuck at whatever size it was first given.
#[test]
fn clearing_the_destination_returns_the_surface_to_its_buffer() {
    let f = fixture();
    let mut app = Application::new(&f);
    app.configure();
    app.attach();
    app.sync();

    assert!(app.set_a_viewport_destination(200, 120), "no viewporter");
    app.sync();
    assert_eq!(the_only_frame(&f).expect("mapped").size().width(), 200);

    assert!(app.clear_the_viewport_destination(), "no viewporter");
    app.sync();

    let frame = the_only_frame(&f).expect("one frame is still mapped");
    assert_eq!(
        (frame.size().width(), frame.size().height()),
        (16, 16),
        "clearing the destination did not return the surface to its buffer's size"
    );
}

/// Keep the import used when the harness changes shape.
#[allow(dead_code)]
fn _surface_type(_: &wl_surface::WlSurface) {}
