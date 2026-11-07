//! Use the microphone's sample clock for both tracks before recording starts.

use crate::{recorder::MediaError, words::t};
use gst::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) fn select(
    pipeline: &gst::Pipeline,
    branch: &gst::Bin,
    paused: &AtomicBool,
) -> Result<(), MediaError> {
    let previous_clock = pipeline.clock();
    let previous_time = pipeline.current_running_time();
    let connected = crate::capture_connection::Connected::native(pipeline);
    // No recording buffer has been admitted. Pause the existing source without
    // disconnecting its permission-scoped stream and prepare the microphone.
    paused.store(true, Ordering::Relaxed);
    let result = (|| {
        pipeline.set_state(gst::State::Paused).map_err(failed)?;
        let clock = branch
            .by_name("microphone")
            .and_then(|source| source.provide_clock())
            .ok_or_else(|| {
                MediaError(t("The recording clock is unavailable. Please try again."))
            })?;
        pipeline.use_clock(Some(&clock));
        // The microphone's clock begins at its own first sample, not at the
        // previous preview's running time. Old preview buffers are rejected by
        // recording_start before this new epoch is admitted to the encoder.
        pipeline.set_start_time(Some(gst::ClockTime::ZERO));
        pipeline.set_state(gst::State::Playing).map_err(failed)?;
        Ok(())
    })();
    if result.is_err() {
        pipeline.use_clock(previous_clock.as_ref());
        pipeline.set_start_time(previous_time);
        let _ = pipeline.set_state(gst::State::Playing);
    }
    let restored = connected
        .map(crate::capture_connection::Connected::release)
        .transpose()
        .map(|_| ());
    paused.store(false, Ordering::Relaxed);
    result.and(restored)
}

fn failed(error: impl std::fmt::Display) -> MediaError {
    MediaError(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gst::glib;

    mod microphone {
        use gst::{glib, prelude::*, subclass::prelude::*};

        #[derive(Default)]
        pub struct Input;

        #[glib::object_subclass]
        impl ObjectSubclass for Input {
            const NAME: &'static str = "StudioInputWithRecordingClock";
            type Type = super::Microphone;
            type ParentType = gst::Bin;
        }
        impl ObjectImpl for Input {}
        impl GstObjectImpl for Input {}
        impl ElementImpl for Input {
            fn provide_clock(&self) -> Option<gst::Clock> {
                Some(glib::Object::new::<gst::SystemClock>().upcast())
            }
        }
        impl BinImpl for Input {}
    }

    glib::wrapper! {
        pub struct Microphone(ObjectSubclass<microphone::Input>)
            @extends gst::Bin, gst::Element, gst::Object;
    }

    #[test]
    fn microphone_recording_replaces_the_preview_clock_before_admitting_video()
    -> Result<(), Box<dyn std::error::Error>> {
        gst::init()?;
        let pipeline = gst::Pipeline::new();
        let preview_clock = gst::SystemClock::obtain();
        pipeline.use_clock(Some(&preview_clock));
        pipeline.set_state(gst::State::Playing)?;
        let branch = gst::Bin::new();
        let microphone = glib::Object::builder::<Microphone>()
            .property("name", "microphone")
            .build();
        branch.add(&microphone)?;
        pipeline.add(&branch)?;
        let paused = AtomicBool::new(false);
        select(&pipeline, &branch, &paused)?;
        let selected = pipeline.clock().ok_or("No recording clock")?;
        let elapsed = pipeline
            .current_running_time()
            .ok_or("No new recording timeline")?;
        pipeline.set_state(gst::State::Null)?;
        assert_ne!(selected, preview_clock.upcast::<gst::Clock>());
        assert!(elapsed < gst::ClockTime::from_mseconds(100));
        assert!(!paused.load(Ordering::Relaxed));
        Ok(())
    }

    #[test]
    fn missing_microphone_clock_restores_the_running_preview()
    -> Result<(), Box<dyn std::error::Error>> {
        gst::init()?;
        let pipeline = gst::Pipeline::new();
        let clock = gst::SystemClock::obtain();
        pipeline.use_clock(Some(&clock));
        let branch = gst::Bin::new();
        pipeline.add(&branch)?;
        pipeline.set_state(gst::State::Playing)?;
        let paused = AtomicBool::new(false);
        let result = select(&pipeline, &branch, &paused);
        let selected = pipeline.clock();
        let state = pipeline.current_state();
        pipeline.set_state(gst::State::Null)?;
        assert!(result.is_err());
        assert_eq!(selected, Some(clock.upcast::<gst::Clock>()));
        assert_eq!(state, gst::State::Playing);
        assert!(!paused.load(Ordering::Relaxed));
        Ok(())
    }
}
