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

pub(crate) fn prepare(bin: &gst::Bin) -> Result<(), MediaError> {
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
        limit(&queue, count);
    }
    Ok(())
}

fn limit(queue: &gst::Element, count: u32) {
    queue.set_property("max-size-time", 0_u64);
    queue.set_property("max-size-bytes", MAX_BYTES);
    queue.set_property("max-size-buffers", count);
    // A full non-leaky queue blocks upstream; it has not discarded audio.
    // PulseAudio's actual sample-loss warning is handled by Recorder::poll.
    let reported = Arc::new(AtomicBool::new(false));
    queue.connect("overrun", false, move |values| {
        if reported.swap(true, Ordering::Relaxed) {
            return None;
        }
        if let Some(queue) = values
            .first()
            .and_then(|value| value.get::<gst::Element>().ok())
        {
            eprintln!(
                "Audio queue {} is applying backpressure: {} buffers, {} bytes",
                queue.name(),
                queue.property::<u32>("current-level-buffers"),
                queue.property::<u32>("current-level-bytes")
            );
        }
        None
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn a_full_bounded_queue_recovers_without_losing_audio_or_inventing_a_capture_failure()
    -> Result<(), Box<dyn std::error::Error>> {
        gst::init()?;
        let pipeline = gst::parse::launch("appsrc name=input is-live=true format=time ! queue name=audio-input ! fakesink name=consumer sync=false async=false")?
            .downcast::<gst::Pipeline>().map_err(|_| "No test pipeline")?;
        let source = pipeline
            .by_name("input")
            .ok_or("No test source")?
            .downcast::<gst_app::AppSrc>()
            .map_err(|_| "No app source")?;
        let queue = pipeline.by_name("audio-input").ok_or("No test queue")?;
        let full = Arc::new(AtomicBool::new(false));
        limit(&queue, INPUT_BUFFERS);
        let observed = full.clone();
        queue.connect("overrun", false, move |_| {
            observed.store(true, Ordering::Release);
            None
        });
        let received = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counted = received.clone();
        pipeline
            .by_name("consumer")
            .ok_or("No test consumer")?
            .static_pad("sink")
            .ok_or("No consumer input")?
            .add_probe(gst::PadProbeType::BUFFER, move |_, _| {
                counted.fetch_add(1, Ordering::Relaxed);
                gst::PadProbeReturn::Ok
            })
            .ok_or("Could not observe received audio")?;
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
                    return Err("The bounded queue did not apply backpressure".into());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Ok(())
        })();
        pad.remove_probe(blocked);
        let drained = (|| -> Result<(), Box<dyn std::error::Error>> {
            result?;
            source.end_of_stream()?;
            let message = pipeline
                .bus()
                .ok_or("No pipeline bus")?
                .timed_pop_filtered(
                    gst::ClockTime::from_seconds(3),
                    &[gst::MessageType::Eos, gst::MessageType::Error],
                )
                .ok_or("The recovered queue did not drain")?;
            assert_eq!(message.type_(), gst::MessageType::Eos);
            assert_eq!(received.load(Ordering::Relaxed), INPUT_BUFFERS as usize + 4);
            Ok(())
        })();
        pipeline.set_state(gst::State::Null)?;
        drained
    }
}
