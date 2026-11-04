//! The recording state machine. No device can start as a side effect of a view.

use std::time::{Duration, Instant};

/// The one source a person deliberately asks the system to let them select.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceKind {
    /// One whole display, chosen in the system's own picker.
    #[default]
    Screen,
    /// One application window, chosen in the system's own picker.
    Window,
}

/// A visible recording size, never silently changed when encoding struggles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Quality {
    /// Full HD, or the source's own size when smaller.
    #[default]
    FullHd,
    /// HD, or the source's own size when smaller.
    Hd,
    /// The full source's dimensions.
    Original,
}

impl Quality {
    /// Fit inside the chosen bounds without stretching or enlarging the source.
    pub fn size(self, width: i32, height: i32) -> Result<(i32, i32), &'static str> {
        if width <= 0 || height <= 0 {
            return Err("The selected screen did not report a usable size.");
        }
        let (max_width, max_height) = match self {
            Self::FullHd => (1920.0_f64, 1080.0_f64),
            Self::Hd => (1280.0, 720.0),
            Self::Original => (f64::from(width), f64::from(height)),
        };
        let scale = (max_width / f64::from(width))
            .min(max_height / f64::from(height))
            .min(1.0);
        // Even sizes are required by the encoder's 4:2:0 pixel format.
        let even = |number: f64| ((number.floor() as i32) / 2 * 2).max(2);
        Ok((
            even(f64::from(width) * scale),
            even(f64::from(height) * scale),
        ))
    }
}

/// What is happening now, kept independent of the widget tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// No screen is being read.
    Ready,
    /// Waiting for the person's system-picker answer.
    Choosing,
    /// The selected source can be previewed, but nothing is being saved.
    Selected,
    /// A countdown precedes the first frame written.
    Countdown(u8),
    /// Video, and microphone only if chosen, are being saved.
    Recording,
    /// Capture is paused; the recording clock is paused too.
    Paused,
    /// The encoder is closing the file. It cannot be saved or discarded yet.
    Finishing,
    /// A playable, durable recording is ready.
    Review,
}

/// Invalid transitions refuse rather than starting another device or file.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("That action is not available while {0:?}.")]
pub struct InvalidTransition(pub Phase);

/// One session, with a monotonic clock that excludes pauses.
pub struct Session {
    phase: Phase,
    elapsed: Duration,
    running_since: Option<Instant>,
    microphone: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            phase: Phase::Ready,
            elapsed: Duration::ZERO,
            running_since: None,
            microphone: false,
        }
    }
}

