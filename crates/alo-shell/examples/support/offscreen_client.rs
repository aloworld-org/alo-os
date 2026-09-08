//! Pixel expectations and protocol coordination for the offscreen graphics fixture.
use crate::{Fixture, application::Application};
use alo_shell::{FrameTarget, RenderError, render_scanout};
use smithay::{
    backend::renderer::gles::GlesRenderer,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};
use wayland_client::Proxy;

/// Fixture-only sink: success models submission but never claims a DRM commit.
pub struct Target<'a> {
    /// Real graphics context used by the production preparation path.
    pub renderer: &'a mut GlesRenderer,
    /// Inject transport rejection after successful preparation.
    pub refuse: bool,
}
impl FrameTarget for Target<'_> {
    fn size(&self) -> Size<i32, Physical> {
        (33, 32).into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        self.submit_popups(roots, &[], &alo_shell::Cursor::Default)
    }
    fn submit_popups(
        &mut self,
        roots: &[WlSurface],
        popups: &[alo_shell::Popup],
        cursor: &alo_shell::Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        let prepared = render_scanout(self.renderer, (33, 32).into(), roots, popups, cursor)?;
        verify(prepared.pixels().pixels());
        if self.refuse {
            return Err(RenderError::Submission(
                "injected transport rejection after readback".into(),
            ));
        }
        Ok(prepared.into_parts().1)
    }
}

/// Independent expected pixel map: opaque layers at known client coordinates.
pub fn verify(pixels: &[u8]) {
    assert_eq!(pixels.len(), 33 * 32 * 4);
    for (index, pixel) in pixels.as_chunks::<4>().0.iter().enumerate() {
        let (x, y) = (index % 33, index / 33);
        let expected = if (10..26).contains(&x) && (14..30).contains(&y) {
            [255, 255, 0, 0] // cyan cursor above the popup
        } else if (8..24).contains(&x) && (12..28).contains(&y) {
            [255, 0, 0, 0] // blue popup above its parent
        } else if (24..40).contains(&x) && (4..20).contains(&y) {
            [0, 255, 255, 0] // yellow child clipped at the right edge
        } else if x < 16 && y < 16 {
            if y < 8 {
                [0, 0, 255, 0]
            } else {
                [0, 255, 0, 0]
            }
        } else {
            [0; 4]
        };
        assert_eq!(*pixel, expected, "pixel ({x},{y})");
    }
}

/// Solid, opaque native little-endian ARGB buffer bytes.
fn solid(bgra: [u8; 4]) -> [u8; 1024] {
    let mut pixels = [0; 1024];
    for pixel in pixels.as_chunks_mut::<4>().0 {
        *pixel = bgra;
    }
    pixels
}

