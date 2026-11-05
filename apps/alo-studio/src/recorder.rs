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
    /// Video or microphone capture fell behind: seal the captured media.
    FellBehind,
}

/// One pipeline, stopped before its permission-scoped remote can be closed.
pub struct Recorder {
    source_ended: bool,
    pipeline: gst::Pipeline,
    bus: gst::Bus,
    frame: Arc<Mutex<Option<Frame>>>,
    preview_visible: Arc<AtomicBool>,
    video_ready: Arc<AtomicBool>,
    finishing: bool,
    fell_behind: Arc<AtomicBool>,
    video: Option<Arc<Mutex<crate::recorded_video::Progress>>>,
    paused: Arc<AtomicBool>,
    #[cfg(feature = "verification")]
    generated_audio: bool,
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
        Self::build(source)
    }
    /// Begin encoding on the preview's stream, at the chosen 1–30 fps ceiling.
    pub fn start_recording(
        &mut self,
        path: &Path,
        width: i32,
        height: i32,
        frame_rate: i32,
        microphone: bool,
    ) -> Result<(), MediaError> {
        let audio = if microphone {
            #[cfg(feature = "verification")]
            if self.generated_audio {
                return self.start_with_audio(
                    path,
                    width,
                    height,
                    frame_rate,
                    crate::recording_branch::Audio::Generated,
                );
            }
            crate::recording_branch::Audio::Microphone
        } else {
            crate::recording_branch::Audio::Off
        };
        self.start_with_audio(path, width, height, frame_rate, audio)
    }
    /// Exercise the selected stream with generated sound, without hardware.
    #[cfg(feature = "verification")]
    pub fn start_with_generated_audio(
        &mut self,
        path: &Path,
        width: i32,
        height: i32,
        frame_rate: i32,
    ) -> Result<(), MediaError> {
        self.start_with_audio(
            path,
            width,
            height,
            frame_rate,
            crate::recording_branch::Audio::Generated,
        )
    }
    fn start_with_audio(
        &mut self,
        path: &Path,
        width: i32,
        height: i32,
        frame_rate: i32,
        audio: crate::recording_branch::Audio,
    ) -> Result<(), MediaError> {
        if self.video.is_some() || self.finishing || self.source_ended || width <= 0 || height <= 0
        {
            return Err(MediaError(t("Please select a source for a new recording.")));
        }
        if !(1..=30).contains(&frame_rate) {
            return Err(MediaError(t(
                "Please choose a frame rate between 1 and 30 fps.",
            )));
        }
        self.video = Some(crate::recording_branch::attach(
            &self.pipeline,
            path,
            width,
            height,
            frame_rate,
            audio,
            crate::recording_branch::Signals {
                fell_behind: self.fell_behind.clone(),
                video_ready: self.video_ready.clone(),
            },
        )?);
        Ok(())
    }
    fn build(source: Input) -> Result<Self, MediaError> {
        gst::init().map_err(failed)?;
        let source_text = match &source {
            Input::PipeWire { .. } => {
                // Queued frames must own their bytes when Finish disconnects
                // the PipeWire stream and releases its shared buffer pool.
                // Preserve live scheduling while excluding cached keepalive
                // frames, whose clock domain differs in older plugins.
                "pipewiresrc name=source do-timestamp=true always-copy=true keepalive-time=0 resend-last=false ! video/x-raw,pixel-aspect-ratio=1/1"
            }
            #[cfg(feature = "verification")]
            Input::VerificationDisplay(_) => {
                "ximagesrc name=source use-damage=false show-pointer=true do-timestamp=true"
            }
        };
        let pipeline = gst::parse::launch(&format!(
            "{source_text} ! queue name=video-input max-size-buffers=2 max-size-bytes=0 max-size-time=0 ! tee name=frames \
             frames. ! queue name=preview-input leaky=downstream max-size-buffers=1 max-size-bytes=0 max-size-time=0 ! videoconvert ! videoscale ! video/x-raw,format=RGBA,width=[1,640],height=[1,480],pixel-aspect-ratio=1/1 ! appsink name=preview sync=false async=false max-buffers=1 drop=true"
        )).map_err(failed)?.downcast::<gst::Pipeline>()
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
        #[cfg(feature = "verification")]
        let generated_audio = matches!(&source, Input::VerificationDisplay(_));
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
        let fell_behind = Arc::new(AtomicBool::new(false));
        let preview_visible = Arc::new(AtomicBool::new(true));
        let video_ready = Arc::new(AtomicBool::new(false));
        let converting_preview = preview_visible.clone();
        let encoding_video = video_ready.clone();
        by_name("preview-input")?
            .static_pad("src")
            .and_then(|pad| {
                pad.add_probe(gst::PadProbeType::BUFFER, move |_, _| {
                    if converting_preview.load(Ordering::Relaxed)
                        || !encoding_video.load(Ordering::Acquire)
                    {
                        gst::PadProbeReturn::Ok
                    } else {
                        // Consume queued native frames before conversion. The
                        // capture stream and its recording branch keep running.
                        gst::PadProbeReturn::Handled
                    }
                })
            })
            .ok_or_else(|| MediaError(t("The preview could not start.")))?;
        let showing_preview = preview_visible.clone();
        let frame = Arc::new(Mutex::new(None));
        let receiving = frame.clone();
        let sink = by_name("preview")?
            .downcast::<gst_app::AppSink>()
            .map_err(|_| MediaError(t("The preview could not start.")))?;
        sink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    // Keep draining the native stream even when the workspace
                    // is hidden. Skip mapping and copying images nobody sees.
                    if !showing_preview.load(Ordering::Relaxed) {
                        return Ok(gst::FlowSuccess::Ok);
                    }
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
            for _ in 0..32 {
                let Some(message) = bus.pop() else {
                    break;
                };
                if let gst::MessageView::Error(detail) = message.view() {
                    eprintln!(
                        "Starting recording media: {} ({:?})",
                        detail.error(),
                        detail.debug()
                    );
                }
            }
            let _ = pipeline.set_state(gst::State::Null);
            return Err(failed(error));
        }
        paused.store(false, Ordering::Relaxed);
        Ok(Self {
            source_ended: false,
            pipeline,
            bus,
            frame,
            preview_visible,
            video_ready,
            finishing: false,
            fell_behind,
            video: None,
            paused,
            #[cfg(feature = "verification")]
            generated_audio,
        })
    }
    /// The duration actually encoded, available before stopping the engine.
    pub fn recorded_duration(&self) -> Option<std::time::Duration> {
        self.video.as_ref()?.lock().ok()?.duration()
    }
    /// Time since fresh video joined the selected audio timeline. Engine state
    /// changes alone do not establish that either source has supplied media.
    pub fn capture_elapsed(&self) -> Option<std::time::Duration> {
        let first = self.video.as_ref()?.lock().ok()?.input_time()?;
        self.pipeline
            .current_running_time()?
            .checked_sub(first)
            .map(|elapsed| std::time::Duration::from_nanos(elapsed.nseconds()))
    }
    /// The newest frame only. Slow UI painting never builds an unbounded queue.
    pub fn frame(&self) -> Option<Frame> {
        self.frame.lock().ok()?.take()
    }
    /// Skip conversion and copying while the workspace is hidden. Native
    /// capture buffers keep draining and encoding continues independently.
    pub fn set_preview_visible(&self, visible: bool) {
        self.preview_visible.store(visible, Ordering::Relaxed);
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
        self.video_ready.store(false, Ordering::Release);
        self.paused.store(true, Ordering::Relaxed);
        if let Err(error) = self.pipeline.set_state(gst::State::Paused) {
            self.paused.store(false, Ordering::Relaxed);
            return Err(failed(error));
        }
        Ok(())
    }
    /// Continue both streams on the same pipeline clock.
    pub fn resume(&self) -> Result<(), MediaError> {
        self.video_ready.store(false, Ordering::Release);
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
        // Stop admitting video at Finish, then close the microphone before a
        // slow native video disconnect can extend the audio recording.
        self.paused.store(true, Ordering::Relaxed);
        let mut inputs = Vec::new();
        for (name, queue) in [("microphone", "audio-input"), ("source", "video-input")] {
            if let Some(source) = self.pipeline.by_name(name) {
                source.set_locked_state(true);
                source.set_state(gst::State::Null).map_err(failed)?;
                #[cfg(feature = "verification")]
                if name == "source" && std::env::var_os("ALO_STUDIO_VERIFY_STOP_DELAY").is_some() {
                    std::thread::sleep(std::time::Duration::from_millis(750));
                }
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
                gst::MessageView::Warning(warning)
                    if !self.finishing
                        && warning.error().matches(gst::CoreError::Clock)
                        && self
                            .pipeline
                            .by_name("microphone")
                            .is_some_and(|microphone| {
                                warning.src().is_some_and(|origin| {
                                    origin == microphone.upcast_ref::<gst::Object>()
                                        || origin.has_as_ancestor(&microphone)
                                })
                            }) =>
                {
                    // GstAudioBaseSrc reports lost samples as CORE/CLOCK, not
                    // as an error. Do not silently continue with broken audio.
                    eprintln!(
                        "Microphone capture fell behind: {} ({:?})",
                        warning.error(),
                        warning.debug()
                    );
                    return Some(Event::FellBehind);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn idle() -> Result<Recorder, Box<dyn std::error::Error>> {
        gst::init()?;
        let pipeline = gst::parse::launch(
            "fakesrc name=microphone ! fakesink fakesrc name=source ! fakesink",
        )?
        .downcast::<gst::Pipeline>()
        .map_err(|_| "No test pipeline")?;
        let bus = pipeline.bus().ok_or("No test bus")?;
        Ok(Recorder {
            source_ended: false,
            pipeline,
            bus,
            frame: Arc::new(Mutex::new(None)),
            preview_visible: Arc::new(AtomicBool::new(true)),
            video_ready: Arc::new(AtomicBool::new(false)),
            finishing: false,
            fell_behind: Arc::new(AtomicBool::new(false)),
            video: None,
            paused: Arc::new(AtomicBool::new(false)),
            #[cfg(feature = "verification")]
            generated_audio: false,
        })
    }

    #[test]
    fn lost_microphone_samples_request_a_controlled_finish()
    -> Result<(), Box<dyn std::error::Error>> {
        let recorder = idle()?;
        let microphone = recorder
            .pipeline
            .by_name("microphone")
            .ok_or("No microphone")?;
        recorder.bus.post(
            gst::message::Warning::builder(gst::CoreError::Clock, "Lost audio samples")
                .src(&microphone)
                .build(),
        )?;
        assert!(matches!(recorder.poll(), Some(Event::FellBehind)));
        assert!(recorder.poll().is_none());
        Ok(())
    }

    #[test]
    fn unrelated_warnings_do_not_stop_recording() -> Result<(), Box<dyn std::error::Error>> {
        let recorder = idle()?;
        let microphone = recorder
            .pipeline
            .by_name("microphone")
            .ok_or("No microphone")?;
        let source = recorder.pipeline.by_name("source").ok_or("No source")?;
        for message in [
            gst::message::Warning::builder(gst::CoreError::Clock, "Other clock warning")
                .src(&source)
                .build(),
            gst::message::Warning::builder(gst::StreamError::Failed, "Other audio warning")
                .src(&microphone)
                .build(),
        ] {
            recorder.bus.post(message)?;
        }
        assert!(recorder.poll().is_none());
        Ok(())
    }

    #[test]
    fn audio_warning_does_not_interrupt_finalization() -> Result<(), Box<dyn std::error::Error>> {
        let mut recorder = idle()?;
        recorder.finishing = true;
        let microphone = recorder
            .pipeline
            .by_name("microphone")
            .ok_or("No microphone")?;
        recorder.bus.post(
            gst::message::Warning::builder(gst::CoreError::Clock, "Late audio warning")
                .src(&microphone)
                .build(),
        )?;
        recorder.bus.post(gst::message::Eos::new())?;
        assert!(matches!(recorder.poll(), Some(Event::Finished)));
        Ok(())
    }
}
