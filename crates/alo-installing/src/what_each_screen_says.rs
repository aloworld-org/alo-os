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
const THE_CARET: char = '▌';

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
        },

        WhichScreen::TheReview => TheWordsOfAScreen {
            eyebrow: say(&words::CHECK_THIS_CAREFULLY),
            // What is about to be erased, named. The disk's own name is the
            // title because it is the thing a person has to read and then type
            // back, and reading it off anywhere but the top of the screen
            // would make that harder than it needs to be.
            title: asking.disk().unwrap_or_default().to_owned(),
            introduction: say(&words::WHAT_WILL_BE_GONE),
            cards: vec![(say(&words::TYPE_THE_NAME_TO_CONTINUE), the_field(asking))],
            consequence: say(&words::NO_OTHER_DISK_IS_TOUCHED),
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
