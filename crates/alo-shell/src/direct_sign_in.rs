//! The sign-in screen on this machine's own display: keys in, the screen out,
//! and a session at the end of it.
//!
//! The nested twin of this file is `crate::nested_sign_in`, which shows the
//! same screen inside somebody else's compositor. The difference between them
//! is only where the keys come from and where the pixels go: a parent window's
//! events and a parent's EGL there, this seat's libinput and this card's
//! scanout here. The screen, what it shows and what a keystroke means to it are
//! `alo-greeting`'s and `crate::sign_in_screen`'s, both times.
//!
//! **Nothing here is a second display lifetime.** Polling the seat, pausing,
//! retiring the output and flushing are `crate::direct_loop`'s, exactly as they
//! are for a session — this is a different driver on that one loop, not a loop
//! of its own.

use std::io;

use alo_greeting::Knocking;

use crate::{
    Cursor, DirectFrame, DirectLoopError, DirectLoopResult, InputError, InputUpdate, RenderError,
    SeatInput, Server, SessionError, SignInScreen, Signing, WindowControlLabels,
    direct_input_loop::LoopInput,
    scene_native::{NativeLayers, NativeScene},
    sign_in_raster::{SignInLook, picture},
};

/// What one sign-in lifetime came to.
///
/// Both halves are reported, never one instead of the other: a session can open
/// in the same lifetime whose display then fails to retire, and a machine that
/// reported only the failure would have signed somebody in and not said so.
#[must_use = "a session may have opened, and the display's own outcome is separate"]
pub struct SignedIn<K: Knocking> {
    /// The session that opened, or the screen as it stood when the display ended.
    pub signing: Signing<K>,
    /// Everything `DirectSession::run_compositor_with_input` reports about the
    /// display: the loop's outcome, input cleanup, both flushes and retirement.
    pub display: DirectLoopResult,
}

impl crate::DirectSession {
    /// Show the sign-in screen on this machine's display until somebody signs in.
    ///
    /// The one call a machine booting to a compositor makes. It takes the
    /// display through libseat, finds the connector, mode and CRTC on it, takes
    /// the seat's input devices, and then draws the screen for every frame
    /// `next` asks for until a key opens a session — at which point it draws no
    /// more, because the next frame would be a password field in front of
    /// somebody already signed in.
    ///
    /// Every key goes through `Server::sign_in_key`, which intercepts it at the
    /// seat and forwards it to no client; a pointer event moves nothing,
    /// because there is nothing on this screen to point at. `next` is trusted
    /// pacing, exactly as in `run_compositor`, and returning `Stop` from it
    /// ends the screen with nobody signed in.
    ///
    /// # Errors
    /// [`SessionError`] when the seat refuses the display or the device cannot
    /// be acquired; everything else is reported inside [`SignedIn`] rather than
    /// thrown away, because the session that opened is worth more than the
    /// reason the display afterwards did not close cleanly.
    pub fn sign_in<K: Knocking>(
        &mut self,
        server: &mut Server,
        screen: SignInScreen<K>,
        labels: &mut WindowControlLabels,
        look: SignInLook,
        mut next: impl FnMut() -> DirectFrame,
    ) -> Result<crate::ActiveSessionResult<SignedIn<K>>, SessionError> {
        server.clear_input();
        let manager = self.input_session();
        let mut signing = Some(Signing::Still(Box::new(screen)));
        let display = self.with_active_device(|fd, poll| {
            let setup = (|| {
                let output = crate::discover_atomic_output(fd)?;
                poll()?;
                let painter = crate::software_scanout::SoftwarePainter::new()?;
                Ok::<_, DirectLoopError>((output, painter, SeatInput::new(manager)?))
            })();
            match setup {
                Ok((output, painter, owner)) => crate::direct_loop::run_with_input(
                    server,
                    crate::direct_target::Target::new(
                        painter,
                        crate::drm_inventory::Inventory(fd),
                        output,
                    ),
                    poll,
                    &mut next,
                    Greeter {
                        owner,
                        screen: Lane::of(&mut signing, labels, look),
                    },
                ),
                Err(error) => DirectLoopResult {
                    outcome: Err(error),
                    input_cleanup: None,
                    input_flush: Some(server.flush()),
                    retirement: None,
                    flush: None,
                },
            }
        });
        // The screen is only ever taken out of this cell and put straight back
        // in the same expression, so it is absent only if the machine stopped
        // between the two. That cannot happen, and saying so with an error
        // rather than a panic is the difference between a machine that refuses
        // to sign somebody in and one that dies at the sign-in screen.
        let Some(signing) = signing else {
            return Err(SessionError::Backend {
                stage: "keep the sign-in screen across a keystroke",
                source: io::Error::other("the screen was not put back"),
            });
        };
        display.map(
            |crate::ActiveSessionResult { outcome, cleanup }| crate::ActiveSessionResult {
                outcome: SignedIn {
                    signing,
                    display: outcome,
                },
                cleanup,
            },
        )
    }
}

