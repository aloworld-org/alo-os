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
use smithay::backend::input::ButtonState;
use smithay::utils::Size;

use crate::display_scale::DisplayScale;
use crate::sign_in_pointer::ThePointerOnTheSignInScreen;
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
                        chose:
                            crate::the_keyboard_a_person_chose::TheKeyboardAPersonChose::nothing_yet(),
                        language:
                            crate::the_language_a_person_chose::TheLanguageAPersonChose::nothing_yet(),
                        region:
                            crate::the_region_a_person_chose::TheRegionAPersonChose::nothing_yet(),
                        zone: crate::the_zone_a_person_chose::TheZoneAPersonChose::nothing_yet(),
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
    /// The keyboard a person picked on the language sheet, and whether it
    /// still needs writing down and applying.
    ///
    /// Here rather than in `Lane` because applying it needs the `Server` and
    /// `Lane` deliberately has none -- that is the whole reason a keystroke
    /// can be checked on a machine with no input devices.
    chose: crate::the_keyboard_a_person_chose::TheKeyboardAPersonChose,
    /// The language a person picked on the language sheet, and whether it
    /// still needs writing down as the language this machine was set up in.
    language: crate::the_language_a_person_chose::TheLanguageAPersonChose,
    /// The region a person took on the region sheet, and whether it still
    /// needs writing down as the region this machine is in.
    region: crate::the_region_a_person_chose::TheRegionAPersonChose,
    /// The time zone the region a person took sets, and whether it is set yet.
    zone: crate::the_zone_a_person_chose::TheZoneAPersonChose,
}

/// Set the time zone the region a person took decided, if it just decided
/// one (ADR 0140). Every outcome ends on the log, as the region's does.
fn the_zone_is_now<K: Knocking>(
    zone: &mut crate::the_zone_a_person_chose::TheZoneAPersonChose,
    screen: &Lane<'_, K>,
) {
    use crate::the_zone_a_person_chose::AboutTheZone;
    match zone.chose(
        screen.the_zone_chosen(),
        std::path::Path::new(alo_formats::the_machines_zone::THE_LOCALTIME),
        std::path::Path::new(alo_formats::the_machines_zone::THE_ZONEINFO),
    ) {
        AboutTheZone::Unchanged => {}
        AboutTheZone::Set(zone) => eprintln!(
            "alo-compositor: this machine's time zone is now {zone}, from the region chosen"
        ),
        AboutTheZone::NotSet(why) => eprintln!(
            "alo-compositor: a time zone was chosen and could not be set, so the machine keeps \
             the one it had: {why}"
        ),
    }
}

/// Keep the region a person took as the region this machine is in, if they
/// just took one. Every outcome ends on the log, as the language's does.
fn the_region_is_now<K: Knocking>(
    region: &mut crate::the_region_a_person_chose::TheRegionAPersonChose,
    screen: &Lane<'_, K>,
) {
    use crate::the_region_a_person_chose::AboutTheRegion;
    match region.chose(
        screen.the_region_chosen(),
        std::path::Path::new(alo_formats::set_up_in::THE_MACHINES_SET_UP_IN),
    ) {
        AboutTheRegion::Unchanged => {}
        AboutTheRegion::Kept(code) => {
            eprintln!("alo-compositor: this machine is in {code}, chosen on the region sheet")
        }
        AboutTheRegion::NotKept(why) => eprintln!(
            "alo-compositor: a region was chosen and could not be written down, so dates on \
             this machine follow its time zone or its language instead: {why}"
        ),
    }
}

