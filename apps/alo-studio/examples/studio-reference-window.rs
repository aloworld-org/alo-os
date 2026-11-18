//! An independent flashing window, generated tone and real Pulse monitor.
//! Excluded from the normal installed application's build.

use gst::prelude::*;
use gtk::prelude::*;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[path = "../verification/reference_signal.rs"]
mod signal;

#[derive(Default)]
struct Trace {
    draws: Vec<Value>,
    audio: Vec<Value>,
    errors: Vec<String>,
}

type Shared = Arc<Mutex<Trace>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gst::init()?;
    gtk::glib::set_application_name("Studio verification window");
    let trace = Shared::default();
    let audio = pipeline(
        "audiotestsrc name=signal is-live=true samplesperbuffer=160 wave=silence ! \
         audio/x-raw,format=S16LE,rate=16000,channels=1 ! \
         pulsesink device=studio-verification-audio client-name=Studio-reference-signal \
         sync=true slave-method=resample buffer-time=200000 latency-time=10000",
    )?;
    audio
        .by_name("signal")
        .and_then(|source| source.static_pad("src"))
        .ok_or("the generated signal has no output")?
        .add_probe(gst::PadProbeType::BUFFER, |_, info| {
            let Some(buffer) = info.buffer_mut() else {
                return gst::PadProbeReturn::Drop;
            };
            let buffer = buffer.make_mut();
            let Some(stamp) = buffer.pts() else {
                return gst::PadProbeReturn::Drop;
            };
            buffer.unset_flags(gst::BufferFlags::GAP);
            let Ok(mut mapped) = buffer.map_writable() else {
                return gst::PadProbeReturn::Drop;
            };
            for (index, sample) in mapped.as_mut_slice().chunks_exact_mut(2).enumerate() {
                let offset = u64::try_from(index)
                    .unwrap_or_default()
                    .saturating_mul(62_500);
                sample.copy_from_slice(
                    &signal::tone_sample(stamp.nseconds().saturating_add(offset)).to_le_bytes(),
                );
            }
            gst::PadProbeReturn::Ok
        })
        .ok_or("the generated signal could not be attached")?;

    // This reads the real server monitor, not the samples passed to pulsesink.
    let monitor = pipeline(
        "pulsesrc name=reference-input device=studio-verification-audio.monitor \
         client-name=Studio-reference-monitor buffer-time=2000000 latency-time=10000 ! \
         audioconvert ! audioresample ! audio/x-raw,format=S16LE,rate=16000,channels=1 ! \
         appsink name=monitor sync=false max-buffers=100 drop=false",
    )?;
    monitor.set_state(gst::State::Paused)?;
    let clock = monitor
        .by_name("reference-input")
        .and_then(|source| source.provide_clock())
        .ok_or("the real monitor has no sample clock")?;
    audio.use_clock(Some(&clock));
    monitor.use_clock(Some(&clock));
    let sink = monitor
        .by_name("monitor")
        .ok_or("no monitor sink")?
        .downcast::<gst_app::AppSink>()
        .map_err(|_| "the monitor sink is unavailable")?;
    let monitored = monitor.downgrade();
    let heard = trace.clone();
    sink.set_callbacks(gst_app::AppSinkCallbacks::builder().new_sample(move |sink| {
        let sample = sink.pull_sample().map_err(|_| gst::FlowError::Error)?;
        let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
        let stamp = buffer.pts().ok_or(gst::FlowError::Error)?;
        let base = monitored.upgrade().and_then(|pipeline| pipeline.base_time())
            .ok_or(gst::FlowError::Error)?;
        let mapped = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
        let mut heard = heard.lock().map_err(|_| gst::FlowError::Error)?;
        for (index, chunk) in mapped.as_slice().chunks(320).enumerate() {
            let offset = u64::try_from(index).unwrap_or_default().saturating_mul(10_000_000);
            let peak = chunk.chunks_exact(2).filter_map(|sample| <[u8; 2]>::try_from(sample).ok())
                .map(i16::from_le_bytes).map(i16::unsigned_abs).max().unwrap_or_default();
            heard.audio.push(json!({
                "clock_ns": base.nseconds().saturating_add(stamp.nseconds()).saturating_add(offset),
                "duration_ns": u64::try_from(chunk.len() / 2).unwrap_or_default().saturating_mul(62_500),
                "tone": peak > 1200, "peak": peak,
            }));
        }
        Ok(gst::FlowSuccess::Ok)
    }).build());

    let application = gtk::Application::builder()
        .application_id("org.aloworld.Studio.Verification")
        .build();
    let watches = [watch(&audio, &trace)?, watch(&monitor, &trace)?];
    let playing = audio.clone();
    let listening = monitor.clone();
    let shown = trace.clone();
    let startup = trace.clone();
    application.connect_activate(move |application| {
        let area = gtk::DrawingArea::new();
        let drawing = playing.clone();
        let shown = shown.clone();
        let clock = clock.clone();
        area.set_draw_func(move |_, cairo, width, height| {
            let Some(base) = drawing.base_time() else {
                return;
            };
            let now = clock.time();
            let Some(position) = now.checked_sub(base) else {
                return;
            };
            let lit = signal::lit_at(position.nseconds());
            let gray = if lit { 1.0 } else { 0.02 };
            cairo.set_source_rgb(gray, gray, gray);
            if cairo.paint().is_err() {
                return;
            }
            let ink = if lit { 0.04 } else { 0.9 };
            cairo.set_source_rgb(ink, ink, ink);
            cairo.set_font_size(22.0);
            cairo.move_to(f64::from(width) * 0.08, f64::from(height) * 0.50);
            let _ = cairo.show_text("Recording synchronization");
            cairo.move_to(f64::from(width) * 0.08, f64::from(height) * 0.65);
            let _ = cairo.show_text(&format!("{:.2} seconds", position.seconds_f64()));
            if let Ok(mut shown) = shown.lock() {
                shown.draws.push(json!({
                    "clock_ns": now.nseconds(),
                    "position_ns": position.nseconds(), "lit": lit,
                }));
            }
        });
        let window = gtk::ApplicationWindow::builder()
            .application(application)
            .title("Studio verification window")
            .default_width(600)
            .default_height(420)
            .child(&area)
            .build();
        let weak_area = area.downgrade();
        gtk::glib::timeout_add_local(Duration::from_millis(40), move || {
            let Some(area) = weak_area.upgrade() else {
                return gtk::glib::ControlFlow::Break;
            };
            area.queue_draw();
            gtk::glib::ControlFlow::Continue
        });
        window.present();
        if let Err(error) = listening
            .set_state(gst::State::Playing)
            .and_then(|_| playing.set_state(gst::State::Playing))
        {
            if let Ok(mut shown) = startup.lock() {
                shown.errors.push(error.to_string());
            }
            application.quit();
        }
    });
    let quit = gtk::gio::SimpleAction::new("finish-reference", None);
    let weak_app = application.downgrade();
    quit.connect_activate(move |_, _| {
        if let Some(application) = weak_app.upgrade() {
            application.quit();
        }
    });
    application.add_action(&quit);
    let weak_app = application.downgrade();
    gtk::glib::timeout_add_local_once(Duration::from_secs(900), move || {
        if let Some(application) = weak_app.upgrade() {
            application.quit();
        }
    });
    application.run_with_args::<&str>(&[]);
    audio.set_state(gst::State::Null)?;
    monitor.set_state(gst::State::Null)?;
    drop(watches);
    let trace = trace.lock().map_err(|_| "the reference trace was lost")?;
    let output = gtk::glib::user_data_dir().join("studio-walk");
    std::fs::create_dir_all(&output)?;
    std::fs::write(
        output.join("reference.json"),
        serde_json::to_vec(&json!({
            "version": 1, "draws": &trace.draws, "audio": &trace.audio, "errors": &trace.errors,
        }))?,
    )?;
    if trace.draws.is_empty() || trace.audio.is_empty() || !trace.errors.is_empty() {
        return Err("the independent reference did not remain healthy".into());
    }
    Ok(())
}

fn pipeline(description: &str) -> Result<gst::Pipeline, Box<dyn std::error::Error>> {
    gst::parse::launch(description)?
        .downcast::<gst::Pipeline>()
        .map_err(|_| "the reference pipeline is unavailable".into())
}

fn watch(
    pipeline: &gst::Pipeline,
    trace: &Shared,
) -> Result<gst::bus::BusWatchGuard, Box<dyn std::error::Error>> {
    let weak_pipeline = pipeline.downgrade();
    let trace = trace.clone();
    let bus = pipeline.bus().ok_or("the reference has no message bus")?;
    Ok(bus.add_watch_local(move |_, message| {
        match message.view() {
            gst::MessageView::Latency(_) => {
                if let Some(pipeline) = weak_pipeline.upgrade() {
                    let _ = pipeline.recalculate_latency();
                }
            }
            gst::MessageView::Error(error) => {
                if let Ok(mut trace) = trace.lock() {
                    trace.errors.push(error.error().to_string());
                }
            }
            _ => {}
        }
        gtk::glib::ControlFlow::Continue
    })?)
}
