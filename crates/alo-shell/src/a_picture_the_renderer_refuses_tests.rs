//! **Why pixman refused Firefox's miniature, measured on pixman every run**,
//! and the refusal path: the renderer refuses a picture, the frame is still
//! finished, the log hears of it once, and the window's square is drawn by its
//! application's mark.

use drm::buffer::DrmFourcc;
use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::pixman::PixmanRenderer;
use smithay::backend::renderer::{Bind, Frame, ImportMem, Offscreen, Renderer};
use smithay::reexports::wayland_server::{
    Client, DataInit, Dispatch, Display, DisplayHandle, Resource, protocol::wl_surface,
    protocol::wl_surface::WlSurface,
};
use smithay::utils::{Physical, Rectangle, Transform};

use super::{ARefusal, TheRefusedPictures, WhatTheRendererRefused, WhereItWasGoing, noted};

/// The screen the faults were seen on.
const SCREEN: (i32, i32) = (1280, 800);

/// Surfaces with identities and nothing else: never dispatched.
struct Identities;
impl Dispatch<WlSurface, ()> for Identities {
    fn request(
        _: &mut Self,
        _: &Client,
        surface: &WlSurface,
        _: wl_surface::Request,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        surface.post_error(0u32, "an identity fixture takes no requests");
    }
}

/// `how_many` distinct surfaces, and what keeps them alive.
fn surfaces(how_many: usize) -> (Display<Identities>, Vec<WlSurface>) {
    let display = Display::<Identities>::new().expect("a display");
    let mut handle = display.handle();
    let (socket, peer) = std::os::unix::net::UnixStream::pair().expect("a socket pair");
    std::mem::forget(peer);
    let client = handle
        .insert_client(socket, std::sync::Arc::new(()))
        .expect("a client");
    let made = (0..how_many)
        .map(|_| {
            client
                .create_resource::<WlSurface, (), Identities>(&handle, 1, ())
                .expect("a surface")
        })
        .collect();
    (display, made)
}

/// **Where a window `size` big is drawn in the first square of the panel**
/// on the screen the faults were seen on, laid out by the panel's own code.
fn where_its_miniature_goes(size: (i32, i32)) -> Rectangle<i32, Physical> {
    let look = crate::desktop_testing::noon_look(
        &crate::desktop_testing::an_appearance(),
        alo_strings::Direction::LeftToRight,
    );
    let mut panel = alo_put_aside::Panel::new();
    let window = alo_dock::Window::of(
        alo_dock::WindowId::numbered(7),
        Some(alo_dock::AppId::named("org.mozilla.firefox").expect("a name")),
        "Firefox",
        alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 800, 600).expect("a patch"),
        alo_dock::HowItSits::OnTheCanvas,
    );
    panel
        .put_aside(
            &window,
            alo_canvas::Zoom::LIFE_SIZE,
            alo_canvas::Place::FIRST,
            alo_put_aside::whether_it_is_private::Privacy::Ordinary,
        )
        .expect("put aside");
    let laid_out = crate::panel_raster::picture(
        &panel,
        look,
        SCREEN,
        crate::panel_raster::the_edge_for(look.reading()),
        crate::panel_raster::WhetherRevealed::Revealed,
        crate::display_scale::DisplayScale::of_per_cent(100),
    )
    .expect("the panel laid out");
    let slot = *laid_out.slots.first().expect("a square for the window");
    let parts = crate::a_window_in_miniature::in_the_slot(slot);
    let (picture, _) =
        crate::a_window_in_miniature::the_picture_in(parts, (f64::from(size.0), f64::from(size.1)))
            .expect("a picture");
    picture
}

