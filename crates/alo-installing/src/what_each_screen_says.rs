//! What each screen says, from where the asking has got to.
//!
//! Every sentence here is one `words.rs` already declares. **Not one was
//! written for this file**, which is worth saying: the screens were worded
//! before they could be drawn, so the job here is choosing among sentences
//! rather than inventing them, and a screen that needed a new sentence would
//! be a screen somebody had not designed.
//!
//! # The two things that are never translated
//!
//! A **network's own name** and a **disk's own name**, which arrive from the
//! machine and are put on a card as they came. `words.rs` says so for the
//! disk — *{disk} is what the maker calls the disk — Samsung SSD 870 EVO, WD
//! Elements — and is not translated* — and a network's name is the same kind of
//! thing: it is what somebody typed into a router.
//!
//! # What a field looks like
//!
//! A card, whose lower line is what has been typed. See
//! [`crate::the_asking_walks`] for why it is a card and not a box of its own.
//! A password's characters become dots, **one dot each**, because a person
//! counting that they typed twelve characters is the only honest feedback a
//! hidden field can give.

use crate::ANetwork;
use crate::a_screen_is_drawn::TheWordsOfAScreen;
use crate::the_asking_walks::{TheAsking, WhichScreen};
use crate::which_disks_may_be_offered::Offered;
use crate::words;
use alo_strings::{Filling, Strings, Word};

/// What a person is looking at when they are typing: the caret.
///
/// A line with nothing after it looks like a screen that has stopped, and this
/// environment has no animation to show otherwise — so the caret is a
/// character on the end of the line rather than something that blinks.
///
/// **It was `▌` (U+2590 LEFT HALF BLOCK) until 2026-10-21, and the bundled
/// face does not have that character.** `Manrope-SemiBold.ttf`'s own `cmap`
/// maps it to glyph 0, so the caret drew nothing — and since the caret is the
/// *only* thing on an empty field, the password screen showed no field at all.
/// A person was asked for a password and given nowhere to type it.
///
/// Nothing caught it: the sentence was built correctly, the screen laid out,
/// the test asked whether it laid out. `the_face_has_every_character_it_is_ever_asked_to_draw`
/// is the check that now cannot let a character without a glyph into this
/// crate, and `every_screen_has_ink_on_it_and_is_left_as_a_picture` draws the
/// screens so a person can look.
const THE_CARET: char = '|';

/// What a hidden character looks like.
const A_HIDDEN_CHARACTER: char = '•';

/// How many cards the screen in front of a person has.
///
/// Asked separately from the words because the walk needs it before anything is
/// drawn — the ring has to be somewhere on a list before a person can move it.
#[must_use]
pub fn how_many_cards(
    asking: &TheAsking,
    layouts: &[(&str, &str)],
    networks: &[ANetwork],
    disks: &[Offered],
) -> usize {
    match asking.on() {
        // The guess, and *Something else*.
        WhichScreen::TheKeyboard => 2,
        WhichScreen::EveryLayout => layouts.len(),
        WhichScreen::TheNetwork => networks.len(),
        WhichScreen::TheDisk => disks.len(),
        // A field is one card and the ring does not move over it.
        WhichScreen::ThePassword | WhichScreen::TheReview => 1,
    }
}

