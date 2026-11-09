//! *How much can alo do?* on this machine's own display, before the desktop
//! stands up: keys and presses in, the sheet out, and the desktop after it.
//!
//! The same road as the sign-in screen's (`crate::direct_sign_in`) — this
//! seat's libinput, this card's scanout, one lifetime of
//! `crate::direct_loop::run_with_input` — with the sheet in place of the
//! sign-in screen. It runs **inside the person's own session**, after their
//! first sign-in, because that is the owner's answer of 2026-11-09 and because
//! nothing else can write the file: the sign-in screen runs as root with every
//! capability dropped, and `/var/lib/alo` is the person's and `0700`.
//!
//! **Nothing here decides anything.** What a key or a press means is
//! `crate::how_much_alo_can_do`'s, and what is drawn is
//! `crate::how_much_alo_can_do_sheet`'s.

use std::io;

use smithay::backend::input::ButtonState;

use crate::display_scale::DisplayScale;
use crate::how_much_alo_can_do::{HowMuchAloCanDo, Went};
use crate::how_much_alo_can_do_sheet::HowMuchSheet;
use crate::sign_in_pointer::ThePointerOnTheSignInScreen;
use crate::{
    Cursor, DirectFrame, DirectLoopError, DirectLoopResult, InputError, InputUpdate, RenderError,
    SeatInput, Server, SessionError, WindowControlLabels,
    direct_input_loop::LoopInput,
    scene_native::{NativeLayers, NativeScene},
    sign_in_raster::{Measure, SignInLook, SignInPicture},
};

/// What one showing of the sheet came to.
///
/// Both halves, as [`crate::SignedIn`] keeps them: a level can be written in
/// the same lifetime whose display then fails to retire.
#[must_use = "a level may have been written, and the display's own outcome is separate"]
pub(crate) struct Asked {
    /// What the sheet ended on: [`Went::Stayed`] when the display ended first,
    /// or the sheet could not be drawn on it.
    pub(crate) went: Went,
    /// Whether the sheet could not be drawn on this display at all.
    pub(crate) could_not_draw: bool,
    /// Everything the display lifetime reported.
    pub(crate) display: DirectLoopResult,
}

/// What the sheet needs besides its state.
pub(crate) struct TheSheetsMeans<'a> {
    /// The bundled font every sentence is laid out with.
    pub(crate) labels: &'a mut WindowControlLabels,
    /// The machine's words, in the person's language.
    pub(crate) strings: &'a alo_strings::Strings,
    /// Light or dark, the text scale, the contrast.
    pub(crate) look: SignInLook,
    /// Whether the sealed box can start a program here, as Settings reads it.
    pub(crate) the_sealed_box_runs_here: bool,
}

