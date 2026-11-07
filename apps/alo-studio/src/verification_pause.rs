//! Inspect the live native source while recording and microphone input pause.

use super::{MediaError, Recorder};
use gst::prelude::*;

impl Recorder {
    /// Check the real PipeWire input remains connected while recording is gated.
    /// This inspection is excluded from the normal installed application.
    pub fn verify_paused_native_capture(&self) -> Result<(), MediaError> {
        let source = self
            .pipeline
            .by_name("source")
            .ok_or_else(|| MediaError("No native capture source".into()))?;
        if !self.paused.load(std::sync::atomic::Ordering::Relaxed)
            || self.pipeline.current_state() != gst::State::Paused
            || source.current_state() != gst::State::Playing
            || !source.is_locked_state()
            || !source
                .factory()
                .is_some_and(|factory| factory.name() == "pipewiresrc")
        {
            return Err(MediaError(
                "The paused native video connection was interrupted".into(),
            ));
        }
        if self
            .pipeline
            .by_name("microphone")
            .is_some_and(|source| source.current_state() == gst::State::Playing)
        {
            return Err(MediaError("The paused microphone is still active".into()));
        }
        Ok(())
    }
}
