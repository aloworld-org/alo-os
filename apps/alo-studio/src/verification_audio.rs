//! Delay generated audio without changing installed recording behavior.

use crate::recorder::MediaError;
use gst::prelude::*;

pub(crate) fn delay(source: &gst::Element, pipeline: &gst::Pipeline) -> Result<(), MediaError> {
    let Ok(delay) = std::env::var("ALO_STUDIO_VERIFY_AUDIO_START_DELAY_MS") else {
        return Ok(());
    };
    let delay = delay
        .parse::<u64>()
        .map_err(|error| MediaError(error.to_string()))?;
    if !(1..=1000).contains(&delay) {
        return Err(MediaError("Audio fixture delay must be 1–1000 ms".into()));
    }
    // Live audiotestsrc ignores timestamp-offset. Delay its first delivery and
    // shift its sample timeline by the same amount instead. This models a live
    // microphone that changes state before it can supply timestamped samples.
    source.set_property("do-timestamp", false);
    let pipeline = pipeline.downgrade();
    let offset = std::sync::OnceLock::new();
    source
        .static_pad("src")
        .and_then(|pad| {
            pad.add_probe(gst::PadProbeType::BUFFER, move |_, info| {
                let Some(buffer) = info.buffer_mut() else {
                    return gst::PadProbeReturn::Ok;
                };
                let Some(stamp) = buffer.pts() else {
                    return gst::PadProbeReturn::Ok;
                };
                let adjustment = *offset.get_or_init(|| {
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                    pipeline
                        .upgrade()
                        .and_then(|pipeline| pipeline.current_running_time())
                        .unwrap_or(stamp)
                        .saturating_sub(stamp)
                });
                let buffer = buffer.make_mut();
                buffer.set_pts(stamp.saturating_add(adjustment));
                buffer.set_dts(None);
                gst::PadProbeReturn::Ok
            })
        })
        .ok_or_else(|| MediaError("The delayed audio fixture could not start".into()))?;
    Ok(())
}
