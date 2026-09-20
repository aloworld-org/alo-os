//! The record window on the nested compositor: keys in, the account out.
//!
//! Two calls, used in turn by whoever runs the session while the window is
//! open: [`Nested::pump_record`] takes the parent window's keys through the
//! seat and hands each to the window, and [`Nested::submit_with_record`] draws
//! the session's frame with the account above every client — and with a
//! waiting question above the account and the egress indicator above both,
//! because reading what the machine did is never a reason to hide what it is
//! asking or what is leaving it.

use alo_recounting::Recounting;
use alo_strings::Strings;
use smithay::backend::{
    input::{Event, InputEvent, KeyboardKeyEvent},
    winit::WinitEvent,
};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::approval_raster::ApprovalPicture;
use crate::egress_status_raster::EgressStatusPicture;
use crate::nested_egress_status::status_picture;
use crate::record_raster::{RecordPicture, picture};
use crate::scene_native::{NativeLayers, NativeScene};
use crate::{
    ApprovalFrame, EgressStatusFrame, FrameTarget, InputError, Nested, RecordLook, RecordOpened,
    RecordWindow, RenderError, Server, WindowControlLabels,
};

/// What a frame needs to draw the record window.
#[derive(Clone, Copy)]
pub struct RecordFrame<'a> {
    /// The window, as it stands.
    pub window: &'a RecordWindow,
    /// The vocabulary the person reads, which words every clause.
    pub strings: &'a Strings,
    /// Their scheme, text scale and reading direction.
    pub look: RecordLook,
}

impl Nested {
    /// Route the parent window's keys to the record window, in order, and hand
    /// back what each key that read the record again did.
    ///
    /// Keys go through `server`'s seat and are never forwarded to any client.
    /// **Once the window is closed, nothing more is routed**, and while the
    /// parent window does not have the keyboard nothing is routed at all. A
    /// host with a question open routes keys to the approval surface first.
    ///
    /// # Errors
    /// [`RenderError::Closed`] when the parent window closed, and
    /// [`RenderError::Input`] when `server` has no keyboard.
    pub fn pump_record(
        &mut self,
        server: &mut Server,
        window: &mut RecordWindow,
        recounting: &Recounting,
    ) -> Result<Vec<RecordOpened>, RenderError> {
        let mut read = Vec::new();
        let mut failure: Option<InputError> = None;
        let pumped = self.pump_events(|event, focused| {
            if failure.is_some() || !focused || !window.is_open() {
                return;
            }
            let Some(WinitEvent::Input(InputEvent::Keyboard { event })) = event else {
                return;
            };
            let code = u32::from(event.key_code()).saturating_sub(8);
            match server.record_key(code, event.state(), event.time_msec()) {
                Ok(Some(key)) => read.extend(window.pressed(key, recounting)),
                Ok(None) | Err(InputError::InvalidKey) => {}
                Err(error) => failure = Some(error),
            }
        });
        match failure {
            Some(error) => Err(RenderError::Input(error)),
            None => pumped.map(|()| read),
        }
    }

    /// Submit clients, popups and native controls with the record window above
    /// them, a waiting question above the window, the egress indicator above
    /// that, and the cursor on top.
    ///
    /// While the window is closed it draws nothing, and the frame is exactly
    /// the one `Nested::submit_with_approval` or
    /// `Nested::submit_with_egress_status` would submit.
    ///
    /// # Errors
    /// [`RenderError::RecordScene`] when the open window cannot be held whole
    /// on this output; every refusal the indicator and the approval surface
    /// make; and the backend's own submission failures. A refused frame draws
    /// nothing.
    #[expect(
        clippy::too_many_arguments,
        reason = "one session frame: clients, popups, cursor, controls, fonts, the indicator, the record and the question"
    )]
    pub fn submit_with_record(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
        controls: Option<crate::WindowControlScene<'_>>,
        labels: &mut WindowControlLabels,
        egress: EgressStatusFrame<'_>,
        record: RecordFrame<'_>,
        approval: Option<ApprovalFrame<'_>>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        let size = self.size();
        let (status, account, question) =
            frame_pictures(egress, record, approval, labels, (size.w, size.h))?;
        self.submit_native_layers(
            roots,
            popups,
            cursor,
            NativeLayers {
                scene: controls.map(NativeScene::Controls),
                desktop: None,
                record: Some(&account),
                settings: None,
                approval: question.as_ref(),
                status: Some(&status),
            },
        )
    }
}

/// The indicator, the record window and any question for one frame, or the
/// refusal that stops the whole frame — the indicator's first, because a frame
/// that cannot say what is leaving is refused whatever else it holds.
fn frame_pictures(
    egress: EgressStatusFrame<'_>,
    record: RecordFrame<'_>,
    approval: Option<ApprovalFrame<'_>>,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
) -> Result<(EgressStatusPicture, RecordPicture, Option<ApprovalPicture>), RenderError> {
    let status = status_picture(egress, labels, size)?;
    let account = picture(
        record.window.shows(),
        record.strings,
        labels,
        size,
        record.look,
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
    Ok((status, account, question))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::Contrast;
    use crate::record_testing::{an_afternoon_kept, light, words};
    use crate::{EgressStatus, EgressStatusLook};
    use alo_appearance::{Scheme, TextScale};
    use alo_dock::Dock;
    use alo_egress::Indicator;
    use alo_indicator::{Drew, Indicating};
    use alo_strings::Direction;

    /// **A frame carrying the record still carries the indicator**: one whose
    /// indicator was never told is refused whole, record and all; told, the
    /// frame holds both; and a window that cannot hold an entry whole refuses
    /// the frame rather than cutting it.
    #[test]
    fn a_frame_with_the_record_is_refused_whole_without_the_indicator_or_room() {
        let kept = an_afternoon_kept();
        let strings = words();
        let dock = Dock::shipped();
        let mut labels = WindowControlLabels::new().unwrap();
        let egress_look = EgressStatusLook {
            contrast: Contrast::AsDesigned,
            scheme: Scheme::Light,
            scale: TextScale::ordinary(),
            reading: Direction::LeftToRight,
        };
        let mut window = RecordWindow::on_an_output();
        assert_eq!(
            window.opened_by_hand(&kept.recounting()),
            RecordOpened::Shown
        );
        let record = RecordFrame {
            window: &window,
            strings: &strings,
            look: light(),
        };

        let never_told = EgressStatus::on_an_output();
        let egress = EgressStatusFrame {
            status: &never_told,
            strings: &strings,
            dock: &dock,
            look: egress_look,
        };
        assert!(matches!(
            frame_pictures(egress, record, None, &mut labels, (1920, 1080)),
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
        let (indicator, account, question) =
            frame_pictures(egress, record, None, &mut labels, (1920, 1080)).unwrap();
        assert!(indicator.is_empty());
        assert!(!account.is_empty());
        assert!(!account.entries.is_empty());
        assert!(question.is_none());

        assert!(matches!(
            frame_pictures(egress, record, None, &mut labels, (1920, 90)),
            Err(RenderError::RecordScene)
        ));
    }
}
