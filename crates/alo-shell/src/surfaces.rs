//! XDG surface handshake, renderer buffer ownership and lifecycle cleanup.

use smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
use smithay::{
    backend::renderer::utils::{on_commit_buffer_handler, with_renderer_surface_state},
    delegate_compositor, delegate_data_device, delegate_pointer_gestures, delegate_shm,
    delegate_text_input_manager, delegate_viewporter, delegate_xdg_decoration,
    delegate_xdg_shell,
    input::{Seat, SeatHandler, SeatState, pointer::CursorImageStatus},
    reexports::wayland_server::{
        Client, DisplayHandle,
        backend::ClientData,
        protocol::{wl_buffer::WlBuffer, wl_seat::WlSeat, wl_surface::WlSurface},
    },
    utils::Serial,
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorClientState, CompositorHandler, CompositorState, with_states},
        pointer_gestures::PointerGesturesState,
        shell::xdg::decoration::{XdgDecorationHandler, XdgDecorationState},
        // The protocol's own enum, from the generated bindings.

        selection::{
            SelectionHandler,
            data_device::{
                ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
            },
        },
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            XdgToplevelSurfaceData,
        },
        shm::{ShmHandler, ShmState},
        text_input::TextInputManagerState,
        viewporter::ViewporterState,
    },
};

/// Every accepted connection gets independent transaction state.
#[derive(Default)]
pub(crate) struct ClientState {
    /// Smithay's per-client surface transactions.
    compositor: CompositorClientState,
}
impl ClientData for ClientState {}

/// A toplevel's presentation eligibility, separate from merely owning a role.
struct Window {
    /// Fresh identity for each uninterrupted visible mapping; never a counter.
    visibility: std::sync::Arc<()>,
    /// Smithay's role handle.
    surface: ToplevelSurface,
    /// A configured buffer is currently attached.
    mapped: bool,
    /// Shell visibility is independent of the client's buffer mapping.
    minimized: bool,
    /// The desktop this window is on is not the one a person is looking at.
    ///
    /// **A separate reason from `minimized`, deliberately.** A person minimised
    /// a window and expects to find it minimised; a person who switched desktop
    /// expects to find every window exactly as they left it when they switch
    /// back. Folding the two would make a desktop switch look like somebody
    /// minimising every window on the way out, and restore them un-minimised on
    /// the way in. Which desktop is which is `alo-desktops`' answer and this
    /// field is only where it is written down; see `crate::desktop_membership`.
    elsewhere: bool,
}

impl Window {
    /// Whether this window is one a frame draws.
    ///
    /// Three reasons it is not, and the caller never has to remember all
    /// three: it has no configured buffer, the person minimised it, or it is on
    /// a desktop that is not the current one.
    ///
    /// # Two pieces of correctness in other crates rest on `!minimized`, and
    /// neither is visible from this line
    ///
    /// Putting a window aside minimises it. So a put-aside window is not drawn,
    /// not in `Server::mapped_surfaces`, not among the frames on the plane, and
    /// **its Place is absent from `Server::the_world`.** Two crates depend on
    /// that, in opposite directions:
    ///
    /// - `alo_put_aside::a_place_groups_its_windows` has a branch for *the
    ///   Places the World cannot name*, reachable **only because** of this. If
    ///   `!minimized` is relaxed, every Place the panel holds is in the World,
    ///   that branch is dead, and it should be **removed** rather than kept
    ///   looking like care.
    /// - `alo_arranging::HowItWasShowing::PutAside` cannot be produced by the
    ///   shell's own mapping, **only because** of this. If `!minimized` is
    ///   relaxed, a put-aside frame starts arriving at
    ///   `crate::canvas_remembered`'s mapping and would be recorded as
    ///   `Ordinary` — wrong, and silent. **Fill the case; do not widen the
    ///   match.**
    ///
    /// Relaxing this condition is a correct thing somebody may want to do — a
    /// put-aside window drawn live in place is not an absurd future. **It is not
    /// a local change**, and the two notes downstream say the same thing from
    /// their own ends, where an editor of this line would not look.
    fn drawn(&self) -> bool {
        self.mapped && !self.minimized && !self.elsewhere && self.surface.alive()
    }
}

