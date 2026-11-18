//! Reject stale video instead of giving WebM repeated or invented timestamps.

use crate::{recorder::MediaError, words::t};
use gst::prelude::*;
use std::sync::Mutex;

/// WebM uses one-millisecond ticks, explicitly selected on our muxer.
const TICK: gst::ClockTime = gst::ClockTime::from_mseconds(1);

/// A fresh instance belongs to one recording and survives its pause/resume.
#[derive(Default)]
struct Order {
    last: Option<gst::ClockTime>,
}

impl Order {
    /// Do not change either the timestamp or the duration of an accepted frame.
    fn admit(&mut self, stamp: Option<gst::ClockTime>) -> bool {
        let Some(stamp) = stamp else { return false };
        if self
            .last
            .is_some_and(|last| last.checked_add(TICK).is_none_or(|minimum| stamp < minimum))
        {
            return false;
        }
        self.last = Some(stamp);
        true
    }
}

/// Install after rate conversion, before stale buffers reach the encoder.
pub(crate) fn prepare(encoder: &gst::Element) -> Result<(), MediaError> {
    let order = Mutex::new(Order::default());
    encoder
        .static_pad("sink")
        .ok_or_else(|| MediaError(t("The video encoder is unavailable.")))?
        .add_probe(gst::PadProbeType::BUFFER, move |_, info| {
            let stamp = info.buffer().and_then(|buffer| buffer.pts());
            if order.lock().is_ok_and(|mut order| order.admit(stamp)) {
                gst::PadProbeReturn::Ok
            } else {
                // Rust consumes the refused buffer; never retimestamp or pad it.
                gst::PadProbeReturn::Handled
            }
        })
        .ok_or_else(|| MediaError(t("The recording could not be prepared.")))?;
    Ok(())
}

#[cfg(test)]
#[path = "recording_order_tests.rs"]
mod tests;
