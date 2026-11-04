//! Record a real X11 framebuffer with the production encoder, pause, resume,
//! finalize, and decode it. This diagnostic is excluded from installed builds.

use alo_studio::recorder::{Event, Input, Recorder};
use gst::prelude::*;
use std::os::{fd::AsRawFd, unix::net::UnixStream};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("Pass an output .webm path")?;
    let remote = if let Some(node) = std::env::args().nth(2) {
        let socket = std::env::args()
            .nth(3)
            .ok_or("Pass the isolated PipeWire socket path")?;
        Some((node.parse::<u32>()?, UnixStream::connect(socket)?))
    } else {
        None
    };
    let display = std::env::var("DISPLAY").unwrap_or_default();
    let source = || match &remote {
        Some((node, socket)) => Input::PipeWire {
            fd: socket.as_raw_fd(),
            node: *node,
        },
        None => Input::VerificationDisplay(display.clone()),
    };
    // A PipeWire fixture never opens the real microphone. Opus is exercised
    // separately by the Xvfb source's synthetic audio.
    let microphone = remote.is_none();
    alo_studio::performance::check(640, 360)?;
    let preview = Recorder::preview(source())?;
    let preview_started = Instant::now();
    loop {
        if preview.frame().is_some() {
            break;
        }
        if let Some(Event::Failed(error)) = preview.poll() {
            return Err(error.into());
        }
        if preview_started.elapsed() > Duration::from_secs(8) {
            return Err("No preview frame arrived".into());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    let mut recorder = Recorder::record(source(), &output, 640, 360, microphone)?;
    drop(preview);
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
        if std::env::var_os("ALO_STUDIO_VERIFY_SOURCE_END").is_some()
            && elapsed > Duration::from_millis(2500)
            && !finishing
        {
            recorder.finish_after_source_ended()?;
            finishing = true;
        }
        if elapsed > Duration::from_secs(3) && !resumed && !finishing {
            recorder.resume()?;
            resumed = true;
        }
        if elapsed > Duration::from_secs(5) && !finishing {
            recorder.finish()?;
            finishing = true;
        }
        match recorder.poll() {
            Some(Event::Finished) => break,
            Some(Event::Failed(error) | Event::SourceEnded(error)) => return Err(error.into()),
            Some(Event::FellBehind) => return Err("The recording could not keep up".into()),
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
    let decode = if microphone {
        "filesrc name=input ! matroskademux name=demux demux.video_0 ! queue ! vp9dec ! fakesink sync=false demux.audio_0 ! queue ! opusdec ! fakesink sync=false"
    } else {
        "filesrc name=input ! matroskademux ! vp9dec ! fakesink sync=false"
    };
    let pipeline = gst::parse::launch(decode)?
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
    let duration = pipeline
        .query_duration::<gst::ClockTime>()
        .ok_or("No video duration")?;
    pipeline.set_state(gst::State::Null)?;
    result?;
    if duration > gst::ClockTime::from_mseconds(4700) {
        return Err("Paused time was included in the recording".into());
    }
    println!(
        "PASS: {frames} preview frames; preview-to-record handoff; pause/resume; EOS; {} bytes; {streams} decoded to completion",
        std::fs::metadata(&output)?.len(),
        streams = if microphone {
            "VP9 + synthetic Opus/WebM"
        } else {
            "PipeWire VP9/WebM"
        }
    );
    Ok(())
}