/// **Ask, if nobody has chosen**, and say on the error stream what came of it.
///
/// The level is read through Settings' own value
/// (`crate::how_much_alo_can_do::HowMuchAloCanDo::to_ask`), from the place
/// Settings is told it is kept. A desktop with no settings places asks nothing,
/// because it has nowhere the answer could be written. Nothing here can take
/// the desktop away: every outcome, the display's own failures among them, ends
/// on the log and the desktop stands up after it.
pub(crate) fn ask_if_nobody_has_chosen(
    session: &mut crate::DirectSession,
    server: &mut Server,
    desktop: &mut dyn crate::TheDesktop,
    labels: &mut WindowControlLabels,
    strings: &alo_strings::Strings,
    next: &mut dyn FnMut() -> DirectFrame,
) {
    let Some(places) = desktop.the_settings_places() else {
        eprintln!("how much can alo do: not asked, this desktop has no settings places");
        return;
    };
    let asking = match HowMuchAloCanDo::to_ask(places.how_code_runs()) {
        crate::how_much_alo_can_do::ToAsk::Ask(asking) => asking,
        crate::how_much_alo_can_do::ToAsk::AlreadyChosen(level) => {
            eprintln!("how much can alo do: not asked, {level:?} was chosen already");
            return;
        }
        crate::how_much_alo_can_do::ToAsk::WouldNotRead => {
            eprintln!(
                "how much can alo do: not asked, {} is there and would not read, and is left as it \
                 is",
                places.how_code_runs().display()
            );
            return;
        }
    };
    // Settings' own reading of whether the box can start a program here.
    let the_sealed_box_runs_here = match std::fs::read_to_string(places.mounts()) {
        Ok(mounts) => !alo_capability::the_sealed_box_here::the_root_is_composefs(&mounts),
        Err(_) => true,
    };
    let look = desktop.now().look;
    let means = TheSheetsMeans {
        labels,
        strings,
        look: SignInLook {
            scheme: look.scheme(),
            scale: look.scale(),
            contrast: look.contrast(),
        },
        the_sealed_box_runs_here,
    };
    match session.ask_how_much_alo_can_do(server, asking, means, next) {
        Ok(crate::ActiveSessionResult { outcome, cleanup }) => {
            match (outcome.went, outcome.could_not_draw) {
                (Went::Kept(level), _) => eprintln!("how much can alo do: {level:?} was chosen"),
                (Went::OnWithNothingChosen, _) => eprintln!(
                    "how much can alo do: the choice could not be written twice, so nothing is \
                     chosen and the next sign-in asks again"
                ),
                (Went::Stayed, true) => eprintln!(
                    "how much can alo do: this display is too small for the sheet, so nothing is \
                     chosen and the next sign-in asks again"
                ),
                (Went::Stayed, false) => eprintln!(
                    "how much can alo do: the display ended before Continue, so nothing is chosen"
                ),
            }
            if let Err(why) = outcome.display.outcome {
                eprintln!("how much can alo do: the sheet's display ended: {why}");
            }
            if let Err(why) = cleanup {
                eprintln!("how much can alo do: the sheet's display did not close cleanly: {why}");
            }
        }
        Err(why) => eprintln!("how much can alo do: the sheet could not take the display: {why}"),
    }
}

impl crate::DirectSession {
    /// Show *How much can alo do?* until the person presses Continue, the
    /// display is taken away, or `next` says stop.
    ///
    /// # Errors
    /// [`SessionError`] when the seat refuses the display. Everything else is
    /// inside [`Asked`].
    pub(crate) fn ask_how_much_alo_can_do(
        &mut self,
        server: &mut Server,
        asking: HowMuchAloCanDo,
        means: TheSheetsMeans<'_>,
        next: &mut dyn FnMut() -> DirectFrame,
    ) -> Result<crate::ActiveSessionResult<Asked>, SessionError> {
        server.clear_input();
        let manager = self.input_session();
        let mut lane = TheSheet::of(asking, means);
        let result = self.with_active_device(|fd, poll| {
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
                    next,
                    Asking {
                        owner,
                        sheet: &mut lane,
                    },
                ),
                Err(error) => DirectLoopResult {
                    outcome: Err(error),
                    input_cleanup: None,
                    input_flush: Some(server.flush()),
                    retirement: None,
                    rest_retirement: Vec::new(),
                    flush: None,
                },
            }
        })?;
        Ok(crate::ActiveSessionResult {
            outcome: Asked {
                went: lane.went,
                could_not_draw: lane.could_not_draw,
                display: result.outcome,
            },
            cleanup: result.cleanup,
        })
    }
}

/// The sheet, and what a press on it is read against.
///
/// Apart from the seat that feeds it, as `crate::direct_sign_in::Lane` is, so
/// what a key does can be checked on a machine with no input devices.
pub(crate) struct TheSheet<'a> {
    /// What is chosen and where the ring is.
    asking: HowMuchAloCanDo,
    /// What drawing it needs.
    means: TheSheetsMeans<'a>,
    /// Where the pointer is, kept here rather than in the seat.
    pointer: ThePointerOnTheSignInScreen,
    /// The size and scale the last frame was drawn at.
    drawn_at: Option<((i32, i32), DisplayScale)>,
    /// The controls the last frame drew, in its pixels; emptied by every key
    /// and press, because each may have changed the sheet.
    controls: Vec<crate::a_press_on_a_setup_sheet::SheetControl>,
    /// What the sheet has come to.
    went: Went,
    /// Whether the display was too small for the sheet.
    could_not_draw: bool,
}

