//! Display ownership and bounded, nonblocking dispatch for graphics backends.

use std::{io, path::Path, sync::Arc};

use smithay::reexports::wayland_server::{Display, protocol::wl_surface::WlSurface};
use smithay::utils::{Physical, Point, Rectangle};

use crate::{
    SocketError,
    socket::Socket,
    surfaces::{ClientState, Surfaces},
};

/// A native Wayland display, independent of the nested or direct graphics backend.
///
/// The caller supplies a private runtime directory and a fresh session name.
/// No environment is mutated and no client process is launched. Drop closes
/// clients and removes the owned socket. Dispatch must be driven by the backend.
pub struct Server {
    /// Submitted opaque label and pointer releases owned independently of pixels.
    pub(crate) control_overlay: crate::window_control_overlay::Overlay,
    /// Native control ownership is separate from client pointer grabs.
    pub(crate) control_press: Option<crate::window_control_input::Press>,
    /// Explicitly composed strip and mapping-bound native label focus.
    pub(crate) control_presentation: Option<crate::window_control_presentation::Presentation>,
    /// Successfully submitted reader/page identity and exact hit bounds.
    pub(crate) reader_presentation: Option<crate::window_control_reader_frame::ReaderPresentation>,
    /// The display is private so every inserted client has our client state.
    display: Display<Surfaces>,
    /// Protocol state and mapped toplevels.
    pub(crate) surfaces: Surfaces,
    /// Listener and private directory lifetime.
    socket: Socket,
    /// One output and its successfully submitted surface membership.
    pub(crate) presentation: crate::presentation::Presentation,
    /// Stable mapped-root order for trusted window cycling.
    pub(crate) switch_order: crate::window_switch::SwitchOrder,
    /// **How each display is divided and what desktops are on it.**
    ///
    /// Held here because it is a session's, and kept across windows opening
    /// and closing and displays arriving and leaving. Nothing about a layout
    /// or a desktop is decided in this crate: `crate::server_desk` says what
    /// that means and which crate answers which question.
    pub(crate) desk: crate::server_desk::Desk,
    /// **Where the fixed controls are, as the last draw laid them out.**
    ///
    /// Held here because a drag has to ask, and the Dock's bounds otherwise exist
    /// only inside the raster path at draw time — which is why
    /// `crate::canvas_never_lost`'s rule had no caller until this field existed.
    /// `crate::canvas_fixed_controls` carries the reasoning and the units.
    pub(crate) fixed_controls: crate::canvas_fixed_controls::FixedControls,
    /// **Where the put-aside panel is, as the last draw laid it out.**
    ///
    /// Held for exactly the reason `fixed_controls` above is, and the parallel is
    /// the argument: the panel's slots are computed in `crate::panel_raster` and
    /// nowhere else, so outside a frame they do not exist. A pointer arriving
    /// between frames has nothing to be tested against, which is why
    /// `crate::which_preview_the_pointer_is_on` had no geometry to ask about
    /// until this field existed.
    ///
    /// `None` until the first draw, and that is the honest answer rather than an
    /// empty picture: a panel nobody has laid out has no slots *and* no reserved
    /// column, and a default would claim an edge that has not been measured on
    /// this display. `crate::peeking_at_a_put_aside_window` answers `Ok(None)`
    /// there instead of inventing one.
    pub(crate) panel_as_drawn: Option<crate::which_preview_the_pointer_is_on::ThePanelAsDrawn>,
    /// Windows the **person** has asked to put aside, not yet acted on.
    ///
    /// # Why a request rather than the act
    ///
    /// `Server::put_this_window_aside` needs the `Panel`, and the `Panel` belongs to
    /// whoever implements `crate::TheDesktop` — the compositor holds no session state, by
    /// that trait's own argument. But the two places a person asks are
    /// `crate::window_command` (the keyboard) and `crate::window_control_input` (the
    /// button on a window's own controls), and both are `Server` methods with no desktop
    /// in scope.
    ///
    /// So the ask is recorded here and performed in `crate::direct_desktop`, which is the one
    /// place holding both. The same shape as `panel_as_drawn` above, in the other direction:
    /// that one carries what the draw knew to a question asked later, this one carries what
    /// the input knew to an act performed later.
    ///
    /// **A list rather than one, because two presses in one batch are two windows.** Keeping
    /// only the latest would silently drop a window a person asked to put away, which is the
    /// one outcome this whole surface exists to prevent.
    pub(crate) asked_to_put_aside: Vec<WlSurface>,
    /// Where the **person** clicked the panel, not yet acted on.
    ///
    /// The point of the press, rather than the pointer read again later. A peek asks where the
    /// pointer *is*; a click asks where it **was when the button went down** — re-reading the
    /// live pointer when the ask is met would act on whichever preview the pointer had moved
    /// to, which is a different window from the one the person chose.
    ///
    /// Recorded here and met in `crate::direct_desktop` for the same reason as
    /// `asked_to_put_aside`: deciding *which* preview needs the live `Panel` for the identity
    /// check, and input has no desktop in scope.
    pub(crate) asked_to_bring_back: Vec<Point<i32, Physical>>,
    /// Buttons whose press the panel took, so their release is taken too.
    ///
    /// **A client must never see a release for a press it did not see.** The press is swallowed
    /// where the panel claims it, so the matching release has to be swallowed as well or the
    /// focused client receives an unpaired release and believes a button it never saw held has
    /// gone up.
    pub(crate) clicks_the_panel_took: std::collections::HashSet<u32>,
    /// **What this person's chords mean**, told once as the session stands up.
    ///
    /// Held here for the reason `asked_to_put_aside` gives in the other
    /// direction: a key arrives at the seat, where no desktop is in scope, and
    /// what a chord means is the person's own settings, which belong to
    /// whoever implements `crate::TheDesktop`. `crate::a_chord_reaches_its_action`
    /// carries the argument.
    ///
    /// `None` until a desktop says otherwise, which is what keeps the sign-in
    /// screen, the nested lane and every test on the road they already had: a
    /// seat that has been told no shortcuts looks none up.
    pub(crate) shortcuts: Option<alo_shortcuts::Shortcuts>,
    /// **What a touchpad gesture means, as `alo-desktops` recognises it.**
    ///
    /// One recogniser for the seat: at most one gesture is in flight at a
    /// time, and a second would be two answers to one person's fingers. What a
    /// swipe does with what it answers is `crate::desktop_swipes`.
    pub(crate) gestures: alo_desktops::gestures::Gestures,
}

