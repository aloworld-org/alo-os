//! The asking: a screen on the display, a keyboard under a person's hands, and
//! the loop between them.
//!
//! # What this joins, and what it deliberately does not disturb
//!
//! Before this, `alo-installing` **never asked anybody anything.** It read the
//! disk out of the kernel command line — staged by the program a person ran on
//! Windows — installed, and ended. Every screen, every key reader and the whole
//! renderer sat on the crate's surface with nothing calling them.
//!
//! **The road that already works is left exactly as it was.** Where the command
//! line names a disk, this does nothing at all: that is the Windows road, the
//! answers were given before the restart, and a screen asking them again would
//! be asking a person to repeat themselves.
//!
//! What it is for is the other road. A person who booted a stick was never
//! asked anything by anybody, and today the environment refuses with *no disk
//! chosen* — a true sentence about a question it never put. **That refusal is
//! what this replaces.**
//!
//! # The loop is four lines and the decisions are not in it
//!
//! [`what_the_person_did`] turns one key event into one intention, and
//! [`TheAsking`] is a state machine over those. Both are tested on any machine.
//! [`ask`] opens `/dev/fb0` and a keyboard and turns the handle; it is the only
//! part a test here cannot judge, and the certified machine's boot is the
//! measurement it is owed.
//!
//! That is the third time this crate has taken that split — `fill`,
//! `draw_a_screen`, and now this — and it is why a fault in any of them is a
//! fault a test can reach.

use crate::a_key_reaches_here::{Pressed, what_was_pressed};

/// What a person meant by the key they pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatThePersonDid {
    /// Move up the cards.
    WentUp,
    /// Move down the cards.
    WentDown,
    /// Take the card the ring is on.
    Chose,
    /// Go back to the screen before.
    WentBack,
    /// Nothing this screen acts on.
    NothingThisScreenActsOn,
}

/// What this key event meant.
///
/// `None` for an event that is not a key going down at all — a release, or a
/// short read — because a screen that moved on both the press and the release
/// would move twice for one keystroke. That is
/// [`crate::what_was_pressed`]'s rule and this does not restate it.
#[must_use]
pub fn what_the_person_did(event: &[u8]) -> Option<WhatThePersonDid> {
    Some(match what_was_pressed(event)? {
        Pressed::Up => WhatThePersonDid::WentUp,
        Pressed::Down => WhatThePersonDid::WentDown,
        Pressed::Enter => WhatThePersonDid::Chose,
        Pressed::Escape => WhatThePersonDid::WentBack,
        // Tab and Backspace mean something on a screen a person types into and
        // nothing on a screen of cards. They are named here as understood and
        // unacted-on rather than falling through a catch-all, so that the
        // screen which does type into a field has somewhere to take them.
        Pressed::Tab | Pressed::Backspace => WhatThePersonDid::NothingThisScreenActsOn,
    })
}

/// Which screen the environment is on.
///
/// Only the two the Wi-Fi road needs first. The rest of the eleven frames are
/// added here as each one's own state exists, and the enum is deliberately
/// small rather than a placeholder for all of them: a variant nothing can draw
/// is a screen a person could be sent to and left on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichScreen {
    /// S0 — which keyboard, the two-card question.
    TheKeyboard,
    /// What *Something else* opens: every layout, as a list.
    EveryLayout,
}

/// Where the asking has got to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheAsking {
    /// Which screen is on the display.
    on: WhichScreen,
    /// Which card the ring is on, on that screen.
    card: usize,
    /// How many cards that screen has.
    how_many: usize,
    /// Which layout has been settled on, once one has been.
    settled: Option<String>,
}

impl TheAsking {
    /// Begin, on the keyboard question.
    ///
    /// It begins there because typing presupposes the layout: the two things
    /// this environment asks a person to type are a network password and a
    /// disk's own name, and both fail silently and identically on the wrong
    /// keyboard.
    #[must_use]
    pub fn begins() -> Self {
        Self {
            on: WhichScreen::TheKeyboard,
            card: 0,
            how_many: 2,
            settled: None,
        }
    }

    /// Which screen is on the display.
    #[must_use]
    pub const fn on(&self) -> WhichScreen {
        self.on
    }

    /// Which card the ring is on.
    #[must_use]
    pub const fn card(&self) -> usize {
        self.card
    }

    /// The layout settled on, once one has been.
    #[must_use]
    pub fn settled(&self) -> Option<&str> {
        self.settled.as_deref()
    }

    /// How many cards the screen in front of a person has.
    ///
    /// Told rather than worked out, because on the list screen it is how many
    /// fit on this display — which only the display can answer.
    pub const fn there_are_this_many_cards(&mut self, how_many: usize) {
        self.how_many = how_many;
        if self.card >= how_many {
            self.card = how_many.saturating_sub(1);
        }
    }

    /// Act on what a person did, and say whether the screen has to be redrawn.
    ///
    /// `named` is what the ring is on, where the screen is a list of layouts —
    /// the caller has the names and this does not hold them.
    ///
    /// Returns `true` when anything a person can see has changed. A redraw for
    /// a key that changed nothing would make the whole screen flash on a key
    /// the environment does not even act on.
    pub fn and_then(&mut self, did: WhatThePersonDid, named: Option<&str>) -> bool {
        match did {
            WhatThePersonDid::WentUp => {
                let was = self.card;
                self.card = self.card.saturating_sub(1);
                self.card != was
            }
            WhatThePersonDid::WentDown => {
                let was = self.card;
                let last = self.how_many.saturating_sub(1);
                self.card = (self.card + 1).min(last);
                self.card != was
            }
            WhatThePersonDid::Chose => self.chose(named),
            WhatThePersonDid::WentBack => {
                // From the list, back to the question. From the question there
                // is nowhere back to: this is the first screen, and Escape on
                // it does nothing rather than leaving a person on a blank
                // display.
                if self.on == WhichScreen::EveryLayout {
                    self.on = WhichScreen::TheKeyboard;
                    self.card = 0;
                    self.how_many = 2;
                    true
                } else {
                    false
                }
            }
            WhatThePersonDid::NothingThisScreenActsOn => false,
        }
    }

