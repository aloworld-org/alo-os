//! Inspect real native clock ownership during the integration probe's drain.

use super::{MediaError, Recorder};
use gst::prelude::*;

impl Recorder {
    /// Verify capture has stopped without destroying a clock still in use.
    /// Available only to explicitly built integration probes.
    pub fn verify_stopped_inputs(&self) -> Result<(), MediaError> {
        if !self.finishing {
            return Err(MediaError("Media is not draining".into()));
        }
        for name in ["microphone", "source"] {
            if let Some(source) = self.pipeline.by_name(name)
                && (source.current_state() == gst::State::Playing || !source.is_locked_state())
            {
                return Err(MediaError(format!("Capture input {name} is still active")));
            }
        }
        if let Some(clock) = self.pipeline.clock()
            && clock.name() == "GstPulseSrcClock"
        {
            let provider = self
                .pipeline
                .by_name("microphone")
                .and_then(|mic| mic.provide_clock());
            if provider.as_ref() != Some(&clock) {
                return Err(MediaError(
                    "The draining pipeline lost its native microphone clock provider".into(),
                ));
            }
        }
        Ok(())
    }
}
