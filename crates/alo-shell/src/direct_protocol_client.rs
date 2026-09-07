//! Minimal wire client for frame completion and output membership assertions.
use std::{os::unix::net::UnixStream, sync::mpsc, time::Duration};
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, delegate_noop,
    protocol::{wl_callback, wl_compositor, wl_output, wl_registry, wl_surface},
};

/// Only events relevant to the direct presentation contract.
#[derive(Default)]
struct Events {
    /// Surface factory bound at registry discovery.
    compositor: Option<wl_compositor::WlCompositor>,
    /// Received callback timestamps.
    frames: Vec<u32>,
    /// Received output enter/leave counts.
    membership: (usize, usize),
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
            match interface.as_str() {
                "wl_compositor" => {
                    state.compositor = Some(registry.bind(name, version.min(6), qh, ()))
                }
                "wl_output" => {
                    let _: wl_output::WlOutput = registry.bind(name, version.min(4), qh, ());
                }
                _ => {}
            }
        }
    }
}
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
delegate_noop!(Events: ignore wl_compositor::WlCompositor);
delegate_noop!(Events: ignore wl_output::WlOutput);

/// Drive a real surface and callbacks; scene eligibility is controlled by the server fixture.
pub(super) fn run(
    socket: UnixStream,
    send: mpsc::Sender<(u32, u32)>,
    responses: mpsc::Receiver<()>,
) -> Result<(), String> {
    run_inner(socket, send, responses).map_err(|error| error.to_string())
}

/// Typed errors stay inside the client thread, avoiding non-Send error objects.
fn run_inner(
    socket: UnixStream,
    send: mpsc::Sender<(u32, u32)>,
    responses: mpsc::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    socket.set_read_timeout(Some(Duration::from_secs(3)))?;
    let connection = Connection::from_socket(socket)?;
    let mut queue = connection.new_event_queue();
    let qh = queue.handle();
    connection.display().get_registry(&qh, ());
    let mut events = Events::default();
    queue.roundtrip(&mut events)?;
    let surface = events
        .compositor
        .as_ref()
        .ok_or("missing compositor")?
        .create_surface(&qh, ());
    for stage in 0..=6 {
        if stage == 0 || stage == 2 || stage == 6 {
            surface.frame(&qh, ());
            surface.commit();
        }
        queue.roundtrip(&mut events)?;
        send.send((stage, surface.id().protocol_id()))?;
        responses.recv_timeout(Duration::from_secs(5))?;
        queue.roundtrip(&mut events)?;
        let expected: &[u32] = match stage {
            0 => &[],
            1..=4 => &[71],
            _ => &[71, 75],
        };
        assert_eq!(events.frames, expected);
        assert_eq!(
            events.membership,
            match stage {
                0 => (0, 0),
                1..=3 => (1, 0),
                4 => (1, 1),
                _ => (2, 1),
            }
        );
    }
    Ok(())
}
