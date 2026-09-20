//! Settings on the nested compositor: keys in, the sections out.
//!
//! Two calls, used in turn by whoever runs the session while Settings is open:
//! [`Nested::pump_settings`] takes the parent window's keys through the seat
//! and hands each to Settings, and [`Nested::submit_with_settings`] draws the
//! session's frame with Settings above every client — and with a waiting
//! question above Settings and the egress indicator above both, because
//! changing a setting is never a reason to hide what the machine is asking or
//! what is leaving it.

use std::time::SystemTime;

use alo_strings::Strings;
use smithay::backend::{
    input::{Event, InputEvent, KeyboardKeyEvent},
    winit::WinitEvent,
};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::approval_raster::ApprovalPicture;
use crate::egress_status_raster::EgressStatusPicture;
use crate::nested_egress_status::status_picture;
use crate::scene_native::{NativeLayers, NativeScene};
use crate::settings_raster::{SettingsPicture, picture};
use crate::{
    ApprovalFrame, EgressStatusFrame, FrameTarget, InputError, Nested, RenderError, Server,
    SettingsDid, SettingsDoors, SettingsLook, SettingsWindow, WindowControlLabels,
};

/// What a frame needs to draw Settings.
#[derive(Clone, Copy)]
pub struct SettingsFrame<'a> {
    /// Settings, as it stands.
    pub window: &'a SettingsWindow,
    /// The vocabulary the person reads, which words every row.
    pub strings: &'a Strings,
    /// Their scheme, text scale and reading direction.
    pub look: SettingsLook,
    /// The moment drawn, for the grants and pairings in force.
    pub now: SystemTime,
}

impl Nested {
    /// Route the parent window's keys to Settings, in order, and hand back what
    /// each key that did something did.
    ///
    /// Keys go through `server`'s seat and are never forwarded to any client.
    /// **Once Settings is closed, nothing more is routed**, and while the
    /// parent window does not have the keyboard nothing is routed at all. A
    /// host with a question open routes keys to the approval surface first.
    ///
    /// # Errors
    /// [`RenderError::Closed`] when the parent window closed, and
    /// [`RenderError::Input`] when `server` has no keyboard.
    pub fn pump_settings(
        &mut self,
        server: &mut Server,
        window: &mut SettingsWindow,
        doors: SettingsDoors<'_>,
    ) -> Result<Vec<SettingsDid>, RenderError> {
        let mut did = Vec::new();
        let mut failure: Option<InputError> = None;
        let pumped = self.pump_events(|event, focused| {
            if failure.is_some() || !focused || !window.is_open() {
                return;
            }
            let Some(WinitEvent::Input(InputEvent::Keyboard { event })) = event else {
                return;
            };
            let code = u32::from(event.key_code()).saturating_sub(8);
            match server.settings_key(code, event.state(), event.time_msec()) {
                Ok(Some(press)) => did.extend(window.pressed(press, doors)),
                Ok(None) | Err(InputError::InvalidKey) => {}
                Err(error) => failure = Some(error),
            }
        });
        match failure {
            Some(error) => Err(RenderError::Input(error)),
            None => pumped.map(|()| did),
        }
    }

    /// Submit clients, popups and native controls with Settings above them, a
    /// waiting question above Settings, the egress indicator above that, and
    /// the cursor on top.
    ///
    /// While Settings is closed it draws nothing, and the frame is exactly the
    /// one `Nested::submit_with_approval` or `Nested::submit_with_egress_status`
    /// would submit.
    ///
    /// # Errors
    /// [`RenderError::SettingsScene`] when open Settings cannot be held on this
    /// output; every refusal the indicator and the approval surface make; and
    /// the backend's own submission failures. A refused frame draws nothing.
    #[expect(
        clippy::too_many_arguments,
        reason = "one session frame: clients, popups, cursor, controls, fonts, the indicator, Settings and the question"
    )]
    pub fn submit_with_settings(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
        controls: Option<crate::WindowControlScene<'_>>,
        labels: &mut WindowControlLabels,
        egress: EgressStatusFrame<'_>,
        settings: SettingsFrame<'_>,
        approval: Option<ApprovalFrame<'_>>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        let size = self.size();
        let (status, sections, question) =
            frame_pictures(egress, settings, approval, labels, (size.w, size.h))?;
        self.submit_native_layers(
            roots,
            popups,
            cursor,
            NativeLayers {
                scene: controls.map(NativeScene::Controls),
                desktop: None,
                record: None,
                settings: Some(&sections),
                approval: question.as_ref(),
                status: Some(&status),
            },
        )
    }
}

