//! Measure the selected encoder configuration before a recording starts.

use crate::recorder::MediaError;
use crate::words::t;
use gst::prelude::*;
use std::time::{Duration, Instant};

/// Advice from a short sample; the person still chooses their recording size.
pub enum Assessment {
    /// The sample encoded faster than its playback time.
    Ready,
    /// The sample was slow, so continuing needs an explicit choice.
    Demanding,
}

/// Encode about 1.5 seconds at the chosen rate without opening a device or file.
/// This is a preflight, not a claim that arbitrary future content has a fixed
/// cost. The recording queue also stops a real recording if its encoder stalls.
pub fn check(width: i32, height: i32, frame_rate: i32) -> Result<Assessment, MediaError> {
    if !(1..=30).contains(&frame_rate) {
        return Err(MediaError(t(
            "Please choose a frame rate between 1 and 30 fps.",
        )));
    }
    gst::init().map_err(|error| MediaError(error.to_string()))?;
    let frames = (frame_rate * 3 + 1) / 2;
    let element = gst::parse::launch(&format!("videotestsrc num-buffers={frames} pattern=smpte ! capsfilter name=size ! videoconvert ! vp9enc deadline=1 cpu-used=8 threads=4 lag-in-frames=0 target-bitrate=4500000 ! fakesink sync=false"))
        .map_err(|error| MediaError(error.to_string()))?;
    let pipeline = element
        .downcast::<gst::Pipeline>()
        .map_err(|_| MediaError(t("Recording quality could not be checked.")))?;
    let caps = gst::Caps::builder("video/x-raw")
        .field("format", "I420")
        .field("width", width)
        .field("height", height)
        .field("framerate", gst::Fraction::new(frame_rate, 1))
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
        Some(gst::MessageView::Eos(_))
            if elapsed
                <= Duration::from_secs_f64(f64::from(frames) / f64::from(frame_rate))
                    .saturating_sub(Duration::from_millis(100)) =>
        {
            Ok(Assessment::Ready)
        }
        Some(gst::MessageView::Error(error)) => Err(MediaError(format!(
            "{} {}",
            t("Recording quality could not be checked."),
            error.error()
        ))),
        _ => Ok(Assessment::Demanding),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_frame_rates_outside_the_supported_range() {
        assert!(check(640, 360, 0).is_err());
        assert!(check(640, 360, 31).is_err());
    }
}