/// Protocol globals and toplevel roots shared by display backends.
pub(crate) struct Surfaces {
    /// Per-mapping normal geometry and normal/maximized/tiled response boundaries.
    pub(crate) window_modes: Vec<crate::window_mode::ModeWindow>,
    /// Last successfully submitted extent, shared by maximize and tile transactions.
    pub(crate) window_mode_output: Option<(i32, i32)>,
    /// Resize protocol state owned by one held press and mapping lifetime.
    pub(crate) window_resize: Option<crate::resize_transaction::Resize>,
    /// Pointer-authorized interactive movement of one mapped root.
    pub(crate) window_move: Option<crate::window_move::Move>,
    /// Opt-in popup handshake and parent lifetime tracking.
    pub(crate) popups: crate::popups::Popups,
    /// Seat-scoped explicit popup input ownership.
    pub(crate) popup_grab: Option<crate::popup_grabs::Grab>,
    /// Core surface/subsurface protocol.
    compositor: CompositorState,
    /// `wp_viewporter`: a surface says how big it is drawn, and which part of
    /// its buffer to draw.
    ///
    /// **Held and never read, like `gestures` above.** What the field does is
    /// keep the global alive; the protocol's effect arrives through
    /// `on_commit_buffer_handler`, which smithay already has read the viewport
    /// into `SurfaceView` before anything of ours looks — see this field's note
    /// in `crates/alo-shell/tests/a_surface_says_how_big_it_is_drawn/mod.rs` for
    /// why that makes the global sufficient here and not elsewhere.
    #[expect(
        dead_code,
        reason = "holding the global alive is the whole purpose; the effect arrives through the renderer's own commit handler"
    )]
    viewporter: ViewporterState,
    /// `zwp_text_input_v3`: how text a person did not type on a keyboard reaches
    /// an application.
    ///
    /// **Held and never read**, like `viewporter` and `gestures`. Smithay sets
    /// text-input focus from the keyboard focus itself, so nothing of ours
    /// follows it — which is the opposite of the clipboard, where a selection is
    /// held by nobody unless something calls `set_data_device_focus`.
    ///
    /// **This is half of a pair, and the half that is safe to advertise.** An
    /// application binds this to say *I will take composed text*; an input method
    /// binds `zwp_input_method_v2` to produce it, and an application is never told
    /// it has the text input until one exists — `smithay`'s own seat code:
    /// *only notify on `enter` once we have an actual IME.*
    ///
    /// So this cannot fire yet, and
    /// [ADR 0083](../../../docs/decisions/0083-an-input-method-is-a-grant-not-a-global.md)
    /// says why the other half is absent: an input method receives every keystroke
    /// before the application does, so who may become one is a grant a person
    /// makes rather than a global anybody may bind.
    #[expect(
        dead_code,
        reason = "holding the global alive is the whole purpose; focus is smithay's own"
    )]
    text_input: TextInputManagerState,
    /// CPU-backed application buffers.
    shm: ShmState,
    /// XDG shell role and configure tracking.
    xdg: XdgShellState,
    /// The display this shell's globals were made on.
    ///
    /// Kept because `set_data_device_focus` needs one every time the keyboard
    /// moves, and a handle is cheap to clone and impossible to obtain again
    /// from here. It was passed to `new` and dropped until 2026-10-02.
    pub(crate) display: DisplayHandle,
    /// **Who draws a window's frame.** This shell, always — see the handler.
    ///
    /// Held and never read, deliberately. `XdgDecorationHandler` has no
    /// accessor the way `DataDeviceHandler` has `data_device_state`: this
    /// value's entire purpose is to **stay alive**, because dropping it
    /// withdraws the global and clients stop being told who draws their frames.
    /// An accessor added to quiet the lint would make the field look read while
    /// leaving the thing that actually matters unsaid.
    #[expect(
        dead_code,
        reason = "the global lives as long as this value; being alive is what it is for"
    )]
    xdg_decoration: XdgDecorationState,
    /// **Copy and paste, and drag-and-drop between applications.**
    ///
    /// Held here rather than per-window because a selection is per *seat*: one
    /// person's clipboard, whichever window they were in when they took it.
    data_device: DataDeviceState,
    /// Live toplevel roots in front-to-back stacking order.
    windows: Vec<Window>,
    /// **Which Place the person is looking at.**
    ///
    /// One home, not two. A new toplevel is created here, on `Surfaces`, so the
    /// Place it is put on has to be reachable from here, and the shell reads it
    /// back through `Server::the_place_now`.
    ///
    /// **This note used to name the camera as the counter-example** — *`Server::camera`
    /// has a second copy in `self.popups.camera` that three mutators keep in step
    /// by hand, and this deliberately does not repeat that.* It was right to, and
    /// it is why [`Self::camera`] below now sits beside this field instead. It was
    /// also an undercount: there were **seven** places keeping the two in step, not
    /// three.
    pub(crate) place: alo_canvas::Place,
    /// **Where on the plane the person is looking, and how far in — one
    /// camera per display.**
    ///
    /// **A camera each, not a camera copied.** `more-than-one-display-plan.md`
    /// task 7, and the distinction its constraint turns on: what was forbidden
    /// is the shape collapsed on 2026-10-04, where *one* camera had a second
    /// home kept in step by hand. These are different cameras holding
    /// different views, which is what a viewport is — and the note below says
    /// so outright: *canvas task 9 makes a camera per viewport.*
    ///
    /// Keyed by display name, as `Server::presentations` and the per-display
    /// fixed controls are. A display with no entry is looking at
    /// [`alo_canvas::Camera::new`] — the origin at life size — which is what
    /// every display looked at before any of them had a camera of its own.
    ///
    /// # What it was, and why that had to go first
    ///
    /// One home, and this is it. It had two until 2026-10-04: a field on `Server`
    /// and a copy on `self.popups`, which **five** places kept in step by hand —
    /// the four mutators in `crate::canvas_camera` and `crate::canvas_show_all`,
    /// and `Server::render_frame`, which re-assigned the copy once a frame before
    /// drawing. That last one is why nothing was ever stale: the per-frame
    /// assignment made a forgotten mutator harmless, so the pattern was a real
    /// design rather than an oversight, and the note on [`Self::place`] right above
    /// records that it deliberately did not copy it.
    ///
    /// **It is collapsed now because the next task cannot be done over it.** Canvas
    /// task 9 — *every screen is a view onto the canvas* — makes a camera **per
    /// viewport**, and *the* camera syncing into *the* popups has no meaning with
    /// two displays at their own zoom: which display's plane constrains a popup is
    /// a question the old shape could not be asked. `alo-displays` already models
    /// more than one display, so the state's home was the only thing in the way.
    ///
    /// Here rather than on `Server` for [`Self::place`]'s own reason: a popup is
    /// created in this file and constrained to the **screen**, so where the plane
    /// sits has to be reachable from here. `Popups` is handed it as an argument
    /// instead of keeping a copy — an argument cannot be left un-synced, and the
    /// three entry points that place a popup all had to answer for it to compile.
    /// The shell reads it back through `Server::the_camera`.
    pub(crate) cameras: std::collections::BTreeMap<String, alo_canvas::Camera>,
    /// **Which level the person is looking at**: one Place, or every Place.
    ///
    /// Stored rather than derived because the World and a Place are different
    /// coordinate spaces — `alo_canvas::Showing`'s own note carries why, and that
    /// note was wrong until the shell's half of task 2 was built. Never set by a
    /// person: it changes only by stepping out of the last rung or into a tile.
    pub(crate) showing: alo_canvas::Showing,
    /// Seat globals, created only when the backend enables input.
    pub(crate) seats: SeatState<Self>,
    /// Optional keyboard seat and routing state.
    pub(crate) keyboard: Option<crate::keyboard::Keyboard>,
    /// Optional trusted backend pointer routing state.
    pub(crate) pointer: Option<crate::pointer::Pointer>,
    /// Latest request accepted by Smithay's focus, serial and role validation.
    pub(crate) cursor: CursorImageStatus,
    /// `zwp_pointer_gestures_v1`, held so the global outlives this compositor's
    /// clients rather than being dropped the moment it is created.
    ///
    /// **Never read, and that is what it is for.** The other globals beside it
    /// are reached through their handler traits — `CompositorHandler` hands back
    /// `&mut self.compositor`, and so on — but nothing asks this one for
    /// anything: `delegate_pointer_gestures!` needs no accessor, and a pinch
    /// arrives through the seat's pointer rather than through here. What the
    /// field does is stay alive. Dropping it would take the global with it and
    /// the protocol would simply stop being advertised, which is a fault with no
    /// error message at either end — a client would see no gestures and nothing
    /// would say why.
    #[expect(
        dead_code,
        reason = "holding the global alive is the whole purpose; there is nothing to read"
    )]
    pub(crate) gestures: PointerGesturesState,
    /// Buffer sharing with the graphics card, made here and advertised later.
    ///
    /// **The state is not the global.** `DmabufState::new` creates nothing a
    /// client can see; `create_global` is what advertises. So this can be held
    /// from the start while the promise waits for a renderer, which is what the
    /// owner's ruling of 2026-10-02 asks for and the reason this field is not
    /// an `Option`: an absent state would need an accessor that panics, and
    /// `DmabufHandler::dmabuf_state` returns a plain reference.
    dmabuf: DmabufState,
    /// Absent until a renderer exists — see `advertise_importable_buffers`.
    ///
    /// Its presence is the record that the promise has been made, so making it
    /// twice can be refused rather than quietly advertising two globals.
    dmabuf_global: Option<DmabufGlobal>,
    /// Buffers clients offered, until a renderer has tried to import them.
    ///
    /// Filled by `dmabuf_imported` and emptied by the loop, which is the only
    /// place holding both this and the renderer. A buffer left here would be a
    /// client waiting forever, so the drain is not optional — see
    /// `crate::buffers_clients_hand_over`.
    handed_over: Vec<crate::buffers_clients_hand_over::HandedOver>,
}

