//! Which elements are which window's, and which edges go with them.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or missing surface is the failure \
              being reported"
)]

use super::*;
use smithay::reexports::wayland_server::{
    Client, DataInit, Dispatch, Display, DisplayHandle, Resource, protocol::wl_surface,
};
use smithay::utils::{Logical, Point, Size};

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

/// **A popup's elements are drawn with the window it belongs to.**
/// `crate::scene::trees` lists a window's popups just before it, front to
/// back, so a run ends at a window and takes every popup before it.
#[test]
fn a_window_takes_its_popups_into_its_run() {
    let (_display, made) = surfaces(4);
    let [front, popup, back, its_popup] = [0, 1, 2, 3].map(|at| made[at].clone());
    let roots = vec![front.clone(), back.clone()];
    let mut runs = Vec::new();
    // The order `trees` gives: front's popup, front, back's popup, back.
    a_surface_was_imported(&mut runs, &roots, &popup, 2);
    a_surface_was_imported(&mut runs, &roots, &front, 5);
    a_surface_was_imported(&mut runs, &roots, &its_popup, 6);
    a_surface_was_imported(&mut runs, &roots, &back, 9);
    assert_eq!(
        runs,
        vec![
            WhoseElements {
                window: front,
                elements: 0..5
            },
            WhoseElements {
                window: back,
                elements: 5..9
            },
        ]
    );
}

/// The bundled faces.
fn fonts() -> cosmic_text::FontSystem {
    let mut fonts = cosmic_text::FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
    fonts
}

/// An edge over a window at `y`.
fn an_edge(y: i32, fonts: &mut cosmic_text::FontSystem) -> EdgePicture {
    let window = Rectangle::<i32, Logical>::new(Point::from((40, y)), Size::from((400, 200)));
    EdgePicture::of(
        &crate::window_edge::edge_of(
            window,
            crate::window_edge::Decorations::TheShellDraws,
            true,
            Size::from((1440, 960)),
        ),
        crate::window_edge_paint::Pointing::default(),
        crate::window_edge_picture::OverTheEdge::default(),
        crate::window_edge_picture::TheWindow {
            body: Some(window),
            ..crate::window_edge_picture::TheWindow::default()
        },
        fonts,
        (1440, 960),
        100,
    )
    .expect("fits")
}

/// **Each edge goes with its own window, and one that names no window — or a
/// window the painter was not handed — is still drawn**, after every window, as
/// every edge was before.
#[test]
fn each_edge_is_drawn_once_with_its_window_or_after_them_all() {
    let (_display, made) = surfaces(3);
    let mut fonts = fonts();
    let edges = vec![
        an_edge(100, &mut fonts).on_the_window(made[0].clone()),
        an_edge(400, &mut fonts).on_the_window(made[1].clone()),
        an_edge(700, &mut fonts),
        an_edge(700, &mut fonts).on_the_window(made[2].clone()),
    ];
    let textures: Vec<AnEdgesTextures<()>> = edges
        .iter()
        .map(|_| AnEdgesTextures {
            behind: Vec::new(),
            over: Vec::new(),
        })
        .collect();
    let runs = vec![
        WhoseElements {
            window: made[0].clone(),
            elements: 0..1,
        },
        WhoseElements {
            window: made[1].clone(),
            elements: 1..2,
        },
    ];
    assert_eq!(the_edges_of(Some(&made[0]), &edges, &textures).count(), 1);
    assert_eq!(the_edges_of(Some(&made[1]), &edges, &textures).count(), 1);
    // The edge with no window, and the one whose window was not drawn.
    assert_eq!(the_edges_of_no_run(&runs, &edges, &textures).count(), 2);
    assert!(made[0].is_alive());
}

/// **What goes under a window really is under it**: an application's edge has
/// its line, its shadow and its strip in `behind`, and nothing in `behind`
/// lands on the window's own pixels.
#[test]
fn an_applications_edge_has_its_frame_behind_and_none_of_it_on_the_window() {
    let mut fonts = fonts();
    let picture = an_edge(100, &mut fonts);
    assert!(
        picture.behind().len() >= 2,
        "a shown edge with a body has bands and a strip: {}",
        picture.behind().len()
    );
    let window = Rectangle::<i32, Physical>::new((40, 100).into(), (400, 200).into());
    for raster in picture.behind() {
        assert!(
            raster.area.intersection(window).is_none(),
            "{:?} is on the window",
            raster.area
        );
    }
}
