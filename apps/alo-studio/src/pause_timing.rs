//! Keep time spent stopping native inputs out of a recording's paused timeline.

use crate::{recorder::MediaError, words::t};
use gst::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) fn pause(pipeline: &gst::Pipeline, paused: &AtomicBool) -> Result<(), MediaError> {
    if let Some(microphone) = pipeline.by_name("microphone")
        && let Some(clock) = microphone.provide_clock()
        && pipeline.clock().as_ref() == Some(&clock)
    {
        // Retained microphone samples belong to its sample clock. Stop that
        // clock before saving the boundary; freezing it while capture is still
        // advancing makes those retained samples overlap at Resume.
        microphone
            .set_state(gst::State::Paused)
            .map_err(|error| MediaError(error.to_string()))?;
    }
    paused.store(true, Ordering::Relaxed);
    // Close admission before sampling the cutoff. A buffer that passed the
    // final source-gate check was timestamped before this boundary.
    let Some(requested) = pipeline.current_running_time() else {
        paused.store(false, Ordering::Relaxed);
        return Err(MediaError(t(
            "The recording clock is unavailable. Please try again.",
        )));
    };
    if let Err(error) = pipeline.set_state(gst::State::Paused) {
        paused.store(false, Ordering::Relaxed);
        return Err(MediaError(error.to_string()));
    }
    // The pipeline samples time again after pausing its children. Capture was
    // already gated above, so that native shutdown wait belongs to the pause.
    pipeline.set_start_time(Some(requested));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gst::glib;
    use std::time::{Duration, Instant};

    mod delayed {
        use gst::glib;
        use gst::prelude::*;
        use gst::subclass::prelude::*;

        #[derive(Default)]
        pub struct Input {
            clock: std::sync::OnceLock<gst::SystemClock>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for Input {
            const NAME: &'static str = "StudioInputWithDelayedPause";
            type Type = super::SlowInput;
            type ParentType = gst::Bin;
        }
        impl ObjectImpl for Input {}
        impl GstObjectImpl for Input {}
        impl ElementImpl for Input {
            fn provide_clock(&self) -> Option<gst::Clock> {
                Some(
                    self.clock
                        .get_or_init(gst::glib::Object::new::<gst::SystemClock>)
                        .clone()
                        .upcast(),
                )
            }
            fn change_state(
                &self,
                transition: gst::StateChange,
            ) -> Result<gst::StateChangeSuccess, gst::StateChangeError> {
                if transition == gst::StateChange::PlayingToPaused {
                    std::thread::sleep(std::time::Duration::from_millis(300));
                }
                self.parent_change_state(transition)
            }
        }
        impl BinImpl for Input {}
    }

    glib::wrapper! {
        pub struct SlowInput(ObjectSubclass<delayed::Input>)
            @extends gst::Bin, gst::Element, gst::Object;
    }

    #[test]
    fn a_slow_input_pause_does_not_advance_the_saved_timeline()
    -> Result<(), Box<dyn std::error::Error>> {
        gst::init()?;
        let pipeline = gst::Pipeline::new();
        pipeline.add(&glib::Object::new::<SlowInput>())?;
        pipeline.set_state(gst::State::Playing)?;
        let before = pipeline.current_running_time().ok_or("No running clock")?;
        let started = Instant::now();
        let paused = AtomicBool::new(false);
        pause(&pipeline, &paused)?;
        let elapsed = started.elapsed();
        let saved = pipeline.start_time().ok_or("No paused time")?;
        pipeline.set_state(gst::State::Null)?;
        assert!(elapsed >= Duration::from_millis(250));
        assert!(paused.load(Ordering::Relaxed));
        assert!(saved >= before);
        assert!(
            saved - before < gst::ClockTime::from_mseconds(100),
            "stopping the input added {} to the recording's paused time",
            saved - before
        );
        Ok(())
    }

    #[test]
    fn microphone_pause_completes_before_its_clock_boundary_is_saved()
    -> Result<(), Box<dyn std::error::Error>> {
        gst::init()?;
        let pipeline = gst::Pipeline::new();
        let microphone = glib::Object::builder::<SlowInput>()
            .property("name", "microphone")
            .build();
        let clock = microphone.provide_clock().ok_or("No microphone clock")?;
        pipeline.use_clock(Some(&clock));
        pipeline.add(&microphone)?;
        pipeline.set_state(gst::State::Playing)?;
        let before = pipeline.current_running_time().ok_or("No running clock")?;
        pause(&pipeline, &AtomicBool::new(false))?;
        let saved = pipeline.start_time().ok_or("No paused time")?;
        pipeline.set_state(gst::State::Null)?;
        assert!(saved - before >= gst::ClockTime::from_mseconds(250));
        Ok(())
    }
}