impl Surfaces {
    /// What the display showing this desk point is looking at.
    ///
    /// The first display's camera for a point on none of them, and the
    /// origin at life size when no display has been described yet — the same
    /// two fallbacks `crate::popup_placement` makes, and for the same reason:
    /// a pointer is somewhere whether or not the arrangement can place it.
    pub(crate) fn camera_at(
        &self,
        here: smithay::utils::Point<i32, smithay::utils::Physical>,
    ) -> alo_canvas::Camera {
        let named = self
            .popups
            .screens
            .iter()
            .find(|view| view.rect.contains(here))
            .or_else(|| self.popups.screens.first())
            .map_or("", |view| view.named.as_str());
        // **Read now, from its one home.** A camera copied into `screens`
        // once a frame is stale before the first frame and wrong after any
        // pan, which is how the first version of this broke four tests that
        // press a pointer before anything is drawn.
        self.camera_of(named)
    }

    /// What this display is looking at, or the origin at life size.
    ///
    /// **A display with no camera of its own has not been looked away from**,
    /// which is a real state rather than a missing one: every display starts
    /// at [`alo_canvas::Camera::new`] and only a pan or a zoom gives it an
    /// entry. Answering with the default rather than with `None` is what lets
    /// every reader keep its shape.
    pub(crate) fn camera_of(&self, named: &str) -> alo_canvas::Camera {
        self.cameras
            .get(named)
            .copied()
            .unwrap_or_else(alo_canvas::Camera::new)
    }
}

