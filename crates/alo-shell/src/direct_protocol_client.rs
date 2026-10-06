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
    /// Unmodified output wire events from the direct backend metadata path.
    outputs: Vec<wl_output::Event>,
    /// Registry withdrawal count; output objects may remain client-owned.
    removed: usize,
    /// Advertised names retained to exercise a delayed bind after withdrawal.
    output_globals: Vec<u32>,
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
        if matches!(event, wl_registry::Event::GlobalRemove { .. }) {
            state.removed += 1;
        }
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
                    state.output_globals.push(name);
                    let _: wl_output::WlOutput = registry.bind(name, version.min(4), qh, ());
                }
                _ => {}
            }
        }
    }
}

/// Real protocol retirement, failed-disable refusal and a fresh output lifetime.
pub(crate) fn retirement(
    socket: UnixStream,
    send: mpsc::Sender<(u32, u32)>,
    responses: mpsc::Receiver<()>,
    refuse: bool,
) -> Result<(), String> {
    retirement_inner(socket, send, responses, refuse).map_err(|error| error.to_string())
}

fn retirement_inner(
    socket: UnixStream,
    send: mpsc::Sender<(u32, u32)>,
    responses: mpsc::Receiver<()>,
    refuse: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    socket.set_read_timeout(Some(Duration::from_secs(3)))?;
    let connection = Connection::from_socket(socket)?;
    let mut queue = connection.new_event_queue();
    let qh = queue.handle();
    let registry = connection.display().get_registry(&qh, ());
    let mut events = Events::default();
    queue.roundtrip(&mut events)?;
    let surface = events
        .compositor
        .as_ref()
        .ok_or("missing compositor")?
        .create_surface(&qh, ());
    for stage in 0..=3 {
        if stage == 2 && !refuse {
            let _: wl_output::WlOutput = registry.bind(
                *events
                    .output_globals
                    .first()
                    .ok_or("missing retired global")?,
                4,
                &qh,
                (),
            );
        }
        if stage <= 1 {
            surface.frame(&qh, ());
            surface.commit();
        }
        queue.roundtrip(&mut events)?;
        send.send((stage, surface.id().protocol_id()))?;
        responses.recv_timeout(Duration::from_secs(5))?;
        queue.roundtrip(&mut events)?;
        queue.roundtrip(&mut events)?;
        assert_eq!(events.removed, usize::from(stage >= 1 && !refuse));
        let resumed = stage == 3 && !refuse;
        assert_eq!(events.frames, if resumed { vec![80, 83] } else { vec![80] });
        assert_eq!(
            events.membership,
            if resumed {
                (2, 1)
            } else if stage >= 1 && !refuse {
                (1, 1)
            } else {
                (1, 0)
            }
        );
        let names: Vec<_> = events
            .outputs
            .iter()
            .filter_map(|event| match event {
                wl_output::Event::Name { name } => Some(name.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            names,
            if resumed {
                vec!["alo-drm-1", "alo-drm-1", "alo-drm-4"]
            } else if stage >= 2 && !refuse {
                vec!["alo-drm-1", "alo-drm-1"]
            } else {
                vec!["alo-drm-1"]
            }
        );
    }
    Ok(())
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
impl Dispatch<wl_output::WlOutput, ()> for Events {
    fn event(
        state: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        state.outputs.push(event);
    }
}

/// Drive a real surface and callbacks; scene eligibility is controlled by the server fixture.
pub(crate) fn run(
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
        queue.roundtrip(&mut events)?;
        let expected: &[u32] = match stage {
            0 => &[],
            1..=4 => &[71],
            _ => &[71, 75],
        };
        assert_eq!(events.frames, expected);
        if stage == 0 {
            assert!(events.outputs.is_empty());
        } else {
            assert!(events.outputs.iter().any(|event| matches!(event,
                wl_output::Event::Name { name } if name == "alo-drm-1")));
            assert!(events.outputs.iter().any(|event| matches!(event,
                wl_output::Event::Geometry { physical_width: 310, physical_height: 170, make, model, .. }
                if make == "unknown" && model == "unknown")));
            assert!(events.outputs.iter().any(|event| matches!(
                event,
                wl_output::Event::Mode {
                    width: 1280,
                    height: 720,
                    refresh: 60_000,
                    ..
                }
            )));
        }
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

/// One surface, two displays, and only one of them drawing it.
///
/// `more-than-one-display-plan.md` task 4's second acceptance clause: **a
/// client drawn on a display gets its frame callback from the display that
/// drew it.** The client asks for one callback, the server draws both
/// displays in one pass, and only the first reports having submitted the
/// surface — so a compositor that sent callbacks from the wrong display, or
/// from every display, is caught by the count rather than by inspection.
pub(crate) fn one_of_two_displays(
    socket: UnixStream,
    send: mpsc::Sender<(u32, u32)>,
    responses: mpsc::Receiver<()>,
    drawn_at: u32,
) -> Result<(), String> {
    one_of_two_displays_inner(socket, send, responses, drawn_at).map_err(|error| error.to_string())
}

fn one_of_two_displays_inner(
    socket: UnixStream,
    send: mpsc::Sender<(u32, u32)>,
    responses: mpsc::Receiver<()>,
    drawn_at: u32,
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
    surface.frame(&qh, ());
    surface.commit();
    queue.roundtrip(&mut events)?;
    send.send((0, surface.id().protocol_id()))?;
    responses.recv_timeout(Duration::from_secs(5))?;
    queue.roundtrip(&mut events)?;
    queue.roundtrip(&mut events)?;
    // **Exactly one**, carrying the time the drawing display drew at. Two
    // would be the fault worth naming: a callback from a display that never
    // put this surface on a screen tells the client to draw again for a frame
    // nobody saw.
    assert_eq!(
        events.frames,
        vec![drawn_at],
        "the frame callback did not come from the display that drew"
    );
    // And it entered that display's output alone. A surface drawn on one
    // screen that is a member of both is a client told it is somewhere it is
    // not — which is what decides where it looks for scale and transform.
    assert_eq!(
        events.membership,
        (1, 0),
        "the surface entered an output that never drew it"
    );
    Ok(())
}
