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
    /// The XKB variant of that layout — `oss` for `fr(oss)` — and empty for
    /// the layout's plain form.
    ///
    /// **Added 2026-10-14, because it was missing here and present in
    /// [`crate::AMachineToStandOn`].** The greeter has carried a variant
    /// since it learnt to read this machine's keyboard file; this struct took
    /// only the layout, so a person who chose `fr(oss)` signed in through
    /// `fr(oss)` and then worked all day through plain `fr`. Empty rather
    /// than [`None`] for the reason the greeter's is: empty is XKB's own word
    /// for *the layout's default*, and a second spelling of one absence is a
    /// second thing to translate at the one place it is handed over.
    pub variant: &'a str,
}

/// How a desktop's day ended.
#[derive(Debug)]
#[must_use = "a display lifetime that ended has an outcome worth reading"]
pub struct StoodUp {
    /// Everything the display lifetime reported: the loop's outcome, input
    /// cleanup, both flushes and retirement, each independent of the others.
    pub display: crate::DirectLoopResult,
    /// **What this seat made of a person's input**, over the whole session.
    ///
    /// Carried out because a lane that survives a refusal has to be able to say
    /// it happened — `InputError::is_about_one_event` is where that is argued,
    /// and the short of it is that a pointer silently doing nothing for ever is
    /// worse than a desktop that ends loudly.
    ///
    /// **It was one number until 2026-10-18 and one number could not answer the
    /// question.** On that day a press landing on a Dock entry opened nothing,
    /// and *could not make sense of: 0* would have meant either **nothing
    /// arrived** or **everything worked** — opposites. So the refusals are
    /// carried beside what the seat routed, with the key presses as the
    /// control. [`crate::WhatTheSeatMadeOfIt`] holds the argument.
    ///
    /// **And the same three numbers are said while the session runs**, through
    /// `TheDesktop::what_the_seat_made_of_its_input`, because on that day this
    /// field could not be read at all: nothing handles `SIGTERM`, so every way
    /// of stopping that machine killed it before this was printed. This is the
    /// last word rather than the only one.
    pub input: crate::WhatTheSeatMadeOfIt,
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
    the_terminal: Option<alo_dock::AppId>,
    the_artwork: &alo_dock::artwork::TheArtwork,
) -> Result<StoodUp, WouldNotStand> {
    let mut server = Server::bind_keyboard(
        display.runtime,
        display.socket,
        XkbConfig {
            layout: display.layout,
            variant: display.variant,
            ..XkbConfig::default()
        },
    )?;
    // **And a pointer on the same seat, which nothing in production did until
    // 2026-10-17.**
    //
    // Measured on a booted machine: the desktop drew its Dock, the first
    // pointer event arrived, and the display loop **ended** —
    // `DirectLoopResult { outcome: Err(Input(InputDispatchError { error:
    // PointerUnavailable })) }` at 759.98 s, 113 seconds after it came up. A
    // person who moved the mouse lost their desktop.
    //
    // `Server::enable_pointer` has existed and been tested all along. Every one
    // of its callers was an **example or a test** — seven of them — and no
    // shipping binary called it, so `surfaces.pointer` was `None` on every real
    // machine and `direct_pointer` refused the first motion it was handed.
    //
    // This is the shape this lane keeps meeting: finished, tested code that
    // nothing calls. A passing test cannot tell you whether anything reaches
    // the code; `examples/a_real_application` enabled the pointer and proved
    // the pointer worked, which is what made the gap invisible.
    //
    // **Here rather than in the binary**, because this function is where the
    // seat is stood up and where the keyboard is bound: a caller that had to
    // remember one of the two would be a caller that forgets.
    server.enable_pointer()?;
    // **How this machine starts an application, handed in rather than built.**
    // `alo_portals::activated` is the implementation and `alo-portals` is a
    // dev-dependency here, so the caller supplies it — see
    // `Server::asks_an_application_to_start`. `None` is a machine that cannot
    // start anything and reports every ask, which is the honest default.
    if let Some(ask) = asks_an_application_to_start {
        server.it_can_start_applications_with(ask);
    }
    // **Which application this machine's terminal is, handed in for the reason
    // the sender above is**: it is a reading of the machine, and this crate
    // shows rather than measures. `None` is a machine with no terminal
    // installed, and `crate::a_chord_opens_a_terminal` says so on the error
    // stream rather than quietly doing nothing.
    if let Some(terminal) = the_terminal {
        server.its_terminal_is(terminal);
    }
    // **What each application draws on the Dock, handed in for the reason the
    // two above are**: which file holds an application icon is a reading of
    // this machine, and this crate shows rather than measures
    // (`crate::what_the_dock_is_filled_with`). An empty one is a Dock of first
    // letters, which is the decided fallback and not a failure.
    server.its_applications_draw(the_artwork.clone());
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
    // **What the Settings entry is called, told to the server once.** A native
    // entry has no identifier, so its translated label is the only string a
    // search can match it by — and the server has no vocabulary of its own.
    //
    // **After the strings and not before**, which is why this is here rather
    // than beside the pins above: the bundle is read from this machine and the
    // order of these lines is the order each thing becomes knowable.
    server.its_settings_entry_is_called(
        strings
            .say(
                &alo_dock::words::SETTINGS_ON_THE_DOCK.key(),
                &alo_strings::Filling::nothing(),
            )
            .into_text(),
    );
    // **And the market's, which is the other native entry.** One of these
    // without the other is a Dock row a search cannot find — and before
    // 2026-10-17 it was worse than that: the market borrowed this label, so
    // typing *Settings* matched two rows.
    server.its_applications_entry_is_called(
        strings
            .say(
                &alo_dock::words::APPLICATIONS_ON_THE_DOCK.key(),
                &alo_strings::Filling::nothing(),
            )
            .into_text(),
    );
    let mut session = DirectSession::new(display.display.to_owned())?;
    let stood = session.desktop(&mut server, desktop, &mut labels, &strings, &mut next)?;
    // **Read after the loop, from the seat that did the routing and the
    // refusing.** The counts live on the `Server` rather than in the loop's
    // result because they belong to the seat and outlive any one lifetime of
    // the display.
    let input = server.what_the_seat_made_of_its_input();
    Ok(StoodUp {
        display: stood.outcome,
        input,
    })
}

#[cfg(test)]
#[path = "session_desktop_tests.rs"]
mod tests;