impl Surfaces {
    /// Advertise only protocols this component implements.
    pub(crate) fn new(display: &DisplayHandle) -> Self {
        Self {
            window_modes: Vec::new(),
            window_mode_output: None,
            window_resize: None,
            window_move: None,
            popups: Default::default(),
            popup_grab: None,
            compositor: CompositorState::new::<Self>(display),
            shm: ShmState::new::<Self>(display, vec![]),
            xdg: XdgShellState::new::<Self>(display),
            xdg_decoration: XdgDecorationState::new::<Self>(display),
            display: display.clone(),
            // Copy and paste. The global alone is not enough: see
            // `crate::keyboard`, where the selection is made to follow the
            // keyboard, without which a client binds this and finds nothing
            // ever holds the selection.
            data_device: DataDeviceState::new::<Self>(display),
            // The touchpad gesture protocol. Advertised so a pinch reaches an
            // application that wants one; `crate::canvas_pinch` is what happens
            // when the pinch is over the plane instead of over a frame.
            gestures: PointerGesturesState::new::<Self>(display),
            viewporter: ViewporterState::new::<Self>(display),
            text_input: TextInputManagerState::new::<Self>(display),
            // Buffer sharing. **The global is deliberately not made here**, by
            // the owner's ruling of 2026-10-02: advertising DMA-BUF before a
            // renderer exists is a promise this shell cannot keep, and a client
            // would discover that only when its buffer was refused. Every other
            // global above is advertised at birth because every other protocol
            // is implemented by this file alone.
            dmabuf: DmabufState::new(),
            dmabuf_global: None,
            handed_over: Vec::new(),
            windows: Vec::new(),
            // A machine that has never been used is looking at its first Place.
            cameras: std::collections::BTreeMap::new(),
            place: alo_canvas::Place::FIRST,
            showing: alo_canvas::Showing::OnePlace(alo_canvas::Place::FIRST),
            seats: SeatState::new(),
            keyboard: None,
            pointer: None,
            cursor: CursorImageStatus::default_named(),
        }
    }

