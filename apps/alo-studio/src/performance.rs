//! Measure the selected encoder configuration before a recording starts.

use crate::recorder::MediaError;
use crate::words::t;
use gst::prelude::*;
use std::time::{Duration, Instant};

/// Encode 1.5 seconds of frames without reading a screen, microphone, or file.
/// This is a preflight, not a claim that arbitrary future content has a fixed
/// cost. The recording queue also stops a real recording if its encoder stalls.
pub fn check(width: i32, height: i32) -> Result<(), MediaError> {
    gst::init().map_err(|error| MediaError(error.to_string()))?;
    let element = gst::parse::launch("videotestsrc num-buffers=45 pattern=smpte ! capsfilter name=size ! videoconvert ! vp9enc deadline=1 cpu-used=8 threads=4 target-bitrate=4500000 ! fakesink sync=false")
        .map_err(|error| MediaError(error.to_string()))?;
    let pipeline = element
        .downcast::<gst::Pipeline>()
        .map_err(|_| MediaError(t("Recording quality could not be checked.")))?;
    let caps = gst::Caps::builder("video/x-raw")
        .field("width", width)
        .field("height", height)
        .field("framerate", gst::Fraction::new(30, 1))
        .build();
    let size = pipeline
        .by_name("size")
        .ok_or_else(|| MediaError(t("Recording quality could not be checked.")))?;
    size.set_property("caps", caps);
    let started = Instant::now();
    if let Err(error) = pipeline.set_state(gst::State::Playing) {
        let _ = pipeline.set_state(gst::State::Null);
        return Err(MediaError(error.to_string()));
    }
    let message = pipeline.bus().and_then(|bus| {
        bus.timed_pop_filtered(
            gst::ClockTime::from_seconds(8),
            &[gst::MessageType::Eos, gst::MessageType::Error],
        )
    });
    let elapsed = started.elapsed();
    let _ = pipeline.set_state(gst::State::Null);
    match message.as_ref().map(|message| message.view()) {
        Some(gst::MessageView::Eos(_)) if elapsed <= Duration::from_millis(1400) => Ok(()),
        Some(gst::MessageView::Error(error)) => Err(MediaError(format!(
            "{} {}",
            t("Recording quality could not be checked."),
            error.error()
        ))),
        _ => Err(MediaError(t(
            "This recording size is too demanding right now. Choose 720p, close other busy applications, or select a smaller window. Nothing has been recorded.",
        ))),
    }
}
