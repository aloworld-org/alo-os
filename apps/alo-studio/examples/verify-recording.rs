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
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};

// Integration-only reuse of the OS wire codec. Independent installed builds
// never compile this example or require the OS source tree.
#[path = "../../../crates/alo-capture-protocol/src/descriptor.rs"]
pub mod descriptor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--duration") {
        let path = std::env::args().nth(2).ok_or("Pass the saved video path")?;
        println!(
            "{}",
            alo_studio::completed_media::duration(std::path::Path::new(&path))?.as_millis()
        );
        return Ok(());
    }
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("Pass an output .webm path")?;
    gst::init()?;
    gst::log::set_threshold_for_name("matroskamux", gst::DebugLevel::Error);
    let rejected_timestamps = Arc::new(AtomicBool::new(false));
    let rejected = rejected_timestamps.clone();
    let timing_log = gst::log::add_log_function(move |category, level, _, _, _, _, _| {
        if category.name() == "matroskamux" && level == gst::DebugLevel::Error {
            rejected.store(true, Ordering::Relaxed);
        }
    });
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
    let pulse_audio = std::env::var_os("ALO_STUDIO_VERIFY_PULSE_AUDIO").is_some();
    let microphone = remote.is_none() || generated_audio || pulse_audio;
    // Software-emulated OS guests need a longer observation window. The
    // default CI timing and minimum continuous-frame requirement stay strict.
    let time_scale: u64 = match std::env::var("ALO_STUDIO_VERIFY_TIME_SCALE") {
        Ok(value) => value.parse()?,
        Err(_) => 1,
    };
    if !(1..=6).contains(&time_scale) {
        return Err("The verification time scale must be between one and six".into());
    }
    let mut recorder = Recorder::preview(source())?;
    let preview_started = Instant::now();
    let preview_frame = loop {
        if let Some(frame) = recorder.frame() {
            break frame;
        }
        if let Some(Event::Failed(error)) = recorder.poll() {
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
    // An actual audio source joins an already running preview, as it does after
    // the person's quality check. A private Pulse server must be selected by
    // the caller; this opt-in diagnostic can otherwise open a real microphone.
    if pulse_audio {
        let preview_seconds = if std::env::var_os("ALO_STUDIO_VERIFY_LONG_PREVIEW").is_some() {
            12
        } else {
            3
        };
        std::thread::sleep(Duration::from_secs(preview_seconds));
    }
    let quality = if std::env::var_os("ALO_STUDIO_VERIFY_LIGHTWEIGHT").is_some() {
        alo_studio::model::Quality::Lightweight
    } else {
        alo_studio::model::Quality::Original
    };
    let (width, height) = quality.size(preview_frame.width, preview_frame.height)?;
    let frame_rate = quality.frame_rate();
    alo_studio::performance::check(width, height, frame_rate)?;
    if generated_audio {
        recorder.start_with_generated_audio(&output, width, height, frame_rate)?;
    } else {
        recorder.start_recording(&output, width, height, frame_rate, microphone)?;
    }
    let started = Instant::now();
    let mut paused = false;
    let mut resumed = false;
    let mut finishing = false;
    let mut preview_hidden = false;
    let mut preview_restored = false;
    let mut frames = 0_u64;
    let mut frames_after_resume = 0_u64;
    loop {
        let elapsed = started.elapsed();
        if recorder.frame().is_some() {
            if preview_hidden
                && !preview_restored
                && elapsed > Duration::from_millis(900 * time_scale)
            {
                return Err("A hidden preview continued converting frames".into());
            }
            frames += 1;
            if resumed && !finishing {
                frames_after_resume += 1;
            }
        }
        if elapsed > Duration::from_millis(500 * time_scale) && !preview_hidden {
            recorder.set_preview_visible(false);
            preview_hidden = true;
        }
        if elapsed > Duration::from_millis(1600 * time_scale) && !preview_restored {
            if recorder.recorded_duration().is_none() {
                return Err("Hiding the preview interrupted video encoding".into());
            }
            recorder.set_preview_visible(true);
            preview_restored = true;
        }
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
    gst::log::remove_log_function(timing_log);
    if rejected_timestamps.load(Ordering::Relaxed) {
        return Err("The WebM writer rejected recorded timestamps".into());
    }
    if resumed && frames_after_resume < 2 {
        return Err(format!(
            "The selected source delivered {frames_after_resume} frames after Resume; at least two new frames are required"
        )
        .into());
    }
    if resumed {
        println!("Frames received after Resume: {frames_after_resume}");
    }
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
        "filesrc name=input ! matroskademux name=demux demux.video_0 ! queue ! vp9dec name=video ! fakesink sync=false async=false demux.audio_0 ! queue ! opusdec name=audio ! fakesink sync=false async=false"
    } else {
        "filesrc name=input ! matroskademux ! vp9dec name=video ! fakesink sync=false async=false"
    };
    let pipeline = gst::parse::launch(decode)?
        .downcast::<gst::Pipeline>()
        .map_err(|_| "No decoder pipeline")?;
    pipeline
        .by_name("input")
        .ok_or("No input")?
        .set_property("location", output.to_string_lossy().as_ref());
    let first = [
        Arc::new(AtomicU64::new(u64::MAX)),
        Arc::new(AtomicU64::new(u64::MAX)),
    ];
    for (name, first) in ["video", "audio"].into_iter().zip(first.iter()) {
        if let Some(decoder) = pipeline.by_name(name) {
            let first = first.clone();
            decoder
                .static_pad("src")
                .ok_or("No decoded output")?
                .add_probe(gst::PadProbeType::BUFFER, move |_, info| {
                    if let Some(stamp) = info.buffer().and_then(|buffer| buffer.pts()) {
                        first.fetch_min(stamp.nseconds(), Ordering::Relaxed);
                    }
                    gst::PadProbeReturn::Ok
                });
        }
    }
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
    let caps = pipeline
        .by_name("video")
        .and_then(|element| element.static_pad("src"))
        .and_then(|pad| pad.current_caps())
        .ok_or("No decoded video format")?;
    let video_info = gst_video::VideoInfo::from_caps(&caps)?;
    pipeline.set_state(gst::State::Null)?;
    result?;
    let saved_duration = alo_studio::completed_media::duration(&output)?;
    // Header discovery and decoded stream queries can include different final
    // packet padding. Both must fit the observed active recording window. The
    // separate GTK flow requires library metadata to equal discovery exactly.
    if saved_duration < Duration::from_secs(time_scale)
        || saved_duration > Duration::from_millis(4000 * time_scale + 700)
    {
        return Err("Discovered playback duration does not match active capture".into());
    }
    if video_info.width() != width as u32
        || video_info.height() != height as u32
        || video_info.fps() <= gst::Fraction::new(0, 1)
        || video_info.fps() > gst::Fraction::new(frame_rate, 1)
    {
        return Err("The recording did not preserve the chosen size and frame-rate ceiling".into());
    }
    println!(
        "Decoded video: {width} × {height}, {} fps",
        video_info.fps()
    );
    if microphone {
        let video = first[0].load(Ordering::Relaxed);
        let audio = first[1].load(Ordering::Relaxed);
        if video == u64::MAX || audio == u64::MAX {
            return Err("Both video and audio must decode actual buffers".into());
        }
        println!("Decoded first video/audio timestamps: {video} / {audio} ns");
        if video.abs_diff(audio) > 250_000_000 * time_scale {
            return Err("Video and microphone did not start on the same timeline".into());
        }
    }
    if duration < gst::ClockTime::from_seconds(time_scale) {
        return Err("The decoded recording is too short for the observed capture".into());
    }
    if duration > gst::ClockTime::from_mseconds(4000 * time_scale + 700) {
        return Err("Paused time was included in the recording".into());
    }
    println!(
        "PASS: {frames} preview frames; preview-to-record handoff; pause/resume; EOS; {} bytes; {streams} decoded to completion",
        std::fs::metadata(&output)?.len(),
        streams = if pulse_audio {
            "VP9 + isolated PulseAudio/Opus/WebM"
        } else if microphone {
            "VP9 + synthetic Opus/WebM"
        } else {
            "PipeWire VP9/WebM"
        }
    );
    Ok(())
}
