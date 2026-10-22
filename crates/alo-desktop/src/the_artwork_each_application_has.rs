//! **The picture each installed application draws on the Dock**, read off this
//! machine once and handed to the shell as data.
//!
//! Three crates meet here and each does one part of it, which is why this file
//! is short and says mostly why rather than how:
//!
//! - `alo_installed::read_with_their_icons` — what is installed, and what each
//!   one *calls* its icon;
//! - `alo_icons::the_icon_called` — where that name's file actually is;
//! - `image` — the bytes of that file, turned into pixels.
//!
//! The shell does none of it. `crates/alo-shell` is forbidden to take a
//! reading of the machine (`what_the_dock_is_filled_with`: *the shell does not
//! scan, and that is a rule rather than a layering taste*), so the answer
//! arrives there as [`alo_dock::artwork::TheArtwork`], exactly as the pins
//! arrive as a `Discovery`.
//!
//! # Once, when the desktop stands up
//!
//! This walks the filesystem — every icon directory, for every installed
//! application — so it is asked once and kept, which is what
//! [`crate::what_this_machine_offers`] already does for the pins and for the
//! terminal. Asking per frame would walk the disk per frame.
//!
//! # Why the count of each outcome is carried out of here
//!
//! `CLAUDE.md`: *a report cannot tell them apart unless the writer says which
//! it was.* A Dock showing letters has four possible causes — nothing
//! installed declared an icon, the icon was named and is not on disk, it is on
//! disk and is scalable, or it decoded to nothing — and they are four different
//! machines. [`HowItWent`] carries all four so a report about a real machine
//! can say which, instead of *icons did not appear*.

use alo_dock::artwork::{APicture, TheArtwork};
use alo_dock::window::AppId;
use alo_icons::WhatWasFound;

/// How many applications ended up in each outcome.
///
/// **Counts and not a list**, because this is for saying what a machine is
/// like rather than for deciding anything: nothing branches on it, and a list
/// of names would invite a caller to re-derive the artwork from it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HowItWent {
    /// Applications that drew a real picture.
    pub drew_a_picture: usize,
    /// Applications whose entry named no icon at all.
    pub named_none: usize,
    /// Applications whose named icon is nowhere on this machine.
    pub named_one_that_is_not_here: usize,
    /// Applications whose only artwork is scalable, which nothing here can
    /// rasterise. See `alo-icons`, *PNG only*.
    pub only_scalable: usize,
    /// Applications whose artwork is on disk and would not decode.
    ///
    /// **Its own count rather than folded into the one above.** A file that is
    /// there and unreadable is a broken machine; a file that is not there is
    /// an ordinary one, and a report that cannot tell them apart cannot say
    /// which this is.
    pub would_not_decode: usize,
}

/// Read what every installed application draws, from these directories.
///
/// `entries` and `icons` are `alo_installed::where_entries_live` and
/// `alo_icons::where_icons_live`; `wanted` is how many pixels across the Dock
/// will draw it, which decides **which size is picked** and nothing else — the
/// picture is handed over at its own size and scaled where it is drawn.
///
/// # Keyed by the identifier, because that is what an `AppId` holds here
///
/// `AppId::named` is given the **desktop file identifier** by both callers that
/// build one for a real application — `filled_by` beside this file, and
/// `alo_shell::an_application_verb_is_carried_out` — so that is what artwork is
/// keyed by. The shown name was the first thing this file used, and it would
/// have matched nothing: `Application::shown` is *the name and the identifier
/// together*, a fragment for putting in a row, and no `AppId` anywhere is built
/// from it. The Dock would have drawn letters with a full set of artwork
/// loaded and no way to tell.
#[must_use]
pub fn the_artwork_each_application_has(
    entries: &[std::path::PathBuf],
    icons: &[std::path::PathBuf],
    wanted: u32,
) -> (TheArtwork, HowItWent) {
    let (installed, named) = alo_installed::read_with_their_icons(entries);
    let mut artwork = TheArtwork::none();
    let mut went = HowItWent::default();
    for application in installed.all() {
        let Some(calls_it) = named.named(application.identifier()) else {
            went.named_none += 1;
            continue;
        };
        let at = match alo_icons::the_icon_called(calls_it, icons, wanted) {
            WhatWasFound::APicture(at) => at,
            WhatWasFound::OnlyScalable => {
                went.only_scalable += 1;
                continue;
            }
            WhatWasFound::Nothing => {
                went.named_one_that_is_not_here += 1;
                continue;
            }
        };
        let Some(picture) = decoded(&at) else {
            went.would_not_decode += 1;
            continue;
        };
        // An identifier `AppId` refuses — blank, or only space — is one the
        // Dock cannot name either, so there is nothing to hang a picture on.
        // `alo_applications::Application` already refuses those, so this is
        // unreachable rather than rare; it is written as a refusal because the
        // alternative is an `expect` on somebody else's invariant.
        let Ok(known_as) = AppId::named(application.identifier()) else {
            continue;
        };
        artwork.draws(known_as, picture);
        went.drew_a_picture += 1;
    }
    (artwork, went)
}

/// One file's pixels, or [`None`] where it would not become any.
///
/// **Every failure is [`None`] and none of them stops the walk.** One
/// application with a truncated PNG must not cost every other application its
/// icon, which is `alo_installed::read`'s own rule: *a file that does not parse
/// is skipped rather than failing the walk.*
fn decoded(at: &std::path::Path) -> Option<APicture> {
    let bytes = std::fs::read(at).ok()?;
    // **Told to read it as a PNG rather than asked to guess.** `alo-icons`
    // offers a `.png` and nothing else, so a file here that is not one is a
    // machine whose icon theme is wrong — and sniffing the contents would
    // quietly decode whatever it actually is, under an application's name.
    let decoded = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png).ok()?;
    let rgba = decoded.to_rgba8();
    let (across, down) = rgba.dimensions();
    APicture::of(across, down, rgba.into_raw()).ok()
}

#[cfg(test)]
#[path = "the_artwork_each_application_has_tests.rs"]
mod tests;
