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
    // The selected source supplies live frames. Keep live scheduling; cached
    // keepalive frames are disabled when constructing the source instead.
    source.set_property(
        "stream-properties",
        gst::Structure::builder("props")
            .field("stream.is-live", true)
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