/// Keep the language a person chose as the language this machine was set up
/// in, if they just chose one, so their desktop can give it to them.
///
/// Every outcome ends on the log, for the reason [`the_keyboard_is_now`]
/// gives: the screen already speaks the language, and a sentence about a file
/// in front of them would be this program's problem printed where their name
/// goes.
fn the_language_is_now<K: Knocking>(
    language: &mut crate::the_language_a_person_chose::TheLanguageAPersonChose,
    screen: &Lane<'_, K>,
) {
    use crate::the_language_a_person_chose::AboutTheLanguage;
    match language.chose(
        screen.the_language_chosen(),
        std::path::Path::new(alo_formats::set_up_in::THE_MACHINES_SET_UP_IN),
    ) {
        AboutTheLanguage::Unchanged => {}
        AboutTheLanguage::Kept(tag) => eprintln!(
            "alo-compositor: this machine was set up in {tag}, chosen on the language sheet"
        ),
        AboutTheLanguage::NotKept(why) => eprintln!(
            "alo-compositor: a language was chosen and could not be written down, so the \
             person's desktop will not be given it: {why}"
        ),
    }
}

/// Keep and apply the keyboard a person chose, if they just chose one.
///
/// **A free function rather than a method**, because it is the one place the
/// three halves meet: the screen that knows the choice, the file that keeps
/// it, and the `Server` that holds the keymap. Each of them is somebody
/// else's, and a method on any of the three would have made that one the owner
/// of the other two.
///
/// Every outcome ends on the log rather than on the screen. A person at a
/// language sheet whose keyboard would not change has a working machine and a
/// wrong layout; a sentence about `keyboards.toml` in front of them would be
/// this program's problem printed where their name goes.
fn the_keyboard_is_now<K: Knocking>(
    chose: &mut crate::the_keyboard_a_person_chose::TheKeyboardAPersonChose,
    screen: &Lane<'_, K>,
    server: &mut Server,
) {
    use crate::the_keyboard_a_person_chose::AboutTheKeyboard;
    match chose.chose(
        screen.the_keyboard_chosen().as_ref(),
        std::path::Path::new(alo_keyboards::THE_MACHINES_KEYBOARDS),
    ) {
        AboutTheKeyboard::Unchanged => {}
        AboutTheKeyboard::Apply(layout) => {
            let config = smithay::input::keyboard::XkbConfig {
                layout: layout.name(),
                variant: layout.variant().unwrap_or_default(),
                ..smithay::input::keyboard::XkbConfig::default()
            };
            match server.the_keyboard_is_now(config) {
                Ok(()) => eprintln!(
                    "alo-compositor: the keyboard is now {}, chosen on the language sheet",
                    layout
                ),
                // **The file is already written, and that is the right way
                // round.** The next boot reads it and comes up in the layout
                // the person asked for, so a keymap that would not rebuild
                // now costs them this session rather than the choice.
                Err(why) => eprintln!(
                    "alo-compositor: {} was chosen and the keymap would not take it: {why}",
                    layout
                ),
            }
        }
        AboutTheKeyboard::NotKept(why) => eprintln!(
            "alo-compositor: a keyboard was chosen and could not be written down, so it \
             was not applied: {why}"
        ),
    }
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
    /// Where the pointer is, kept here and never in the seat's client-facing
    /// one. `crate::sign_in_pointer`'s header says why.
    pointer: ThePointerOnTheSignInScreen,
    /// The size the last frame was drawn at and the scale it was drawn at, or
    /// [`None`] before the first one.
    ///
    /// **A press is hit-tested against the size that was drawn, not against
    /// the display's mode.** They are the same number on this machine, and
    /// keeping one of them is how they stay the same: two layouts mean a press
    /// landing where no control was painted, which is the fault
    /// `crate::a_press_in_the_menu` records in its own header.
    ///
    /// **And at the scale that was drawn**, for the same reason: a press
    /// arrives in pixels and the layout is logical, so the one conversion
    /// between them has to be the one the frame used.
    ///
    /// Nothing before the first draw, which is `crate::direct_desktop`'s rule
    /// for its panel and true here for the same reason: a pointer moving on a
    /// machine that has not painted is an ordinary moment, not something to
    /// invent an answer for.
    drawn_at: Option<((i32, i32), DisplayScale)>,
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
            pointer: ThePointerOnTheSignInScreen::default(),
            drawn_at: None,
        }
    }

    /// Move the pointer, if a frame has been drawn for it to move across.
    ///
    /// # Errors
    /// [`InputError`] for a malformed event, which the caller reports rather
    /// than taking the screen away from somebody over.
    pub(crate) fn pointed(&mut self, event: &crate::DirectPointerEvent) -> Result<(), InputError> {
        let Some((extent, _)) = self.drawn_at else {
            return Ok(());
        };
        self.pointer.moved(event, extent)?;
        Ok(())
    }

    /// Act on a press wherever the pointer is.
    ///
    /// Answers whether anything was pressed, which is what lets the caller
    /// tell a click on a control from one on the screen's ground.
    pub(crate) fn pressed_by_the_pointer(&mut self) -> bool {
        let Some((extent, display)) = self.drawn_at else {
            return false;
        };
        // The layout the frame was drawn from: logical, the drawn pixels
        // through the scale that frame used.
        let logical = (
            i32::try_from(display.logical(extent.0)).unwrap_or(i32::MAX),
            i32::try_from(display.logical(extent.1)).unwrap_or(i32::MAX),
        );
        let Some(at) = crate::sign_in_layout::sign_in_laid_out(Size::from(logical)) else {
            return false;
        };
        let Some(on) = self.pointer.pressed(&at, display) else {
            return false;
        };
        // **A control with nothing to do is still a control that was pressed.**
        // The press is consumed either way: `crate::a_press_on_the_sign_in_screen`
        // holds the table of which ones act, and the power target is in it
        // precisely so that it does nothing.
        if let Some(meant) = crate::a_press_on_the_sign_in_screen::what_a_press_does(on) {
            self.pressed(meant);
        }
        true
    }

    /// The time zone the region a person took sets, or [`None`].
    pub(crate) fn the_zone_chosen(&self) -> Option<&str> {
        match self.signing.as_ref() {
            Some(Signing::Still(screen)) => screen.the_zone_chosen(),
            Some(Signing::HandedOver(_)) | None => None,
        }
    }

    /// The region a person took on the region sheet, or [`None`].
    pub(crate) fn the_region_chosen(&self) -> Option<&str> {
        match self.signing.as_ref() {
            Some(Signing::Still(screen)) => screen.the_region_chosen(),
            Some(Signing::HandedOver(_)) | None => None,
        }
    }

    /// The keyboard to type on now, chosen on the first-start sheets, or
    /// [`None`].
    pub(crate) fn the_keyboard_chosen(&self) -> Option<alo_keyboards::Layout> {
        match self.signing.as_ref() {
            Some(Signing::Still(screen)) => screen.the_keyboard_chosen(),
            Some(Signing::HandedOver(_)) | None => None,
        }
    }

    /// The language a person chose on the sheet, or [`None`].
    pub(crate) fn the_language_chosen(&self) -> Option<&str> {
        match self.signing.as_ref() {
            Some(Signing::Still(screen)) => screen.the_language_chosen(),
            Some(Signing::HandedOver(_)) | None => None,
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
        // **The screen's own scale**, worked out from its pixels and its
        // glass, because nobody has signed in to have a setting of their own.
        // A display that cannot describe itself is one that cannot be worked
        // out, and is drawn one-to-one: what `Scale::worked_out_for` answers
        // for a screen that gives no size.
        let display = target.metadata().map_or(DisplayScale::ONE, |metadata| {
            crate::the_sign_in_screens_scale::the_sign_in_screens_scale(
                &metadata.name,
                (size.w, size.h),
                metadata.physical_size,
            )
        });
        let drawn = picture(
            screen.shows(),
            self.labels,
            screen.words(),
            (size.w, size.h),
            self.look,
            crate::sign_in_raster::WhoseScreen::ItsOwn,
            display,
        )?;
        self.drawn_at = Some(((size.w, size.h), display));
        // **An arrow only once a device has moved**, so a machine with no
        // mouse draws no cursor rather than one parked in a corner that
        // nothing can move.
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
                ..NativeLayers::nothing()
            },
        )?;
        Ok(())
    }

    /// A session opened, so this screen has nothing left to show.
    pub(crate) fn finished(&self) -> bool {
        !matches!(self.signing, Some(Signing::Still(_)))
    }

    /// A device went away, so the pointer has no position any more.
    ///
    /// The same rule the seat applies to a held Shift whose keyboard was
    /// unplugged: what a removed device said is no longer true. An arrow left
    /// on the screen after its mouse was pulled out is a target a person can
    /// see and cannot move, and a press arriving from somewhere else would
    /// land on whatever it was last over.
    pub(crate) fn the_pointer_went_away(&mut self) {
        self.pointer = ThePointerOnTheSignInScreen::default();
    }
}

