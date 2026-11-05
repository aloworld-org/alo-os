//! Timestamp and publish complete BGRx frames with a bounded engine queue.

use alo_capture_protocol::{MediaNode, frame_bytes, write_message};
use gst::prelude::*;
use pipewire::{self as pw, types::ObjectType};
use std::{
    cell::Cell,
    io::{self, Read},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

/// One worker owns one native stream until its portal closes the input socket.
pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let width: u32 = args.next().ok_or("width required")?.parse()?;
    let height: u32 = args.next().ok_or("height required")?.parse()?;
    let name = args.next().ok_or("unique node name required")?;
    let application = args.next().ok_or("authenticated application required")?;
    if args.next().is_some() || !valid(&name) || !valid(&application) {
        return Err("invalid capture worker arguments".into());
    }
    let bytes = frame_bytes(width, height)?;
    gst::init()?;
    let pipeline = gst::Pipeline::new();
    // This worker drives video from compositor arrivals. Its PipeWire output
    // clock can remain at zero until a consumer starts the graph, which would
    // stamp those arrivals with identical times and stall rate conversion.
    pipeline.use_clock(Some(&gst::SystemClock::obtain()));
    let source = gst_app::AppSrc::builder()
        .is_live(true)
        .do_timestamp(false)
        .format(gst::Format::Time)
        .max_buffers(2)
        .max_bytes(0)
        .block(false)
        .leaky_type(gst_app::AppLeakyType::Downstream)
        .caps(
            &gst::Caps::builder("video/x-raw")
                .field("format", "BGRx")
                .field("width", i32::try_from(width)?)
                .field("height", i32::try_from(height)?)
                .field("framerate", gst::Fraction::new(0, 1))
                .field("pixel-aspect-ratio", gst::Fraction::new(1, 1))
                .build(),
        )
        .build();
    let sink = gst::ElementFactory::make("pipewiresink")
        .property_from_str("mode", "provide")
        // PipeWire's header carries monotonic capture time, not this
        // GStreamer pipeline's running time. Do not schedule that absolute
        // timestamp as a relative presentation time in the publisher.
        .property("sync", false)
        .property(
            "stream-properties",
            gst::Structure::builder("props")
                .field("node.name", &name)
                .field("media.class", "Stream/Output/Video")
                .field("media.type", "Video")
                .field("media.category", "Capture")
                .field("pipewire.access.portal.app_id", &application)
                .build(),
        )
        .property(
            "client-properties",
            gst::Structure::builder("props")
                .field("application.name", "alo screen sharing")
                .field("pipewire.access.portal.app_id", &application)
                .build(),
        )
        .build()?;
    pipeline.add_many([source.upcast_ref(), &sink])?;
    source.link(&sink)?;
    pipeline.set_state(gst::State::Playing)?;
    let ended = Arc::new(AtomicBool::new(false));
    let input_ended = ended.clone();
    let input = source.clone();
    std::thread::Builder::new()
        .name("compositor-frames".into())
        .spawn(move || {
            let result = feed(input.clone(), bytes);
            if let Err(why) = result {
                eprintln!("capture input ended: {why}");
            }
            let _ = input.end_of_stream();
            input_ended.store(true, Ordering::Release);
        })?;
    let node = identify(&name, &ended)?;
    write_message(&mut io::stdout().lock(), &node)?;
    let bus = pipeline.bus().ok_or("the capture engine has no bus")?;
    let result = loop {
        if let Some(message) = bus.timed_pop(gst::ClockTime::from_mseconds(100)) {
            match message.view() {
                gst::MessageView::Error(error) => break Err(error.error().to_string()),
                gst::MessageView::Eos(..) => break Ok(()),
                _ => {}
            }
        }
        if ended.load(Ordering::Acquire) {
            break Ok(());
        }
    };
    pipeline.set_state(gst::State::Null)?;
    result.map_err(Into::into)
}

/// The broker supplies opaque ASCII node and application identities only.
fn valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
}

/// Stamp capture arrivals in the monotonic domain shared by PipeWire clients.
fn feed(source: gst_app::AppSrc, bytes: usize) -> io::Result<()> {
    let mut input = io::stdin().lock();
    let clock = gst::SystemClock::obtain();
    loop {
        let mut frame = vec![0; bytes];
        input.read_exact(&mut frame)?;
        let mut frame = gst::Buffer::from_mut_slice(frame);
        let captured = clock.time();
        frame.make_mut().set_pts(captured);
        frame.make_mut().set_dts(captured);
        source.push_buffer(frame).map_err(io::Error::other)?;
    }
}

/// Ask the actual registry for the immutable identity of this worker's own node.
fn identify(name: &str, ended: &AtomicBool) -> io::Result<MediaNode> {
    let main_loop = pw::main_loop::MainLoopBox::new(None).map_err(failed)?;
    let context = pw::context::ContextBox::new(main_loop.loop_(), None).map_err(failed)?;
    let core = context.connect(None).map_err(failed)?;
    let registry = core.get_registry().map_err(failed)?;
    let found = Rc::new(Cell::new(None));
    let seen = found.clone();
    let name = name.to_owned();
    let _listener = registry
        .add_listener_local()
        .global(move |global| {
            if global.type_ != ObjectType::Node {
                return;
            }
            let Some(props) = global.props else {
                return;
            };
            if props.get("node.name") == Some(name.as_str())
                && let Some(serial) = props.get("object.serial").and_then(|s| s.parse().ok())
            {
                seen.set(Some(MediaNode {
                    id: global.id,
                    serial,
                }));
            }
        })
        .register();
    let until = Instant::now() + Duration::from_secs(5);
    while Instant::now() < until && !ended.load(Ordering::Acquire) {
        if let Some(node) = found.get() {
            return Ok(node);
        }
        if main_loop
            .loop_()
            .iterate(pw::loop_::Timeout::Finite(Duration::from_millis(25)))
            < 0
        {
            break;
        }
    }
    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "the capture node did not appear",
    ))
}

/// Keep media-engine errors at this fixed worker boundary.
fn failed(why: impl std::fmt::Display) -> io::Error {
    io::Error::other(why.to_string())
}
