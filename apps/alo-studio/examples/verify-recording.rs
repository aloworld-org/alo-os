//! Record a real X11 framebuffer with the production encoder, pause, resume,
//! finalize, and decode it. This diagnostic is excluded from installed builds.

use alo_studio::recorder::{Event, Input, Recorder};
use gst::prelude::*;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("Pass an output .webm path")?;
    let display = std::env::var("DISPLAY")?;
    let mut recorder = Recorder::record(
        Input::VerificationDisplay(display),
        &output,
        640,
        360,
        false,
    )?;
    let started = Instant::now();
    let mut paused = false;
    let mut resumed = false;
    let mut finishing = false;
    let mut frames = 0_u64;
    loop {
        if recorder.frame().is_some() {
            frames += 1;
        }
        let elapsed = started.elapsed();
        if elapsed > Duration::from_secs(2) && !paused {
            recorder.pause()?;
            paused = true;
        }
        if elapsed > Duration::from_secs(3) && !resumed {
            recorder.resume()?;
            resumed = true;
        }
        if elapsed > Duration::from_secs(5) && !finishing {
            recorder.finish()?;
            finishing = true;
        }
        match recorder.poll() {
            Some(Event::Finished) => break,
            Some(Event::Failed(error)) => return Err(error.into()),
            None => {}
        }
        if elapsed > Duration::from_secs(20) {
            return Err("The pipeline failed to finalize".into());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    drop(recorder);
    if frames == 0 {
        return Err("No real preview frames arrived".into());
    }
    if std::fs::metadata(&output)?.len() < 1024 {
        return Err("The recording is empty".into());
    }
    // Decoding proves the muxed output is a usable video, not just a nonempty file.
    let pipeline = gst::parse::launch(
        "filesrc name=input ! matroskademux ! vp9dec ! videoconvert ! fakesink sync=false",
    )?
    .downcast::<gst::Pipeline>()
    .map_err(|_| "No decoder pipeline")?;
    pipeline
        .by_name("input")
        .ok_or("No input")?
        .set_property("location", output.to_string_lossy().as_ref());
    pipeline.set_state(gst::State::Playing)?;
    let bus = pipeline.bus().ok_or("No bus")?;
    let message = bus
        .timed_pop_filtered(
            gst::ClockTime::from_seconds(15),
            &[gst::MessageType::Eos, gst::MessageType::Error],
        )
        .ok_or("Decoding timed out")?;
    let result = match message.view() {
        gst::MessageView::Eos(_) => Ok(()),
        gst::MessageView::Error(error) => Err(error.error().to_string()),
        _ => Err("No decoding result".to_owned()),
    };
    pipeline.set_state(gst::State::Null)?;
    result?;
    println!(
        "PASS: {frames} preview frames; pause/resume; EOS; {} bytes; VP9/WebM decoded to completion",
        std::fs::metadata(&output)?.len()
    );
    Ok(())
}
