//! XDG surface handshake, renderer buffer ownership and lifecycle cleanup.

use smithay::{
    backend::renderer::utils::{on_commit_buffer_handler, with_renderer_surface_state},
    delegate_compositor, delegate_data_device, delegate_pointer_gestures, delegate_shm,
    delegate_viewporter, delegate_xdg_shell,
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
    /// **Copy and paste, and drag-and-drop between applications.**
    ///
    /// Held here rather than per-window because a selection is per *seat*: one
    /// person's clipboard, whichever window they were in when they took it.
    data_device: DataDeviceState,
    /// Live toplevel roots in front-to-back stacking order.
    windows: Vec<Window>,
    /// **Which Place the person is looking at.**
    ///
    /// One home, not two. `Server::camera` has a second copy in
    /// `self.popups.camera` that three mutators keep in step by hand, and this
    /// deliberately does not repeat that: a new toplevel is created here, on
    /// `Surfaces`, so the Place it is put on has to be reachable from here, and
    /// the shell reads it back through `Server::the_place_now`.
    pub(crate) place: alo_canvas::Place,
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
            windows: Vec::new(),
            // A machine that has never been used is looking at its first Place.
            place: alo_canvas::Place::FIRST,
            showing: alo_canvas::Showing::OnePlace(alo_canvas::Place::FIRST),
            seats: SeatState::new(),
            keyboard: None,
            pointer: None,
            cursor: CursorImageStatus::default_named(),
        }
    }

    /// Remove resources whose client disappeared without orderly destruction.
    pub(crate) fn prune(&mut self) {
        self.windows.retain(|window| window.surface.alive());
        self.prune_window_move();
        self.prune_window_resize();
        self.prune_window_modes();
        let parents: Vec<_> = self.mapped().cloned().collect();
        self.popups.prune(&parents);
        self.popups.refresh(&parents);
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

    /// Identity changes at each unmap or visibility transition, including within dispatch.
    pub(crate) fn window_visibility(&self, surface: &WlSurface) -> Option<std::sync::Arc<()>> {
        self.windows
            .iter()
            .find(|w| w.drawn() && w.surface.wl_surface() == surface)
            .map(|w| w.visibility.clone())
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
        self.popups.insert(surface, positioner, &parents);
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
            .reposition(&surface, positioner, token, &parents);
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

delegate_data_device!(Surfaces);

delegate_shm!(Surfaces);
delegate_xdg_shell!(Surfaces);

delegate_pointer_gestures!(Surfaces);
delegate_viewporter!(Surfaces);
