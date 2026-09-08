//! Display ownership and bounded, nonblocking dispatch for graphics backends.

use std::{io, path::Path, sync::Arc};

use smithay::reexports::wayland_server::{Display, protocol::wl_surface::WlSurface};

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
            display,
            surfaces,
            socket,
            presentation: Default::default(),
            switch_order: Default::default(),
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
        self.switch_order.refresh(self.surfaces.mapped());
        self.display.flush_clients()
    }

    /// Live, configured toplevel roots with buffers, in front-to-back order.
    /// New roles initially follow existing roles; explicit raising changes order.
    ///
    /// This is an internal renderer input, not an agent context or window API.
    /// Buffer removal, surface destruction and disconnect remove a root here.
    pub fn mapped_surfaces(&self) -> impl Iterator<Item = &WlSurface> {
        self.surfaces.mapped()
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
        let size = target.size();
        self.presentation.validate_target(target)?;
        if size.w > 0 && size.h > 0 {
            // Reactive popup negotiation follows the backend's desired extent,
            // even on submission refusal. wl_output describes only submitted modes.
            self.surfaces.popups.output_size = Some(size);
        }
        self.surfaces.prune();
        let roots: Vec<_> = self.mapped_surfaces().cloned().collect();
        let cursor = self.cursor();
        let popups = self.popup_surfaces();
        self.presentation.render(
            &self.display.handle(),
            target,
            (&roots, &popups),
            &cursor,
            time,
        )
    }
}
