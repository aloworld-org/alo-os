//! Titles shown always, or under the pointer, as the person said.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use crate::Server;
use alo_appearance::WindowTitles;
use smithay::input::keyboard::XkbConfig;
use smithay::reexports::wayland_server::{
    Client, DataInit, Dispatch, Display, DisplayHandle, Resource,
    protocol::wl_surface::{self, WlSurface},
};
use std::os::unix::fs::PermissionsExt;

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

/// A window no pointer has ever been near, and what keeps it alive.
fn a_window() -> (Display<Identities>, std::os::unix::net::UnixStream, WlSurface) {
    let display = Display::<Identities>::new().expect("a display");
    let mut handle = display.handle();
    let (socket, peer) = std::os::unix::net::UnixStream::pair().expect("a socket pair");
    let client = handle
        .insert_client(socket, std::sync::Arc::new(()))
        .expect("a client");
    let surface = client
        .create_resource::<WlSurface, (), Identities>(&handle, 1, ())
        .expect("a surface");
    (display, peer, surface)
}

/// A seat with a keyboard.
fn a_session(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind_keyboard(
        runtime.path(),
        what,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    (runtime, server)
}

/// **Always, until somebody says otherwise** — and then exactly what they said.
#[test]
fn a_seat_shows_every_title_until_told_otherwise() {
    let (_runtime, mut server) = a_session("titles-told");
    assert_eq!(server.the_window_titles(), WindowTitles::Always);
    server.the_window_titles_are(WindowTitles::WhenPointedAt);
    assert_eq!(server.the_window_titles(), WindowTitles::WhenPointedAt);
}

/// **A window nobody has pointed at shows its edge, while titles are always
/// shown — and does not, once the person chose the quieter edge.** The choice
/// is not taken away: the machine §5 describes still answers for it.
#[test]
fn an_untouched_windows_edge_is_shown_only_while_titles_are_always_shown() {
    let (_runtime, mut server) = a_session("titles-revealed");
    let (_display, _peer, window) = a_window();
    assert!(window.is_alive());
    assert!(server.is_this_windows_edge_revealed(&window));
    server.the_window_titles_are(WindowTitles::WhenPointedAt);
    assert!(
        !server.is_this_windows_edge_revealed(&window),
        "the quieter edge was chosen and the edge still showed with nothing pointing at it"
    );
}

/// **A window the keyboard is not on does not wear the focused look**, which
/// is the refusal: a seat with no keyboard focus focuses nothing.
#[test]
fn a_window_the_keyboard_is_not_on_is_not_focused() {
    let (_runtime, server) = a_session("titles-focus");
    let (_display, _peer, window) = a_window();
    assert!(!server.is_the_window_typing_goes_to(&window));
}