    /// Advertise buffer sharing, now that a renderer can actually import.
    ///
    /// **Called once a renderer exists and never from `new`.** The owner ruled
    /// on 2026-10-02 that the renderer is initialised before DMA-BUF is
    /// advertised, and this is where the two meet: the caller is the graphics
    /// backend's own setup, which has just built a renderer and asks it what it
    /// can take.
    ///
    /// `formats` are that renderer's own, read through
    /// [`crate::direct_target::ScenePainter::importable_formats`] and never a
    /// list written here. A hand-written list is the same broken promise one
    /// size smaller: the client is told a format is available and finds out
    /// otherwise when it offers one.
    ///
    /// A renderer that can import nothing advertises nothing, because a global
    /// with an empty format set is a promise with no content.
    ///
    /// Advertising twice is ignored rather than doubled: two globals for one
    /// protocol would have clients bind either one and be answered by the same
    /// handler, which no client expects and nothing here needs.
    pub(crate) fn advertise_importable_buffers(&mut self, formats: FormatSet) {
        if self.dmabuf_global.is_some() || formats.iter().next().is_none() {
            return;
        }
        let display = self.display.clone();
        self.dmabuf_global = Some(self.dmabuf.create_global::<Self>(&display, formats));
    }

    /// Take every buffer waiting for a renderer to try it.
    ///
    /// Taken rather than borrowed: each answer consumes the notifier that
    /// carries it, so the holder has to give them up to answer them.
    pub(crate) fn buffers_awaiting_import(
        &mut self,
    ) -> Vec<crate::buffers_clients_hand_over::HandedOver> {
        std::mem::take(&mut self.handed_over)
    }

    /// Remove resources whose client disappeared without orderly destruction.
    pub(crate) fn prune(&mut self) {
        self.windows.retain(|window| window.surface.alive());
        self.prune_window_move();
        self.prune_window_resize();
        self.prune_window_modes();
        let parents: Vec<_> = self.mapped().cloned().collect();
        self.popups.prune(&parents);
        self.popups.refresh(&parents, &self.cameras);
        self.prune_popup_grab();
        self.prune_keyboard_focus();
        self.prune_pointer_focus();
    }

    /// Roots eligible for rendering; dead handles never escape this iterator.
    pub(crate) fn mapped(&self) -> impl Iterator<Item = &WlSurface> {
        self.windows
            .iter()
            .filter(|w| w.drawn())
            .map(|w| w.surface.wl_surface())
    }

    /// All buffered mappings, including windows hidden by the person.
    pub(crate) fn buffered(&self) -> impl Iterator<Item = &WlSurface> {
        self.windows
            .iter()
            .filter(|w| w.mapped && w.surface.alive())
            .map(|w| w.surface.wl_surface())
    }

    /// Current hidden mappings in stacking order, for trusted restore controls.
    pub(crate) fn minimized(&self) -> impl Iterator<Item = &WlSurface> {
        self.windows
            .iter()
            .filter(|w| w.mapped && w.minimized && w.surface.alive())
            .map(|w| w.surface.wl_surface())
    }

    /// Say whether this window's desktop is one being looked at.
    ///
    /// A fresh visibility identity on a change, exactly as minimising does:
    /// whatever was counting on this window being on screen must be told it is
    /// not the same showing any more.
    pub(crate) fn set_elsewhere(&mut self, surface: &WlSurface, value: bool) {
        let Some(window) = self
            .windows
            .iter_mut()
            .find(|w| w.mapped && w.surface.alive() && w.surface.wl_surface() == surface)
        else {
            return;
        };
        if window.elsewhere != value {
            window.visibility = Default::default();
        }
        window.elsewhere = value;
    }

    /// Change visibility only for an exact live buffered root.
    pub(crate) fn minimize(&mut self, surface: &WlSurface, value: bool) -> Option<bool> {
        let window = self
            .windows
            .iter_mut()
            .find(|w| w.mapped && w.surface.alive() && w.surface.wl_surface() == surface)?;
        let changed = window.minimized != value;
        if changed {
            window.visibility = Default::default();
        }
        window.minimized = value;
        Some(changed)
    }

    /// Live roles, regardless of buffer state.
    pub(crate) fn count(&self) -> usize {
        self.windows.iter().filter(|w| w.surface.alive()).count()
    }

