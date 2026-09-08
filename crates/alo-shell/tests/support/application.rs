//! Real XDG client, protocol events and shared-memory buffer fixture.
#![expect(
    clippy::unwrap_used,
    reason = "unexpected results fail the integration test"
)]
use super::Fixture;
#[path = "keyboard_events.rs"]
mod keyboard_events;
#[path = "pixel_buffers.rs"]
mod pixel_buffers;
#[path = "pointer_events.rs"]
mod pointer_events;
#[path = "popup_events.rs"]
mod popup_events;
use std::{
    fs::File,
    io::Write,
    os::{fd::AsFd, unix::net::UnixStream},
    time::Duration,
};
use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle, delegate_noop,
    protocol::{
        wl_buffer, wl_callback, wl_compositor, wl_output, wl_registry, wl_shm, wl_shm_pool,
        wl_subcompositor, wl_subsurface, wl_surface,
    },
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
/// Registry and configure events actually received from the compositor.
#[derive(Default)]
pub struct Events {
    /// Logical sizes suggested by successive XDG toplevel configures.
    pub sizes: Vec<(i32, i32)>,
    /// Activated flags in successive XDG toplevel configures, observed on wire.
    pub activation: Vec<bool>,
    /// Resizing flags received in successive configurations.
    pub resizing: Vec<bool>,
    /// Maximized flags received in successive configurations.
    pub maximized: Vec<bool>,
    /// Cooperative close requests received; the fixture never closes implicitly.
    pub close_requests: usize,
    /// Popup configuration and terminal dismissal wire events.
    pub popups: popup_events::PopupEvents,
    /// Pointer events observed on the wire.
    pub pointer: pointer_events::PointerEvents,
    /// Keyboard wire events, separate from buffer lifecycle observations.
    pub keyboard: keyboard_events::KeyboardEvents,
    /// Advertised globals with their names and versions.
    globals: Vec<(u32, String, u32)>,
    /// Most recent XDG configure serial.
    pub serial: Option<u32>,
    /// Buffer releases received from the server.
    pub releases: usize,
    /// Frame callback timestamps received, in completion order.
    pub frames: Vec<u32>,
    /// Current output modes received from the single advertised output.
    pub modes: Vec<(i32, i32)>,
    /// Output metadata events retained for protocol assertions.
    pub output_events: Vec<wl_output::Event>,
    /// Number of output globals bound.
    pub outputs: usize,
    /// Output enter and leave event counts.
    pub membership: (usize, usize),
}

