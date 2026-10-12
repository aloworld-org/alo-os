//! What a session does once somebody is signed in: put their desktop on the
//! display and keep it there.
//!
//! `crate::booting` is the other half of a machine's morning — the sign-in
//! screen, before anybody is there. This is after: the dock, the status area
//! and what is leaving, on the same display, for the person whose session it
//! is.
//!
//! # Two processes, and the reason is whose they are
//!
//! The greeter runs before anybody has signed in and ends when a session opens;
//! this runs inside that session. They are separate programs because they
//! belong to different people — one to the machine, one to the person — and a
//! single process that carried on after the handover would be drawing a
//! person's desktop with whatever the greeter was.
//!
//! # What it is handed, and what it never reads
//!
//! Everything on the desktop arrives through [`crate::TheDesktop`]. The dock is
//! `alo-dock`'s, the appearance `alo-appearance`'s, the readings
//! `alo-measuring`'s and the division `alo-dividing`'s, and this file reads none
//! of them: a compositor that opened `/sys` would be a compositor measuring.

use std::path::Path;

use smithay::input::keyboard::XkbConfig;

use crate::{DirectFrame, DirectSession, Server, TheDesktop, WindowControlLabels, WouldNotStand};

/// Everything a session's compositor is told about the display it stands on.
///
/// Fewer facts than a greeter needs: no accounts, no uid and no door, because
/// nobody is being signed in here — that already happened, and this draws for
/// whoever it happened to.
#[derive(Debug, Clone, Copy)]
pub struct ADisplayToStandOn<'a> {
    /// The display device, opened through libseat and never directly.
    pub display: &'a Path,
    /// The directory the Wayland socket is bound in.
    pub runtime: &'a Path,
    /// The name of the socket inside it, which is what a client connects to.
    pub socket: &'a str,
    /// The XKB layout this machine's keyboard has.
    pub layout: &'a str,
}

/// How a desktop's day ended.
#[derive(Debug)]
#[must_use = "a display lifetime that ended has an outcome worth reading"]
pub struct StoodUp {
    /// Everything the display lifetime reported: the loop's outcome, input
    /// cleanup, both flushes and retirement, each independent of the others.
    pub display: crate::DirectLoopResult,
}

/// Put this person's desktop on the display, and keep it there.
///
/// Returns when `next` says stop or the display lifetime ends. Clients connect
/// to the socket this binds and are drawn underneath the desktop, with their
/// input routed to them.
///
/// # Errors
/// [`WouldNotStand`], one variant per thing to go and fix — the seat that would
/// not lend the display, the socket that would not bind, the keymap that does
/// not exist, the font that is not there.
pub fn stand_the_desktop_up(
    display: ADisplayToStandOn<'_>,
    desktop: &mut dyn TheDesktop,
    mut next: impl FnMut() -> DirectFrame,
    asks_an_application_to_start: Option<crate::AWayToStartApplications>,
    what_this_machine_offers: &alo_dock::pinning::Discovery,
) -> Result<StoodUp, WouldNotStand> {
    let mut server = Server::bind_keyboard(
        display.runtime,
        display.socket,
        XkbConfig {
            layout: display.layout,
            ..XkbConfig::default()
        },
    )?;
    // **How this machine starts an application, handed in rather than built.**
    // `alo_portals::activated` is the implementation and `alo-portals` is a
    // dev-dependency here, so the caller supplies it — see
    // `Server::asks_an_application_to_start`. `None` is a machine that cannot
    // start anything and reports every ask, which is the honest default.
    if let Some(ask) = asks_an_application_to_start {
        server.it_can_start_applications_with(ask);
    }
    // **What this person's Dock is pinned to, worked out once.** Two readings
    // meet here and neither is this crate's to take: what they kept, which is a
    // file in their folder, and what the machine has, which is a scan. Both
    // arrive as data for the reason the sender above does.
    //
    // **And the write goes straight back out.** `crates/alo-shell` keeps no
    // settings file of its own (ADR 0038), so the answer to *write this down*
    // is handed to the desktop that read it — at most once in a session's life,
    // and never for a scan that failed or has not finished.
    let kept = desktop.the_pins_they_kept();
    if let Some(owed) =
        server.its_dock_is_filled_from_what_they_kept(&kept, what_this_machine_offers)
    {
        desktop.keep_these_pins(&owed);
    }
    let mut labels = WindowControlLabels::new()?;
    // The machine's own sentences, for the names a reader is told. The same
    // bundle the sign-in screen words a refusal from, read the same way.
    let strings = crate::what_this_machine_can_say()?;
    let mut session = DirectSession::new(display.display.to_owned())?;
    let stood = session.desktop(&mut server, desktop, &mut labels, &strings, &mut next)?;
    Ok(StoodUp {
        display: stood.outcome,
    })
}

#[cfg(test)]
#[path = "session_desktop_tests.rs"]
mod tests;
