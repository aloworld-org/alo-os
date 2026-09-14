//! The approval surface on the nested compositor: keys in, the question out.
//!
//! Two calls, used in turn by whoever runs the session while a question is
//! open: [`Nested::pump_approval`] takes the parent window's keys through the
//! seat and hands each to the screen, and [`Nested::submit_with_approval`]
//! draws the session's frame with the question above every client — and with
//! the egress indicator above the question, because a frame that could carry a
//! question and not what is leaving would be a frame in which something leaves
//! silently.

use std::time::SystemTime;

use alo_capability::Grants;
use alo_strings::Strings;
use alo_turn::Turning;
use smithay::backend::{
    input::{Event, InputEvent, KeyboardKeyEvent},
    winit::WinitEvent,
};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::approval_raster::{ApprovalLook, ApprovalPicture, picture};
use crate::nested_egress_status::status_picture;
use crate::scene_native::{NativeLayers, NativeScene};
use crate::{
    ApprovalOutcome, ApprovalScreen, EgressStatusFrame, FrameTarget, InputError, Nested,
    RenderError, Server, WindowControlLabels,
};

/// What a frame needs to draw the approval surface.
#[derive(Clone, Copy)]
pub struct ApprovalFrame<'a> {
    /// The surface, as it stands.
    pub screen: &'a ApprovalScreen,
    /// The vocabulary the person reads, which words the two answers.
    pub strings: &'a Strings,
    /// Their scheme, text scale and reading direction.
    pub look: ApprovalLook,
}

impl Nested {
    /// Route the parent window's keys to the approval surface, in order, and
    /// hand back what each key that did something did.
    ///
    /// Keys go through `server`'s seat and are never forwarded to any client.
    /// **Once nothing is open, nothing more is routed** — the keys left in this
    /// pump answer nothing and reach no window — and while the parent window
    /// does not have the keyboard, nothing is routed at all.
    ///
    /// # Errors
    /// [`RenderError::Closed`] when the parent window closed, with every
    /// outcome up to then lost to the caller but not to the record, which the
    /// turn wrote before anything came back; and [`RenderError::Input`] when
    /// `server` has no keyboard.
    pub fn pump_approval(
        &mut self,
        server: &mut Server,
        screen: &mut ApprovalScreen,
        turning: &mut Turning<'_, '_>,
        grants: &Grants,
        now: SystemTime,
    ) -> Result<Vec<ApprovalOutcome>, RenderError> {
        let mut outcomes = Vec::new();
        let mut failure: Option<InputError> = None;
        let pumped = self.pump_events(|event, focused| {
            if failure.is_some() || !focused || !screen.is_open() {
                return;
            }
            let Some(WinitEvent::Input(InputEvent::Keyboard { event })) = event else {
                return;
            };
            let code = u32::from(event.key_code()).saturating_sub(8);
            match server.approval_key(code, event.state(), event.time_msec()) {
                Ok(Some(key)) => {
                    let outcome = screen.pressed(key, turning, grants, now);
                    if outcome != ApprovalOutcome::default() {
                        outcomes.push(outcome);
                    }
                }
                Ok(None) | Err(InputError::InvalidKey) => {}
                Err(error) => failure = Some(error),
            }
        });
        match failure {
            Some(error) => Err(RenderError::Input(error)),
            None => pumped.map(|()| outcomes),
        }
    }

    /// Submit clients, popups and native controls with the approval surface
    /// above them, the egress indicator above that, and the cursor on top.
    ///
    /// While nothing is open the surface draws nothing and the frame is exactly
    /// the one `Nested::submit_with_egress_status` would submit.
    ///
    /// # Errors
    /// [`RenderError::ApprovalScene`] when the open question or refusal cannot
    /// be held whole on this window; every refusal
    /// `Nested::submit_with_egress_status` makes about the indicator; and the
    /// backend's own submission failures. A refused frame draws nothing.
    #[expect(
        clippy::too_many_arguments,
        reason = "one session frame: clients, popups, cursor, controls, fonts, the indicator and the question"
    )]
    pub fn submit_with_approval(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
        controls: Option<crate::WindowControlScene<'_>>,
        labels: &mut WindowControlLabels,
        egress: EgressStatusFrame<'_>,
        approval: ApprovalFrame<'_>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        let size = self.size();
        let (status, question) = frame_pictures(egress, approval, labels, (size.w, size.h))?;
        self.submit_native_layers(
            roots,
            popups,
            cursor,
            NativeLayers {
                scene: controls.map(NativeScene::Controls),
                desktop: None,
                record: None,
                approval: Some(&question),
                status: Some(&status),
            },
        )
    }
}

/// The indicator and the question for one frame, or the refusal that stops the
/// whole frame — the indicator's first, because a frame that cannot say what is
/// leaving is refused whatever else it holds.
fn frame_pictures(
    egress: EgressStatusFrame<'_>,
    approval: ApprovalFrame<'_>,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
) -> Result<
    (
        crate::egress_status_raster::EgressStatusPicture,
        ApprovalPicture,
    ),
    RenderError,
> {
    let status = status_picture(egress, labels, size)?;
    let question = picture(
        approval.screen.shows(),
        approval.strings,
        labels,
        size,
        approval.look,
    )?;
    Ok((status, question))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::approval_testing::{archiving, noon, on_a_machine, words};
    use crate::{EgressStatus, EgressStatusLook};
    use alo_appearance::{Scheme, TextScale};
    use alo_dock::Dock;
    use alo_egress::Indicator;
    use alo_indicator::{Drew, Indicating};
    use alo_strings::Direction;

    /// The ordinary light look, read left to right.
    fn looks() -> (EgressStatusLook, ApprovalLook) {
        (
            EgressStatusLook {
                scheme: Scheme::Light,
                scale: TextScale::ordinary(),
                reading: Direction::LeftToRight,
            },
            ApprovalLook {
                scheme: Scheme::Light,
                scale: TextScale::ordinary(),
                reading: Direction::LeftToRight,
            },
        )
    }

    /// **A frame carrying a question still carries the indicator**: one whose
    /// indicator was never told is refused whole, question and all; told, the
    /// frame holds both; and a question that does not fit refuses the frame
    /// rather than cutting the sentence.
    #[test]
    fn a_frame_with_a_question_is_refused_whole_without_the_indicator_or_room() {
        on_a_machine(|turning, grants, places| {
            let strings = words();
            let dock = Dock::shipped();
            let mut labels = WindowControlLabels::new().unwrap();
            let (egress_look, approval_look) = looks();
            let march = archiving(turning, grants, places, "march");
            let mut screen = ApprovalScreen::on_an_output();
            screen.arrived(turning, march, noon());
            let approval = ApprovalFrame {
                screen: &screen,
                strings: &strings,
                look: approval_look,
            };

            let never_told = EgressStatus::on_an_output();
            let egress = EgressStatusFrame {
                status: &never_told,
                strings: &strings,
                dock: &dock,
                look: egress_look,
            };
            assert!(matches!(
                frame_pictures(egress, approval, &mut labels, (1920, 1080)),
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
            let (indicator, question) =
                frame_pictures(egress, approval, &mut labels, (1920, 1080)).unwrap();
            assert!(indicator.is_empty());
            assert!(!question.is_empty());
            assert_eq!(question.answers.len(), 2);

            assert!(matches!(
                frame_pictures(egress, approval, &mut labels, (1920, 90)),
                Err(RenderError::ApprovalScene)
            ));
        });
    }
}
