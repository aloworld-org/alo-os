//! Admit fresh video only after the chosen audio source actually starts.

use crate::{recorder::MediaError, words::t};
use gst::prelude::*;
use std::sync::{
    Arc,
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
        sink: &gst::Pad,
        ready: gst::ClockTime,
    ) -> Result<(), MediaError> {
        sink.add_probe(gst::PadProbeType::BUFFER, move |_, info| {
            let earliest = ready.nseconds().max(self.audio.load(Ordering::Acquire));
            if info
                .buffer()
                .and_then(|buffer| buffer.pts())
                .is_some_and(|stamp| stamp.nseconds() >= earliest)
            {
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