    /// Resolve only a live mapped root owned by this display, never a child.
    pub(crate) fn mapped_toplevel(&self, surface: &WlSurface) -> Option<&ToplevelSurface> {
        self.windows
            .iter()
            .find(|w| w.drawn() && w.surface.wl_surface() == surface)
            .map(|w| &w.surface)
    }

    /// Raise only a live mapped root; refusal leaves the entire order unchanged.
    pub(crate) fn raise(&mut self, surface: &WlSurface) -> bool {
        let Some(index) = self
            .windows
            .iter()
            .position(|w| w.drawn() && w.surface.wl_surface() == surface)
        else {
            return false;
        };
        if let Some(prefix) = self.windows.get_mut(..=index) {
            prefix.rotate_right(1);
            true
        } else {
            false
        }
    }
}

impl CompositorHandler for Surfaces {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        // Server owns insertion exclusively. A wrong data type is an internal
        // invariant violation, never a client-controlled request. Fail closed
        // rather than share fallback transaction state between unrelated clients.
        &client
            .get_data::<ClientState>()
            .unwrap_or_else(|| std::process::abort())
            .compositor
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
        self.popups.commit(surface);
        let Some(window) = self
            .windows
            .iter_mut()
            .find(|w| w.surface.wl_surface() == surface)
        else {
            return;
        };
        let has_buffer =
            with_renderer_surface_state(surface, |state| state.buffer().is_some()).unwrap_or(false);
        if has_buffer {
            window.mapped = window.surface.ensure_configured();
        } else if window.mapped {
            window.visibility = Default::default();
            window.mapped = false;
            window.minimized = false;
            crate::window_placement::reset(surface);
            // XDG unmap requires a fresh handshake. Smithay resets its initial
            // configure flag, but retains `configured` and old acknowledgements.
            // Reset role state too so an old configure cannot authorize remapping.
            with_states(surface, |states| {
                if let Some(data) = states.data_map.get::<XdgToplevelSurfaceData>() {
                    *data.lock().unwrap_or_else(|_| std::process::abort()) = Default::default();
                }
            });
        } else if !window.surface.is_initial_configure_sent() {
            window.surface.with_pending_state(|pending| {
                // **Fullscreen joined these on 2026-09-30, when it became
                // true.** It was deliberately absent before — the set said
                // exactly what this compositor could do, and a capability
                // advertised without a handler is a promise a client acts on
                // and is answered with silence. `tests/support/wm_capabilities`
                // pinned the set to refuse exactly that, and it changes here
                // with the thing it was refusing rather than ahead of it.
                pending.capabilities.replace([
                    smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::WmCapabilities::Maximize,
                    smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::WmCapabilities::Minimize,
                    smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::WmCapabilities::Fullscreen,
                ]);
            });
            window.surface.send_configure();
        }
        self.prune_window_move();
        self.commit_window_resize(surface);
        self.commit_window_mode(surface);
        let parents: Vec<_> = self.mapped().cloned().collect();
        self.popups.prune(&parents);
    }
}

