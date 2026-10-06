//! **A key pressed while Settings is open reaches Settings** —
//! `docs/autonomy/the-shell-plan.md` task 18, the half a chord does not cover.
//!
//! `crate::a_chord_reaches_its_action` opens the window and that was the whole
//! of what landed first, which left a worse machine than none: `Escape` is
//! `SettingsKey::Close`, `SettingsWindow::close` had **no production caller**,
//! and a person who pressed `⊞`+`I` got a window they could not dismiss. Every
//! link below this one was finished and tested — `Server::settings_key` builds
//! a press from a keysym, `SettingsWindow::pressed` carries it out — and
//! nothing on a running machine called either. This file is the entrance.
//!
//! # Focus is left exactly where it is
//!
//! `crate::settings_seat`'s own header settles it: *while Settings is open
//! every key is intercepted at the seat and never forwarded … keyboard focus is
//! left where it is, as it is for the record window.* Interception is what
//! stops the client hearing anything, so clearing focus would buy nothing and
//! cost two events — `crate::keyboard`'s `keyboard_focus(None)` *tells a client
//! its keys went up*, and the enter on the way back would be a second lie. The
//! application behind Settings is told nothing, because nothing happened to it.
//!
//! **Held keys are likewise not forgotten**, for the reason
//! `crate::direct_keyboard` gives in `clear_input`: a focus change deliberately
//! does not, and this is not even a focus change.
//!
//! # The pointer keeps its own road
//!
//! Only keys are taken. `settings_seat.rs` says *every key*, and says nothing
//! about the pointer, so motion and buttons go exactly where they went before
//! this file existed. The sign-in screen takes everything because no client is
//! mapped behind it; Settings opens over a session that is running.
//!
//! # The one line nothing here can test, said plainly
//!
//! `crate::direct_desktop`'s lane asks [`Server::settings_is_taking_the_keys`]
//! per input batch and calls in. That call needs a libinput context, which
//! needs a login seat and a real device, so no test in this crate drives it —
//! the same gap every other line of the direct lane has. Everything from the
//! evdev code inward is driven by a test on a real seat, through
//! [`the_key_settings_took`]; the branch itself is read and not measured, and
//! is only verified by a person pressing `⊞`+`I` on a machine.
//!
//! # What a press can change outside the window
//!
//! Two things, and both are done here because this is where the press is known:
//!
//! - **A binding kept is told to the seat again.** `SettingsDid::Kept` with
//!   `SettingsSection::Shortcuts` and `SettingsKept::Written` means the
//!   person's file now says something else, and
//!   `Server::the_shortcuts_are` is what makes a chord mean it. Without this
//!   call Settings would write the file and nothing would change until the next
//!   sign-in, which is the same shape of fault as the window that would not
//!   close.
//! - **A revocation is knocked through to the daemon.** The door is
//!   `alo_changing::TheDaemonsDoor` at the socket the host was told, and
//!   `door.rs` flattens every failure into *the next sign-in* because the
//!   change is already on the disk by then. A door is made per press rather
//!   than held: one connection, one line, one answer.

use std::io;
use std::time::SystemTime;

use alo_strings::Strings;

use crate::{
    InputError, InputUpdate, Server, SettingsDid, SettingsDoors, SettingsKept, SettingsSection,
};

impl Server {
    /// Whether this seat's keys belong to Settings rather than to a client.
    ///
    /// Asked per input batch by whoever drives the lane, because the answer
    /// changes on a keystroke: the press that closes the window is the last one
    /// Settings takes.
    #[must_use]
    pub fn settings_is_taking_the_keys(&self) -> bool {
        self.settings.is_open()
    }
}

