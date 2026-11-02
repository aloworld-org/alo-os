//! **A window's edge stays on its window when the canvas pans and zooms.**
//!
//! Found by reading the code on 2026-11-01: every road that lays an edge out —
//! the draw, a press, a press in the menu, the pointer revealing it, the
//! keyboard arriving on it — read the window's rectangle on the **plane** as
//! though it were on the screen. The window itself is drawn through the camera,
//! so the moment a person panned or zoomed, the window moved and its edge, line
//! and shadow stayed where it had been at 100 %.
//!
//! Asked here of the reveal, which is a road a test can drive end to end with a
//! real pointer: with titles shown only when pointed at, pointing at where the
//! edge is drawn reveals it, and pointing at where it used to be does not.
//! `every_windows_edge` reads the same `Server::where_a_window_is_drawn`, which
//! a guard in that file's own tests holds.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure this test reports"
)]

use super::support::{Application, Fixture, looking, motion, on_the_screen};

/// The viewport every test here renders to.
const VIEWPORT: (i32, i32) = (1280, 720);

/// Where the window is put on the plane.
const PLACED: (i32, i32) = (400, 300);

/// How far above a window's top the pointer is held: inside the 44-tall region
/// the edge occupies above it (`alo_shell::window_edge::THE_REGION_IS_TALL`).
const ABOVE_THE_WINDOW: f64 = 20.0;

/// A display with a pointer and an output, titles shown only when pointed at,
/// and one window placed on the plane.
fn a_window_on_the_canvas() -> (Fixture, Application) {
    let f = Fixture::keyboard();
    assert!(f.backend(|s| s.enable_pointer()).is_ok());
    f.render(VIEWPORT, false, 1).expect("an output to look at");
    let mut app = Application::new(&f);
    app.configure();
    app.attach();
    app.sync();
    f.backend(|s| s.the_window_titles_are(alo_appearance::WindowTitles::WhenPointedAt));
    let root = f.root();
    assert!(f.backend(move |s| s.place_window(&root, PLACED)).is_ok());
    (f, app)
}

/// Whether pointing here reveals the window's edge.
fn pointing_here_reveals_it(f: &Fixture, at: (f64, f64)) -> bool {
    motion(f, at);
    let root = f.root();
    f.backend(move |s| {
        s.the_edges_were_told_where_the_pointer_is();
        s.is_this_windows_edge_revealed(&root)
    })
}

/// **Panned and zoomed, the edge is where the window is drawn**, and not
/// where the window would be at 100 % with the canvas unmoved.
#[test]
fn a_windows_edge_is_revealed_where_the_window_is_drawn_at_every_pan_and_zoom() {
    let (f, _app) = a_window_on_the_canvas();
    let unmoved = (
        f64::from(PLACED.0) + 8.0,
        f64::from(PLACED.1) - ABOVE_THE_WINDOW,
    );
    for (at, zoom) in [((-300, -200), 1000), ((150, 100), 500), ((300, 250), 2500)] {
        looking(&f, at, zoom);
        let corner = on_the_screen((0.0, 0.0), PLACED, at, zoom);
        let wide = 16.0 * f64::from(zoom) / 1000.0;
        let over_the_edge = (corner.0 + wide / 2.0, corner.1 - ABOVE_THE_WINDOW);
        assert!(
            pointing_here_reveals_it(&f, over_the_edge),
            "looking from {at:?} at {zoom} thousandths, pointing above the window as drawn \
             at {over_the_edge:?} did not reveal its edge"
        );
        assert!(
            !pointing_here_reveals_it(&f, unmoved),
            "looking from {at:?} at {zoom} thousandths, the edge was revealed where the window \
             would be with the canvas unmoved, so it stayed behind when the window moved"
        );
    }
}

/// **The control: with the canvas unmoved, the two places are one**, so the
/// test above is about the camera and not about the pointer.
#[test]
fn with_the_canvas_unmoved_the_edge_is_where_it_always_was() {
    let (f, _app) = a_window_on_the_canvas();
    looking(&f, (0, 0), 1000);
    let over = (
        f64::from(PLACED.0) + 8.0,
        f64::from(PLACED.1) - ABOVE_THE_WINDOW,
    );
    assert!(pointing_here_reveals_it(&f, over));
    assert!(
        !pointing_here_reveals_it(&f, (5.0, 5.0)),
        "pointing far from the window revealed its edge, so nothing here measures anything"
    );
}