/// The sign-in lane: this seat's keys to one screen, and that screen to the card.
struct Greeter<'a, K: Knocking> {
    /// Same-seat libinput context, consumed at loop exit.
    owner: SeatInput,
    /// Everything about the screen itself, which needs no input devices to
    /// exercise — and so can be checked on a machine that has none.
    screen: Lane<'a, K>,
}

/// The screen a machine boots to, and what becomes of it.
///
/// Separate from the seat that feeds it because they fail in different places
/// and are worth checking apart: a libinput context needs a login seat and real
/// devices, and what a keystroke does to the screen needs neither.
pub(crate) struct Lane<'a, K: Knocking> {
    /// The caller's own screen, so a handover survives this driver being dropped.
    signing: &'a mut Option<Signing<K>>,
    /// The bundled font every sentence on the screen is laid out with.
    labels: &'a mut WindowControlLabels,
    /// Light or dark, the scale, and whether high contrast is on.
    look: SignInLook,
}

impl<'a, K: Knocking> Lane<'a, K> {
    /// Take a screen, the font its sentences are laid out with, and its look.
    pub(crate) fn of(
        signing: &'a mut Option<Signing<K>>,
        labels: &'a mut WindowControlLabels,
        look: SignInLook,
    ) -> Self {
        Self {
            signing,
            labels,
            look,
        }
    }

    /// Hand one key's meaning to the screen, and keep whatever it becomes.
    pub(crate) fn pressed(&mut self, meant: crate::SignInKey) {
        *self.signing = self.signing.take().map(|now| match now {
            Signing::Still(screen) => (*screen).pressed(meant),
            handed_over @ Signing::HandedOver(_) => handed_over,
        });
    }

    /// Lay the screen out for this display and put it on the card.
    ///
    /// Submission goes straight to the target rather than through
    /// `Server::render`, as the nested screen's does: there is no client to
    /// import, no output membership to publish and no frame callback to
    /// complete, because nobody is connected to this compositor yet.
    pub(crate) fn present(
        &mut self,
        target: &mut impl crate::presentation::NativeTarget,
    ) -> Result<(), RenderError> {
        let Some(Signing::Still(screen)) = self.signing.as_ref() else {
            return Ok(());
        };
        let size = target.size();
        let drawn = picture(screen.shows(), self.labels, (size.w, size.h), self.look)?;
        target.submit_native_layers(
            &[],
            &[],
            &Cursor::Default,
            NativeLayers {
                scene: Some(NativeScene::SignIn(&drawn)),
                ..NativeLayers::nothing()
            },
        )?;
        Ok(())
    }

    /// A session opened, so this screen has nothing left to show.
    pub(crate) fn finished(&self) -> bool {
        !matches!(self.signing, Some(Signing::Still(_)))
    }
}

impl<K: Knocking> LoopInput for Greeter<'_, K> {
    /// Hand every key to the screen, in order, and ignore everything else.
    ///
    /// A pointer has nowhere to go: no client is mapped and the screen has no
    /// control a click reaches. A device going away resets the seat, which lets
    /// go of every key the screen believed held — a Shift still down after its
    /// keyboard was unplugged would capitalise everything typed on the next one.
    fn dispatch(
        &mut self,
        server: &mut Server,
        poll: &mut dyn FnMut() -> Result<(), SessionError>,
    ) -> Result<(), DirectLoopError> {
        let screen = &mut self.screen;
        self.owner.dispatch(
            || poll().map_err(io::Error::other),
            |update| {
                let InputUpdate::Event(event) = update else {
                    server.clear_input();
                    return Ok(());
                };
                let Some(crate::DirectSeatEvent::Key(key, _)) =
                    crate::libinput_routing::translate(event).map_err(io::Error::other)?
                else {
                    return Ok(());
                };
                match server.sign_in_key(key.code, key.state, key.time) {
                    Ok(Some(meant)) => {
                        screen.pressed(meant);
                        Ok(())
                    }
                    // A release, a repeat, and a key this keyboard has a code
                    // for that evdev does not: none of them is a reason to take
                    // the sign-in screen away from somebody.
                    Ok(None) | Err(InputError::InvalidKey) => Ok(()),
                    Err(error) => Err(io::Error::other(error)),
                }
            },
        )?;
        Ok(())
    }

    /// Draw the screen, and nothing of any client's.
    fn present<T: crate::direct_loop::LoopTarget + crate::presentation::NativeTarget>(
        &mut self,
        _server: &mut Server,
        target: &mut T,
        _time: u32,
    ) -> Result<(), DirectLoopError> {
        self.screen.present(target)?;
        Ok(())
    }

    /// A session opened, so this screen has nothing left to show.
    fn finished(&self) -> bool {
        self.screen.finished()
    }

    fn shutdown(self, server: &mut Server) -> Option<io::Result<()>> {
        Some(self.owner.shutdown(|| server.clear_input()))
    }
}
