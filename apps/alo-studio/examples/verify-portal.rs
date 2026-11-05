//! Exercise the actual ScreenCast portal and its permission-scoped PipeWire fd.
//! Run only with an explicitly chosen, isolated test desktop.

use alo_studio::{
    model::{Quality, SourceKind},
    portal,
    recorder::{Event, Recorder},
};
use gst::prelude::*;
use gtk::gdk::prelude::TextureExt;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("Pass an output .webm path")?;
    let kind = match std::env::args().nth(2).as_deref() {
        None | Some("screen") => SourceKind::Screen,
        Some("window") => SourceKind::Window,
        _ => return Err("Choose screen or window".into()),
    };
    let time_scale: u64 = std::env::var("ALO_STUDIO_VERIFY_TIME_SCALE")
        .unwrap_or_else(|_| "1".to_owned())
        .parse()?;
    if !(1..=6).contains(&time_scale) {
        return Err("The verification time scale must be between one and six".into());
    }
    gtk::init()?;
    gtk::glib::MainContext::default().block_on(async {
        let capture = portal::choose(kind, None).await?;
        let mut recorder = Recorder::preview((&capture).into())?;
        let mut size = None;
        for _ in 0..(160 * time_scale) {
            if let Some(Event::Failed(error)) = recorder.poll() { return Err(error.into()); }
            if let Some(frame) = recorder.frame() {
                let texture = gtk::gdk::MemoryTexture::new(
                    frame.width, frame.height, gtk::gdk::MemoryFormat::R8g8b8a8,
                    &gtk::glib::Bytes::from_owned(frame.bytes), frame.width as usize * 4,
                );
                texture.save_to_png(output.with_extension("preview.png"))?;
                size = recorder.source_size();
                break;
            }
            gtk::glib::timeout_future(Duration::from_millis(50)).await;
        }
        let (width, height) = size.ok_or("No real portal preview frame arrived")?;
        let (width, height) = Quality::Hd.size(width, height)?;
        recorder.start_recording(&output, width, height, false)?;
        let started = Instant::now();
        let mut finishing = false;
        let mut frames = 0;
        loop {
            if recorder.frame().is_some() { frames += 1; }
            if started.elapsed() > Duration::from_secs(5 * time_scale) && !finishing {
                recorder.finish()?;
                finishing = true;
            }
            match recorder.poll() {
                Some(Event::Finished) => break,
                Some(Event::Failed(error) | Event::SourceEnded(error)) => return Err(error.into()),
                Some(Event::FellBehind) => return Err("Portal recording fell behind".into()),
                None => {}
            }
            if started.elapsed() > Duration::from_secs(20 * time_scale) { return Err("Portal recording did not finalize".into()); }
            gtk::glib::timeout_future(Duration::from_millis(25)).await;
        }
        drop(recorder);
        drop(capture);
        gtk::glib::timeout_future(Duration::from_millis(100)).await;
        if frames < 15 { return Err(format!("The portal recording delivered {frames} preview frames; at least 15 are required").into()); }
        let decoder = gst::parse::launch("filesrc name=input ! matroskademux ! vp9dec ! fakesink sync=false")?
            .downcast::<gst::Pipeline>().map_err(|_| "No decoder pipeline")?;
        decoder.by_name("input").ok_or("No decoder input")?.set_property("location", output.to_string_lossy().as_ref());
        decoder.set_state(gst::State::Playing)?;
        let message = decoder.bus().ok_or("No decoder bus")?.timed_pop_filtered(gst::ClockTime::from_seconds(15 * time_scale), &[gst::MessageType::Eos, gst::MessageType::Error]);
        let _ = decoder.set_state(gst::State::Null);
        if !matches!(message.as_ref().map(|message| message.view()), Some(gst::MessageView::Eos(_))) {
            return Err("The real portal video could not be decoded to completion".into());
        }
        println!("PASS: real ScreenCast portal, scoped PipeWire remote, {width} × {height}, {frames} captured frames, VP9/WebM decoded to EOS");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
