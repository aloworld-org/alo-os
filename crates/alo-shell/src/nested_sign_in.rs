//! The sign-in screen on the nested compositor: keystrokes in, the screen out.
//!
//! Two calls, used in turn by whoever runs the greeter: [`Nested::pump_sign_in`]
//! takes the parent window's keys through the seat and hands each to the
//! screen, and [`Nested::submit_sign_in`] draws the screen as it now stands.
//! When a key opens a session the pump answers `Signing::HandedOver`, which
//! holds no screen — so there is nothing to pass to the second call, and the
//! greeter's drawing ends in the same step that the session begins.

use alo_greeting::Knocking;
use smithay::backend::{
    input::{Event, InputEvent, KeyboardKeyEvent},
    winit::WinitEvent,
};

use crate::scene_native::NativeScene;
use crate::sign_in_raster::{SignInLook, picture};
use crate::{
    Cursor, FrameTarget, InputError, Nested, RenderError, Server, SignInScreen, Signing,
    WindowControlLabels,
};

impl Nested {
    /// Route the parent window's keys to the sign-in screen, in order.
    ///
    /// Keys go through `server`'s seat — its layout and its modifiers — and are
    /// never forwarded to any client. While the parent window does not have the
    /// keyboard nothing is routed, and every key the seat believed held is let
    /// go, so a Shift held as focus left is not still held when it returns.
    ///
    /// **Once a session is handed over, nothing more is routed**, and the
    /// handover is returned even if the parent window closed in the same pump:
    /// the session is open whatever happens to this window, and losing it here
    /// would leave a person signed in to something nothing shows.
    ///
    /// A key code outside evdev's range is ignored rather than ending the
    /// screen; a person at a sign-in screen is not locked out by a strange key.
    ///
    /// # Errors
    /// [`RenderError::Closed`] when the parent window closed with nobody signed
    /// in, and [`RenderError::Input`] when `server` has no keyboard. The screen
    /// is dropped with either — and what was typed at it zeroed — because the
    /// nested backend cannot be pumped again after either (`Nested`).
    pub fn pump_sign_in<K: Knocking>(
        &mut self,
        server: &mut Server,
        screen: SignInScreen<K>,
    ) -> Result<Signing<K>, RenderError> {
        let mut signing = Some(Signing::Still(Box::new(screen)));
        let mut failure: Option<InputError> = None;
        let pumped = self.pump_events(|event, focused| {
            if failure.is_some() || !matches!(signing, Some(Signing::Still(_))) {
                return;
            }
            if !focused {
                if let Err(error) = server.sign_in_keys_released() {
                    failure = Some(error);
                }
                return;
            }
            let Some(WinitEvent::Input(InputEvent::Keyboard { event })) = event else {
                return;
            };
            let code = u32::from(event.key_code()).saturating_sub(8);
            match server.sign_in_key(code, event.state(), event.time_msec()) {
                Ok(Some(key)) => {
                    signing = signing.take().map(|now| match now {
                        Signing::Still(screen) => (*screen).pressed(key),
                        handed_over @ Signing::HandedOver(_) => handed_over,
                    });
                }
                Ok(None) | Err(InputError::InvalidKey) => {}
                Err(error) => failure = Some(error),
            }
        });
        match signing {
            Some(handed_over @ Signing::HandedOver(_)) => Ok(handed_over),
            Some(still) => match failure {
                Some(error) => Err(RenderError::Input(error)),
                None => pumped.map(|()| still),
            },
            // `signing` is only ever taken and put straight back.
            None => Err(RenderError::Closed),
        }
    }

    /// Draw the sign-in screen as it stands, as the whole output.
    ///
    /// Laid out for the parent window's current size with `labels`' bundled
    /// font, in the person's `look`. No client, popup or control is drawn with
    /// it, and the default cursor is left to the parent.
    ///
    /// # Errors
    /// [`RenderError::SignInScene`] when the window cannot hold the screen,
    /// [`RenderError::Closed`] after the parent window closed, and the
    /// backend's own submission failures.
    pub fn submit_sign_in<K: Knocking>(
        &mut self,
        screen: &SignInScreen<K>,
        labels: &mut WindowControlLabels,
        look: SignInLook,
    ) -> Result<(), RenderError> {
        let size = self.size();
        let drawn = picture(screen.shows(), labels, (size.w, size.h), look)?;
        self.submit_native_scene(
            &[],
            &[],
            &Cursor::Default,
            Some(NativeScene::SignIn(&drawn)),
        )
        .map(|_| ())
    }
}
