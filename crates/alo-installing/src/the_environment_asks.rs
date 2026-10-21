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
    use crate::a_surface_to_draw_on::{Grey, WHERE_THE_PIXELS_GO};
    use crate::alos_own_face::AlosOwnFace;
    use crate::what_a_person_typed::{Typed, WhatAPersonTyped};
    use std::io::Read as _;

    let Ok(face) = AlosOwnFace::carried() else {
        // The typeface compiled into this program would not read, which can
        // only mean the file in the repository is damaged. Nothing can be
        // drawn, so the caller falls back to the console it already has.
        return Ok(None);
    };
    let Some(surface) = the_surface()? else {
        return Ok(None);
    };
    let Some(mut keyboard) = a_keyboard()? else {
        return Ok(None);
    };
    let display = std::fs::OpenOptions::new()
        .write(true)
        .open(WHERE_THE_PIXELS_GO)?;

    let beneath = Grey::of(0x18);
    let ink = Grey::of(0xE8);
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
            if let Some(shows) = crate::a_screen_is_drawn::what_a_screen_shows(
                surface.across(),
                surface.down(),
                None,
                face,
                &words,
            ) {
                crate::a_screen_is_drawn::draw_a_screen(
                    &display, surface, beneath, ink, face, &shows,
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
            wants_a_password: false,
        },
        WhichScreen::TheKeyboard => WhatTheRingIsOn::default(),
        WhichScreen::EveryLayout => WhatTheRingIsOn {
            named: layouts.get(asking.card()).map(|&(named, _)| named),
            wants_a_password: false,
        },
        WhichScreen::TheNetwork => {
            networks
                .get(asking.card())
                .map_or_else(WhatTheRingIsOn::default, |network| WhatTheRingIsOn {
                    named: Some(network.name()),
                    wants_a_password: network.locked(),
                })
        }
        WhichScreen::TheDisk => WhatTheRingIsOn {
            named: disks.get(asking.card()).map(|disk| disk.called.as_str()),
            wants_a_password: false,
        },
        WhichScreen::ThePassword | WhichScreen::TheReview => WhatTheRingIsOn::default(),
    }
}

/// The surface the firmware left, where there is one.
#[cfg(unix)]
fn the_surface() -> std::io::Result<Option<crate::a_surface_to_draw_on::TheSurface>> {
    use crate::a_surface_to_draw_on::{
        TheSurface, WHERE_THE_DEPTH_IS, WHERE_THE_SIZE_IS, WHERE_THE_STRIDE_IS,
    };
    let Ok(size) = std::fs::read_to_string(WHERE_THE_SIZE_IS) else {
        return Ok(None);
    };
    let Ok(stride) = std::fs::read_to_string(WHERE_THE_STRIDE_IS) else {
        return Ok(None);
    };
    let Ok(depth) = std::fs::read_to_string(WHERE_THE_DEPTH_IS) else {
        return Ok(None);
    };
    Ok(TheSurface::from_what_the_kernel_says(&size, &stride, &depth).ok())
}

/// The first device that is a keyboard, opened.
///
/// **A keyboard is recognised by A, Z and Enter together**, which is
/// `is_a_keyboard`'s rule and not restated here: a power button reports one
/// key and would otherwise pass as a keyboard, and a person would be left
/// pressing arrows at a device that has none.
#[cfg(unix)]
fn a_keyboard() -> std::io::Result<Option<std::fs::File>> {
    use crate::a_key_reaches_here::{WHERE_THE_KEYS_ARE, is_a_keyboard};
    let Ok(every) = std::fs::read_dir(WHERE_THE_KEYS_ARE) else {
        return Ok(None);
    };
    let mut named: Vec<String> = every
        .filter_map(Result::ok)
        .map(|one| one.file_name().to_string_lossy().into_owned())
        .filter(|one| one.starts_with("event"))
        .collect();
    // In order, so that two keyboards plugged in give the same answer twice.
    named.sort();

    for one in named {
        let capabilities = std::path::Path::new(WHERE_THE_KEYS_ARE)
            .join(&one)
            .join("device/capabilities/key");
        let Ok(says) = std::fs::read_to_string(&capabilities) else {
            continue;
        };
        if !is_a_keyboard(&says) {
            continue;
        }
        if let Ok(opened) = std::fs::File::open(format!("/dev/input/{one}")) {
            return Ok(Some(opened));
        }
    }
    Ok(None)
}
