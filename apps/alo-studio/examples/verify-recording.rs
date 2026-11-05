//! Record a real X11 framebuffer with the production encoder, pause, resume,
//! finalize, and decode it. This diagnostic is excluded from installed builds.

use alo_studio::recorder::{Event, Input, Recorder};
use gst::prelude::*;
use std::os::{
    fd::{AsRawFd, OwnedFd},
    unix::net::UnixStream,
};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

// Integration-only reuse of the OS wire codec. Independent installed builds
// never compile this example or require the OS source tree.
#[path = "../../../crates/alo-capture-protocol/src/descriptor.rs"]
pub mod descriptor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("Pass an output .webm path")?;
    let remote = if let Some(node) = std::env::args().nth(2) {
        let socket = std::env::args()
            .nth(3)
            .ok_or("Pass the isolated PipeWire socket path")?;
        let node = node.parse::<u32>()?;
        let descriptor: OwnedFd =
            if let Some(helper) = std::env::var_os("ALO_STUDIO_CAPTURE_REMOTE") {
                let serial = std::env::var("ALO_STUDIO_VERIFY_NODE_SERIAL")?;
                let (receiving, output) = UnixStream::pair()?;
                receiving.set_read_timeout(Some(Duration::from_secs(15)))?;
                let mut child = Command::new(helper)
                    .args([node.to_string(), serial, "org.aloworld.Studio".to_owned()])
                    .stdout(Stdio::from(OwnedFd::from(output)))
                    .spawn()?;
                let received = descriptor::receive_descriptor(&receiving)?;
                if !child.wait()?.success() {
                    return Err("Preparing the selected-node connection failed".into());
                }
                received
            } else {
                UnixStream::connect(socket)?.into()
            };
        Some((node, descriptor))
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
    // A PipeWire fixture never opens the real microphone. The second pass adds
    // a generated tone to exercise simultaneous video and audio on that source.
    let generated_audio = std::env::var_os("ALO_STUDIO_VERIFY_GENERATED_AUDIO").is_some();
    let microphone = remote.is_none() || generated_audio;
    // Software-emulated OS guests need a longer observation window. The
    // default CI timing and minimum continuous-frame requirement stay strict.
    let time_scale: u64 = match std::env::var("ALO_STUDIO_VERIFY_TIME_SCALE") {
        Ok(value) => value.parse()?,
        Err(_) => 1,
    };
    if !(1..=6).contains(&time_scale) {
        return Err("The verification time scale must be between one and six".into());
    }
    let preview = Recorder::preview(source())?;
    let preview_started = Instant::now();
    let preview_frame = loop {
        if let Some(frame) = preview.frame() {
            break frame;
        }
        if let Some(Event::Failed(error)) = preview.poll() {
            return Err(error.into());
        }
        if preview_started.elapsed() > Duration::from_secs(8 * time_scale) {
            return Err("No preview frame arrived".into());
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    if let Ok(expected) = std::env::var("ALO_STUDIO_EXPECT_PREVIEW_HEIGHT")
        && (preview_frame.width != 640 || preview_frame.height != expected.parse::<i32>()?)
    {
        return Err("The preview changed the source's aspect ratio".into());
    }
    alo_studio::performance::check(preview_frame.width, preview_frame.height)?;
    let mut recorder = if generated_audio {
        Recorder::record_with_generated_audio(
            source(),
            &output,
            preview_frame.width,
            preview_frame.height,
        )?
    } else {
        Recorder::record(
            source(),
            &output,
            preview_frame.width,
            preview_frame.height,
            microphone,
        )?
    };
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
        if elapsed > Duration::from_secs(2 * time_scale) && !paused {
            recorder.pause()?;
            paused = true;
        }
        if std::env::var_os("ALO_STUDIO_VERIFY_SOURCE_END").is_some()
            && elapsed > Duration::from_millis(2500 * time_scale)
            && !finishing
        {
            recorder.finish_after_source_ended()?;
            finishing = true;
        }
        if elapsed > Duration::from_secs(3 * time_scale) && !resumed && !finishing {
            recorder.resume()?;
            resumed = true;
        }
        if elapsed > Duration::from_secs(5 * time_scale) && !finishing {
            recorder.finish()?;
            finishing = true;
        }
        match recorder.poll() {
            Some(Event::Finished) => break,
            Some(Event::Failed(error) | Event::SourceEnded(error)) => return Err(error.into()),
            Some(Event::FellBehind) => {
                return Err(format!(
                    "The recording could not keep up: {frames} preview frames in {elapsed:?}, encoded {:?}",
                    recorder.recorded_duration()
                ).into());
            }
            None => {}
        }
        if elapsed > Duration::from_secs(20 * time_scale) {
            return Err("The pipeline failed to finalize".into());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    drop(recorder);
    if frames < 15 {
        return Err(format!(
            "The recording delivered {frames} preview frames; at least 15 are required"
        )
        .into());
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
            gst::ClockTime::from_seconds(15 * time_scale),
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
    if duration < gst::ClockTime::from_seconds(time_scale) {
        return Err("The decoded recording is too short for the observed capture".into());
    }
    if duration > gst::ClockTime::from_mseconds(4000 * time_scale + 700) {
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