/// What this screen says.
#[must_use]
pub fn what_this_screen_says(
    strings: &Strings,
    asking: &TheAsking,
    layouts: &[(&str, &str)],
    networks: &[ANetwork],
    disks: &[Offered],
) -> TheWordsOfAScreen {
    let say = |word: &Word| {
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
                    // **`{keys}` is filled here and was filled nowhere before.**
                    // Every sentence on every screen is said with
                    // `Filling::nothing()`, so this one placeholder reached a
                    // person as the word `[keys]` on the first screen of the
                    // install. The four characters are derived from the
                    // keymap, never written down — see
                    // `the_keys_that_prove_a_layout`.
                    layouts
                        .first()
                        .and_then(|&(code, _)| {
                            crate::the_keys_that_prove_a_layout::the_keys_that_prove_a_layout(code)
                        })
                        .map_or_else(
                            // A layout with no keys to show is a layout this
                            // environment cannot check, so it says the honest
                            // thing rather than an empty check.
                            || say(&words::EVERY_LAYOUT_THIS_COMPUTER_KNOWS),
                            |keys| {
                                strings
                                    .say(
                                        &words::PRESS_THESE_TO_CHECK.key(),
                                        &Filling::of("keys", keys),
                                    )
                                    .text()
                                    .to_owned()
                            },
                        ),
                ),
                (
                    say(&words::SOMETHING_ELSE),
                    say(&words::EVERY_LAYOUT_THIS_COMPUTER_KNOWS),
                ),
            ],
            consequence: say(&words::IF_IT_DOES_NOT_MATCH),
            onward: say(&words::CONTINUE),
            back: say(&words::GO_BACK),
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
            onward: say(&words::CONTINUE),
            back: say(&words::GO_BACK),
        },

        WhichScreen::TheNetwork => TheWordsOfAScreen {
            eyebrow: say(&words::NETWORK),
            title: say(&words::HOW_SHOULD_THIS_COMPUTER_CONNECT),
            introduction: say(&words::WHY_A_NETWORK_IS_NEEDED),
            // A network's own name, as it came off the air, and underneath it
            // nothing: whether it wants a password is said by the password
            // screen appearing, and a card that said *locked* would be saying
            // in words what the next screen says by existing.
            cards: networks
                .iter()
                .map(|network| (network.name().to_owned(), String::new()))
                .collect(),
            consequence: say(&words::THE_PASSWORD_IS_NEVER_WRITTEN),
            onward: say(&words::CONTINUE),
            back: say(&words::GO_BACK),
        },

        WhichScreen::ThePassword => TheWordsOfAScreen {
            eyebrow: say(&words::NETWORK),
            // The network's own name is the question: *this* is the one being
            // joined, and a person who chose the wrong one finds out here
            // rather than after the router refuses them.
            title: asking.network().unwrap_or_default().to_owned(),
            introduction: say(&words::WHY_A_NETWORK_IS_NEEDED),
            cards: vec![(the_field(asking), String::new())],
            consequence: say(&words::THE_PASSWORD_IS_NEVER_WRITTEN),
            onward: say(&words::CONTINUE),
            back: say(&words::GO_BACK),
        },

        WhichScreen::TheDisk => TheWordsOfAScreen {
            eyebrow: say(&words::WHERE_TO_INSTALL),
            title: say(&words::WHICH_DISK),
            introduction: say(&words::NOTHING_UNTIL_YOU_TYPE_IT),
            // `called` is already *Samsung SSD 870 EVO · 1 TB* — joined by
            // `A_DISK_AND_ITS_SIZE` where the list was read — and `holding` is
            // already a sentence. Saying either again here would be saying it
            // twice, and two joins of the same two parts is how they come to
            // disagree.
            cards: disks
                .iter()
                .map(|disk| (disk.called.clone(), disk.holding.clone()))
                .collect(),
            consequence: say(&words::THE_STICK_IS_NOT_SHOWN),
            onward: say(&words::CONTINUE),
            back: say(&words::GO_BACK),
        },

        WhichScreen::TheReview => TheWordsOfAScreen {
            eyebrow: say(&words::CHECK_THIS_CAREFULLY),
            // What is about to be erased, named. The disk's own name is the
            // title because it is the thing a person has to read and then type
            // back, and reading it off anywhere but the top of the screen
            // would make that harder than it needs to be.
            title: asking.disk().unwrap_or_default().to_owned(),
            // **`{holding}` is filled here and was filled nowhere before.** A
            // person confirming the one irreversible step of the install read
            // *"{holding} on this disk will be gone"* — the placeholder — on
            // the screen whose entire job is telling them exactly what they
            // are about to lose. `holding` is already a said sentence where
            // the disk was offered, so it is carried across rather than built
            // a second way.
            introduction: {
                let holding = disks
                    .iter()
                    .find(|disk| Some(disk.called.as_str()) == asking.disk())
                    .map(|disk| disk.holding.clone());
                holding.map_or_else(
                    // No disk matched, so nothing can be named. The sentence
                    // that names nothing in particular is the honest one, and
                    // it is already written.
                    || say(&words::NOTHING_UNTIL_YOU_TYPE_IT),
                    |holding| {
                        strings
                            .say(
                                &words::WHAT_WILL_BE_GONE.key(),
                                &Filling::of("holding", holding),
                            )
                            .text()
                            .to_owned()
                    },
                )
            },
            cards: vec![(say(&words::TYPE_THE_NAME_TO_CONTINUE), the_field(asking))],
            consequence: say(&words::NO_OTHER_DISK_IS_TOUCHED),
            // **Not *Continue*.** `words.rs` says why on the word itself:
            // a button that erases a disk should say so on its face.
            onward: say(&words::ERASE_AND_INSTALL),
            back: say(&words::GO_BACK),
        },
    }
}

/// What has been typed, as it is shown, with the caret on the end.
fn the_field(asking: &TheAsking) -> String {
    let mut shown: String = if asking.is_hidden() {
        asking.typed().chars().map(|_| A_HIDDEN_CHARACTER).collect()
    } else {
        asking.typed().to_owned()
    };
    shown.push(THE_CARET);
    shown
}

#[cfg(test)]
#[path = "what_each_screen_says_tests.rs"]
mod tests;