impl<K: Knocking> LoopInput for Greeter<'_, K> {
    /// Hand every key and every press to the screen, in order.
    ///
    /// **The pointer reaches the screen and no client.** It did not until
    /// 2026-10-12, when this comment still said *a pointer has nowhere to go*
    /// — true while the screen had no control a click could reach, and false
    /// from the moment the design put a submit arrow inside the entry.
    /// Motion and presses go through `crate::sign_in_pointer`, which is the
    /// screen's own pointer: `Server::pointer_motion` is the seat's, and that
    /// one offers a position to a window move, a resize band and whatever
    /// surface it hit-tests to, none of which exists before a session.
    ///
    /// **Only the left button acts.** A right or middle click on a sign-in
    /// screen opens nothing, so it is translated, validated and then does
    /// nothing, rather than being made to mean something.
    ///
    /// A device going away resets the seat, which lets go of every key the
    /// screen believed held — a Shift still down after its keyboard was
    /// unplugged would capitalise everything typed on the next one — and
    /// takes the pointer's position with it, for the same reason.
    fn dispatch(
        &mut self,
        server: &mut Server,
        poll: &mut dyn FnMut() -> Result<(), SessionError>,
    ) -> Result<(), DirectLoopError> {
        let screen = &mut self.screen;
        // Borrowed beside the screen, because the closure below takes both and
        // `self` cannot be lent twice into it.
        let chose = &mut self.chose;
        let language = &mut self.language;
        let region = &mut self.region;
        let zone = &mut self.zone;
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
                                screen.pressed(meant);
                                // **The keyboard a person just chose, applied
                                // before their next key.** This is the whole
                                // point of the language sheet coming before
                                // the name: a password typed through the
                                // wrong layout is a password nobody can type.
                                //
                                // Asked after every press and answers
                                // `Unchanged` for all but one of them, so an
                                // arrow does not rebuild a keymap.
                                the_keyboard_is_now(chose, screen, server);
                                the_language_is_now(language, screen);
                                the_region_is_now(region, screen);
                                the_zone_is_now(zone, screen);
                                Ok(())
                            }
                            // A release, a repeat, and a key this keyboard has
                            // a code for that evdev does not: none of them is a
                            // reason to take the sign-in screen away from
                            // somebody.
                            Ok(None) | Err(InputError::InvalidKey) => Ok(()),
                            Err(error) => Err(io::Error::other(error)),
                        }
                    }
                    Some(crate::DirectSeatEvent::Pointer(moved)) => {
                        match screen.pointed(&moved) {
                            // A device that reported nonsense is refused and
                            // the arrow stays where it was — the same judgement
                            // the key path makes above, and for its reason.
                            Ok(()) | Err(InputError::InvalidPointer) => Ok(()),
                            Err(error) => Err(io::Error::other(error)),
                        }
                    }
                    Some(crate::DirectSeatEvent::Button { code, state, .. }) => {
                        // `BTN_LEFT`. Nothing else on this screen means anything.
                        if code == 0x110 && state == ButtonState::Pressed {
                            screen.pressed_by_the_pointer();
                        }
                        Ok(())
                    }
                    Some(crate::DirectSeatEvent::Removed) => {
                        screen.the_pointer_went_away();
                        server.clear_input();
                        Ok(())
                    }
                    None => Ok(()),
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
