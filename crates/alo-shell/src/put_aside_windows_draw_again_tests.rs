//! When a window put aside is asked to draw again, who is asked — and **what a
//! live miniature costs the compositor on the software renderer**, measured
//! every time these tests run.

use std::time::{Duration, Instant};

use drm::buffer::DrmFourcc;
use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::pixman::PixmanRenderer;
use smithay::backend::renderer::{Bind, Frame, ImportMem, Offscreen, Renderer};
use smithay::utils::{Physical, Rectangle, Transform};

use super::{THEY_DRAW_AGAIN_EVERY, it_is_time};

/// **Four times a second**: asked at once the first time, then not again
/// until a quarter of a second has passed.
#[test]
fn a_window_put_aside_is_asked_four_times_a_second() {
    assert_eq!(THEY_DRAW_AGAIN_EVERY, Duration::from_millis(250));
    let now = Instant::now();
    assert!(it_is_time(None, now), "never asked: ask now");
    assert!(!it_is_time(Some(now), now + Duration::from_millis(100)));
    assert!(!it_is_time(Some(now), now + Duration::from_millis(249)));
    assert!(it_is_time(Some(now), now + Duration::from_millis(250)));
    // A clock that went backwards asks nothing early.
    assert!(!it_is_time(Some(now + Duration::from_secs(1)), now));
}

/// **The refusal path: with no panel nobody is asked**, and an empty panel
/// asks nobody.
#[test]
fn with_no_panel_nobody_is_asked() {
    use std::os::unix::fs::PermissionsExt as _;
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server =
        crate::Server::bind(runtime.path(), "draw-again-no-panel").expect("a display");
    assert_eq!(server.the_put_aside_windows_draw_again(None, Instant::now()), 0);
    assert_eq!(
        server.the_put_aside_windows_draw_again(
            Some(&alo_put_aside::Panel::new()),
            Instant::now() + Duration::from_secs(1)
        ),
        0
    );
}

/// **What eight live miniatures cost the compositor each frame, on the
/// processor alone.** A full-HD window's picture is drawn scaled into a
/// miniature's 42 × 24 by pixman — the renderer of a machine with no graphics
/// card, and the road `crate::a_window_in_miniature` takes there — eight times,
/// a full rail's worth, and the time is printed and held under a ceiling.
///
/// The ceiling is **two milliseconds for all eight**, an eighth of a 60 Hz
/// frame: generous on purpose, so the test catches a change that makes the
/// miniatures cost as much as the screen, not a slow build machine. What it
/// does not measure is the application's own drawing four times a second,
/// which is the application's processor and not this compositor's.
#[test]
fn eight_miniatures_cost_the_software_renderer_a_small_share_of_a_frame() {
    const WINDOW: (i32, i32) = (1920, 1080);
    const MINIATURES: i32 = 8;
    const ROUNDS: u32 = 30;
    let mut renderer = PixmanRenderer::new().expect("pixman, the software renderer");
    let pixels: Vec<u8> = (0..WINDOW.0 * WINDOW.1 * 4)
        .map(|at| u8::try_from(at % 251).unwrap_or_default())
        .collect();
    let window = renderer
        .import_memory(&pixels, Fourcc::Abgr8888, WINDOW.into(), false)
        .expect("the window's picture");
    let mut buffer = renderer
        .create_buffer(DrmFourcc::Abgr8888, (112, 600).into())
        .expect("a panel's column to draw into");
    let mut target = renderer.bind(&mut buffer).expect("bound");
    let started = Instant::now();
    for _ in 0..ROUNDS {
        let mut frame = renderer
            .render(&mut target, (112, 600).into(), Transform::Normal)
            .expect("a frame");
        for which in 0..MINIATURES {
            let area: Rectangle<i32, Physical> =
                Rectangle::new((35, 30 + which * 56).into(), (42, 24).into());
            crate::painted::wash_scaled(&mut frame, &window, WINDOW, area)
                .expect("a miniature drawn");
        }
        let _ = frame.finish().expect("finished");
    }
    let each_frame = started.elapsed() / ROUNDS;
    eprintln!(
        "MEASURED: {MINIATURES} miniatures of a {}x{} window, pixman: {} microseconds a frame",
        WINDOW.0,
        WINDOW.1,
        each_frame.as_micros()
    );
    assert!(
        each_frame < Duration::from_millis(2),
        "eight miniatures took {each_frame:?} a frame on the software renderer"
    );
}
