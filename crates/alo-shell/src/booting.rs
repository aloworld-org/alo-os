//! What a machine does when it boots: put a sign-in screen on the display it
//! has, and wait for somebody.
//!
//! Every piece of this existed before this file did — the greeting, the screen,
//! the seat, the display, the drawing — and none of them had ever been put in
//! order by anything a machine could start. `crates/alo-shell` had no binary at
//! all, so a machine that installed alo OS booted to a console. This is the
//! order, and `src/bin/alo-compositor.rs` is the process that runs it.
//!
//! # What it is told, and what it reads
//!
//! Everything about the machine arrives as [`AMachineToStandOn`]. Nothing here
//! reads `/etc/alo/agentd.toml`: the uid is a number the caller was given, for
//! the reason `alo_greeting::Greeting` gives for the same parameter — a file
//! with one owner does not grow a second parser. The accounts are read by
//! `alo-accounts` and the door is knocked on by `alo-greeting`; this file
//! decides nothing about either.
//!
//! # What it does not do yet, said here rather than discovered
//!
//! - **The keyboard is the layout the caller names, and nothing on this machine
//!   decides that layout.** No crate in this repository does yet. A person
//!   whose keyboard is not the one named cannot type their password, which is
//!   why the layout is a parameter that must be passed rather than a default
//!   hidden in here.
//! - **The screen is drawn light, at the ordinary text scale, in the designed
//!   palette.** `alo-appearance` holds what a person chose, and what a person
//!   chose is kept in their own session — which nobody has opened yet at a
//!   sign-in screen. Reading it for the greeter is a question about where a
//!   pre-sign-in preference lives, and that question has no answer in this
//!   repository today.
//! - **It ends when a session opens.** Standing the desktop up in that session
//!   is the next task, and a compositor that drew a desktop here would be
//!   drawing one nothing had decided.

use std::path::Path;

use alo_greeting::{Greeting, TheOpenersDoor};
use alo_strings::Strings;
use smithay::input::keyboard::XkbConfig;

use crate::{
    DirectFrame, DirectSession, Server, SignInScreen, Signing, WindowControlLabels,
    sign_in_raster::SignInLook,
};

/// Everything a compositor is told about the machine it is standing on.
///
/// Trusted configuration, all of it: these are a unit file's words, never an
/// agent's. A path here is opened with the privilege this process has.
#[derive(Debug, Clone)]
pub struct AMachineToStandOn<'a> {
    /// The display device, opened through libseat and never directly.
    pub display: &'a Path,
    /// Where this machine keeps its accounts.
    pub accounts: &'a Path,
    /// The uid `/etc/alo/agentd.toml` names as this machine's person.
    pub person: u32,
    /// The door a session is asked for at: `TheOpenersDoor::on_this_machine`
    /// on a machine, and a socket a test may bind in a test. The door itself
    /// rather than its path, because where `alo-sessiond` listens is that
    /// crate's constant and a second spelling of it here is a second answer.
    pub door: TheOpenersDoor,
    /// The directory the Wayland socket is bound in.
    pub runtime: &'a Path,
    /// The name of the socket inside it.
    pub socket: &'a str,
    /// The XKB layout this machine's keyboard has — `gb`, `de`, `fr`.
    pub layout: &'a str,
}

/// How a compositor's morning ended.
#[derive(Debug)]
#[must_use = "a machine that signed somebody in and did not say so has lost the session"]
pub enum Stood {
    /// A session opened for this uid, and it is open now.
    SomebodySignedIn {
        /// Whose session it is.
        person: u32,
    },
    /// The screen ended with nobody signed in: the scheduler said stop, or the
    /// seat took the display away.
    NobodyDid,
}

/// Why a machine could not put a sign-in screen on its display.
///
/// Each of these is a different thing to go and fix, which is the whole reason
/// they are not one error with a string in it: a display that is not there, a
/// seat that would not lend it, a socket already bound and a keymap that does
/// not exist send whoever is standing the machine up to four different places.
#[derive(Debug, thiserror::Error)]
pub enum WouldNotStand {
    /// The login seat would not lend this process the display.
    #[error("this machine's display could not be taken from its seat: {0}")]
    Seat(#[from] crate::SessionError),
    /// The Wayland socket could not be bound, or the keymap could not be built.
    #[error("this machine's compositor could not start: {0}")]
    Compositor(#[from] crate::InputError),
    /// The machine's own sentences could not be read.
    #[error("this machine's own words could not be read: {0}")]
    Words(String),
    /// The font every sentence on the screen is laid out with is not there.
    #[error("the screen's own font could not be loaded: {0}")]
    Font(#[from] crate::WindowControlLabelError),
}

/// Put the sign-in screen on this machine's display, and keep it there.
///
/// Returns when somebody signs in, when `next` says stop, or when the display
/// lifetime ends. **A session that opened is reported even if the display then
/// failed to retire**: the person is signed in either way, and a machine that
/// reported only the display's failure would have opened a session and not said
/// so.
///
/// `next` is trusted pacing, exactly as it is for `run_compositor`. It is
/// called before every frame and while idle, and it must return promptly: the
/// seat is polled around it, and a caller that blocked in there would be a
/// machine that cannot be paused by its own login manager.
///
/// # Errors
/// [`WouldNotStand`], one variant per thing to go and fix. Everything that
/// happened *after* the screen stood up is inside [`Stood`] and in the log the
/// caller writes, never thrown away.
pub fn stand_the_sign_in_screen_up(
    machine: AMachineToStandOn<'_>,
    words: Strings,
    mut next: impl FnMut() -> DirectFrame,
) -> Result<Stood, WouldNotStand> {
    let mut server = Server::bind_keyboard(
        machine.runtime,
        machine.socket,
        XkbConfig {
            layout: machine.layout,
            ..XkbConfig::default()
        },
    )?;
    let mut labels = WindowControlLabels::new()?;
    let screen = SignInScreen::of(
        Greeting::at(machine.accounts, machine.person, machine.door),
        words,
    );
    let mut session = DirectSession::new(machine.display.to_owned())?;
    let stood = session.sign_in(&mut server, screen, &mut labels, first_look(), &mut next)?;
    // The seat's own close is separate from everything the display reported,
    // and both are separate from whether somebody signed in.
    let signed_in = match stood.outcome.signing {
        Signing::HandedOver(session) => Stood::SomebodySignedIn {
            person: session.uid(),
        },
        Signing::Still(_) => Stood::NobodyDid,
    };
    Ok(signed_in)
}

/// The look a machine's first screen is drawn in.
///
/// Not a preference and not a default hiding one: see this file's header. When
/// there is somewhere for a person to have said what a sign-in screen should
/// look like, this reads it.
fn first_look() -> SignInLook {
    SignInLook {
        scheme: alo_appearance::Scheme::Light,
        scale: alo_appearance::TextScale::ordinary(),
        contrast: crate::Contrast::AsDesigned,
    }
}

/// Every sentence this machine can say, read from the bundle it ships with.
///
/// # Errors
/// [`WouldNotStand::Words`] when the bundle is missing or unreadable, which is
/// a machine that cannot word a refusal — and a sign-in screen that could not
/// say *that password is wrong* is one nobody can use.
pub fn what_this_machine_can_say() -> Result<Strings, WouldNotStand> {
    alo_saying::everything_this_machine_can_say()
        .map(Strings::of)
        .map_err(|why| WouldNotStand::Words(why.to_string()))
}

#[cfg(test)]
#[path = "booting_tests.rs"]
mod tests;
