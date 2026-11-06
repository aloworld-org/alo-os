//! Join typed automation to the same visible controls and lifecycle as a person.

use super::*;
use crate::control::{Command, phase_name};

type Error = (&'static str, String);

impl State {
    /// A bounded read of this session; it neither opens a source nor reads files.
    pub(super) fn automation_status(&self) -> serde_json::Value {
        let session = self.session.borrow();
        let phase = session.phase();
        let preview = !self.view.placeholder.is_visible();
        let clip = self.clip.borrow();
        serde_json::json!({
            "version": 1,
            "phase": phase_name(phase),
            "source_kind": if self.kind.get() == SourceKind::Screen { "screen" } else { "window" },
            "microphone_enabled": self.view.microphone.is_active(),
            "microphone_active": phase == Phase::Recording && session.microphone(),
            "quality": match self.view.quality.selected() { 1 => "hd", 2 => "original", 3 => "lightweight", _ => "full_hd" },
            "countdown_seconds": match self.view.countdown.selected() { 0 => 3, 1 => 5, _ => 0 },
            "elapsed_ms": session.elapsed_at(Instant::now()).as_millis().min(u128::from(u64::MAX)) as u64,
            "recording_id": clip.as_ref().map(|clip| clip.id.as_str()),
            "preview_ready": preview,
            "export_pending": self.export_pending.get(),
            "can_start": Command::Start.allowed(phase, preview, clip.is_some()),
            "can_pause": Command::Pause.allowed(phase, preview, clip.is_some()),
            "can_resume": Command::Resume.allowed(phase, preview, clip.is_some()),
            "can_finish": Command::Finish.allowed(phase, preview, clip.is_some()),
        })
    }

    /// Validate state first, and preserve the system picker for capture/export.
    pub(super) fn automation_command(self: &Rc<Self>, command: Command) -> Result<(), Error> {
        let phase = self.session.borrow().phase();
        let preview = !self.view.placeholder.is_visible();
        if !command.allowed(phase, preview, self.clip.borrow().is_some()) {
            return Err((
                "org.aloworld.Studio.Error.InvalidState",
                t("This action is unavailable in the current recording state."),
            ));
        }
        let failed = |message: String| ("org.aloworld.Studio.Error.Unavailable", message);
        match command {
            Command::NewRecording => self.new_recording(),
            Command::ChooseScreen | Command::ChooseWindow => {
                if self.library.is_none() {
                    return Err(failed(t("Your recording library is unavailable.")));
                }
                Recorder::available().map_err(|error| failed(error.to_string()))?;
                if phase == Phase::Review {
                    self.new_recording();
                }
                if command == Command::ChooseScreen {
                    self.view.screen.set_active(true);
                } else {
                    self.view.one_window.set_active(true);
                }
                self.choose();
            }
            Command::Start => {
                self.preflight();
                if self.session.borrow().phase() != Phase::Checking {
                    return Err(failed(t(
                        "The recording could not start. Please check the recording window.",
                    )));
                }
            }
            Command::Pause | Command::Resume => {
                self.pause();
                let wanted = if command == Command::Pause {
                    Phase::Paused
                } else {
                    Phase::Recording
                };
                if self.session.borrow().phase() != wanted {
                    return Err(failed(t("The recording state could not be changed.")));
                }
            }
            Command::Finish => self.stop(),
            Command::Microphone(enabled) => self.view.microphone.set_active(enabled),
            Command::Quality(quality) => {
                let index = match quality {
                    Quality::FullHd => 0,
                    Quality::Hd => 1,
                    Quality::Original => 2,
                    Quality::Lightweight => 3,
                };
                if !self.view.quality.select(index) {
                    return Err(failed(t("The recording quality is unavailable.")));
                }
            }
            Command::Countdown(seconds) => {
                let index = match seconds {
                    3 => 0,
                    5 => 1,
                    _ => 2,
                };
                if !self.view.countdown.select(index) {
                    return Err(failed(t("The countdown is unavailable.")));
                }
            }
            Command::CancelCountdown => self.cancel_countdown(),
            Command::ShowLibrary => self.show_library(),
            Command::Review(id) => {
                let library = self
                    .library
                    .as_ref()
                    .ok_or_else(|| failed(t("Your recording library is unavailable.")))?;
                library
                    .video(&id)
                    .map_err(|error| failed(error.to_string()))?;
                let (clips, _) = library.clips().map_err(|error| failed(error.to_string()))?;
                let clip = clips
                    .into_iter()
                    .find(|clip| clip.id == id)
                    .ok_or_else(|| failed(t("That recording could not be found.")))?;
                self.open_clip(&clip);
            }
            Command::Rename(title) => {
                let library = self
                    .library
                    .as_ref()
                    .ok_or_else(|| failed(t("Your recording library is unavailable.")))?;
                let mut clip = self
                    .clip
                    .borrow()
                    .clone()
                    .ok_or_else(|| failed(t("Choose a completed recording first.")))?;
                clip.title = title;
                library
                    .update(&clip)
                    .map_err(|error| failed(error.to_string()))?;
                *self.clip.borrow_mut() = Some(clip.clone());
                self.view.review.title.set_text(&clip.title);
            }
            Command::Export => {
                if self.export_pending.get() {
                    return Err((
                        "org.aloworld.Studio.Error.Busy",
                        t("A recording export is already in progress."),
                    ));
                }
                self.export();
            }
        }
        self.render();
        Ok(())
    }

    pub(super) fn release_automation(&self) {
        if let Some((connection, registration)) = self.automation.borrow_mut().take() {
            let _ = connection.unregister_object(registration);
        }
    }
}