impl BufferHandler for Surfaces {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {
        // Renderer buffer state owns attached buffers and releases them on replacement.
    }
}
impl SeatHandler for Surfaces {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;
    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seats
    }
    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        self.cursor = image;
    }
}
impl ShmHandler for Surfaces {
    fn shm_state(&self) -> &ShmState {
        &self.shm
    }
}
impl XdgShellHandler for Surfaces {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg
    }
    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        // **A window is on a Place before it is anything else.** Assigned here
        // rather than when it maps, so that no toplevel this compositor knows
        // about is ever without one; `crate::canvas_place` says why that matters
        // more than it looks like it does.
        crate::canvas_place::put_on(surface.wl_surface(), self.place);
        // Initial configure is sent only after the client's first empty commit.
        self.windows.push(Window {
            visibility: Default::default(),
            elsewhere: false,
            surface,
            mapped: false,
            minimized: false,
        });
    }
    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        self.windows.retain(|w| w.surface != surface);
    }
    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        self.prune();
        let parents: Vec<_> = self.mapped().cloned().collect();
        self.popups
            .insert(surface, positioner, &parents, &self.cameras);
    }
    fn grab(&mut self, surface: PopupSurface, seat: WlSeat, serial: Serial) {
        self.grab_popup(surface, seat, serial);
    }
    /// **Refused, always.** ADR 0071: the name above a frame is what moves it, and
    /// the shell owns that band. A frame shows nothing but its content, so every
    /// press a client can hold a serial for is a press inside its own content —
    /// which means an application asking to be moved is asking on behalf of
    /// something that belongs to it.
    ///
    /// Refused outright rather than only when the press was in the content,
    /// because that test would depend on mapping a serial back to where its press
    /// began, and an implicit grab is exactly the case where that mapping stops
    /// being trustworthy: a client holding a button pressed inside itself receives
    /// the next press too, wherever the pointer has since travelled. A rule that
    /// depends on nothing beats a rule that depends on bookkeeping being right.
    ///
    /// **What this costs is visible and is understood.** An application that draws
    /// its own title bar — anything that has not negotiated server-side
    /// decorations — has a title bar that looks draggable and does nothing. That is
    /// not a reason to soften the rule; it is why decoration negotiation matters,
    /// and it is task 3 of `docs/autonomy/applications-people-already-use.md`,
    /// which names `xdg_decoration` as *whether the application or the shell draws
    /// the frame, which on a canvas is the shell*. Until that lands, a dead title
    /// bar is the symptom.
    fn move_request(&mut self, _surface: ToplevelSurface, _seat: WlSeat, _serial: Serial) {}
    fn maximize_request(&mut self, surface: ToplevelSurface) {
        self.client_window_maximize(surface, true);
    }
    fn minimize_request(&mut self, surface: ToplevelSurface) {
        // XDG has no minimized state or response handshake. Ignore unbuffered
        // intent; only this request's own mapped role may become hidden.
        let _ = self.set_window_minimized(surface.wl_surface(), true);
    }
    fn unmaximize_request(&mut self, surface: ToplevelSurface) {
        self.client_window_maximize(surface, false);
    }
    /// **Answered, where until 2026-09-30 it was met with silence.**
    ///
    /// Smithay's default implementation is empty, so a client asking to fill
    /// the screen received neither a configure nor a refusal — and a
    /// compositor that advertises nothing and answers nothing is
    /// indistinguishable, from the client's side, from one that is broken.
    ///
    /// The output is ignored. A client may name which screen it wants to fill;
    /// this compositor fills the one the window is on, because a request to
    /// move a window to another display is a *move* and is not something a
    /// window asks for on its own behalf (ADR 0071's reasoning, one request
    /// over).
    fn fullscreen_request(
        &mut self,
        surface: ToplevelSurface,
        _output: Option<smithay::reexports::wayland_server::protocol::wl_output::WlOutput>,
    ) {
        self.client_window_full_screen(surface, true);
    }
    fn unfullscreen_request(&mut self, surface: ToplevelSurface) {
        self.client_window_full_screen(surface, false);
    }
    /// **Refused outright, as ADR 0071 decided** — and it lands with the gesture
    /// that replaces it, never before.
    ///
    /// A chrome-less frame has no non-content area a client can be pressed in, so
    /// a client asking to be resized is asking on behalf of a press that belongs
    /// to it. The shell owns the edges and the corners exactly as it owns the
    /// name, and `crate::canvas_resize` is where a person reaches them.
    ///
    /// **The decisive argument is that it cannot be half-refused.** Resizing from
    /// the left edge and then from the right composes into a translation — same
    /// size, moved — so honouring resize while refusing move would leave move
    /// reachable in two gestures. A refusal that can be composed around is not a
    /// refusal, which is why ADR 0071 made this the twin of the move refusal
    /// rather than a separate decision.
    ///
    /// What this does **not** do is freeze a window's size. This request is
    /// specifically an interactive, pointer-driven resize; a client may still
    /// commit whatever size it likes and be drawn at it. What it loses is taking
    /// over the pointer to do it.
    fn resize_request(
        &mut self,
        _surface: ToplevelSurface,
        _seat: WlSeat,
        _serial: Serial,
        _edges: smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
    ) {
    }
    fn popup_destroyed(&mut self, surface: PopupSurface) {
        self.popup_grab_destroyed(&surface);
    }
    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        self.prune();
        let parents: Vec<_> = self.mapped().cloned().collect();
        self.popups
            .reposition(&surface, positioner, token, &parents, &self.cameras);
    }
}

