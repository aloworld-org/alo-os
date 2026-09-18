//! Nested lock input is intercepted by the existing sign-in seat.
use crate::{InputError, LockPressed, LockSurface, Nested, RenderError, Server};
use alo_accounts::Accounts;
use alo_greeting::NotReadable;
use smithay::backend::{
    input::{Event, InputEvent, KeyboardKeyEvent},
    winit::WinitEvent,
};

impl Nested {
    /// Route keyboard events through the existing sign-in seat, never to clients.
    /// Accounts are read anew only for an explicit credential submission.
    /// # Errors
    /// A backend failure is returned alongside the still-locked surface; it never
    /// hands back an open session. Successful authentication wins over window close.
    pub fn pump_lock<N>(
        &mut self,
        server: &mut Server,
        surface: LockSurface<N>,
        mut read: impl FnMut() -> Result<Accounts, NotReadable>,
    ) -> Result<LockPressed<N>, (RenderError, Box<LockSurface<N>>)> {
        let mut state = Some(LockPressed::Still(Box::new(surface)));
        let mut failure = None;
        let pumped = self.pump_events(|event, focused| {
            if failure.is_some() || !matches!(state, Some(LockPressed::Still(_))) {
                return;
            }
            if !focused {
                if let Some(LockPressed::Still(surface)) = state.as_mut() {
                    surface.lost_focus();
                }
                if let Err(error) = server.sign_in_keys_released() {
                    failure = Some(error);
                }
                return;
            }
            let Some(WinitEvent::Input(InputEvent::Keyboard { event })) = event else {
                return;
            };
            match server.sign_in_key(
                u32::from(event.key_code()).saturating_sub(8),
                event.state(),
                event.time_msec(),
            ) {
                Ok(Some(key)) => {
                    state = state.take().map(|state| match state {
                        LockPressed::Still(surface) => (*surface).pressed(key, &mut read),
                        opened => opened,
                    });
                }
                Ok(None) | Err(InputError::InvalidKey) => {}
                Err(error) => failure = Some(error),
            }
        });
        // Every take above puts the state back synchronously.
        match state {
            Some(opened @ LockPressed::Opened { .. }) => Ok(opened),
            Some(LockPressed::Still(surface)) => {
                match failure.map(RenderError::Input).or_else(|| pumped.err()) {
                    Some(error) => Err((error, surface)),
                    None => Ok(LockPressed::Still(surface)),
                }
            }
            None => unreachable!("lock input transition did not retain its state"),
        }
    }
}
