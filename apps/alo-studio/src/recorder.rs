//! GStreamer supplies capture, VP9, Opus and WebM. No codec is implemented here.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use gst::prelude::*;

use crate::portal::Capture;
use crate::words::t;

/// A real frame for the on-screen preview, copied away from the streaming thread.
pub struct Frame {
    /// Packed RGBA bytes, with no padding copied between rows.
    pub bytes: Vec<u8>,
    /// Preview width.
    pub width: i32,
    /// Preview height.
    pub height: i32,
}

/// A refusal from the media engine. It never substitutes a microphone or codec.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct MediaError(pub String);

/// The event the UI must respond to, polled without blocking its main thread.
pub enum Event {
    /// A live source ended; seal captured video and stop the microphone.
    SourceEnded(String),
    /// The WebM trailer has been written and the file can be committed.
    Finished,
    /// The source or encoder failed. The pending file is preserved.
    Failed(String),
    /// A bounded queue filled: finish the clip instead of degrading it silently.
    FellBehind,
}

/// One pipeline, stopped before its permission-scoped remote can be closed.
pub struct Recorder {
    source_ended: bool,
    pipeline: gst::Pipeline,
    bus: gst::Bus,
    frame: Arc<Mutex<Option<Frame>>>,
    finishing: bool,
    fell_behind: Arc<AtomicBool>,
    video: Option<Arc<Mutex<crate::recorded_video::Progress>>>,
    paused: Arc<AtomicBool>,
}

/// The pipeline's source, separately selected for explicit diagnostic builds.
pub enum Input {
    /// A stream chosen through the desktop portal.
    PipeWire { fd: i32, node: u32 },
    /// A real X11 framebuffer, used by the integration probe only.
    #[cfg(feature = "verification")]
    VerificationDisplay(String),
}

impl From<&Capture> for Input {
    fn from(capture: &Capture) -> Self {
        #[cfg(feature = "verification")]
        if let Some(display) = &capture.verification_display {
            return Self::VerificationDisplay(display.clone());
        }
        Self::PipeWire {
            fd: capture.fd(),
            node: capture.node,
        }
    }
}

