//! Put permission-scoped PipeWire frames on the recorder's arrival timeline.

use crate::{recorder::MediaError, words::t};
use gst::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub(crate) fn receive(
    pipeline: &gst::Pipeline,
    source: &gst::Element,
    paused: Arc<AtomicBool>,
) -> Result<(), MediaError> {
    // Frames already arrive live from the selected source. The plugin's
    // pseudo-live clock scheduling can interpret a remote time base as a
    // future presentation deadline, blocking the stream before our queues.
    source.set_property(
        "stream-properties",
        gst::Structure::builder("props")
            .field("stream.is-live", false)
            .build(),
    );
    let pipeline = pipeline.downgrade();
    source
        .static_pad("src")
        .ok_or_else(|| MediaError(t("The selected source is unavailable.")))?
        .add_probe(gst::PadProbeType::BUFFER, move |_, info| {
            if paused.load(Ordering::Relaxed) {
                return gst::PadProbeReturn::Handled;
            }
            let Some(pipeline) = pipeline.upgrade() else {
                return gst::PadProbeReturn::Handled;
            };
            let Some(arrived) = pipeline.current_running_time() else {
                return gst::PadProbeReturn::Handled;
            };
            // A pause can begin while the clock lookup waits on the pipeline.
            // Do not admit that late buffer after its capture gate has closed.
            if paused.load(Ordering::Relaxed) {
                return gst::PadProbeReturn::Handled;
            }
            if let Some(buffer) = info.buffer_mut() {
                let buffer = buffer.make_mut();
                buffer.set_pts(arrived);
                buffer.set_dts(arrived);
            }
            gst::PadProbeReturn::Ok
        })
        .ok_or_else(|| MediaError(t("The selected source could not be prepared.")))?;
    Ok(())
}