impl Dispatch<wl_registry::WlRegistry, ()> for Events {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == "wl_output" {
                let _: wl_output::WlOutput = registry.bind(name, version.min(4), qh, ());
                state.outputs += 1;
            }
            if interface == "wl_seat" {
                let _: wayland_client::protocol::wl_seat::WlSeat =
                    registry.bind(name, version.min(9), qh, ());
            }
            state.globals.push((name, interface, version));
        }
    }
}
impl Dispatch<xdg_surface::XdgSurface, ()> for Events {
    fn event(
        state: &mut Self,
        _: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            state.serial = Some(serial);
        }
    }
}
impl Dispatch<xdg_wm_base::XdgWmBase, ()> for Events {
    fn event(
        _: &mut Self,
        proxy: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            proxy.pong(serial);
        }
    }
}
delegate_noop!(Events: ignore wl_compositor::WlCompositor);
delegate_noop!(Events: ignore wl_subcompositor::WlSubcompositor);
delegate_noop!(Events: ignore wl_subsurface::WlSubsurface);
impl Dispatch<wl_surface::WlSurface, ()> for Events {
    fn event(
        state: &mut Self,
        _: &wl_surface::WlSurface,
        event: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_surface::Event::Enter { .. } => state.membership.0 += 1,
            wl_surface::Event::Leave { .. } => state.membership.1 += 1,
            _ => {}
        }
    }
}
impl Dispatch<wl_callback::WlCallback, ()> for Events {
    fn event(
        state: &mut Self,
        _: &wl_callback::WlCallback,
        event: wl_callback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { callback_data } = event {
            state.frames.push(callback_data);
        }
    }
}
impl Dispatch<wl_output::WlOutput, ()> for Events {
    fn event(
        state: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_output::Event::Mode {
            flags: wayland_client::WEnum::Value(flags),
            width,
            height,
            ..
        } = event
            && flags.contains(wl_output::Mode::Current)
        {
            state.modes.push((width, height));
        }
        state.output_events.push(event);
    }
}
delegate_noop!(Events: ignore wl_shm::WlShm);
delegate_noop!(Events: ignore wl_shm_pool::WlShmPool);
impl Dispatch<wl_buffer::WlBuffer, ()> for Events {
    fn event(
        state: &mut Self,
        _: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_buffer::Event::Release = event {
            state.releases += 1;
        }
    }
}
impl Dispatch<xdg_toplevel::XdgToplevel, ()> for Events {
    fn event(
        state: &mut Self,
        _: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            xdg_toplevel::Event::Close => state.close_requests += 1,
            xdg_toplevel::Event::Configure {
                width,
                height,
                states,
            } => {
                state.sizes.push((width, height));
                state
                    .maximized
                    .push(states.as_chunks::<4>().0.iter().any(|bytes| {
                        u32::from_ne_bytes(*bytes) == xdg_toplevel::State::Maximized as u32
                    }));
                state
                    .resizing
                    .push(states.as_chunks::<4>().0.iter().any(|bytes| {
                        u32::from_ne_bytes(*bytes) == xdg_toplevel::State::Resizing as u32
                    }));
                state
                    .activation
                    .push(states.as_chunks::<4>().0.iter().any(|bytes| {
                        u32::from_ne_bytes(*bytes) == xdg_toplevel::State::Activated as u32
                    }));
            }
            _ => {}
        }
    }
}

/// A real XDG application with a CPU-backed 16x16 ARGB buffer.
pub struct Application {
    /// XDG role factory retained for popup fixtures.
    shell: xdg_wm_base::XdgWmBase,
    /// Connection retained for explicit flush and protocol error inspection.
    pub connection: Connection,
    /// Client event queue.
    pub queue: EventQueue<Events>,
    /// Received protocol state.
    pub events: Events,
    /// Root surface.
    pub surface: wl_surface::WlSurface,
    /// XDG handshake object.
    pub xdg: xdg_surface::XdgSurface,
    /// Window role.
    pub toplevel: xdg_toplevel::XdgToplevel,
    /// Backing buffer, created through SCM_RIGHTS fd transfer.
    buffer: wl_buffer::WlBuffer,
    /// SHM factory for independent pixel fixtures.
    shm: wl_shm::WlShm,
    /// Surface factory retained for synchronized child fixtures.
    compositor: wl_compositor::WlCompositor,
    /// Subsurface-role factory.
    subcompositor: wl_subcompositor::WlSubcompositor,
}

