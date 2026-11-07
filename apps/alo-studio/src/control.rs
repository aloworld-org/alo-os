//! Typed automation commands share the UI lifecycle and never select a source.

use crate::model::{Phase, Quality};

/// One semantic action; no toggle, interpreter, source identifier or export path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Command {
    NewRecording,
    ChooseScreen,
    ChooseWindow,
    Start,
    Pause,
    Resume,
    Finish,
    Microphone(bool),
    Quality(Quality),
    Countdown(u8),
    CancelCountdown,
    ShowLibrary,
    Review(String),
    Rename(String),
    Export,
}

impl Command {
    /// Refuse stale or repeated actions before touching a widget or device.
    pub(crate) fn allowed(&self, phase: Phase, preview: bool, clip: bool) -> bool {
        let changing = matches!(phase, Phase::Ready | Phase::Selected | Phase::Review);
        match self {
            Self::NewRecording
            | Self::ChooseScreen
            | Self::ChooseWindow
            | Self::Microphone(_)
            | Self::Quality(_)
            | Self::Countdown(_)
            | Self::ShowLibrary
            | Self::Review(_) => changing,
            Self::Start => phase == Phase::Selected && preview,
            Self::Pause => phase == Phase::Recording,
            Self::Resume => phase == Phase::Paused,
            Self::Finish => matches!(phase, Phase::Preparing | Phase::Recording | Phase::Paused),
            Self::CancelCountdown => matches!(phase, Phase::Countdown(_)),
            Self::Rename(_) | Self::Export => phase == Phase::Review && clip,
        }
    }
}

/// Stable protocol names, independent of translated labels and Rust spelling.
pub(crate) fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::Ready => "ready",
        Phase::Choosing => "choosing",
        Phase::Selected => "selected",
        Phase::Checking => "checking",
        Phase::Countdown(_) => "countdown",
        Phase::Preparing => "preparing",
        Phase::Recording => "recording",
        Phase::Paused => "paused",
        Phase::Finishing => "finishing",
        Phase::Review => "review",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_requires_a_selected_source_and_its_live_preview() {
        for phase in [
            Phase::Ready,
            Phase::Choosing,
            Phase::Checking,
            Phase::Countdown(1),
            Phase::Review,
        ] {
            assert!(!Command::Start.allowed(phase, true, true));
        }
        assert!(!Command::Start.allowed(Phase::Selected, false, false));
        assert!(Command::Start.allowed(Phase::Selected, true, false));
    }

    #[test]
    fn repeated_pause_resume_and_finish_do_not_change_meaning() {
        assert!(Command::Pause.allowed(Phase::Recording, true, true));
        assert!(!Command::Pause.allowed(Phase::Paused, true, true));
        assert!(Command::Resume.allowed(Phase::Paused, true, true));
        assert!(!Command::Resume.allowed(Phase::Recording, true, true));
        assert!(!Command::Finish.allowed(Phase::Review, true, true));
        assert!(!Command::Finish.allowed(Phase::Ready, false, false));
    }

    #[test]
    fn busy_capture_refuses_new_sources_configuration_and_library_navigation() {
        for phase in [
            Phase::Choosing,
            Phase::Checking,
            Phase::Countdown(3),
            Phase::Preparing,
            Phase::Recording,
            Phase::Paused,
            Phase::Finishing,
        ] {
            for command in [
                Command::ChooseScreen,
                Command::ChooseWindow,
                Command::NewRecording,
                Command::Microphone(true),
                Command::Quality(Quality::Hd),
                Command::Countdown(0),
                Command::ShowLibrary,
                Command::Review("recording".into()),
            ] {
                assert!(
                    !command.allowed(phase, true, true),
                    "{command:?} in {phase:?}"
                );
            }
        }
    }

    #[test]
    fn only_completed_media_can_be_named_or_exported() {
        for command in [
            Command::Rename("Product demonstration".into()),
            Command::Export,
        ] {
            assert!(!command.allowed(Phase::Review, true, false));
            assert!(!command.allowed(Phase::Finishing, true, true));
            assert!(command.allowed(Phase::Review, true, true));
        }
    }
}