impl Recorder {
    /// Verify required engines before requesting anybody's screen.
    pub fn available() -> Result<(), MediaError> {
        gst::init().map_err(failed)?;
        for name in [
            "pipewiresrc",
            "videoconvert",
            "videoscale",
            "videorate",
            "vp9enc",
            "webmmux",
            "appsink",
            "opusenc",
            "filesink",
        ] {
            if gst::ElementFactory::find(name).is_none() {
                return Err(MediaError(format!(
                    "{} ({name}). {}",
                    t("A required recording component is missing"),
                    t("Please reinstall alo Studio.")
                )));
            }
        }
        Ok(())
    }
    /// Start a source-only preview. A microphone is never part of this pipeline.
    pub fn preview(source: Input) -> Result<Self, MediaError> {
        Self::build(source, None, 0, 0, false)
    }
    /// Start writing only the size and microphone choice the person confirmed.
    pub fn record(
        source: Input,
        path: &Path,
        width: i32,
        height: i32,
        microphone: bool,
    ) -> Result<Self, MediaError> {
        Self::build(source, Some(path), width, height, microphone)
    }
    fn build(
        source: Input,
        path: Option<&Path>,
        width: i32,
        height: i32,
        microphone: bool,
    ) -> Result<Self, MediaError> {
        gst::init().map_err(failed)?;
        let source_text = match &source {
            Input::PipeWire { .. } => {
                // Queued frames must own their bytes when Finish disconnects
                // the PipeWire stream and releases its shared buffer pool.
                // The native publisher supplies live frames. PipeWire 1.4's
                // keepalive path mixes absolute clock time into a running-time
                // stream, scheduling later buffers hours into the future.
                "pipewiresrc name=source do-timestamp=true always-copy=true ! video/x-raw,pixel-aspect-ratio=1/1"
            }
            #[cfg(feature = "verification")]
            Input::VerificationDisplay(_) => {
                "ximagesrc name=source use-damage=false show-pointer=true do-timestamp=true"
            }
        };
        // This is a fixed engine configuration. No filename, device name, or
        // other person-supplied text is interpolated into the pipeline parser.
        let video = if path.is_some() {
            "! queue name=video-input max-size-buffers=2 max-size-bytes=0 max-size-time=0 ! videoconvert ! videoscale ! videorate skip-to-first=true ! capsfilter name=size ! tee name=frames \
             frames. ! queue name=record-buffer max-size-time=0 max-size-bytes=134217728 max-size-buffers=60 ! vp9enc name=encoder deadline=1 cpu-used=8 threads=4 lag-in-frames=0 target-bitrate=4500000 keyframe-max-dist=60 \
             ! queue ! webmmux name=container offset-to-zero=true ! filesink name=output async=false \
             frames. ! queue leaky=downstream max-size-buffers=1 ! videoconvert ! videoscale ! video/x-raw,format=RGBA,width=640,pixel-aspect-ratio=1/1 ! appsink name=preview sync=false async=false max-buffers=1 drop=true"
        } else {
            "! videoconvert ! videoscale ! video/x-raw,format=RGBA,width=640,pixel-aspect-ratio=1/1 ! appsink name=preview sync=false async=false max-buffers=1 drop=true"
        };
        let audio_source = match &source {
            #[cfg(feature = "verification")]
            Input::VerificationDisplay(_) => {
                "audiotestsrc name=microphone is-live=true wave=sine volume=0.05"
            }
            _ => "pulsesrc name=microphone client-name=alo-Studio do-timestamp=true",
        };
        let audio = if microphone && path.is_some() {
            format!(
                " {audio_source} ! queue name=audio-input ! audioconvert ! audioresample ! audio/x-raw,rate=48000,channels=1 ! opusenc bitrate=96000 ! queue ! container."
            )
        } else {
            String::new()
        };
        let pipeline = gst::parse::launch(&format!("{source_text} {video} {audio}"))
            .map_err(failed)?
            .downcast::<gst::Pipeline>()
            .map_err(|_| MediaError(t("The recording engine could not start.")))?;
        // Preview and recording share a permission-scoped PipeWire core. Keep
        // their timelines independent of clock resets in that shared graph.
        pipeline.use_clock(Some(&gst::SystemClock::obtain()));
        let by_name = |name| {
            pipeline.by_name(name).ok_or_else(|| {
                MediaError(format!(
                    "{} ({name}).",
                    t("A recording component is unavailable")
                ))
            })
        };
        let element = by_name("source")?;
        // Do not timestamp a preroll buffer before Playing assigns the base time.
        let paused = Arc::new(AtomicBool::new(true));
        match source {
            Input::PipeWire { fd, node } => {
                element.set_property("fd", fd);
                element.set_property("path", node.to_string());
                crate::capture_timing::receive(&pipeline, &element, paused.clone())?;
            }
            #[cfg(feature = "verification")]
            Input::VerificationDisplay(display) => {
                element.set_property("display-name", display);
            }
        }
        if let Some(path) = path {
            let caps = gst::Caps::builder("video/x-raw")
                .field("format", "I420")
                .field("width", width)
                .field("height", height)
                .field("framerate", gst::Fraction::new(30, 1))
                .field("pixel-aspect-ratio", gst::Fraction::new(1, 1))
                .build();
            by_name("size")?.set_property("caps", &caps);
            by_name("output")?.set_property("location", path.to_string_lossy().as_ref());
        }
        let fell_behind = Arc::new(AtomicBool::new(false));
        let video = if path.is_some() {
            let progress = Arc::new(Mutex::new(crate::recorded_video::Progress::default()));
            let recording = progress.clone();
            let encoded = by_name("encoder")?
                .static_pad("src")
                .ok_or_else(|| MediaError(t("The video encoder is unavailable.")))?;
            encoded.add_probe(gst::PadProbeType::BUFFER, move |_, info| {
                if let Some(buffer) = info.buffer()
                    && let Ok(mut progress) = recording.lock()
                {
                    progress.observe(buffer);
                }
                gst::PadProbeReturn::Ok
            });
            Some(progress)
        } else {
            None
        };
        if let Some(queue) = pipeline.by_name("record-buffer") {
            let stalled = fell_behind.clone();
            queue.connect("overrun", false, move |_| {
                stalled.store(true, Ordering::Relaxed);
                None
            });
        }
        let frame = Arc::new(Mutex::new(None));
        let receiving = frame.clone();
        let sink = by_name("preview")?
            .downcast::<gst_app::AppSink>()
            .map_err(|_| MediaError(t("The preview could not start.")))?;
        sink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    let caps = sample.caps().ok_or(gst::FlowError::Error)?;
                    let info =
                        gst_video::VideoInfo::from_caps(caps).map_err(|_| gst::FlowError::Error)?;
                    let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                    let mapped = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
                    let stride = info
                        .stride()
                        .first()
                        .copied()
                        .ok_or(gst::FlowError::Error)?;
                    if stride <= 0 {
                        return Err(gst::FlowError::Error);
                    }
                    let row_bytes = info.width() as usize * 4;
                    let mut bytes = Vec::with_capacity(row_bytes * info.height() as usize);
                    for row in mapped
                        .as_slice()
                        .chunks(stride as usize)
                        .take(info.height() as usize)
                    {
                        bytes.extend_from_slice(row.get(..row_bytes).ok_or(gst::FlowError::Error)?);
                    }
                    if let Ok(mut latest) = receiving.lock() {
                        *latest = Some(Frame {
                            bytes,
                            width: info.width() as i32,
                            height: info.height() as i32,
                        });
                    }
                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );
        let bus = pipeline
            .bus()
            .ok_or_else(|| MediaError(t("The recording engine cannot report its state.")))?;
        if let Err(error) = pipeline.set_state(gst::State::Playing) {
            let _ = pipeline.set_state(gst::State::Null);
            return Err(failed(error));
        }
        paused.store(false, Ordering::Relaxed);
        Ok(Self {
            source_ended: false,
            pipeline,
            bus,
            frame,
            finishing: false,
            fell_behind,
            video,
            paused,
        })
    }
    /// The duration actually encoded, available before stopping the engine.
    pub fn recorded_duration(&self) -> Option<std::time::Duration> {
        self.video.as_ref()?.lock().ok()?.duration()
    }
    /// The newest frame only. Slow UI painting never builds an unbounded queue.
    pub fn frame(&self) -> Option<Frame> {
        self.frame.lock().ok()?.take()
    }
    /// The negotiated source dimensions, before preview scaling.
    pub fn source_size(&self) -> Option<(i32, i32)> {
        let caps = self
            .pipeline
            .by_name("source")?
            .static_pad("src")?
            .current_caps()?;
        let info = gst_video::VideoInfo::from_caps(&caps).ok()?;
        Some((info.width() as i32, info.height() as i32))
    }
    /// Freeze both streams together; the microphone stops reading while paused.
    pub fn pause(&self) -> Result<(), MediaError> {
        self.paused.store(true, Ordering::Relaxed);
        if let Err(error) = self.pipeline.set_state(gst::State::Paused) {
            self.paused.store(false, Ordering::Relaxed);
            return Err(failed(error));
        }
        Ok(())
    }
    /// Continue both streams on the same pipeline clock.
    pub fn resume(&self) -> Result<(), MediaError> {
        self.pipeline
            .set_state(gst::State::Playing)
            .map_err(failed)?;
        self.paused.store(false, Ordering::Relaxed);
        Ok(())
    }
    /// Stop live inputs and seal queued media. Only bus EOS means it is finished.
    pub fn finish(&mut self) -> Result<(), MediaError> {
        // A native PipeWire provider need not forward a downstream EOS request.
        // Closing video and microphone first also makes Finish stop capture now.
        self.finish_after_source_ended()
    }
    /// Stop all live inputs, then drain only queued media into the WebM trailer.
    /// This also works after a revoked PipeWire source can no longer deliver EOS.
    pub fn finish_after_source_ended(&mut self) -> Result<(), MediaError> {
        if self.finishing {
            return Ok(());
        }
        let mut inputs = Vec::new();
        for (name, queue) in [("source", "video-input"), ("microphone", "audio-input")] {
            if let Some(source) = self.pipeline.by_name(name) {
                source.set_locked_state(true);
                source.set_state(gst::State::Null).map_err(failed)?;
                if let Some(queue) = self.pipeline.by_name(queue) {
                    if let Some(sink) = queue.static_pad("sink")
                        && let Some(upstream) = sink.peer()
                    {
                        upstream.unlink(&sink).map_err(failed)?;
                    }
                    inputs.push(queue);
                }
            }
        }
        self.source_ended = true;
        self.resume()?;
        for queue in inputs {
            if !queue
                .static_pad("sink")
                .is_some_and(|pad| pad.send_event(gst::event::Eos::new()))
            {
                return Err(MediaError(t(
                    "The recording could not finish. Its temporary file has been kept.",
                )));
            }
        }
        self.finishing = true;
        Ok(())
    }
    /// Read errors and EOS without blocking the interface.
    pub fn poll(&self) -> Option<Event> {
        if !self.finishing && self.fell_behind.swap(false, Ordering::Relaxed) {
            return Some(Event::FellBehind);
        }
        while let Some(message) = self.bus.pop() {
            match message.view() {
                gst::MessageView::Eos(_) => {
                    return Some(
                        if self.video.is_some() && self.recorded_duration().is_none() {
                            Event::Failed(t(
                                "No video frames were recorded. Please choose your source again.",
                            ))
                        } else {
                            Event::Finished
                        },
                    );
                }
                gst::MessageView::Error(error) => {
                    let live_input = error.src().is_some_and(|origin| {
                        ["source", "microphone"]
                            .iter()
                            .filter_map(|name| self.pipeline.by_name(name))
                            .any(|source| {
                                origin == source.upcast_ref::<gst::Object>()
                                    || origin.has_as_ancestor(&source)
                            })
                    });
                    if live_input {
                        if self.source_ended {
                            continue;
                        }
                        return Some(Event::SourceEnded(error.error().to_string()));
                    }
                    return Some(Event::Failed(error.error().to_string()));
                }
                _ => {}
            }
        }
        None
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        self.paused.store(true, Ordering::Relaxed);
        if let Err(error) = self.pipeline.set_state(gst::State::Null) {
            eprintln!("Stopping the recording engine: {error}");
        }
    }
}

fn failed(error: impl std::fmt::Display) -> MediaError {
    MediaError(error.to_string())
}