/// Draw a picture `size` big into `onto` on a frame the size of the screen,
/// with pixman, and answer what pixman said.
fn drawn_by_pixman(size: (i32, i32), onto: Rectangle<i32, Physical>) -> Result<(), String> {
    let mut renderer = PixmanRenderer::new().expect("pixman, the software renderer");
    let pixels = vec![0x80_u8; usize::try_from(size.0 * size.1 * 4).expect("a size")];
    let window = renderer
        .import_memory(&pixels, Fourcc::Abgr8888, size.into(), false)
        .expect("the window's picture");
    let mut buffer = renderer
        .create_buffer(DrmFourcc::Abgr8888, SCREEN.into())
        .expect("a screen to draw into");
    let mut target = renderer.bind(&mut buffer).expect("bound");
    let mut frame = renderer
        .render(&mut target, SCREEN.into(), Transform::Normal)
        .expect("a frame");
    let drawn = frame
        .render_texture_from_to(
            &window,
            Rectangle::from_size((f64::from(size.0), f64::from(size.1)).into()),
            onto,
            &[Rectangle::from_size(onto.size)],
            &[],
            Transform::Normal,
            1.0,
        )
        .map_err(|why| why.to_string());
    let _ = frame.finish().expect("the frame is still finished");
    drawn
}

/// **The cause, measured.** A window as wide as the screen, drawn small in the
/// panel's first square, is refused by pixman with the sentence the desktop
/// ended on; Dolphin's size, in the same square, is drawn; and the same wide
/// window drawn as small near the top-left corner is drawn — so it is where
/// the picture goes times how much smaller it is, not the size alone.
#[test]
fn pixman_refuses_a_wide_window_drawn_small_at_the_right_hand_edge() {
    let firefox = SCREEN;
    let dolphin = (760, 586);
    let in_the_panel = where_its_miniature_goes(firefox);
    eprintln!(
        "MEASURED: a {}x{} window's miniature is {}x{} at ({}, {}), {:.1} times smaller",
        firefox.0,
        firefox.1,
        in_the_panel.size.w,
        in_the_panel.size.h,
        in_the_panel.loc.x,
        in_the_panel.loc.y,
        f64::from(firefox.0) / f64::from(in_the_panel.size.w)
    );
    assert_eq!(
        drawn_by_pixman(firefox, in_the_panel),
        Err("The requested operation is not supported".to_owned()),
        "a screen-wide window drawn in the panel"
    );
    assert_eq!(
        drawn_by_pixman(dolphin, where_its_miniature_goes(dolphin)),
        Ok(()),
        "Dolphin's size drawn in the panel"
    );
    let near_the_corner = Rectangle::new((35, 30).into(), in_the_panel.size);
    assert_eq!(
        drawn_by_pixman(firefox, near_the_corner),
        Ok(()),
        "the same window drawn as small near the corner"
    );
}

/// **The refusal path: the renderer refuses a picture, it is noted, and the
/// frame is drawn on and finished** — the same frame then draws a picture it
/// can, which is the whole difference from 2026-11-12.
#[test]
fn a_refused_picture_is_noted_and_the_frame_goes_on() {
    let (_display, made) = surfaces(1);
    let firefox = made.first().expect("a surface").clone();
    let refused = WhatTheRendererRefused::default();
    let mut renderer = PixmanRenderer::new().expect("pixman");
    let big = vec![0x80_u8; usize::try_from(SCREEN.0 * SCREEN.1 * 4).expect("a size")];
    let small = vec![0xff_u8; 16 * 16 * 4];
    let wide = renderer
        .import_memory(&big, Fourcc::Abgr8888, SCREEN.into(), false)
        .expect("a wide picture");
    let square = renderer
        .import_memory(&small, Fourcc::Abgr8888, (16, 16).into(), false)
        .expect("a small picture");
    let mut buffer = renderer
        .create_buffer(DrmFourcc::Abgr8888, SCREEN.into())
        .expect("a screen");
    let mut target = renderer.bind(&mut buffer).expect("bound");
    let mut frame = renderer
        .render(&mut target, SCREEN.into(), Transform::Normal)
        .expect("a frame");
    let onto = where_its_miniature_goes(SCREEN);
    if let Err(why) = frame.render_texture_from_to(
        &wide,
        Rectangle::from_size((f64::from(SCREEN.0), f64::from(SCREEN.1)).into()),
        onto,
        &[Rectangle::from_size(onto.size)],
        &[],
        Transform::Normal,
        1.0,
    ) {
        let why = || crate::RenderError::Submission(why.to_string());
        // **Where nothing will act on it, it is raised**, as before: the
        // nested lane and the offscreen checks read the refusal.
        assert!(
            noted(None, &firefox, WhereItWasGoing::InTheMiniature, why()).is_err(),
            "a refusal with nobody to note it in was swallowed"
        );
        noted(
            Some(&refused),
            &firefox,
            WhereItWasGoing::InTheMiniature,
            why(),
        )
        .expect("noted, not raised");
    }
    crate::painted::wash(
        &mut frame,
        &square,
        Rectangle::new((8, 8).into(), (16, 16).into()),
    )
    .expect("a picture the renderer can draw is drawn after the refusal");
    let _ = frame.finish().expect("the frame is finished");
    let noted = refused.taken();
    assert_eq!(noted.len(), 1, "one refusal noted: {noted:?}");
    assert_eq!(noted[0].surface, firefox);
    assert_eq!(noted[0].going, WhereItWasGoing::InTheMiniature);
    assert!(noted[0].why.contains("not supported"), "{}", noted[0].why);
    assert!(refused.taken().is_empty(), "taken leaves none");
}

