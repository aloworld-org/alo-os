//! Bound audio by retained data, never by holes between capture timestamps.

use crate::{recorder::MediaError, words::t};
use gst::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

const INPUT_BUFFERS: u32 = 200;
const ENCODED_BUFFERS: u32 = 500;
const MAX_BYTES: u32 = 1_048_576;

pub(crate) fn prepare(bin: &gst::Bin, fell_behind: Arc<AtomicBool>) -> Result<(), MediaError> {
    if bin.by_name("microphone").is_none() {
        return Ok(());
    }
    for (name, count) in [
        ("audio-input", INPUT_BUFFERS),
        ("audio-buffer", ENCODED_BUFFERS),
    ] {
        let queue = bin
            .by_name(name)
            .ok_or_else(|| MediaError(t("The microphone could not be prepared.")))?;
        limit(&queue, count, fell_behind.clone());
    }
    Ok(())
}

fn limit(queue: &gst::Element, count: u32, fell_behind: Arc<AtomicBool>) {
    queue.set_property("max-size-time", 0_u64);
    queue.set_property("max-size-bytes", MAX_BYTES);
    queue.set_property("max-size-buffers", count);
    queue.connect("overrun", false, move |values| {
        if let Some(queue) = values
            .first()
            .and_then(|value| value.get::<gst::Element>().ok())
        {
            eprintln!(
                "Audio queue {} reached its limit: {} buffers, {} bytes",
                queue.name(),
                queue.property::<u32>("current-level-buffers"),
                queue.property::<u32>("current-level-bytes")
            );
        }
        fell_behind.store(true, Ordering::Relaxed);
        None
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn timestamp_holes_do_not_fill_the_queue_but_retained_audio_does()
    -> Result<(), Box<dyn std::error::Error>> {
        gst::init()?;
        let pipeline = gst::parse::launch("appsrc name=input is-live=true format=time ! queue name=audio-input ! fakesink sync=false async=false")?
            .downcast::<gst::Pipeline>().map_err(|_| "No test pipeline")?;
        let source = pipeline
            .by_name("input")
            .ok_or("No test source")?
            .downcast::<gst_app::AppSrc>()
            .map_err(|_| "No app source")?;
        let queue = pipeline.by_name("audio-input").ok_or("No test queue")?;
        let full = Arc::new(AtomicBool::new(false));
        limit(&queue, INPUT_BUFFERS, full.clone());
        let pad = queue.static_pad("src").ok_or("No queue output")?;
        let blocked = pad
            .add_probe(
                gst::PadProbeType::BLOCK | gst::PadProbeType::BUFFER,
                |_, _| gst::PadProbeReturn::Ok,
            )
            .ok_or("Could not hold the test consumer")?;
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            pipeline.set_state(gst::State::Playing)?;
            let push = |millis| -> Result<(), gst::FlowError> {
                let mut buffer = gst::Buffer::with_size(1920).map_err(|_| gst::FlowError::Error)?;
                let data = buffer.make_mut();
                data.set_pts(gst::ClockTime::from_mseconds(millis));
                data.set_duration(gst::ClockTime::from_mseconds(20));
                source.push_buffer(buffer)?;
                Ok(())
            };
            for millis in [0, 2000, 2020] {
                push(millis)?;
            }
            let deadline = Instant::now() + Duration::from_secs(2);
            while queue.property::<u32>("current-level-buffers") < 2
                && !full.load(Ordering::Relaxed)
            {
                if Instant::now() >= deadline {
                    return Err("The delayed consumer did not receive audio".into());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            if full.load(Ordering::Relaxed) {
                return Err("A timestamp gap was counted as retained audio".into());
            }
            for index in 0..=INPUT_BUFFERS {
                push(2040 + u64::from(index) * 20)?;
            }
            let deadline = Instant::now() + Duration::from_secs(2);
            while !full.load(Ordering::Relaxed) {
                if Instant::now() >= deadline {
                    return Err("A full audio queue did not request a safe stop".into());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Ok(())
        })();
        pad.remove_probe(blocked);
        pipeline.set_state(gst::State::Null)?;
        result
    }
}
