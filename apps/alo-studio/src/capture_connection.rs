//! Keep a native video graph connected across recording clock transitions.

use crate::{recorder::MediaError, words::t};
use gst::prelude::*;

/// Own only a state lock acquired here; the permission owner remains unchanged.
pub(crate) struct Connected {
    source: Option<gst::Element>,
}

impl Connected {
    /// Native arrivals are discarded by capture_timing while the recorder pauses.
    /// Keeping that source Playing avoids removing/recreating its PipeWire link.
    pub(crate) fn native(pipeline: &gst::Pipeline) -> Option<Self> {
        let source = pipeline.by_name("source")?;
        source
            .factory()
            .is_some_and(|factory| factory.name() == "pipewiresrc")
            .then_some(())?;
        Self::keep(source)
    }

    fn keep(source: gst::Element) -> Option<Self> {
        if source.is_locked_state() || source.current_state() != gst::State::Playing {
            return None;
        }
        source.set_locked_state(true);
        Some(Self {
            source: Some(source),
        })
    }

    /// Restore parent control after Playing, including its updated base time.
    pub(crate) fn release(mut self) -> Result<(), MediaError> {
        restore(self.source.take())
    }

    /// Finish and Drop stop inputs themselves; never restart one during teardown.
    pub(crate) fn retire(mut self) {
        if let Some(source) = self.source.take() {
            source.set_locked_state(false);
        }
    }
}

fn restore(source: Option<gst::Element>) -> Result<(), MediaError> {
    if let Some(source) = source {
        source.set_locked_state(false);
        source.sync_state_with_parent().map_err(|_| {
            MediaError(t(
                "The recording connection could not be restored. Please select your source again.",
            ))
        })?;
    }
    Ok(())
}

impl Drop for Connected {
    fn drop(&mut self) {
        // An early return also gives the source back to its parent's state.
        if let Err(error) = restore(self.source.take()) {
            eprintln!("Restoring the capture connection: {error}");
        }
    }
}

#[cfg(test)]
#[path = "capture_connection_tests.rs"]
mod tests;