impl Session {
    /// The current state.
    pub fn phase(&self) -> Phase {
        self.phase
    }
    /// Microphone permission is a choice for this recording only.
    pub fn microphone(&self) -> bool {
        self.microphone
    }
    /// Whether closing the window would interrupt an unfinished video.
    pub fn busy(&self) -> bool {
        matches!(
            self.phase,
            Phase::Recording | Phase::Paused | Phase::Finishing
        )
    }
    /// Begin asking; a dismissed system picker can return to the prior selection.
    pub fn choosing(&mut self) -> Result<(), InvalidTransition> {
        self.require(&[Phase::Ready, Phase::Selected, Phase::Review])?;
        self.phase = Phase::Choosing;
        Ok(())
    }
    /// The person chose a source. No microphone starts here.
    pub fn selected(&mut self) {
        self.phase = Phase::Selected;
    }
    /// A cancelled or failed source selection reads no screen.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    /// Remember a microphone choice only before recording.
    pub fn set_microphone(&mut self, enabled: bool) -> Result<(), InvalidTransition> {
        self.require(&[Phase::Ready, Phase::Selected, Phase::Review])?;
        self.microphone = enabled;
        Ok(())
    }
    /// Count down without writing a file or opening a microphone.
    pub fn countdown(&mut self, seconds: u8) -> Result<(), InvalidTransition> {
        self.require(&[Phase::Selected])?;
        self.phase = Phase::Countdown(seconds);
        Ok(())
    }
    /// The next countdown number, or the signal to start the media pipeline.
    pub fn tick(&mut self) -> Result<bool, InvalidTransition> {
        match self.phase {
            Phase::Countdown(0 | 1) => Ok(true),
            Phase::Countdown(number) => {
                self.phase = Phase::Countdown(number - 1);
                Ok(false)
            }
            phase => Err(InvalidTransition(phase)),
        }
    }
    /// Nothing has been written; cancelling preserves the selected source.
    pub fn cancel_countdown(&mut self) -> Result<(), InvalidTransition> {
        if !matches!(self.phase, Phase::Countdown(_)) {
            return Err(InvalidTransition(self.phase));
        }
        self.phase = Phase::Selected;
        Ok(())
    }
    /// Called only after the actual recording pipeline has started.
    pub fn started(&mut self, now: Instant) -> Result<(), InvalidTransition> {
        if !matches!(self.phase, Phase::Countdown(_)) {
            return Err(InvalidTransition(self.phase));
        }
        self.phase = Phase::Recording;
        self.elapsed = Duration::ZERO;
        self.running_since = Some(now);
        Ok(())
    }
    /// Pause after the media engine has successfully paused.
    pub fn pause(&mut self, now: Instant) -> Result<(), InvalidTransition> {
        self.require(&[Phase::Recording])?;
        self.elapsed = self.elapsed_at(now);
        self.running_since = None;
        self.phase = Phase::Paused;
        Ok(())
    }
    /// Resume after the media engine has successfully resumed.
    pub fn resume(&mut self, now: Instant) -> Result<(), InvalidTransition> {
        self.require(&[Phase::Paused])?;
        self.running_since = Some(now);
        self.phase = Phase::Recording;
        Ok(())
    }
    /// Start sealing the file; a second Stop cannot race the first.
    pub fn finishing(&mut self, now: Instant) -> Result<(), InvalidTransition> {
        self.require(&[Phase::Recording, Phase::Paused])?;
        self.elapsed = self.elapsed_at(now);
        self.running_since = None;
        self.phase = Phase::Finishing;
        Ok(())
    }
    /// Only a finalized and saved video becomes reviewable.
    pub fn finished(&mut self) -> Result<(), InvalidTransition> {
        self.require(&[Phase::Finishing])?;
        self.phase = Phase::Review;
        self.microphone = false;
        Ok(())
    }
    /// Active recording time, never wall-clock time spent paused.
    pub fn elapsed_at(&self, now: Instant) -> Duration {
        self.elapsed
            + self
                .running_since
                .map(|since| now.saturating_duration_since(since))
                .unwrap_or_default()
    }
    fn require(&self, allowed: &[Phase]) -> Result<(), InvalidTransition> {
        if allowed.contains(&self.phase) {
            Ok(())
        } else {
            Err(InvalidTransition(self.phase))
        }
    }
}

/// A stable clock format for the recording controls and library.
pub fn clock(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cannot_record_without_a_deliberately_selected_source() {
        let mut session = Session::default();
        assert!(session.countdown(3).is_err());
        assert!(session.started(Instant::now()).is_err());
        assert!(!session.microphone());
    }
    #[test]
    fn pause_is_excluded_and_stop_is_spent_once() {
        let now = Instant::now();
        let mut session = Session::default();
        session.selected();
        session.countdown(3).unwrap();
        session.started(now).unwrap();
        session.pause(now + Duration::from_secs(4)).unwrap();
        assert_eq!(
            session.elapsed_at(now + Duration::from_secs(50)).as_secs(),
            4
        );
        session.resume(now + Duration::from_secs(50)).unwrap();
        session.finishing(now + Duration::from_secs(53)).unwrap();
        assert_eq!(
            session.elapsed_at(now + Duration::from_secs(100)).as_secs(),
            7
        );
        assert!(session.finishing(now).is_err());
    }
    #[test]
    fn cancelling_countdown_never_starts_the_recording_clock() {
        let mut session = Session::default();
        session.selected();
        session.countdown(3).unwrap();
        assert!(!session.tick().unwrap());
        session.cancel_countdown().unwrap();
        assert_eq!(session.phase(), Phase::Selected);
        assert_eq!(session.elapsed_at(Instant::now()), Duration::ZERO);
    }
    #[test]
    fn microphone_is_reset_after_each_recording() {
        let mut session = Session::default();
        let now = Instant::now();
        session.selected();
        session.set_microphone(true).unwrap();
        session.countdown(0).unwrap();
        session.started(now).unwrap();
        assert!(session.set_microphone(false).is_err());
        session.finishing(now).unwrap();
        session.finished().unwrap();
        assert!(!session.microphone());
    }
    #[test]
    fn quality_preserves_aspect_ratio_and_never_enlarges() {
        assert_eq!(Quality::FullHd.size(3840, 2160), Ok((1920, 1080)));
        assert_eq!(Quality::FullHd.size(800, 600), Ok((800, 600)));
        assert_eq!(Quality::FullHd.size(1080, 1920), Ok((606, 1080)));
        assert!(Quality::Hd.size(0, 0).is_err());
    }
}
