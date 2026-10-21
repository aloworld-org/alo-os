//! One key event to one intention, and the loop that opens the machine.
//!
//! The walk itself is [`crate::the_asking_walks`] and the sentences are
//! [`crate::what_each_screen_says`]. What is left here is the two things only a
//! real machine can do: turn a keypress into something a screen understands,
//! and hold the handle.
//!
//! # Why a field needs the layout and a list does not
//!
//! An arrow key is an arrow key on every keyboard in the world — evdev 103 is
//! up whatever is printed on it. **A letter is not.** So navigation is read
//! straight off the event, and typing goes through
//! [`crate::WhatAPersonTyped`], which holds the modifier state and knows the
//! layout a person chose on the first screen.
//!
//! That is the whole reason the keyboard question comes first: until it is
//! answered there is no layout, and until there is a layout a letter cannot be
//! read at all.

use crate::a_key_reaches_here::{Pressed, what_was_pressed};
use crate::the_asking_walks::{TheAnswers, TheAsking, WhatThePersonDid, WhatTheRingIsOn};
#[cfg(unix)]
use crate::the_keyboard_a_person_has::the_keyboard_a_person_has;
#[cfg(unix)]
use crate::the_screen_the_kernel_left::the_screen_the_kernel_left;

/// What this key event meant, for a screen a person moves about on.
///
/// `None` for an event that is not a key going down — a release, or a short
/// read — because a screen that moved on both the press and the release would
/// move twice for one keystroke.
///
/// **Typing is not here**, and that is not an omission: a letter depends on the
/// layout and this function has none. See this module's header.
#[must_use]
pub fn what_the_person_did(event: &[u8]) -> Option<WhatThePersonDid> {
    Some(match what_was_pressed(event)? {
        Pressed::Up => WhatThePersonDid::WentUp,
        Pressed::Down => WhatThePersonDid::WentDown,
        Pressed::Enter => WhatThePersonDid::Chose,
        Pressed::Escape => WhatThePersonDid::WentBack,
        Pressed::Backspace => WhatThePersonDid::RubbedOut,
        // Tab moves between things on a screen that has several, and no screen
        // here has more than one. Named as understood and unacted-on rather
        // than falling through a catch-all, so the screen that grows a second
        // field has somewhere to take it.
        Pressed::Tab => WhatThePersonDid::NothingThisScreenActsOn,
    })
}

/// The root the real machine is under.
///
/// Both openers take a root so that a test can build one — a tree of input
/// devices, or a framebuffer that is a file of the right size. **Production is
/// the one caller that passes the real thing**, and it is named rather than
/// written as a bare slash so that reading this says which.
#[cfg(unix)]
const THE_WHOLE_MACHINE: &str = "/";

#[cfg(test)]
#[path = "the_environment_asks_tests.rs"]
mod tests;