/// No callback can arrive before the explicit successful fixture submission.
pub fn run(fixture: Fixture, send: mpsc::Sender<u8>, receive: mpsc::Receiver<()>) {
    let mut app = Application::new(&fixture);
    app.configure();
    let mut pixels = solid([0, 0, 255, 255]);
    for row in pixels.as_chunks_mut::<64>().0.iter_mut().skip(8) {
        for pixel in row.as_chunks_mut::<4>().0 {
            *pixel = [0, 255, 0, 255];
        }
    }
    let (child, _role) = app.child((24, 4));
    let _yellow = app.attach_pixels(&child, &solid([0, 255, 255, 255]));
    let (_hidden, _hidden_role) = app.child((i32::MAX, i32::MAX));
    app.surface.frame(&app.queue.handle(), ());
    let _root = app.attach_pixels(&app.surface, &pixels);
    app.sync();
    let (popup, xdg, _popup_role) = app.popup(true, 1);
    popup.commit();
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(7, 10, 16, 16)));
    app.ack_popup(&xdg);
    // Explicit window origins shift the popup buffer to (8,12).
    app.xdg.set_window_geometry(2, 3, 12, 12);
    app.surface.commit();
    xdg.set_window_geometry(1, 1, 12, 12);
    popup.frame(&app.queue.handle(), ());
    let _blue = app.attach_pixels(&popup, &solid([255, 0, 0, 255]));
    app.sync();
    // Wait for the popup's enter, not the now-stale root enter serial. Motion
    // is delivered after dispatch and must cross the wire before set_cursor.
    let deadline = Instant::now() + Duration::from_secs(3);
    while app.events.pointer.enters.last().map(|event| event.0) != Some(popup.id().protocol_id()) {
        assert!(Instant::now() < deadline, "popup pointer enter missing");
        app.sync();
    }
    let cursor = app.cursor((0, 0));
    let _cyan = app.attach_pixels(&cursor, &solid([255, 255, 0, 255]));
    app.sync();
    for stage in 1..=3 {
        assert!(send.send(stage).is_ok());
        assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
        app.sync();
        if stage < 3 {
            assert!(app.events.frames.is_empty());
            assert_eq!(app.events.membership, (0, 0));
            assert_eq!(app.events.outputs, 0);
            assert!(app.events.output_events.is_empty());
        } else {
            // The first successful submission advertises the output. The first
            // roundtrip discovers it and queues a bind; finish that bind before
            // checking the surface enter events delivered through the output.
            app.sync();
            assert_eq!(app.events.outputs, 1);
            assert_eq!(app.events.frames, [77; 4]);
            assert_eq!(app.events.membership, (4, 0));
        }
    }
    let mut other = Application::new(&fixture);
    other.configure();
    let _magenta = other.attach_pixels(&other.surface, &solid([255, 0, 255, 255]));
    other.sync();
    assert!(send.send(8).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    app.sync();
    other.sync();
    assert_eq!(app.events.activation.last(), Some(&true));
    assert!(other.events.activation.ends_with(&[true, false]));
    drop(other);
    app.sync();
    app.set_cursor(app.events.pointer.serial, None, (0, 0));
    app.sync();
    assert!(send.send(6).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    app.set_cursor(app.events.pointer.serial, Some(&cursor), (0, 0));
    app.sync();
    cursor.destroy();
    app.sync();
    assert!(send.send(7).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    drop(app);
    // A new client's roundtrip ensures the disconnected scene has been pruned.
    let mut fresh = Application::new(&fixture);
    fresh.sync();
    assert!(send.send(4).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    fresh.configure();
    fresh.attach();
    fresh.sync();
    assert!(send.send(9).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    fresh.sync();
    assert_eq!(fresh.events.sizes.last(), Some(&(32, 24)));
    assert!(fresh.events.serial.is_some());
    if let Some(serial) = fresh.events.serial {
        fresh.xdg.ack_configure(serial);
    }
    fresh.attach_resized();
    fresh.sync();
    assert!(send.send(10).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    assert!(send.send(11).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    fresh.sync();
    assert!(fresh.events.keyboard.seat.is_some());
    if let Some(seat) = &fresh.events.keyboard.seat {
        fresh
            .toplevel
            ._move(seat, fresh.events.pointer.button_serial);
    }
    fresh.sync();
    assert!(send.send(12).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    fresh.sync();
    assert!(send.send(13).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    fresh.sync();
    if let Some(seat) = &fresh.events.keyboard.seat {
        fresh.toplevel.resize(
            seat,
            fresh.events.pointer.button_serial,
            wayland_protocols::xdg::shell::client::xdg_toplevel::ResizeEdge::TopLeft,
        );
    }
    fresh.sync();
    assert_eq!(fresh.events.resizing.last(), Some(&true));
    assert!(send.send(14).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    fresh.sync();
    assert_eq!(fresh.events.sizes.last(), Some(&(40, 32)));
    if let Some(serial) = fresh.events.serial {
        fresh.xdg.ack_configure(serial);
    }
    let _white = fresh.attach_pixels(&fresh.surface, &solid([255; 4]));
    fresh.sync();
    assert!(send.send(15).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    fresh.sync();
    assert_eq!(fresh.events.resizing.last(), Some(&false));
    if let Some(serial) = fresh.events.serial {
        fresh.xdg.ack_configure(serial);
    }
    fresh.attach_resized();
    fresh.sync();
    assert!(send.send(16).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    assert!(send.send(17).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    fresh.sync();
    assert_eq!(fresh.events.maximized.last(), Some(&true));
    // The production coordinator last successfully submitted a 33x32 output.
    assert_eq!(fresh.events.sizes.last(), Some(&(33, 32)));
    if let Some(serial) = fresh.events.serial {
        fresh.xdg.ack_configure(serial);
    }
    fresh.sync();
    assert!(send.send(18).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    fresh.attach_maximized();
    fresh.sync();
    assert!(send.send(19).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    fresh.sync();
    assert_eq!(fresh.events.maximized.last(), Some(&false));
    assert_eq!(fresh.events.sizes.last(), Some(&(32, 24)));
    if let Some(serial) = fresh.events.serial {
        fresh.xdg.ack_configure(serial);
    }
    fresh.sync();
    assert!(send.send(20).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    fresh.attach_resized();
    fresh.sync();
    assert!(send.send(21).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    fresh.surface.attach(None, 0, 0);
    fresh.surface.commit();
    fresh.sync();
    fresh.configure();
    fresh.surface.frame(&fresh.queue.handle(), ());
    let storage = fresh.attach_pixels(&fresh.surface, &solid([255, 255, 255, 255]));
    fresh.sync();
    // Truncate after metadata acceptance but before the first renderer import.
    // Smithay catches the resulting buffer access failure and rejects this client.
    assert!(storage.set_len(0).is_ok());
    assert!(send.send(5).is_ok());
    assert!(receive.recv_timeout(Duration::from_secs(5)).is_ok());
    fresh.refused();
    assert!(fresh.events.frames.is_empty());
}