impl Server {
    /// Flush queued protocol events without accepting or dispatching client work.
    pub(crate) fn flush(&mut self) -> io::Result<()> {
        self.display.flush_clients()
    }

    /// Handle kept private to the library's protocol-global initialization.
    pub(crate) fn display_handle(&self) -> smithay::reexports::wayland_server::DisplayHandle {
        self.display.handle()
    }
    /// Bind `runtime/name/wayland`, refusing existing sessions and unsafe paths.
    pub fn bind(runtime: &Path, name: &str) -> Result<Self, SocketError> {
        let display = Display::new().map_err(io::Error::other)?;
        let surfaces = Surfaces::new(&display.handle());
        let socket = Socket::bind(runtime, name)?;
        Ok(Self {
            control_overlay: Default::default(),
            control_press: None,
            control_presentation: None,
            reader_presentation: None,
            display,
            surfaces,
            socket,
            presentation: Default::default(),
            switch_order: Default::default(),
            fixed_controls: crate::canvas_fixed_controls::FixedControls::default(),
            // Nothing has been drawn yet, which is why this is `None` rather than an
            // empty picture — see the field's own note.
            panel_as_drawn: None,
            // Nobody has asked for anything yet.
            asked_to_put_aside: Vec::new(),
            asked_to_bring_back: Vec::new(),
            clicks_the_panel_took: std::collections::HashSet::new(),
            gestures: Default::default(),
            // Nobody has told this seat what a chord means, so it takes none.
            shortcuts: None,
            desk: crate::server_desk::Desk::new(),
        })
    }

    /// Absolute socket path; pass only to applications belonging to this session.
    pub fn socket_path(&self) -> &Path {
        &self.socket.path
    }

    /// Accept at most 16 clients, dispatch pending requests and flush responses.
    ///
    /// Never blocks waiting for a client. Protocol errors disconnect the offending
    /// client; display I/O failures are returned to the backend. The accept budget
    /// ensures a connection flood cannot make acceptance itself an endless loop.
    pub fn dispatch(&mut self) -> io::Result<()> {
        if let Some(listener) = &self.socket.listener {
            for _ in 0..16 {
                let Some(stream) = listener.accept()? else {
                    break;
                };
                self.display
                    .handle()
                    .insert_client(stream, Arc::new(ClientState::default()))?;
            }
        }
        self.display.dispatch_clients(&mut self.surfaces)?;
        self.surfaces.prune();
        // The desktops and the divisions follow the windows: one that opened
        // joins the desktop being looked at, and one that closed loses its
        // share rather than leaving a tree holding a window nobody can see.
        self.the_windows_are_now_these();
        self.switch_order.refresh(self.surfaces.buffered());
        self.display.flush_clients()
    }

    /// Visible, configured toplevel roots with buffers, in front-to-back order.
    /// New roles initially follow existing roles; explicit raising changes order.
    ///
    /// This is an internal renderer input, not an agent context or window API.
    /// Minimization, buffer removal, destruction and disconnect exclude roots here.
    pub fn mapped_surfaces(&self) -> impl Iterator<Item = &WlSurface> {
        self.surfaces.mapped()
    }