/// The indicator, Settings and any question for one frame, or the refusal that
/// stops the whole frame — the indicator's first, because a frame that cannot
/// say what is leaving is refused whatever else it holds.
fn frame_pictures(
    egress: EgressStatusFrame<'_>,
    settings: SettingsFrame<'_>,
    approval: Option<ApprovalFrame<'_>>,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
) -> Result<
    (
        EgressStatusPicture,
        SettingsPicture,
        Option<ApprovalPicture>,
    ),
    RenderError,
> {
    let status = status_picture(egress, labels, size)?;
    let sections = picture(
        settings.window,
        settings.strings,
        labels,
        size,
        settings.look,
        settings.now,
    )?;
    let question = match approval {
        Some(approval) => Some(crate::approval_raster::picture(
            approval.screen.shows(),
            approval.strings,
            labels,
            size,
            approval.look,
        )?),
        None => None,
    };
    Ok((status, sections, question))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::Contrast;
    use crate::settings_testing::{a_persons_machine, light, words};
    use crate::{EgressStatus, EgressStatusLook};
    use alo_appearance::{Scheme, TextScale};
    use alo_dock::Dock;
    use alo_egress::Indicator;
    use alo_indicator::{Drew, Indicating};
    use alo_strings::Direction;

    /// **A frame carrying Settings still carries the indicator**: one whose
    /// indicator was never told is refused whole, Settings and all; told, the
    /// frame holds both; and Settings that cannot hold its focused row whole
    /// refuses the frame rather than cutting it.
    #[test]
    fn a_frame_with_settings_is_refused_whole_without_the_indicator_or_room() {
        let machine = a_persons_machine("frame");
        let strings = words();
        let dock = Dock::shipped();
        let mut labels = WindowControlLabels::new().unwrap();
        let egress_look = EgressStatusLook {
            contrast: Contrast::AsDesigned,
            scheme: Scheme::Light,
            scale: TextScale::ordinary(),
            reading: Direction::LeftToRight,
        };
        let window = machine.opened();
        let settings = SettingsFrame {
            window: &window,
            strings: &strings,
            look: light(),
            now: crate::settings_testing::noon(),
        };

        let never_told = EgressStatus::on_an_output();
        let egress = EgressStatusFrame {
            status: &never_told,
            strings: &strings,
            dock: &dock,
            look: egress_look,
        };
        assert!(matches!(
            frame_pictures(egress, settings, None, &mut labels, (1920, 1080)),
            Err(RenderError::EgressStatusUnknown)
        ));

        let mut status = EgressStatus::on_an_output();
        assert_eq!(
            Indicating::nowhere().show(Some(&mut status), &Indicator::default()),
            Drew::Shown
        );
        let egress = EgressStatusFrame {
            status: &status,
            strings: &strings,
            dock: &dock,
            look: egress_look,
        };
        let (indicator, sections, question) =
            frame_pictures(egress, settings, None, &mut labels, (1920, 1080)).unwrap();
        assert!(indicator.is_empty());
        assert!(!sections.is_empty());
        assert!(!sections.lines.is_empty());
        assert!(question.is_none());

        assert!(matches!(
            frame_pictures(egress, settings, None, &mut labels, (1920, 90)),
            Err(RenderError::SettingsScene)
        ));
    }
}