fn a_refusal(surface: &WlSurface, going: WhereItWasGoing) -> ARefusal {
    ARefusal {
        surface: surface.clone(),
        going,
        why: "The requested operation is not supported".to_owned(),
    }
}

/// **The log hears of each refusal once**, however many frames refuse it, and
/// says which window and what happens instead.
#[test]
fn the_log_is_told_of_a_refusal_once() {
    let (_display, made) = surfaces(2);
    let mut remembered = TheRefusedPictures::default();
    let first = remembered
        .remembered(
            &a_refusal(&made[0], WhereItWasGoing::InTheMiniature),
            Some(7),
        )
        .expect("told the first time");
    assert!(first.contains("window 7"), "{first}");
    assert!(first.contains("not supported"), "{first}");
    assert!(first.contains("application's mark"), "{first}");
    for _ in 0..60 {
        assert_eq!(
            remembered.remembered(
                &a_refusal(&made[0], WhereItWasGoing::InTheMiniature),
                Some(7)
            ),
            None,
            "a second frame's refusal is not told again"
        );
    }
    let on_the_canvas = remembered
        .remembered(&a_refusal(&made[1], WhereItWasGoing::OnTheCanvas), Some(8))
        .expect("another window is told");
    assert!(
        on_the_canvas.contains("rest of the desktop"),
        "{on_the_canvas}"
    );
}

/// **A window put aside whose miniature was refused is drawn by its mark**;
/// a window refused on the canvas is not, and neither is one never refused.
#[test]
fn a_refused_miniature_is_drawn_by_its_mark_and_nothing_else_is() {
    let (_display, made) = surfaces(2);
    let mut remembered = TheRefusedPictures::default();
    let _ = remembered.remembered(
        &a_refusal(&made[0], WhereItWasGoing::InTheMiniature),
        Some(7),
    );
    let _ = remembered.remembered(&a_refusal(&made[1], WhereItWasGoing::OnTheCanvas), Some(8));
    assert!(remembered.is_drawn_by_its_mark(7));
    assert!(
        !remembered.is_drawn_by_its_mark(8),
        "refused on the canvas only"
    );
    assert!(!remembered.is_drawn_by_its_mark(9), "never refused");
    // **A surface with no window number is told, and drawn by nothing.**
    let mut unnumbered = TheRefusedPictures::default();
    assert!(
        unnumbered
            .remembered(&a_refusal(&made[0], WhereItWasGoing::InTheMiniature), None)
            .is_some()
    );
    assert!(!unnumbered.is_drawn_by_its_mark(7));
}

/// **A window brought back is tried again the next time it is put aside**,
/// and told of again if it is refused again.
#[test]
fn a_window_brought_back_is_forgotten() {
    let (_display, made) = surfaces(1);
    let mut remembered = TheRefusedPictures::default();
    let refusal = a_refusal(&made[0], WhereItWasGoing::InTheMiniature);
    let _ = remembered.remembered(&refusal, Some(7));
    remembered.forget_all_but(&[7]);
    assert!(
        remembered.is_drawn_by_its_mark(7),
        "still aside: remembered"
    );
    remembered.forget_all_but(&[]);
    assert!(
        !remembered.is_drawn_by_its_mark(7),
        "brought back: forgotten"
    );
    assert!(
        remembered.remembered(&refusal, Some(7)).is_some(),
        "refused again after being put aside again: told again"
    );
}

