//! Admit fresh video only after the chosen audio source actually starts.

use crate::{recorder::MediaError, words::t};
use gst::prelude::*;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

pub(crate) struct Start {
    audio: Arc<AtomicU64>,
}

impl Start {
    /// A live audio source can finish changing state before its first sample.
    pub(crate) fn observe(bin: &gst::Bin) -> Result<Self, MediaError> {
        let audio = Arc::new(AtomicU64::new(0));
        if let Some(source) = bin.by_name("microphone") {
            audio.store(u64::MAX, Ordering::Relaxed);
            let first = audio.clone();
            source
                .static_pad("src")
                .and_then(|pad| {
                    pad.add_probe(gst::PadProbeType::BUFFER, move |_, info| {
                        if let Some(stamp) = info.buffer().and_then(|buffer| buffer.pts()) {
                            first.store(stamp.nseconds(), Ordering::Release);
                            gst::PadProbeReturn::Remove
                        } else {
                            gst::PadProbeReturn::Ok
                        }
                    })
                })
                .ok_or_else(|| MediaError(t("The microphone could not be prepared.")))?;
        }
        Ok(Self { audio })
    }

    /// Discard preview backlog and video preceding the first real audio sample.
    pub(crate) fn admit_video(
        self,
        pipeline: &gst::Pipeline,
        sink: &gst::Pad,
        ready: gst::ClockTime,
        progress: Arc<Mutex<crate::recorded_video::Progress>>,
    ) -> Result<(), MediaError> {
        let pipeline = pipeline.downgrade();
        sink.add_probe(gst::PadProbeType::BUFFER, move |_, info| {
            let earliest = ready.nseconds().max(self.audio.load(Ordering::Acquire));
            let now = pipeline
                .upgrade()
                .and_then(|pipeline| pipeline.current_running_time());
            if let Some(stamp) = info
                .buffer()
                .and_then(|buffer| buffer.pts())
                .filter(|stamp| stamp.nseconds() >= earliest)
                .filter(|stamp| now.is_some_and(|now| *stamp <= now))
            {
                if let Ok(mut progress) = progress.lock() {
                    progress.input_started(stamp);
                }
                gst::PadProbeReturn::Remove
            } else {
                // The Rust probe consumes this buffer. Handled prevents an
                // older GStreamer core from unrefing that buffer a second time.
                gst::PadProbeReturn::Handled
            }
        })
        .ok_or_else(|| MediaError(t("The recording could not be prepared.")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_frames_from_the_previous_clock_epoch_cannot_start_a_recording()
    -> Result<(), Box<dyn std::error::Error>> {
        gst::init()?;
        let pipeline = gst::parse::launch(
            "appsrc name=input is-live=true format=time ! fakesink name=output sync=false async=false",
        )?.downcast::<gst::Pipeline>().map_err(|_| "No test pipeline")?;
        let input = pipeline
            .by_name("input")
            .ok_or("No input")?
            .downcast::<gst_app::AppSrc>()
            .map_err(|_| "Not an app source")?;
        let sink = pipeline
            .by_name("output")
            .and_then(|sink| sink.static_pad("sink"))
            .ok_or("No sink pad")?;
        let progress = Arc::new(Mutex::new(crate::recorded_video::Progress::default()));
        let start = Start::observe(&gst::Bin::new())?;
        pipeline.set_state(gst::State::Playing)?;
        start.admit_video(&pipeline, &sink, gst::ClockTime::ZERO, progress.clone())?;
        for stamp in [gst::ClockTime::from_seconds(10), gst::ClockTime::ZERO] {
            let mut buffer = gst::Buffer::new();
            buffer.make_mut().set_pts(stamp);
            input.push_buffer(buffer)?;
        }
        input.end_of_stream()?;
        let message = pipeline
            .bus()
            .ok_or("No bus")?
            .timed_pop_filtered(
                gst::ClockTime::from_seconds(2),
                &[gst::MessageType::Eos, gst::MessageType::Error],
            )
            .ok_or("The test pipeline did not finish")?;
        pipeline.set_state(gst::State::Null)?;
        assert_eq!(message.type_(), gst::MessageType::Eos);
        assert_eq!(
            progress.lock().map_err(|_| "No progress")?.input_time(),
            Some(gst::ClockTime::ZERO)
        );
        Ok(())
    }
}