/// Route one seat update while Settings is open.
///
/// Keys go to Settings and reach no client. Everything else — a pointer, a
/// device going away, the context retiring — takes the road
/// [`Server::libinput_update`] already gave it, so a person can still move the
/// mouse over the session behind the window.
///
/// Separate from [`Server`] and taking `&mut Server` rather than `&mut self`
/// because it needs the person's vocabulary, which the server does not hold:
/// `crate::direct_desktop`'s lane does, which is the one place a running
/// desktop's strings and its seat are both in scope.
///
/// # Errors
/// Whatever the seat refuses about the key — [`InputError::Unavailable`] for a
/// display bound without a keyboard — and whatever the ordinary road refuses
/// about everything else. A code outside evdev's range is **not** an error
/// here, for `crate::direct_sign_in`'s reason: a keyboard with a code evdev
/// does not have is not a reason to take a person's window away.
pub(crate) fn a_key_reaches_settings(
    server: &mut Server,
    update: InputUpdate<'_>,
    extent: (i32, i32),
    strings: &Strings,
) -> Result<(), InputError> {
    let InputUpdate::Event(event) = update else {
        return server.libinput_update(update, extent);
    };
    // **One reading of what libinput said**, the same function
    // `crate::libinput_routing` hands the clients and `crate::direct_sign_in`
    // hands the sign-in screen, so a machine cannot disagree with itself about
    // which key a person pressed depending on what is on the screen.
    let Some(crate::DirectSeatEvent::Key(key, _)) = crate::libinput_routing::translate(event)?
    else {
        return server.libinput_update(InputUpdate::Event(event), extent);
    };
    the_key_settings_took(server, key, strings)
}

/// One key, from the evdev code inward.
///
/// **Split from the function above so that a test can press a key.** A
/// libinput `Event` needs a login seat and a device, so a test that had to
/// make one could not run at all — and the first version of this file was
/// tested by a helper that called [`the_press_is_carried_out`] itself. Deleting
/// the line that carries a press out left every test green, which is a test
/// that cannot see what it is named for: `docs/misreadings/` has the family.
/// Everything below this signature is now on the road the lane drives.
///
/// # Errors
/// Whatever the seat refuses about the key — [`InputError::Unavailable`] for a
/// display bound without a keyboard. A code outside evdev's range is **not** an
/// error here, for `crate::direct_sign_in`'s reason: a keyboard with a code
/// evdev does not have is not a reason to take a person's window away.
pub(crate) fn the_key_settings_took(
    server: &mut Server,
    key: crate::DirectKeyEvent,
    strings: &Strings,
) -> Result<(), InputError> {
    match server.settings_key(key.code, key.state, key.time) {
        Ok(Some(press)) => {
            the_press_is_carried_out(server, press, strings);
            Ok(())
        }
        // A release, a repeat, and a code this keyboard has that evdev does
        // not. None of the three is something Settings does.
        Ok(None) | Err(InputError::InvalidKey) => Ok(()),
        Err(error) => Err(error),
    }
}

/// Hand one press to the window, and do what it said outside the window.
///
/// **The window is taken out and put back** rather than borrowed, for the
/// reason `Server::the_chord_does_what_it_names` gives: the press needs
/// `&mut SettingsWindow` while the server is already borrowed. Nothing in the
/// chain replaces it, so what is put back is what was taken.
///
/// **A session nobody told where settings are kept cannot have an open window**
/// — `crate::a_chord_reaches_its_action` is the only road that opens one and it
/// needs the places — so there is nothing here for that case to do, and the
/// press is dropped rather than carried out against places this file invented.
fn the_press_is_carried_out(server: &mut Server, press: crate::SettingsPress, strings: &Strings) {
    let Some(places) = server.settings_places.clone() else {
        return;
    };
    let door = alo_changing::TheDaemonsDoor::at(places.daemon());
    let mut window = std::mem::take(&mut server.settings);
    let did = window.pressed(
        press,
        SettingsDoors {
            strings,
            daemon: &door,
            // **The one clock read on this road.** A press is a thing that
            // happened at a moment, and the grants and pairings in force are
            // the ones in force then. Every layer below takes it as an
            // argument so a test can choose it.
            now: SystemTime::now(),
        },
    );
    server.settings = window;
    if let Some(SettingsDid::Kept(SettingsSection::Shortcuts, SettingsKept::Written)) = did
        && let Some(bound) = server.settings.shortcuts().cloned()
    {
        server.the_shortcuts_are(bound);
    }
}

/// The same routing, with the seat's own refusals turned into the lane's.
///
/// `crate::direct_desktop` drives libinput through a closure that answers
/// [`io::Error`], exactly as `crate::direct_sign_in` does, so the conversion
/// lives beside the road rather than in the lane.
pub(crate) fn routed_or_io(
    server: &mut Server,
    update: InputUpdate<'_>,
    extent: (i32, i32),
    strings: &Strings,
) -> io::Result<()> {
    a_key_reaches_settings(server, update, extent, strings).map_err(io::Error::other)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_key_reaches_settings_tests.rs"]
mod tests;