/// **A refused miniature's square is a private window's square**: the
/// application's mark on a blank card, with no badge.
#[test]
fn the_square_by_its_mark_is_what_a_private_window_shows() {
    use alo_put_aside::whether_it_is_private::Privacy;
    let palette = crate::desktop_testing::noon_look(
        &crate::desktop_testing::an_appearance(),
        alo_strings::Direction::LeftToRight,
    )
    .palette()
    .expect("a palette");
    let mut fonts = cosmic_text::FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(alo_drawing::fonts::MANROPE.to_vec());
    let preview = |privacy: Privacy| {
        let mut panel = alo_put_aside::Panel::new();
        let window = alo_dock::Window::of(
            alo_dock::WindowId::numbered(7),
            Some(alo_dock::AppId::named("org.mozilla.firefox").expect("a name")),
            "Firefox",
            alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 800, 600).expect("a patch"),
            alo_dock::HowItSits::OnTheCanvas,
        );
        panel
            .put_aside(
                &window,
                alo_canvas::Zoom::LIFE_SIZE,
                alo_canvas::Place::FIRST,
                privacy,
            )
            .expect("put aside");
        panel.previews().first().expect("a preview").clone()
    };
    let slot = Rectangle::new((0, 0).into(), (48, 48).into());
    let artwork = alo_dock::artwork::TheArtwork::none();
    let by_its_mark = crate::a_miniatures_card::a_windows_square_with_its_mark(
        slot,
        &preview(Privacy::Ordinary),
        &artwork,
        &palette,
        &mut fonts,
    );
    let private = crate::a_miniatures_card::a_windows_square(
        slot,
        &preview(Privacy::Private(
            alo_put_aside::SafeName::chosen("Bank").expect("a safe name"),
        )),
        &artwork,
        &palette,
        &mut fonts,
    );
    let ordinary = crate::a_miniatures_card::a_windows_square(
        slot,
        &preview(Privacy::Ordinary),
        &artwork,
        &palette,
        &mut fonts,
    );
    assert_eq!(
        by_its_mark.pixels, private.pixels,
        "a private window's square"
    );
    assert_ne!(
        by_its_mark.pixels, ordinary.pixels,
        "not the card a miniature is drawn into"
    );
}

/// **No client picture in the painter ends the frame on its own say.** Every
/// import of a client's tree and every draw of a client's elements in
/// `crate::scene_drawing` goes through `noted`, which raises only where nobody
/// will act on the refusal — never `map_err(submission)?` straight out of the
/// frame, the shape of the fault of 2026-11-12. Held to the source because the
/// road that reaches it needs a real application's buffer.
#[test]
fn the_painter_raises_no_client_picture() {
    let source = include_str!("scene_drawing.rs");
    let code: String = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut draws = 0;
    for (at, _) in code.match_indices("draw_render_elements(&mut frame") {
        draws += 1;
        let after = code.get(at..).expect("in the source");
        let statement = after.split_once(';').map_or(after, |(first, _)| first);
        assert!(
            statement.contains("noted(") && !statement.contains("map_err(submission)"),
            "a client draw that raises: {statement}"
        );
    }
    let mut imports = 0;
    for (at, _) in code.match_indices("drawing::import_at(") {
        imports += 1;
        let before = code.get(..at).expect("in the source");
        let line = before.rsplit_once('\n').map_or(before, |(_, line)| line);
        assert!(
            line.trim_end().ends_with("match"),
            "an import of a client's tree that is not matched: {line}"
        );
    }
    assert_eq!(
        draws, 4,
        "the windows, the rest, the miniatures and the pointer"
    );
    assert_eq!(imports, 3, "the canvas, the pointer and the miniatures");
}