    /// Where each mapped window is on this display, in its physical pixels.
    ///
    /// For the one question the dock asks of the windows — whether any needs
    /// the room it is sitting in. [`crate::dock_room`] holds the rule and
    /// `docs/design/when-the-dock-gives-way.md` settles what *a window needs
    /// the room* has to mean.
    ///
    /// **One is the scale this display is laid out at**, the same one
    /// `crate::desktop_raster` multiplies a division by. The conversion is
    /// here rather than inside the comparison so there is one place to change
    /// when a display is laid out at another scale.
    ///
    /// A minimised window is not among these, because `mapped` excludes it: a
    /// window a person put aside is not a window needing the room.
    pub(crate) fn window_areas(&self) -> Vec<Rectangle<i32, Physical>> {
        self.mapped_surfaces()
            .map(|surface| {
                let origin = crate::window_placement::window_buffer_origin(surface);
                let geometry = crate::scene::geometry(surface);
                Rectangle::new(
                    (origin + geometry.loc).to_physical(1.0).to_i32_round(),
                    geometry.size.to_physical(1.0).to_i32_round(),
                )
            })
            .collect()
    }

    /// Number of live toplevel roles, including those not yet mapped.
    pub fn toplevel_count(&self) -> usize {
        self.surfaces.count()
    }

    /// Draw and submit mapped surface trees, then notify only submitted surfaces.
    ///
    /// The target is trusted compositor plumbing, never supplied by an agent.
    /// Dispatch is not interleaved with submission. Failure preserves all pending
    /// frame callbacks. `time` is milliseconds on the session's monotonic clock,
    /// wrapping at 32 bits as required by Wayland. Success means submission to
    /// the backend, not physical presentation or a presentation-time guarantee.
    pub fn render(
        &mut self,
        target: &mut impl crate::FrameTarget,
        time: u32,
    ) -> Result<usize, crate::RenderError> {
        self.render_window_controls(target, None, time)
    }

    /// Shared submission path; callers own native presentation retirement.
    pub(crate) fn render_frame(
        &mut self,
        target: &mut impl crate::FrameTarget,
        time: u32,
    ) -> Result<usize, crate::RenderError> {
        let size = target.size();
        let metadata = self.presentation.validate_target(target)?;
        if size.w > 0 && size.h > 0 {
            // Reactive popup negotiation follows the backend's desired extent,
            // even on submission refusal. wl_output describes only submitted modes.
            self.surfaces.popups.output_size = Some(size);
            // **The camera used to be assigned beside the extent and no longer is.**
            // `Popups` held a copy of it, and this line is where the copy was made
            // fresh once a frame — which is what kept it from ever being stale.
            // `crate::Surfaces::camera` is the one home now, and the three places
            // that place a popup take it as an argument, so there is nothing left
            // here to keep in step.
        }
        // The one reader of the camera that draws with it, and the reason the
        // backend is told rather than asked: the first version of this seam
        // shipped a backend holding a camera nobody assigned, drawing a zoom it had
        // never been told about.
        target.look_at(self.surfaces.camera)?;
        self.surfaces.prune();
        let roots: Vec<_> = self.mapped_surfaces().cloned().collect();
        let cursor = self.cursor();
        let popups = self.popup_surfaces();
        let submitted = self.presentation.render(
            &self.display.handle(),
            target,
            (&roots, &popups),
            &cursor,
            time,
        )?;
        self.surfaces.update_window_mode_output(Some(size));
        // The output this frame went to is the display a session holds its
        // divisions and desktops on; see `crate::display_lifecycle`.
        self.the_display_submitted(&metadata, (size.w, size.h));
        Ok(submitted)
    }
}

impl Server {
    /// Advertise buffer sharing, now that a renderer exists to import with.
    ///
    /// A pass-through, because the ordering is the whole content: the caller is
    /// the graphics backend's setup, which has a renderer and nothing else to
    /// do with wayland globals. See
    /// [`crate::surfaces::Surfaces::advertise_importable_buffers`].
    pub(crate) fn advertise_importable_buffers(
        &mut self,
        formats: smithay::backend::allocator::format::FormatSet,
    ) {
        self.surfaces.advertise_importable_buffers(formats);
    }

    /// Take every buffer a client offered and nobody has tried yet.
    ///
    /// Called once per turn of `crate::direct_loop::run_with_input`, between
    /// dispatching clients and drawing. Leaving one here would leave a client
    /// waiting for an event with no sender.
    pub(crate) fn buffers_awaiting_import(
        &mut self,
    ) -> Vec<crate::buffers_clients_hand_over::HandedOver> {
        self.surfaces.buffers_awaiting_import()
    }
}
