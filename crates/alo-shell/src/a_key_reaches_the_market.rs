//! **A key pressed while the window of what a person can install is open
//! reaches that window.**
//!
//! `crate::a_press_on_the_dock` opens it and that is all a press can do. Every
//! link below this one was finished and tested before this file existed —
//! `MarketKey::of` reads a symbol, `MarketWindow::pressed` carries it out,
//! `Server::market_key` applies this person's layout — and **nothing on a
//! running machine called any of them.** A person who pressed the Applications
//! entry would have got a window they could not move through and could not
//! dismiss, which is the fault `crate::a_key_reaches_settings` was written to
//! undo for Settings and is worth undoing once rather than twice.
//!
//! # Where this sits in the order of interception
//!
//! Settings first, then this, then the Dock's own surfaces. That is the order
//! the layers are drawn in and so the order a person reads them in:
//! `crate::scene_native::NativeLayers` puts `settings` above `desktop`, and
//! this window is drawn **inside** the desktop picture, over the Dock. A person
//! pressing Escape cancels whatever is in front of them, and in front is
//! decided by the compositor's own stack rather than by the order these
//! questions happen to be asked in.
//!
//! # Focus is left exactly where it is
//!
//! `crate::settings_seat`'s ruling, applied unchanged: interception is what
//! stops the client hearing anything, so clearing focus would buy nothing and
//! cost two events — `crate::keyboard`'s `keyboard_focus(None)` *tells a client
//! its keys went up*, and the enter on the way back would be a second lie. The
//! application behind this window is told nothing, because nothing happened to
//! it.
//!
//! # Only keys, and only while it is open
//!
//! The pointer keeps its own road entirely: motion and buttons go where they
//! went before this file existed, so the Dock is still pressable and so is the
//! session behind. And the question is asked **per input batch** rather than
//! once, because the press that closes the window is the last one this window
//! takes.
//!
//! # What this file does with an ask, and what it refuses to do
//!
//! `MarketPressed::AskedToInstall` is **recorded on the `Server` and carried
//! out nowhere near here.** A person pressing Enter has said what they want;
//! carrying it out means reaching the machine over a network, and a key handler
//! that did it would be the frame loop waiting on a download.
//!
//! So the press becomes `Server::asked_to_install`, the lane takes it once, and
//! `crate::what_a_person_installs` does the work on a thread of its own with
//! the errand on the egress indicator — which `alo_software::install` refuses
//! to run without. `MarketWindow::it_arrived` and `it_was_refused` are the two
//! answers, and the lane gives them when the thread posts back.
//!
//! **This paragraph said *nothing here acts on it* until 2026-10-17**, when
//! the road that acts on it was built. The claim that mattered is unchanged:
//! nothing in this file reaches the machine.

use std::io;

use smithay::backend::input::KeyState;

use crate::{InputError, InputUpdate, MarketPressed, Server};

// **`MarketPressed::AskedToInstall` is acted on after all**, which this file's
// header said it was not. What is unchanged is the claim that mattered: nothing
// *here* reaches the machine. The press is recorded on the `Server` and
// `crate::what_a_person_installs` is what carries it out, on a thread, with the
// errand on the indicator — because reaching the machine from a key handler
// would be the frame loop waiting on a download.

impl Server {
    /// Whether this seat's keys belong to the window of what a person can
    /// install rather than to a client.
    ///
    /// Asked per input batch by whoever drives the lane, because the answer
    /// changes on a keystroke: the press that closes the window is the last one
    /// it takes.
    #[must_use]
    pub fn the_market_is_taking_the_keys(&self) -> bool {
        self.market.is_open()
    }

    /// What a person pressed Install on, waiting to be started.
    ///
    /// **Taken rather than read**, so one press starts one installation however
    /// many frames pass before the lane asks — `Server::take_what_was_asked_of_alo`
    /// keeps the same rule at the other end of the same kind of road, and says
    /// why at length.
    pub fn take_what_was_asked_to_install(&mut self) -> Option<alo_software::Listed> {
        self.asked_to_install.take()
    }