impl<'a> TheSheet<'a> {
    /// The sheet as it arrives.
    pub(crate) fn of(asking: HowMuchAloCanDo, means: TheSheetsMeans<'a>) -> Self {
        Self {
            asking,
            means,
            pointer: ThePointerOnTheSignInScreen::default(),
            drawn_at: None,
            controls: Vec::new(),
            went: Went::Stayed,
            could_not_draw: false,
        }
    }

    /// One key's meaning.
    pub(crate) fn pressed(&mut self, meant: crate::SignInKey) {
        if self.finished() {
            return;
        }
        self.controls.clear();
        self.went = self.asking.pressed(meant);
    }

    /// Move the pointer, once a frame has been drawn for it to move across.
    ///
    /// # Errors
    /// [`InputError`] for a malformed event.
    pub(crate) fn pointed(&mut self, event: &crate::DirectPointerEvent) -> Result<(), InputError> {
        let Some((extent, _)) = self.drawn_at else {
            return Ok(());
        };
        if matches!(event, crate::DirectPointerEvent::Axis(_)) {
            return Ok(());
        }
        self.pointer.moved(event, extent).map(|_| ())
    }

    /// A press wherever the pointer is, on what the last frame drew.
    pub(crate) fn pressed_by_the_pointer(&mut self) {
        if self.finished() {
            return;
        }
        let Some(at) = self.pointer.in_pixels() else {
            return;
        };
        let Some(on) = crate::a_press_on_a_setup_sheet::what_is_under(&self.controls, at) else {
            return;
        };
        self.controls.clear();
        self.went = self.asking.pressed_on(on);
    }

    /// The picture for a display of `size` at `display`.
    ///
    /// # Errors
    /// [`RenderError`] for a display this sheet cannot be laid out on.
    pub(crate) fn drawn_for(
        &mut self,
        size: (i32, i32),
        display: DisplayScale,
    ) -> Result<SignInPicture, RenderError> {
        let colours =
            crate::sign_in_surface::TheColours::of(self.means.look.contrast, self.means.look.scheme);
        let sheet = HowMuchSheet {
            asking: &self.asking,
            the_sealed_box_runs_here: self.means.the_sealed_box_runs_here,
        };
        let scale = self.means.look.scale;
        let strings = self.means.strings;
        let fonts = &mut self.means.labels.fonts;
        let drawn = crate::a_scale_held_to_what_draws::at_the_largest_that_draws(display, |at| {
            crate::how_much_alo_can_do_sheet::drawn(
                &sheet,
                fonts,
                strings,
                size,
                colours,
                &Measure::at(scale, at),
            )
        })?;
        self.drawn_at = Some((size, display));
        self.controls.clone_from(&drawn.controls);
        Ok(drawn)
    }

    /// Lay the sheet out for this display and put it on the card.
    ///
    /// **A display the sheet cannot be laid out on ends the sheet with nothing
    /// chosen** rather than the session: the desktop opens as it would have,
    /// and the next sign-in asks again.
    fn present(&mut self, target: &mut impl crate::presentation::NativeTarget) -> Result<(), RenderError> {
        let size = target.size();
        let display = target.metadata().map_or(DisplayScale::ONE, |metadata| {
            crate::the_sign_in_screens_scale::the_sign_in_screens_scale(
                &metadata.name,
                (size.w, size.h),
                metadata.physical_size,
            )
        });
        let drawn = match self.drawn_for((size.w, size.h), display) {
            Ok(drawn) => drawn,
            Err(RenderError::SignInScene) => {
                self.could_not_draw = true;
                return Ok(());
            }
            Err(other) => return Err(other),
        };
        let cursor = match self.pointer.where_it_is() {
            Some(location) => Cursor::Arrow { location },
            None => Cursor::Default,
        };
        target.submit_native_layers(
            &[],
            &[],
            &cursor,
            NativeLayers {
                scene: Some(NativeScene::SignIn(&drawn)),
                pointer_at: display,
                ..NativeLayers::nothing()
            },
        )?;
        Ok(())
    }