impl Application {
    /// Send a cursor request with an explicit serial for authorization tests.
    pub fn set_cursor(
        &self,
        serial: u32,
        surface: Option<&wl_surface::WlSurface>,
        hotspot: (i32, i32),
    ) {
        self.events
            .pointer
            .proxy
            .as_ref()
            .unwrap()
            .set_cursor(serial, surface, hotspot.0, hotspot.1);
    }
    /// Create a buffered cursor with a pending frame callback using real requests.
    pub fn cursor(&self, hotspot: (i32, i32)) -> wl_surface::WlSurface {
        let qh = self.queue.handle();
        let surface = self.compositor.create_surface(&qh, ());
        self.events.pointer.proxy.as_ref().unwrap().set_cursor(
            self.events.pointer.serial,
            Some(&surface),
            hotspot.0,
            hotspot.1,
        );
        surface.attach(Some(&self.buffer), 0, 0);
        surface.frame(&qh, ());
        surface.commit();
        surface
    }
    /// Commit an empty input region so hits fall through to another surface.
    pub fn empty_input(&self) {
        self.empty_input_on(&self.surface);
    }
    /// Commit an empty input region on a particular popup or child surface.
    pub fn empty_input_on(&self, surface: &wl_surface::WlSurface) {
        let region = self.compositor.create_region(&self.queue.handle(), ());
        surface.set_input_region(Some(&region));
        region.destroy();
        surface.commit();
    }
    /// Connect and create a role without committing or acknowledging anything.
    pub fn new(fixture: &Fixture) -> Self {
        let stream = UnixStream::connect(&fixture.path).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let connection = Connection::from_socket(stream).unwrap();
        let mut queue = connection.new_event_queue();
        let qh = queue.handle();
        let registry = connection.display().get_registry(&qh, ());
        let mut events = Events::default();
        queue.roundtrip(&mut events).unwrap();
        let global = |interface: &str| {
            events
                .globals
                .iter()
                .find(|(_, name, _)| name == interface)
                .unwrap()
                .0
        };
        let compositor: wl_compositor::WlCompositor =
            registry.bind(global("wl_compositor"), 4, &qh, ());
        let subcompositor = registry.bind(global("wl_subcompositor"), 1, &qh, ());
        let shm: wl_shm::WlShm = registry.bind(global("wl_shm"), 1, &qh, ());
        let shell: xdg_wm_base::XdgWmBase = registry.bind(global("xdg_wm_base"), 6, &qh, ());
        let surface = compositor.create_surface(&qh, ());
        let xdg = shell.get_xdg_surface(&surface, &qh, ());
        let toplevel = xdg.get_toplevel(&qh, ());
        let mut file: File = tempfile::tempfile().unwrap();
        file.write_all(&[0xff; 16 * 16 * 4]).unwrap();
        let pool = shm.create_pool(file.as_fd(), 16 * 16 * 4, &qh, ());
        let buffer = pool.create_buffer(0, 16, 16, 16 * 4, wl_shm::Format::Argb8888, &qh, ());
        pool.destroy();
        queue.roundtrip(&mut events).unwrap();
        Self {
            shell,
            connection,
            queue,
            events,
            surface,
            xdg,
            toplevel,
            buffer,
            shm,
            compositor,
            subcompositor,
        }
    }

    /// Attach a synchronized child and request its frame; parent commit applies it.
    pub fn child(
        &self,
        position: (i32, i32),
    ) -> (wl_surface::WlSurface, wl_subsurface::WlSubsurface) {
        self.child_on(&self.surface, position)
    }
    /// Create a synchronized tree child under a popup or toplevel buffer.
    pub fn child_on(
        &self,
        parent: &wl_surface::WlSurface,
        position: (i32, i32),
    ) -> (wl_surface::WlSurface, wl_subsurface::WlSubsurface) {
        let qh = self.queue.handle();
        let surface = self.compositor.create_surface(&qh, ());
        let role = self.subcompositor.get_subsurface(&surface, parent, &qh, ());
        role.set_position(position.0, position.1);
        surface.attach(Some(&self.buffer), 0, 0);
        surface.damage_buffer(0, 0, 16, 16);
        surface.frame(&qh, ());
        surface.commit();
        (surface, role)
    }

    /// Round trip through the actual server.
    pub fn sync(&mut self) {
        self.queue.roundtrip(&mut self.events).unwrap();
    }

    /// Complete a fresh empty-commit/configure/ack sequence.
    pub fn configure(&mut self) -> u32 {
        self.events.serial = None;
        self.surface.commit();
        self.sync();
        let serial = self.events.serial.unwrap();
        self.xdg.ack_configure(serial);
        self.sync();
        serial
    }

    /// Attach and commit a real SHM buffer, without an implicit acknowledgement.
    pub fn attach(&self) {
        self.surface.attach(Some(&self.buffer), 0, 0);
        self.surface.damage_buffer(0, 0, 16, 16);
        self.surface.commit();
    }

    /// Assert that the server rejected this client's protocol request.
    pub fn refused(&mut self) {
        assert!(self.queue.roundtrip(&mut self.events).is_err());
        assert!(self.connection.protocol_error().is_some());
    }
}