    /// Hand one key to the window, and say what it did.
    ///
    /// **Split from the libinput road below so that a test can press a key.** A
    /// libinput `Event` needs a login seat and a device, so a test that had to
    /// make one could not run at all. `crate::a_key_reaches_settings` records
    /// what that costs when it is skipped: a helper that called the window
    /// itself left every test green while the line carrying a press out had
    /// been deleted.
    ///
    /// **The window is taken out and put back** rather than borrowed:
    /// `MarketWindow::pressed` wants `&mut MarketWindow` while this holds
    /// `&mut self`. Nothing in the call replaces it, so what is put back is
    /// what was taken — `Server::do_this_native_action` says the same of
    /// Settings.
    ///
    /// # Errors
    /// Whatever the seat refuses about the key — [`InputError::Unavailable`] for
    /// a display bound without a keyboard. A code outside evdev's range is
    /// **not** an error here, for `crate::direct_sign_in`'s reason: a keyboard
    /// with a code evdev does not have is not a reason to take a person's
    /// window away.
    pub fn the_market_takes_this_key(
        &mut self,
        code: u32,
        state: KeyState,
        time: u32,
    ) -> Result<Option<MarketPressed>, InputError> {
        match self.market_key(code, state, time) {
            Ok(Some(key)) => {
                let mut window = std::mem::take(&mut self.market);
                let did = window.pressed(key);
                // **An ask is recorded here, where the press is known.** The
                // window holds what was asked for so it can draw the row; this
                // is the *event*, which is what the lane needs — see
                // `Server::asked_to_install` for why reading the window's state
                // instead would start the same installation every frame.
                //
                // Cloned out of the window rather than moved, because the
                // window goes on showing it: `MarketWindow::asked` is cleared
                // by `it_arrived` or `it_was_refused`, and until one of those
                // the row says *installing it*.
                if did == MarketPressed::AskedToInstall {
                    self.asked_to_install = window.what_they_asked_for().cloned();
                }
                self.market = window;
                Ok(Some(did))
            }
            // A release, a repeat, and a code this keyboard has that evdev does
            // not. None of the three is something this window does.
            Ok(None) | Err(InputError::InvalidKey) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

/// Route one seat update while the window of what a person can install is open.
///
/// Keys go to the window and reach no client. Everything else — a pointer, a
/// device going away, the context retiring — takes the road
/// [`Server::libinput_update`] already gave it, so a person can still move the
/// mouse over the session behind the window.
///
/// # Errors
/// Whatever the seat refuses about the key, and whatever the ordinary road
/// refuses about everything else.
pub(crate) fn a_key_reaches_the_market(
    server: &mut Server,
    update: InputUpdate<'_>,
    extent: (i32, i32),
) -> Result<(), InputError> {
    let InputUpdate::Event(event) = update else {
        return server.libinput_update(update, extent);
    };
    // **One reading of what libinput said**, the same function the clients, the
    // sign-in screen and Settings are handed, so a machine cannot disagree with
    // itself about which key a person pressed depending on what is on screen.
    let Some(crate::DirectSeatEvent::Key(key, _)) = crate::libinput_routing::translate(event)?
    else {
        return server.libinput_update(InputUpdate::Event(event), extent);
    };
    // **What it did is deliberately dropped here.** Every one of the five
    // answers has already happened inside the window — it moved, it narrowed,
    // it closed, it recorded an ask, or it did nothing — and the only one with
    // work outside the window is `AskedToInstall`, which this file's header
    // says is the host's and not a key's.
    let _did = server.the_market_takes_this_key(key.code, key.state, key.time)?;
    Ok(())
}

/// The same routing, with the seat's own refusals turned into the lane's.
///
/// `crate::direct_desktop` drives libinput through a closure that answers
/// [`io::Error`], so the conversion lives beside the road rather than in the
/// lane — `crate::a_key_reaches_settings::routed_or_io`'s reason exactly.
pub(crate) fn routed_or_io(
    server: &mut Server,
    update: InputUpdate<'_>,
    extent: (i32, i32),
) -> io::Result<()> {
    a_key_reaches_the_market(server, update, extent).map_err(io::Error::other)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_key_reaches_the_market_tests.rs"]
mod tests;
