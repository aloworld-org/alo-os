//! **Writing in VSCodium** — Demo 2's second road, walked: a person asks alo
//! to write something in a file in VSCodium, and alo opens the file in **its
//! own copy** of VSCodium, types the text and saves it
//! ([ADR 0143](../../../docs/decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md),
//! [ADR 0153](../../../docs/decisions/0153-alo-types-by-keys-in-its-own-copy-and-reads-the-text-back.md)).
//!
//! Written before the code that makes it pass, by the owner's rule of
//! 2026-11-14: *write the walk first*. Each check is one hand-over the road
//! has to make, so a run that stops says which one did not happen.
//!
//! # What it does, and what each check is
//!
//! It runs after the provider is added, because alo cannot answer anybody
//! without one, and **at full access** — the level the first-start sheet
//! arrives with chosen, and which the walk's Tab, Enter keeps.
//!
//! 1. **The file is made by the person**, in alo's terminal (Ctrl+Alt+T):
//!    `touch ~/hello.txt`. No verb makes a file, and none is added for this:
//!    `docs/features.md` promises none.
//! 2. **The ask**, in the alo bar (Super+A), in words, with Enter.
//! 3. **The file opens in alo's own copy** — the desktop's own sentence
//!    (`alo_acting::said_on_the_log::a_file_was_opened`), answered
//!    `opened-in-alos-copy`.
//! 4. **The text is typed into that window** and **then it is saved** — the
//!    shell's own sentences (`alo_acting::said_on_the_log::alo_typed`),
//!    answered `typed` and then `asked-to-save`. The turn asks for the save
//!    only after the text read back exactly (ADR 0153), so a save at all says
//!    the read-back matched.
//! 5. **The file on disk holds the text** — the strongest check, because it is
//!    the only one that is not the machine's own account of itself: the person
//!    hands the file to the journal from the terminal (`systemd-cat`), and the
//!    walk reads the text in the serial log.
//!
//! # What is not walked yet
//!
//! **The default level**, where the person approves the typing once with the
//! whole text on the card, needs alo to be **let use VSCodium**, by name — and
//! nothing on a booted machine makes that grant yet: the only surface that
//! grants anything is the folder picker. The design for that surface is not
//! drawn (`docs/design/the-settings-window.md`), so it is not invented here,
//! and the walk does not pretend to have walked it.
//!
//! # Why this is the identifier
//!
//! [`VSCODIUM`] is Flathub's identifier for VSCodium, the one the image
//! installs at first boot (`image/usr/lib/systemd/system/alo-shipping.service`)
//! and the one `alo_launching::alos_copy` gives a copy of its own.

use std::time::Duration;

use alo_acting::said_on_the_log::{A_FILE_WAS_OPENED, ALO_TYPED};
use alo_acting::{Opened, Typed};

use crate::keys::Chord;
use crate::settling::Settling;
use crate::step::{Since, Walk};

/// VSCodium, by the identifier the desktop says it by.
pub const VSCODIUM: &str = "com.vscodium.codium";

/// The file the person makes and alo writes in, in their home.
pub const THE_FILE: &str = "hello.txt";

/// What alo is asked to write. Plain words a model has no reason to change,
/// and no character the walk's keyboard cannot type.
pub const THE_TEXT: &str = "Written by alo in VSCodium";

/// The journal tag the person's check of the file is written under.
pub const THE_TAG: &str = "alo-walk-the-file";

/// The person's request, as they type it.
#[must_use]
pub fn the_ask() -> String {
    format!(
        "Open the file {THE_FILE} in my home folder in VSCodium, type exactly this text into it: \
         {THE_TEXT} and then save it."
    )
}

/// The phrase that begins the desktop's sentence about opening a file in
/// VSCodium.
#[must_use]
pub fn opened_in_vscodium() -> String {
    format!("{A_FILE_WAS_OPENED} {VSCODIUM},")
}

/// The phrase that begins the shell's sentence about typing a text into
/// VSCodium.
#[must_use]
pub fn typed_into_vscodium() -> String {
    format!("{ALO_TYPED} {VSCODIUM}: text,")
}

/// The phrase that begins the shell's sentence about saving in VSCodium.
#[must_use]
pub fn saved_in_vscodium() -> String {
    format!("{ALO_TYPED} {VSCODIUM}: save,")
}

/// Then write in VSCodium at full access.
#[must_use]
pub fn at_full_access(walk: Walk) -> Walk {
    let look = Settling::UNDER_KVM;
    let minutes = |it: u64| Duration::from_secs(it * 60);
    let terminal = Chord::holding(&["ctrl", "alt"], "t");
    let ret = Chord::key("ret");
    walk.press(Chord::key("esc"))
        .press(terminal.clone())
        .look("the-terminal", look.at_most(minutes(1)))
        .type_text(&format!("touch ~/{THE_FILE}"))
        .press(ret.clone())
        .look("the-file-made", look)
        // **No look between Super+A and the ask**: the alo bar takes the
        // keyboard with only a caret, which the first run on GitHub
        // (2026-11-14) measured as no change at all.
        .press(Chord::with("meta_l", "a"))
        .type_text(&the_ask())
        .look("the-ask-typed", look)
        .mark()
        .press(ret.clone())
        .expect_line_saying(
            "alo opens the file in its own copy of VSCodium",
            &opened_in_vscodium(),
            Opened::InAlosCopy.written(),
            Since::Mark,
            minutes(5),
        )
        .look("vscodium-with-the-file", look.at_most(minutes(2)))
        .expect_line_saying(
            "alo types the text into its copy of VSCodium",
            &typed_into_vscodium(),
            &Typed::Typed.written(),
            Since::Mark,
            minutes(5),
        )
        .expect_line_saying(
            "alo saves it, after the text read back exactly",
            &saved_in_vscodium(),
            &Typed::AskedToSave.written(),
            Since::Mark,
            minutes(2),
        )
        .look("vscodium-after-saving", look.at_most(minutes(1)))
        .press(terminal)
        .look("the-terminal-again", look.at_most(minutes(1)))
        .type_text(&format!("systemd-cat -t {THE_TAG} cat ~/{THE_FILE}"))
        .mark()
        .press(ret)
        .expect_line_saying(
            "the file holds what alo typed",
            THE_TAG,
            THE_TEXT,
            Since::Mark,
            minutes(1),
        )
        .look("the-file-checked", look.at_most(minutes(1)))
}

#[cfg(test)]
#[path = "writing_in_vscodium_tests.rs"]
mod tests;
