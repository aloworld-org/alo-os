//! Exercise the actual ScreenCast portal and its permission-scoped PipeWire fd.
//! Run only with an explicitly chosen, isolated test desktop.

use alo_studio::{
    model::{Quality, SourceKind},
    portal,
    recorder::{Event, Recorder},
};
use gst::prelude::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("Pass an output .webm path")?;
    gtk::init()?;
    gtk::glib::MainContext::default().block_on(async {
        let capture = portal::choose(SourceKind::Screen).await?;
        let preview = Recorder::preview((&capture).into())?;
        let mut size = None;
        for _ in 0..100 {
            if let Some(Event::Failed(error)) = preview.poll() { return Err(error.into()); }
            if preview.frame().is_some() { size = preview.source_size(); break; }
            gtk::glib::timeout_future(Duration::from_millis(50)).await;
        }
        let (width, height) = size.ok_or("No real portal preview frame arrived")?;
        let (width, height) = Quality::Hd.size(width, height)?;
        let mut recorder = Recorder::record((&capture).into(), &output, width, height, false)?;
        drop(preview);
        let started = Instant::now();
        let mut finishing = false;
        let mut frames = 0;
        loop {
            if recorder.frame().is_some() { frames += 1; }
            if started.elapsed() > Duration::from_secs(3) && !finishing {
                recorder.finish()?;
                finishing = true;
            }
            match recorder.poll() {
                Some(Event::Finished) => break,
                Some(Event::Failed(error) | Event::SourceEnded(error)) => return Err(error.into()),
                Some(Event::FellBehind) => return Err("Portal recording fell behind".into()),
                None => {}
            }
            if started.elapsed() > Duration::from_secs(15) { return Err("Portal recording did not finalize".into()); }
            gtk::glib::timeout_future(Duration::from_millis(25)).await;
        }
        drop(recorder);
        drop(capture);
        gtk::glib::timeout_future(Duration::from_millis(100)).await;
        if frames == 0 { return Err("The portal recording had no frames".into()); }
        let decoder = gst::parse::launch("filesrc name=input ! matroskademux ! vp9dec ! fakesink sync=false")?
            .downcast::<gst::Pipeline>().map_err(|_| "No decoder pipeline")?;
        decoder.by_name("input").ok_or("No decoder input")?.set_property("location", output.to_string_lossy().as_ref());
        decoder.set_state(gst::State::Playing)?;
        let message = decoder.bus().ok_or("No decoder bus")?.timed_pop_filtered(gst::ClockTime::from_seconds(10), &[gst::MessageType::Eos, gst::MessageType::Error]);
        let _ = decoder.set_state(gst::State::Null);
        if !matches!(message.as_ref().map(|message| message.view()), Some(gst::MessageView::Eos(_))) {
            return Err("The real portal video could not be decoded to completion".into());
        }
        println!("PASS: real ScreenCast portal, scoped PipeWire remote, {width} × {height}, {frames} captured frames, VP9/WebM decoded to EOS");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
