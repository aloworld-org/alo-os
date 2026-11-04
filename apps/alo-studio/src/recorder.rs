//! GStreamer supplies capture, VP9, Opus and WebM. No codec is implemented here.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use gst::prelude::*;

use crate::portal::Capture;

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
    /// The WebM trailer has been written and the file can be committed.
    Finished,
    /// The source or encoder failed. The pending file is preserved.
    Failed(String),
    /// A bounded queue filled: finish the clip instead of degrading it silently.
    FellBehind,
}

/// One pipeline, stopped before its permission-scoped remote can be closed.
pub struct Recorder {
    pipeline: gst::Pipeline,
    bus: gst::Bus,
    frame: Arc<Mutex<Option<Frame>>>,
    finishing: bool,
    fell_behind: Arc<AtomicBool>,
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
                    "A required recording component is missing ({name}). Please reinstall alo Studio."
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
                "pipewiresrc name=source do-timestamp=true keepalive-time=1000"
            }
            #[cfg(feature = "verification")]
            Input::VerificationDisplay(_) => {
                "ximagesrc name=source use-damage=false show-pointer=true do-timestamp=true"
            }
        };
        // This is a fixed engine configuration. No filename, device name, or
        // other person-supplied text is interpolated into the pipeline parser.
        let video = if path.is_some() {
            "! videoconvert ! videoscale ! videorate ! capsfilter name=size ! tee name=frames \
             frames. ! queue name=record-buffer max-size-time=2000000000 max-size-bytes=0 max-size-buffers=0 ! vp9enc name=encoder deadline=1 cpu-used=8 threads=4 target-bitrate=4500000 keyframe-max-dist=60 \
             ! queue ! webmmux name=container ! filesink name=output \
             frames. ! queue leaky=downstream max-size-buffers=1 ! videoconvert ! videoscale ! video/x-raw,format=RGBA,width=640,pixel-aspect-ratio=1/1 ! appsink name=preview sync=false max-buffers=1 drop=true"
        } else {
            "! videoconvert ! videoscale ! video/x-raw,format=RGBA,width=640,pixel-aspect-ratio=1/1 ! appsink name=preview sync=false max-buffers=1 drop=true"
        };
        let audio_source = match &source {
            #[cfg(feature = "verification")]
            Input::VerificationDisplay(_) => "audiotestsrc is-live=true wave=sine volume=0.05",
            _ => "pulsesrc name=microphone client-name=alo-Studio do-timestamp=true",
        };
        let audio = if microphone && path.is_some() {
            format!(
                " {audio_source} ! queue ! audioconvert ! audioresample ! audio/x-raw,rate=48000,channels=1 ! opusenc bitrate=96000 ! queue ! container."
            )
        } else {
            String::new()
        };
        let pipeline = gst::parse::launch(&format!("{source_text} {video} {audio}"))
            .map_err(failed)?
            .downcast::<gst::Pipeline>()
            .map_err(|_| MediaError("The recording engine could not start.".to_owned()))?;
        let by_name = |name| {
            pipeline
                .by_name(name)
                .ok_or_else(|| MediaError(format!("The recording engine has no {name}.")))
        };
        let element = by_name("source")?;
        match source {
            Input::PipeWire { fd, node } => {
                element.set_property("fd", fd);
                element.set_property("path", node.to_string());
            }
            #[cfg(feature = "verification")]
            Input::VerificationDisplay(display) => {
                element.set_property("display-name", display);
            }
        }
        if let Some(path) = path {
            let caps = gst::Caps::builder("video/x-raw")
                .field("width", width)
                .field("height", height)
                .field("framerate", gst::Fraction::new(30, 1))
                .field("pixel-aspect-ratio", gst::Fraction::new(1, 1))
                .build();
            by_name("size")?.set_property("caps", &caps);
            by_name("output")?.set_property("location", path.to_string_lossy().as_ref());
        }
        let fell_behind = Arc::new(AtomicBool::new(false));
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
            .map_err(|_| MediaError("The preview could not start.".to_owned()))?;
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
        let bus = pipeline.bus().ok_or_else(|| {
            MediaError("The recording engine cannot report its state.".to_owned())
        })?;
        if let Err(error) = pipeline.set_state(gst::State::Playing) {
            let _ = pipeline.set_state(gst::State::Null);
            return Err(failed(error));
        }
        Ok(Self {
            pipeline,
            bus,
            frame,
            finishing: false,
            fell_behind,
        })
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
        self.pipeline
            .set_state(gst::State::Paused)
            .map(|_| ())
            .map_err(failed)
    }
    /// Continue both streams on the same pipeline clock.
    pub fn resume(&self) -> Result<(), MediaError> {
        self.pipeline
            .set_state(gst::State::Playing)
            .map(|_| ())
            .map_err(failed)
    }
    /// Seal the container. Only EOS, observed on the bus, means it is finished.
    pub fn finish(&mut self) -> Result<(), MediaError> {
        if self.finishing {
            return Ok(());
        }
        // A paused pipeline needs to run to deliver EOS and flush the muxer.
        self.resume()?;
        if !self.pipeline.send_event(gst::event::Eos::new()) {
            return Err(MediaError(
                "The recording could not finish. Its temporary file has been kept.".to_owned(),
            ));
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
                gst::MessageView::Eos(_) => return Some(Event::Finished),
                gst::MessageView::Error(error) => {
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
        if let Err(error) = self.pipeline.set_state(gst::State::Null) {
            eprintln!("Stopping the recording engine: {error}");
        }
    }
}

fn failed(error: impl std::fmt::Display) -> MediaError {
    MediaError(error.to_string())
}