/// Find a keyboard, open the display, and ask a person everything.
///
/// Returns what they settled on, and [`None`] where there was no display, no
/// keyboard, or they never finished — all of which the caller must be able to
/// go on from, because **a refusal a person can read beats a program that
/// stops.**
///
/// # Errors
///
/// Whatever reading or writing the machine failed with.
///
/// # Why this is gated for Unix
///
/// It opens `/dev/fb0` and `/dev/input/event*` and writes with
/// `FileExt::write_all_at`, and this crate must cross-compile for Windows
/// because `alo-installer`'s tests pull it in.
#[cfg(unix)]
pub fn ask(
    strings: &alo_strings::Strings,
    layouts: &[(&str, &str)],
    networks: &[crate::ANetwork],
    disks: &[crate::Offered],
) -> std::io::Result<Option<TheAnswers>> {
    use crate::a_surface_to_draw_on::Grey;
    use crate::alos_own_face::AlosOwnFace;
    use crate::what_a_person_typed::{Typed, WhatAPersonTyped};
    use std::io::Read as _;
    use std::path::Path;

    let Ok(face) = AlosOwnFace::carried() else {
        // The typeface compiled into this program would not read, which can
        // only mean the file in the repository is damaged. Nothing can be
        // drawn, so the caller falls back to the console it already has.
        return Ok(None);
    };
    // **Which of the four it was, said rather than swallowed.** These two
    // used to be private functions here that answered `Option`, so a machine
    // with no framebuffer, a kernel describing a screen with nothing to write
    // to, and a program not running as root all came back as the same
    // nothing -- and the third is the one that happens on a developer's
    // machine. `NoScreen` tells them apart and its own message says so.
    //
    // Noted and not said: the vocabulary never names a device, and whoever
    // helps somebody afterwards needs to know which file would not open.
    let screen = match the_screen_the_kernel_left(Path::new(THE_WHOLE_MACHINE)) {
        Ok(screen) => screen,
        Err(why) => {
            eprintln!("alo-installing: no screen to draw on: {why}");
            return Ok(None);
        }
    };
    let found = match the_keyboard_a_person_has(Path::new(THE_WHOLE_MACHINE)) {
        Ok(found) => found,
        Err(why) => {
            eprintln!("alo-installing: no keyboard to read: {why}");
            return Ok(None);
        }
    };
    let mut keyboard = std::fs::File::open(&found)?;
    let surface = screen.surface();
    let display = screen.at();

    // **The design's own two tokens, not two numbers chosen here.** This read
    // `Grey::of(0x18)` and `Grey::of(0xE8)` — a near-black surface with
    // near-white ink — which is the design inverted, and no document ever
    // decided it.
    let beneath = Grey::the_canvas();
    let ink = Grey::the_ink();
    let mut asking = TheAsking::begins();
    let mut typing: Option<WhatAPersonTyped> = None;
    let mut event = [0_u8; crate::a_key_reaches_here::ONE_EVENT];
    let mut owed = true;

    loop {
        if owed {
            asking.there_are_this_many_cards(crate::what_each_screen_says::how_many_cards(
                &asking, layouts, networks, disks,
            ));
            // A question with no answers is not a question. This is checked
            // after the count and before the draw, so an empty network list is
            // passed over rather than drawn.
            if asking.there_is_no_network_to_choose() {
                continue;
            }
            let words = crate::what_each_screen_says::what_this_screen_says(
                strings, &asking, layouts, networks, disks,
            );
            // **A list screen is drawn by the layout that scrolls.** The
            // ordinary layout asks for one card slot per card and refuses
            // rather than draw over the accessibility button, so a screen with
            // more choices than fit came back as nothing and was drawn as
            // nothing — a grey screen, on every display, for the eight layouts
            // and for any room with five networks in it. `is_a_list` answers
            // which of the six screens that applies to, and the window is
            // derived from where the ring is.
            let drawn = if asking.is_a_list() {
                crate::a_screen_is_drawn::what_a_list_screen_shows(
                    surface.across(),
                    surface.down(),
                    None,
                    face,
                    &words,
                    asking.card(),
                )
                .map(|list| list.shows)
            } else {
                crate::a_screen_is_drawn::what_a_screen_shows(
                    surface.across(),
                    surface.down(),
                    None,
                    face,
                    &words,
                    Some(asking.card()),
                )
            };
            if let Some(shows) = drawn {
                crate::a_screen_is_drawn::draw_a_screen(
                    display, surface, beneath, ink, face, &shows,
                )?;
            }
            owed = false;
        }

        // A short read is not an event. `read_exact` would be wrong: a partial
        // read on this road means the device went away, and the honest answer
        // is to stop asking rather than to block for ever.
        if keyboard.read(&mut event)? < crate::a_key_reaches_here::ONE_EVENT {
            return Ok(asking.finished().cloned());
        }

        // A field reads letters, which needs the layout a person chose. The
        // reader is made the moment there is one and then kept, because it
        // holds the modifier state between keys.
        if asking.is_a_field() {
            if typing.is_none() {
                typing = asking.layout().and_then(WhatAPersonTyped::on);
            }
            if let Some(reader) = typing.as_mut() {
                let arrived = reader.what_arrived(&event);
                let did = match arrived {
                    Typed::Added(character) => Some(WhatThePersonDid::Typed(character)),
                    Typed::Erased => Some(WhatThePersonDid::RubbedOut),
                    Typed::Nothing => None,
                };
                if let Some(did) = did {
                    let what = ring(&asking, layouts, networks, disks);
                    owed |= asking.and_then(did, what);
                    continue;
                }
            }
        }

        let Some(did) = what_the_person_did(&event) else {
            continue;
        };
        let what = ring(&asking, layouts, networks, disks);
        owed |= asking.and_then(did, what);
        if let Some(answers) = asking.finished() {
            return Ok(Some(answers.clone()));
        }
    }
}

/// What the ring is on, on whichever screen a person is looking at.
#[cfg(unix)]
fn ring<'a>(
    asking: &TheAsking,
    layouts: &'a [(&'a str, &'a str)],
    networks: &'a [crate::ANetwork],
    disks: &'a [crate::Offered],
) -> WhatTheRingIsOn<'a> {
    use crate::the_asking_walks::WhichScreen;
    match asking.on() {
        // The first card carries the layout the environment guessed, which is
        // the first offered until the Windows road stages a better one.
        WhichScreen::TheKeyboard if asking.card() == 0 => WhatTheRingIsOn {
            named: layouts.first().map(|&(named, _)| named),
            as_the_machine_calls_it: None,
            wants_a_password: false,
        },
        WhichScreen::TheKeyboard => WhatTheRingIsOn::default(),
        WhichScreen::EveryLayout => WhatTheRingIsOn {
            named: layouts.get(asking.card()).map(|&(named, _)| named),
            as_the_machine_calls_it: None,
            wants_a_password: false,
        },
        WhichScreen::TheNetwork => {
            networks
                .get(asking.card())
                .map_or_else(WhatTheRingIsOn::default, |network| WhatTheRingIsOn {
                    named: Some(network.name()),
                    as_the_machine_calls_it: None,
                    wants_a_password: network.locked(),
                })
        }
        // **A disk is the one thing with two names**, and both go through:
        // what a person reads and types back, and what the install is given.
        WhichScreen::TheDisk => WhatTheRingIsOn {
            named: disks.get(asking.card()).map(|disk| disk.called.as_str()),
            as_the_machine_calls_it: disks.get(asking.card()).map(|disk| disk.disk.as_str()),
            wants_a_password: false,
        },
        WhichScreen::ThePassword | WhichScreen::TheReview => WhatTheRingIsOn::default(),
    }
}