    /// Whether the sheet has nothing left to show.
    pub(crate) fn finished(&self) -> bool {
        self.could_not_draw || !matches!(self.went, Went::Stayed)
    }

    /// The asking as it stands, for a check.
    #[cfg(test)]
    pub(crate) fn asking(&self) -> &HowMuchAloCanDo {
        &self.asking
    }

    /// What the sheet came to, for a check.
    #[cfg(test)]
    pub(crate) fn went(&self) -> Went {
        self.went
    }
}

/// The sheet's lane: this seat's keys to the sheet, and the sheet to the card.
struct Asking<'s, 'a> {
    /// Same-seat libinput context, consumed at loop exit.
    owner: SeatInput,
    /// The sheet itself.
    sheet: &'s mut TheSheet<'a>,
}

impl LoopInput for Asking<'_, '_> {
    /// Every key and every left press, to the sheet, in order. A device going
    /// away lets go of every key and of the pointer's position.
    fn dispatch(
        &mut self,
        server: &mut Server,
        poll: &mut dyn FnMut() -> Result<(), SessionError>,
    ) -> Result<(), DirectLoopError> {
        let sheet = &mut *self.sheet;
        self.owner.dispatch(
            || poll().map_err(io::Error::other),
            |update| {
                let InputUpdate::Event(event) = update else {
                    server.clear_input();
                    return Ok(());
                };
                match crate::libinput_routing::translate(event).map_err(io::Error::other)? {
                    Some(crate::DirectSeatEvent::Key(key, _)) => {
                        match server.sign_in_key(key.code, key.state, key.time) {
                            Ok(Some(meant)) => {
                                sheet.pressed(meant);
                                Ok(())
                            }
                            Ok(None) | Err(InputError::InvalidKey) => Ok(()),
                            Err(error) => Err(io::Error::other(error)),
                        }
                    }
                    Some(crate::DirectSeatEvent::Pointer(moved)) => match sheet.pointed(&moved) {
                        Ok(()) | Err(InputError::InvalidPointer) => Ok(()),
                        Err(error) => Err(io::Error::other(error)),
                    },
                    Some(crate::DirectSeatEvent::Button { code, state, .. }) => {
                        // `BTN_LEFT`; nothing else on this sheet means anything.
                        if code == 0x110 && state == ButtonState::Pressed {
                            sheet.pressed_by_the_pointer();
                        }
                        Ok(())
                    }
                    Some(crate::DirectSeatEvent::Removed) => {
                        sheet.pointer = ThePointerOnTheSignInScreen::default();
                        server.clear_input();
                        Ok(())
                    }
                    None => Ok(()),
                }
            },
        )?;
        Ok(())
    }

    /// Draw the sheet, and nothing of any client's.
    fn present<T: crate::direct_loop::LoopTarget + crate::presentation::NativeTarget>(
        &mut self,
        _server: &mut Server,
        target: &mut T,
        _time: u32,
    ) -> Result<(), DirectLoopError> {
        self.sheet.present(target)?;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.sheet.finished()
    }

    fn shutdown(self, server: &mut Server) -> Option<io::Result<()>> {
        Some(self.owner.shutdown(|| server.clear_input()))
    }
}

#[cfg(test)]
#[path = "direct_how_much_alo_can_do_tests.rs"]
mod tests;
