//! XDG surface handshake, renderer buffer ownership and lifecycle cleanup.

use smithay::{
    backend::renderer::utils::{on_commit_buffer_handler, with_renderer_surface_state},
    delegate_compositor, delegate_shm, delegate_xdg_shell,
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
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            XdgToplevelSurfaceData,
        },
        shm::{ShmHandler, ShmState},
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
    /// Smithay's role handle.
    surface: ToplevelSurface,
    /// A configured buffer is currently attached.
    mapped: bool,
}

/// Protocol globals and toplevel roots shared by display backends.
pub(crate) struct Surfaces {
    /// Core surface/subsurface protocol.
    compositor: CompositorState,
    /// CPU-backed application buffers.
    shm: ShmState,
    /// XDG shell role and configure tracking.
    xdg: XdgShellState,
    /// Live toplevel roots in creation order.
    windows: Vec<Window>,
    /// Seat globals, created only when the backend enables input.
    pub(crate) seats: SeatState<Self>,
    /// Optional keyboard seat and routing state.
    pub(crate) keyboard: Option<crate::keyboard::Keyboard>,
}

impl Surfaces {
    /// Advertise only protocols this component implements.
    pub(crate) fn new(display: &DisplayHandle) -> Self {
        Self {
            compositor: CompositorState::new::<Self>(display),
            shm: ShmState::new::<Self>(display, vec![]),
            xdg: XdgShellState::new::<Self>(display),
            windows: Vec::new(),
            seats: SeatState::new(),
            keyboard: None,
        }
    }

    /// Remove resources whose client disappeared without orderly destruction.
    pub(crate) fn prune(&mut self) {
        self.windows.retain(|window| window.surface.alive());
        self.prune_keyboard_focus();
    }

    /// Roots eligible for rendering; dead handles never escape this iterator.
    pub(crate) fn mapped(&self) -> impl Iterator<Item = &WlSurface> {
        self.windows
            .iter()
            .filter(|w| w.mapped && w.surface.alive())
            .map(|w| w.surface.wl_surface())
    }

    /// Live roles, regardless of buffer state.
    pub(crate) fn count(&self) -> usize {
        self.windows.iter().filter(|w| w.surface.alive()).count()
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
            window.mapped = false;
            // XDG unmap requires a fresh handshake. Smithay resets its initial
            // configure flag, but retains `configured` and old acknowledgements.
            // Reset role state too so an old configure cannot authorize remapping.
            with_states(surface, |states| {
                if let Some(data) = states.data_map.get::<XdgToplevelSurfaceData>() {
                    *data.lock().unwrap_or_else(|_| std::process::abort()) = Default::default();
                }
            });
        } else if !window.surface.is_initial_configure_sent() {
            window.surface.send_configure();
        }
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
    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {
        // Pointer capability is not advertised until its backend is implemented.
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
        // Initial configure is sent only after the client's first empty commit.
        self.windows.push(Window {
            surface,
            mapped: false,
        });
    }
    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        self.windows.retain(|w| w.surface != surface);
    }
    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        // Popup placement/input is a subsequent component. Dismiss explicitly;
        // never leave a client waiting on a configure we cannot fulfil.
        surface.send_popup_done();
    }
    fn grab(&mut self, surface: PopupSurface, _seat: WlSeat, _serial: Serial) {
        surface.send_popup_done();
    }
    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
        surface.send_popup_done();
    }
}

delegate_compositor!(Surfaces);
delegate_shm!(Surfaces);
delegate_xdg_shell!(Surfaces);
