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

    /// Read anything that changes, before the next frame is drawn.
    ///
    /// **A status area whose clock never moves looks broken rather than
    /// unfinished**, and it is the one surface a person checks to find out
    /// whether their machine is telling them the truth. So the lane asks before
    /// every frame, and whatever is on the other side decides how often that is
    /// worth acting on — this crate does not know what a reading costs and must
    /// not decide how often one is taken.
    ///
    /// Nothing by default: a desktop of fixed values is a real one, and the
    /// display probe is exactly that.
    fn refreshed(&mut self) {}
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
        desktop: &mut dyn TheDesktop,
        labels: &mut WindowControlLabels,
        strings: &alo_strings::Strings,
        mut next: impl FnMut() -> crate::DirectFrame,
    ) -> Result<crate::ActiveSessionResult<crate::DirectLoopResult>, SessionError> {
        server.clear_input();
        // Opened once, here, rather than retried every frame: a machine with no
        // accessibility bus will not grow one mid-session, and asking sixty
        // times a second would be a D-Bus call per frame answering the same no.
        let reader = crate::TheReaderIsTold::opened(
            server,
            strings,
            &[alo_access::Surface::Desktop],
            // **Nothing is turned on and that is read rather than
            // assumed.** This binary has no settings file behind it
            // yet; when it has one, what a person turned on comes
            // from there and a reader is told which way each switch
            // is set without another line changing here.
            &alo_access::TurnedOn::nothing(),
        )
        .ok();
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
                        strings,
                        reader,
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
    desktop: &'a mut dyn TheDesktop,
    /// The bundled font every word on the desktop is laid out with.
    labels: &'a mut WindowControlLabels,
    /// This machine's own sentences, for the names a reader is told.
    strings: &'a alo_strings::Strings,
    /// The tree on the accessibility bus, where there is one to serve it on.
    ///
    /// `None` on a machine with no accessibility bus, which is the ordinary case
    /// for somebody who has never turned a reader on — ADR 0063, *a machine that
    /// cannot run the engine says so rather than failing*. The desktop runs
    /// either way; what changes is whether anybody can read it.
    reader: Option<crate::TheReaderIsTold>,
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
        // Asked before the frame is made, never after: a frame drawn from
        // readings taken after it would show a person the moment before.
        self.desktop.refreshed();
        let size = target.size();
        // **The one place holding both the windows and the frame.** The
        // desktop's own state has no server in it, so it cannot say where the
        // windows are; the server cannot say what the person chose about the
        // dock. The dock's question needs both, and they meet here.
        let windows = server.window_areas();
        let mut frame = self.desktop.now();
        frame.windows = &windows;
        // The server's half, like the windows above: a desktop's own
        // state cannot know that a window has taken the whole screen,
        // and the Dock gives way to one that has.
        frame.filling_the_screen = server.a_window_is_filling_the_screen();
        let pictures = crate::nested_desktop::frame_pictures(
            frame,
            None,
            None,
            self.labels,
            (size.w, size.h),
        )?;
        // **Where the fixed controls ended up, handed to the drag that has to
        // avoid them.** The draw is the only place that knows: these are laid
        // out here and nowhere else. Without this line
        // `crate::canvas_never_lost`'s rule has no bounds to check against,
        // which is why it had no caller at all — see
        // `crate::canvas_fixed_controls`.
        //
        // **Every control this frame laid out, not only the Dock's band.** Until
        // 2026-10-02 the band was the only one handed over, so the rule was in
        // force against one of the three the promise names: a frame could keep
        // its name clear of the Dock and sit entirely under the put-aside panel,
        // and nothing could tell, because the shell had never been given the
        // panel's bounds to check against.
        server.the_fixed_controls_were_drawn(
            crate::canvas_fixed_controls::FixedControlsDrawn {
                dock_band: pictures.desktop.dock.as_ref().map(|dock| dock.band),
                panel_reserved: pictures.desktop.panel.reserved,
            },
            self.desktop.now().look.scale(),
        );
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
        // **What a reader is told follows what is open**, and this is the call
        // that was missing: the tree and the bus were both written and tested
        // and nothing outside a test ever built either, so a screen reader on a
        // running machine found no application at all.
        //
        // Cheap every frame on purpose — it compares the window names it last
        // published and touches the bus only when they differ, so a frame being
        // dragged or redrawn costs one comparison of a short list.
        //
        // A refusal is not allowed to stop the desktop. A reader that cannot be
        // reached is a person without a reader; a compositor that stopped
        // compositing over it would be a machine nobody can use at all.
        if let Some(reader) = self.reader.as_mut() {
            let _ = reader.following(
                server,
                self.strings,
                &[alo_access::Surface::Desktop],
                &alo_access::TurnedOn::nothing(),
            );
        }
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
    /// Forwarded to the display underneath, like the extent and the identity:
    /// what this wrapper adds is the layers above the plane, and it decides
    /// nothing about where the plane is.
    fn look_at(&mut self, camera: alo_canvas::Camera) -> Result<(), RenderError> {
        self.target.look_at(camera)
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
