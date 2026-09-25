//! The desktop on this machine's own display: the dock, the windows, and what
//! is leaving, above whatever clients are mapped.
//!
//! The nested twin is `crate::nested_desktop`, which draws the same desktop
//! inside somebody else's compositor. What differs is where the pixels go and
//! nothing else: the dock, the status area, the two windows and the egress
//! indicator are the same pictures, made by the same `frame_pictures` from the
//! same `DesktopFrame`.
//!
//! # A client's frame is still the server's own
//!
//! This does not submit a desktop *instead of* the clients. It goes through
//! `Server::render_frame` exactly as an ordinary frame does — so the output is
//! published, membership is kept and every client that was drawn gets its frame
//! callback — and hands the desktop's layers to the backend on the way past.
//! A desktop that bypassed that would be a compositor whose clients slowly
//! stopped drawing, for a reason nobody would find.

use crate::scene_native::NativeLayers;
use crate::{
    Cursor, DesktopFrame, DirectLoopError, FrameTarget, Popup, RenderError, Server, SessionError,
    WindowControlLabels, direct_input_loop::LoopInput,
};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};

/// What is on this display now, asked once a frame.
///
/// A trait rather than a struct of nine fields because **nothing here owns any
/// of it**: the dock is `alo-dock`'s, the appearance `alo-appearance`'s, the
/// readings `alo-measuring`'s and the division `alo-dividing`'s. The compositor
/// is handed what they say and draws it, and a type that held them would be a
/// compositor keeping its own copy of somebody else's answer.
pub trait TheDesktop {
    /// Everything one frame of the ordinary desktop needs.
    fn now(&self) -> DesktopFrame<'_>;
}

impl crate::DirectSession {
    /// Stand the desktop up on this machine's display, and keep it there.
    ///
    /// Draws for every frame `next` asks for until it says stop or the seat
    /// takes the display away. Clients are dispatched and drawn underneath, and
    /// their input is routed to them exactly as `run_compositor_with_input`
    /// routes it: this lane changes what is drawn above the clients, never who
    /// hears the keyboard.
    ///
    /// # Errors
    /// [`SessionError`] when the seat refuses the display. Everything the
    /// display lifetime reported is in the result.
    pub fn desktop(
        &mut self,
        server: &mut Server,
        desktop: &dyn TheDesktop,
        labels: &mut WindowControlLabels,
        mut next: impl FnMut() -> crate::DirectFrame,
    ) -> Result<crate::ActiveSessionResult<crate::DirectLoopResult>, SessionError> {
        server.clear_input();
        let manager = self.input_session();
        self.with_active_device(|fd, poll| {
            let setup = (|| {
                let output = crate::discover_atomic_output(fd)?;
                poll()?;
                let (width, height) = output.output.mode.size();
                let painter = crate::software_scanout::SoftwarePainter::new()?;
                let input = crate::direct_input_loop::RoutedInput {
                    owner: crate::SeatInput::new(manager)?,
                    extent: (i32::from(width), i32::from(height)),
                };
                Ok::<_, DirectLoopError>((output, painter, input))
            })();
            match setup {
                Ok((output, painter, input)) => crate::direct_loop::run_with_input(
                    server,
                    crate::direct_target::Target::new(
                        painter,
                        crate::drm_inventory::Inventory(fd),
                        output,
                    ),
                    poll,
                    &mut next,
                    Desk {
                        input,
                        desktop,
                        labels,
                    },
                ),
                Err(error) => crate::DirectLoopResult {
                    outcome: Err(error),
                    input_cleanup: None,
                    input_flush: Some(server.flush()),
                    retirement: None,
                    flush: None,
                },
            }
        })
    }
}

/// The desktop lane: the clients' own input, and the desktop drawn above them.
struct Desk<'a> {
    /// The ordinary routing, unchanged — a client hears the keyboard here.
    input: crate::direct_input_loop::RoutedInput,
    /// What the crates that decide each of these say is on the display now.
    desktop: &'a dyn TheDesktop,
    /// The bundled font every word on the desktop is laid out with.
    labels: &'a mut WindowControlLabels,
}

impl LoopInput for Desk<'_> {
    fn dispatch(
        &mut self,
        server: &mut Server,
        poll: &mut dyn FnMut() -> Result<(), SessionError>,
    ) -> Result<(), DirectLoopError> {
        self.input.dispatch(server, poll)
    }

    /// Make this frame's pictures and submit them with the clients.
    ///
    /// The record and a question are not carried yet: nothing on a machine
    /// opens either, and a frame that reserved room for them would be drawing
    /// for a state that cannot arrive. They are the same two `Option`s in
    /// `frame_pictures` when something does.
    fn present<T: crate::direct_loop::LoopTarget + crate::presentation::NativeTarget>(
        &mut self,
        server: &mut Server,
        target: &mut T,
        time: u32,
    ) -> Result<(), DirectLoopError> {
        let size = target.size();
        let pictures = crate::nested_desktop::frame_pictures(
            self.desktop.now(),
            None,
            None,
            self.labels,
            (size.w, size.h),
        )?;
        server.render_frame(
            &mut Layered {
                target,
                layers: NativeLayers {
                    desktop: Some(&pictures.desktop),
                    status: Some(&pictures.status),
                    ..NativeLayers::nothing()
                },
            },
            time,
        )?;
        Ok(())
    }

    fn shutdown(self, server: &mut Server) -> Option<std::io::Result<()>> {
        self.input.shutdown(server)
    }
}

/// A display with this frame's own layers waiting on it.
///
/// The same arrangement `crate::window_control_frame`'s own wrapper makes, and
/// for the same reason: the server owns what a frame *is* — which clients, which
/// popups, which cursor, and who is told it was drawn — and this only says what
/// is painted above them.
struct Layered<'a, T> {
    /// The display underneath, which decides whether anything was submitted.
    target: &'a mut T,
    /// This shell's own surfaces for this frame.
    layers: NativeLayers<'a>,
}

impl<T: crate::presentation::NativeTarget> FrameTarget for Layered<'_, T> {
    fn metadata(&self) -> Result<crate::OutputMetadata, RenderError> {
        self.target.metadata()
    }
    fn size(&self) -> Size<i32, Physical> {
        self.target.size()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        self.submit_popups(roots, &[], &Cursor::Default)
    }
    fn submit_popups(
        &mut self,
        roots: &[WlSurface],
        popups: &[Popup],
        cursor: &Cursor,
    ) -> Result<Vec<WlSurface>, RenderError> {
        self.target
            .submit_native_layers(roots, popups, cursor, self.layers)
    }
}
