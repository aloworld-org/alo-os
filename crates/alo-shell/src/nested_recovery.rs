//! The recovery screen on the nested compositor: keys in, the screen out, and
//! the one frame that draws the desktop or this instead.
//!
//! Three calls. [`Nested::pump_recovery`] takes the parent window's keys
//! through the seat and hands each to the screen;
//! [`Nested::submit_recovery`] draws it as the whole output; and
//! [`Nested::submit_desktop_or_recovery`] is the frame a session actually
//! submits — it composes the ordinary desktop and, when that refuses for any
//! reason, draws the recovery screen in its place and says which refusal sent
//! it there.

use smithay::backend::{
    input::{Event, InputEvent, KeyboardKeyEvent},
    winit::WinitEvent,
};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::nested_desktop::frame_pictures;
use crate::recovery_raster::RecoveryLook;
use crate::recovery_reached::{Composed, Reached, instead_of_the_desktop};
use crate::scene_native::{NativeLayers, NativeScene};
use crate::{
    ApprovalFrame, Cursor, DesktopFrame, FrameTarget, InputError, Nested, RecordFrame,
    RecoveryChosen, RecoveryScreen, RenderError, Server, WindowControlLabels,
};

/// What a frame needs to draw the recovery screen.
#[derive(Clone, Copy)]
pub struct RecoveryFrame<'a> {
    /// The screen, as it stands.
    pub screen: &'a RecoveryScreen,
    /// The person's scheme and text scale.
    pub look: RecoveryLook,
}

impl Nested {
    /// Route the parent window's keys to the recovery screen, in order.
    ///
    /// Keys go through `server`'s seat and are never forwarded to any client.
    /// **Once a moment is chosen, nothing more is routed**, and the choice is
    /// returned even if the parent window closed in the same pump — a person
    /// who asked for their machine back should not have to ask twice because a
    /// window went away afterwards.
    ///
    /// # Errors
    /// [`RenderError::Closed`] when the parent window closed with nothing
    /// chosen, and [`RenderError::Input`] when `server` has no keyboard.
    pub fn pump_recovery(
        &mut self,
        server: &mut Server,
        screen: RecoveryScreen,
    ) -> Result<RecoveryChosen, RenderError> {
        let mut chosen = Some(RecoveryChosen::Still(Box::new(screen)));
        let mut failure: Option<InputError> = None;
        let pumped = self.pump_events(|event, focused| {
            if failure.is_some() || !matches!(chosen, Some(RecoveryChosen::Still(_))) {
                return;
            }
            if !focused {
                return;
            }
            let Some(WinitEvent::Input(InputEvent::Keyboard { event })) = event else {
                return;
            };
            let code = u32::from(event.key_code()).saturating_sub(8);
            match server.recovery_key(code, event.state(), event.time_msec()) {
                Ok(Some(key)) => {
                    chosen = chosen.take().map(|now| match now {
                        RecoveryChosen::Still(screen) => (*screen).pressed(key),
                        went_back @ RecoveryChosen::GoBack { .. } => went_back,
                    });
                }
                Ok(None) | Err(InputError::InvalidKey) => {}
                Err(error) => failure = Some(error),
            }
        });
        match chosen {
            Some(went_back @ RecoveryChosen::GoBack { .. }) => Ok(went_back),
            Some(still) => match failure {
                Some(error) => Err(RenderError::Input(error)),
                None => pumped.map(|()| still),
            },
            // `chosen` is only ever taken and put straight back.
            None => Err(RenderError::Closed),
        }
    }

    /// Draw the recovery screen as the whole output.
    ///
    /// No client, popup or control is drawn with it, and the default cursor is
    /// left to the parent.
    ///
    /// # Errors
    /// [`RenderError::RecoveryScene`] when the window cannot hold the screen
    /// whole, [`RenderError::Closed`] after the parent window closed, and the
    /// backend's own submission failures.
    pub fn submit_recovery(
        &mut self,
        frame: RecoveryFrame<'_>,
        labels: &mut WindowControlLabels,
    ) -> Result<(), RenderError> {
        let size = self.size();
        let drawn = crate::recovery_raster::picture(
            frame.screen.shows(),
            labels,
            (size.w, size.h),
            frame.look,
        )?;
        self.submit_native_scene(
            &[],
            &[],
            &Cursor::Default,
            Some(NativeScene::Recovery(&drawn)),
            None,
        )
        .map(|_| ())
    }

    /// Submit the ordinary desktop, or the recovery screen when the desktop
    /// will not compose.
    ///
    /// Which one was drawn comes back in [`Reached`], with the desktop's own
    /// refusal beside it so that whoever runs the session can write it where a
    /// maintainer reads it.
    ///
    /// # Errors
    /// [`RenderError::RecoveryScene`] when the desktop refused **and** this
    /// output cannot hold the recovery screen; and the backend's own
    /// submission failures, which are not a screen either way.
    #[expect(
        clippy::too_many_arguments,
        reason = "one session frame: clients, popups, cursor, controls, fonts, the desktop, the record and the screen that is drawn when the desktop will not"
    )]
    pub fn submit_desktop_or_recovery(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &Cursor,
        controls: Option<crate::WindowControlScene<'_>>,
        labels: &mut WindowControlLabels,
        desktop: DesktopFrame<'_>,
        record: Option<RecordFrame<'_>>,
        approval: Option<ApprovalFrame<'_>>,
        recovery: RecoveryFrame<'_>,
    ) -> Result<Reached, RenderError> {
        let size = self.size();
        let composed = instead_of_the_desktop(
            frame_pictures(desktop, record, approval, labels, (size.w, size.h)),
            recovery.screen,
            labels,
            (size.w, size.h),
            recovery.look,
        )?;
        match composed {
            Composed::Desktop(pictures) => self
                .submit_native_layers(
                    roots,
                    popups,
                    cursor,
                    NativeLayers {
                        scene: controls.map(NativeScene::Controls),
                        desktop: Some(&pictures.desktop),
                        record: pictures.record.as_ref(),
                        settings: None,
                        approval: pictures.approval.as_ref(),
                        status: Some(&pictures.status),
                        in_use: None,
                    },
                )
                .map(Reached::TheDesktop),
            Composed::Recovery { picture, why } => {
                self.submit_native_scene(
                    &[],
                    &[],
                    &Cursor::Default,
                    Some(NativeScene::Recovery(&picture)),
                    None,
                )?;
                Ok(Reached::Recovery(why))
            }
        }
    }
}