delegate_compositor!(Surfaces);
/// **What the compositor itself does with a selection: nothing.**
///
/// `SelectionUserData` is `()` because this shell never *offers* a selection of
/// its own — it carries what one client put there to whichever client asks. The
/// moment the shell wants to paste something itself, that unit type is where
/// the thing being pasted goes, and the compiler will say so.
///
/// Both methods keep their defaults. `new_selection` is a notification, and
/// `send_selection` only fires for a selection the *compositor* set, which is
/// the case this shell does not have yet.
impl SelectionHandler for Surfaces {
    type SelectionUserData = ();
}

/// A client's own drag, which this shell does not decorate or interfere with.
///
/// The defaults are correct rather than unfinished: `started` and `dropped` are
/// hooks for a compositor that wants to draw something extra during a drag, and
/// smithay already moves the drag icon with the pointer.
impl ClientDndGrabHandler for Surfaces {}

/// A drag the *compositor* started, which this shell never does.
///
/// Kept as an empty implementation rather than omitted because
/// `DataDeviceHandler` requires it. If this shell ever offers a drag of its own
/// — dragging a file out of a panel, say — these are the methods that stop
/// being empty.
impl ServerDndGrabHandler for Surfaces {}

impl DataDeviceHandler for Surfaces {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device
    }
}

/// **This shell draws every window's frame, and that is not a preference.**
///
/// # Why the answer is the same three times
///
/// `crate::canvas_never_lost` promises a person can always get a frame back,
/// and it keeps that promise by measuring the frame's **name band** — the
/// shell-drawn strip a window is dragged by. A client drawing its own
/// decoration would leave nothing to measure, so the promise would hold for
/// some windows and silently not for others, decided by each application rather
/// than by this compositor.
///
/// So `request_mode` **ignores the mode it is handed.** That is the protocol
/// working as specified rather than a shortcut: `zxdg_toplevel_decoration_v1`
/// lets a client state a preference and leaves the decision with the
/// compositor, precisely so a compositor whose layout depends on drawing the
/// frame can say so.
///
/// `unset_mode` — *I no longer have a preference* — is the same answer for the
/// same reason, and `new_decoration` sends it before the client has asked, so a
/// client that would have drawn its own never starts.
impl XdgDecorationHandler for Surfaces {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        Self::the_shell_draws_this_frame(&toplevel);
    }

    fn request_mode(&mut self, toplevel: ToplevelSurface, _mode: Mode) {
        Self::the_shell_draws_this_frame(&toplevel);
    }

    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        Self::the_shell_draws_this_frame(&toplevel);
    }
}

impl Surfaces {
    /// Tell one toplevel that the shell draws its frame.
    ///
    /// One function rather than three copies, because three identical answers
    /// written out three times is three places for them to stop being
    /// identical — and the one that drifted would be a window drawing its own
    /// frame on a canvas that measures the shell's.
    fn the_shell_draws_this_frame(toplevel: &ToplevelSurface) {
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(Mode::ServerSide);
        });
        // Only a mapped toplevel may be configured; an unmapped one is
        // configured when it maps, carrying the pending state set above.
        if toplevel.is_initial_configure_sent() {
            toplevel.send_pending_configure();
        }
    }
}

delegate_xdg_decoration!(Surfaces);

delegate_data_device!(Surfaces);

delegate_shm!(Surfaces);
delegate_xdg_shell!(Surfaces);

delegate_pointer_gestures!(Surfaces);
delegate_viewporter!(Surfaces);
delegate_text_input_manager!(Surfaces);

use smithay::{
    backend::allocator::{dmabuf::Dmabuf, format::FormatSet},
    delegate_dmabuf,
    wayland::dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier},
};

impl DmabufHandler for Surfaces {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.dmabuf
    }

    /// Hold the offer. The loop answers it, after a real import.
    ///
    /// **Nothing is decided here, and that is the point.** This runs inside
    /// `dispatch_clients`, where the renderer is not reachable: it belongs to
    /// the graphics backend. Answering `successful` from here would be the
    /// unconditional acceptance the owner's ruling of 2026-10-02 forbids —
    /// true-sounding, unverified, and discovered to be false only when the
    /// frame is drawn.
    ///
    /// The global is ignored because this shell advertises exactly one and the
    /// renderer behind it is the same renderer whichever global a buffer came
    /// through.
    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
        notifier: ImportNotifier,
    ) {
        self.handed_over
            .push(crate::buffers_clients_hand_over::HandedOver {
                buffer: dmabuf,
                telling: notifier,
            });
    }
}
delegate_dmabuf!(Surfaces);
