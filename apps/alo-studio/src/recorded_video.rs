//! Measure encoded video rather than assuming the interface timer was recorded.

use std::time::Duration;

#[derive(Default)]
pub(crate) struct Progress {
    input: Option<gst::ClockTime>,
    first: Option<gst::ClockTime>,
    end: Option<gst::ClockTime>,
    #[cfg(feature = "verification")]
    latest: Option<gst::ClockTime>,
}

impl Progress {
    pub(crate) fn input_started(&mut self, stamp: gst::ClockTime) {
        self.input.get_or_insert(stamp);
    }

    pub(crate) fn input_time(&self) -> Option<gst::ClockTime> {
        self.input
    }

    pub(crate) fn observe(&mut self, buffer: &gst::BufferRef) {
        let Some(start) = buffer.pts() else { return };
        let Some(duration) = buffer.duration() else {
            return;
        };
        let Some(end) = start.checked_add(duration) else {
            return;
        };
        self.first = Some(self.first.map_or(start, |first| first.min(start)));
        self.end = Some(self.end.map_or(end, |last| last.max(end)));
        #[cfg(feature = "verification")]
        {
            self.latest = Some(self.latest.map_or(start, |last| last.max(start)));
        }
    }

    /// Compare captured time with a pause boundary, rather than encoder progress.
    #[cfg(feature = "verification")]
    pub(crate) fn latest_time(&self) -> Option<gst::ClockTime> {
        self.latest
    }

    pub(crate) fn duration(&self) -> Option<Duration> {
        let duration = self.end?.checked_sub(self.first?)?;
        (duration > gst::ClockTime::ZERO).then(|| Duration::from_nanos(duration.nseconds()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn video_frame(progress: &mut Progress, start_ms: u64) {
        let mut buffer = gst::Buffer::new();
        let data = buffer.make_mut();
        data.set_pts(gst::ClockTime::from_mseconds(start_ms));
        data.set_duration(gst::ClockTime::from_mseconds(33));
        progress.observe(data);
    }

    #[test]
    fn an_empty_recording_has_no_completed_video_duration() {
        assert_eq!(Progress::default().duration(), None);
    }

    #[test]
    fn one_frame_cannot_be_reported_as_a_long_recording() -> Result<(), gst::glib::Error> {
        gst::init()?;
        let mut progress = Progress::default();
        video_frame(&mut progress, 5000);
        assert_eq!(progress.duration(), Some(Duration::from_millis(33)));
        video_frame(&mut progress, 5033);
        assert_eq!(progress.duration(), Some(Duration::from_millis(66)));
        Ok(())
    }
}