    /// Take whatever the ring is on.
    fn chose(&mut self, named: Option<&str>) -> bool {
        match self.on {
            // The keyboard question's two cards: the layout the environment
            // guessed, and *Something else*.
            WhichScreen::TheKeyboard => {
                if self.card == 0 {
                    self.settled = named.map(str::to_owned);
                    self.settled.is_some()
                } else {
                    self.on = WhichScreen::EveryLayout;
                    self.card = 0;
                    true
                }
            }
            WhichScreen::EveryLayout => {
                self.settled = named.map(str::to_owned);
                self.settled.is_some()
            }
        }
    }
}

#[cfg(test)]
#[path = "the_environment_asks_tests.rs"]
mod tests;

/// Find a keyboard, open the display, and ask.
///
/// Returns the layout a person settled on, and [`None`] where there was no
/// display, no keyboard, or the person never settled — all of which the caller
/// has to be able to go on from, because **a refusal a person can read beats a
/// program that stops.**
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
    offer_these_layouts: &[(&str, &str)],
) -> std::io::Result<Option<String>> {
    use crate::a_surface_to_draw_on::{Grey, WHERE_THE_PIXELS_GO};
    use crate::alos_own_face::AlosOwnFace;
    use std::io::Read as _;

    let Some(face) = AlosOwnFace::carried().ok() else {
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
    let mut event = [0_u8; crate::a_key_reaches_here::ONE_EVENT];
    let mut owed = true;

    loop {
        if owed {
            let words = the_words_for(strings, &asking, offer_these_layouts);
            if let Some(shows) = crate::a_screen_is_drawn::what_a_screen_shows(
                surface.across(),
                surface.down(),
                None,
                face,
                &words,
            ) {
                asking.there_are_this_many_cards(words.cards.len());
                crate::a_screen_is_drawn::draw_a_screen(
                    &display, surface, beneath, ink, face, &shows,
                )?;
            }
            owed = false;
        }

        // A short read is not an event. `read_exact` would be wrong here: a
        // partial read on this road means the device went away, and the honest
        // answer is to stop asking rather than to block for ever.
        if keyboard.read(&mut event)? < crate::a_key_reaches_here::ONE_EVENT {
            return Ok(asking.settled().map(str::to_owned));
        }
        let Some(did) = what_the_person_did(&event) else {
            continue;
        };
        let named = under_the_ring(&asking, offer_these_layouts);
        owed = asking.and_then(did, named);
        if asking.settled().is_some() {
            return Ok(asking.settled().map(str::to_owned));
        }
    }
}

/// What the ring is on, where that is a layout.
#[cfg(unix)]
fn under_the_ring<'a>(asking: &TheAsking, layouts: &'a [(&'a str, &'a str)]) -> Option<&'a str> {
    match asking.on() {
        // The first card carries the layout the environment guessed, which is
        // the first one offered until the Windows road stages a better guess.
        WhichScreen::TheKeyboard if asking.card() == 0 => layouts.first().map(|&(named, _)| named),
        WhichScreen::TheKeyboard => None,
        WhichScreen::EveryLayout => layouts.get(asking.card()).map(|&(named, _)| named),
    }
}

/// What the screen in front of a person says.
#[cfg(unix)]
fn the_words_for(
    strings: &alo_strings::Strings,
    asking: &TheAsking,
    layouts: &[(&str, &str)],
) -> crate::a_screen_is_drawn::TheWordsOfAScreen {
    use crate::a_screen_is_drawn::TheWordsOfAScreen;
    use crate::words;
    use alo_strings::Filling;

    let say = |word: &alo_strings::Word| {
        strings
            .say(&word.key(), &Filling::nothing())
            .text()
            .to_owned()
    };

    match asking.on() {
        WhichScreen::TheKeyboard => TheWordsOfAScreen {
            eyebrow: say(&words::BEFORE_ANYTHING_IS_TYPED),
            title: say(&words::WHICH_KEYBOARD),
            introduction: say(&words::WHY_THE_KEYBOARD_COMES_FIRST),
            cards: vec![
                (
                    layouts
                        .first()
                        .map_or_else(String::new, |&(_, shown)| shown.to_owned()),
                    say(&words::PRESS_THESE_TO_CHECK),
                ),
                (
                    say(&words::SOMETHING_ELSE),
                    say(&words::EVERY_LAYOUT_THIS_COMPUTER_KNOWS),
                ),
            ],
            consequence: say(&words::IF_IT_DOES_NOT_MATCH),
        },
        WhichScreen::EveryLayout => TheWordsOfAScreen {
            eyebrow: say(&words::BEFORE_ANYTHING_IS_TYPED),
            title: say(&words::EVERY_LAYOUT_THIS_COMPUTER_KNOWS),
            introduction: String::new(),
            cards: layouts
                .iter()
                .map(|&(_, shown)| (shown.to_owned(), String::new()))
                .collect(),
            consequence: say(&words::IF_IT_DOES_NOT_MATCH),
        },
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
/// `is_a_keyboard`'s rule and not restated here: a power button reports one key
/// and would otherwise pass as a keyboard, and a person would be left pressing
/// arrows at a device that has none.
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
