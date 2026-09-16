//! What a saved picture is called: the date, and nothing else.
//!
//! The plan's acceptance is one clause and it is absolute — **the file name
//! carries the date and nothing about what was on screen.** This file is that
//! clause, and the argument for it is not tidiness: a folder of pictures is
//! something a person scrolls past, shows somebody over their shoulder, backs
//! up, and hands to a repair shop. A window's title in a file name is a
//! sentence about somebody's medical appointment or their solicitor, readable
//! by everyone who ever sees the folder listing and by every program that
//! indexes it, long after the picture itself was deleted.
//!
//! # The name has no words in it at all
//!
//! Not even *Screenshot*. A name with a word in it is one language's word on
//! every machine in the world, which `CLAUDE.md` calls a bug and which
//! [`crate::words`] would otherwise have to carry a string for — and a
//! translated file name is worse than an English one, because the same folder
//! would then hold files named in whatever language the machine was set to that
//! month. A date is read everywhere, sorts correctly everywhere, and says
//! nothing.
//!
//! # And the only thing it can be told is a moment
//!
//! [`name_for`] takes an [`OnThisDay`] and a number, and there is no argument
//! anywhere in this file for what was captured. A window, a region and the
//! whole screen taken at one moment produce **the same name**, which is a thing
//! a test can hold and a promise a reader can check by looking at the
//! signature.
//!
//! # Several in one second
//!
//! Somebody holding a key down takes several pictures in a second, and each of
//! them is somebody's. So a name that is taken is tried again with a number
//! after it — `2`, then `3` — and after [`HOW_MANY_ONE_MOMENT_MAKES`] the
//! answer is [`None`] and [`crate::NotTaken::NoRoomForAName`] rather than a
//! longer number for ever: a loop with no end in it is how a machine stops
//! answering while looking busy. The number counts pictures in one second and
//! nothing else — it is not a serial number of the person's pictures, and it
//! says nothing about how many they have taken.

use crate::on_this_day::OnThisDay;

/// What a saved picture's file ends with.
///
/// The form the rented mechanism hands back and the form the clipboard is
/// offered ([`crate::Picture::form`]), so the name and the bytes cannot
/// disagree about what the file is.
pub const THE_ENDING: &str = "png";

/// How many pictures one second can be named for.
///
/// A person cannot take a hundred pictures of their screen in one second by
/// hand; something repeating can, and at that point the folder is being filled
/// by a bug rather than by a person, and saying so is better than going on.
pub const HOW_MANY_ONE_MOMENT_MAKES: u32 = 100;

/// What a picture taken at this moment is called, when this many are already
/// there.
///
/// `already_there` counts the names for this moment that are taken: `0` gives
/// the plain name, `1` the same name with `-2` after it, and so on.
///
/// Answers [`None`] past [`HOW_MANY_ONE_MOMENT_MAKES`], which
/// [`crate::NotTaken::NoRoomForAName`] is the sentence for.
#[must_use]
pub fn name_for(day: OnThisDay, already_there: u32) -> Option<String> {
    if already_there >= HOW_MANY_ONE_MOMENT_MAKES {
        return None;
    }
    let (year, month, this_day) = (day.year(), day.month(), day.day());
    let (hour, minute, second) = (day.hour(), day.minute(), day.second());
    let moment = format!("{year:04}-{month:02}-{this_day:02}-{hour:02}{minute:02}{second:02}");
    Some(match already_there {
        0 => format!("{moment}.{THE_ENDING}"),
        // The second picture of a second is `-2`, because a person reading a
        // folder listing counts from one and the first file has no number at
        // all.
        again => format!("{moment}-{}.{THE_ENDING}", again.saturating_add(1)),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    /// Noon on the sixteenth of September 2026, universal time.
    fn a_moment() -> OnThisDay {
        OnThisDay::at(UNIX_EPOCH + Duration::from_secs(1_789_560_000), 0)
    }

    /// **The name is the date and the time, and it ends in the form the bytes
    /// are in.** Written out in full here so that a change to the shape of a
    /// name is a change somebody had to make on purpose.
    #[test]
    fn the_name_is_the_moment_it_was_taken() {
        assert_eq!(
            name_for(a_moment(), 0).unwrap(),
            "2026-09-16-120000.png",
            "the shape of a picture's name changed"
        );
    }

    /// **The name has no word in it**: no *screenshot*, no *picture*, nothing a
    /// translator would have to translate and nothing an English-speaking
    /// machine would impose on everybody else's folder.
    #[test]
    fn the_name_has_no_word_in_it() {
        let name = name_for(a_moment(), 0).unwrap();
        let (before, after) = name.rsplit_once('.').unwrap();
        assert_eq!(after, THE_ENDING);
        assert!(
            before
                .chars()
                .all(|character| character.is_ascii_digit() || character == '-'),
            "{name} has something in it that is not the date"
        );
    }

    /// **A name sorts the way a person expects**, which is why the year comes
    /// first and everything is padded: a folder listed in order is a folder in
    /// the order the pictures were taken.
    #[test]
    fn names_sort_into_the_order_the_pictures_were_taken() {
        let early = OnThisDay::at(UNIX_EPOCH + Duration::from_secs(1_789_516_800 + 9 * 60), 0);
        let later = OnThisDay::at(UNIX_EPOCH + Duration::from_secs(1_789_516_800 + 70 * 60), 0);
        let (first, second) = (name_for(early, 0).unwrap(), name_for(later, 0).unwrap());
        assert!(first < second, "{first} does not sort before {second}");
        assert_eq!(first, "2026-09-16-000900.png");
        assert_eq!(second, "2026-09-16-011000.png");
    }

    /// **Several pictures in one second are told apart by a number**, and the
    /// first of them has no number at all: a person reading a folder listing
    /// counts from one.
    #[test]
    fn several_pictures_in_one_second_are_told_apart() {
        assert_eq!(name_for(a_moment(), 0).unwrap(), "2026-09-16-120000.png");
        assert_eq!(name_for(a_moment(), 1).unwrap(), "2026-09-16-120000-2.png");
        assert_eq!(name_for(a_moment(), 2).unwrap(), "2026-09-16-120000-3.png");
        assert_eq!(
            name_for(a_moment(), 99).unwrap(),
            "2026-09-16-120000-100.png"
        );
    }

    /// **The numbers run out**, rather than a loop going on for ever while the
    /// machine looks busy. Something filling a folder a hundred times in one
    /// second is a bug, and being told so is better than being served.
    #[test]
    fn the_numbers_run_out_rather_than_going_on_for_ever() {
        assert_eq!(name_for(a_moment(), HOW_MANY_ONE_MOMENT_MAKES), None);
        assert_eq!(name_for(a_moment(), u32::MAX), None);
    }

    /// **Every name one moment makes is a different name**, so nothing written
    /// under one of them can overwrite anything written under another.
    #[test]
    fn no_two_names_for_one_moment_are_the_same() {
        let mut seen = std::collections::BTreeSet::new();
        for already_there in 0..HOW_MANY_ONE_MOMENT_MAKES {
            let name = name_for(a_moment(), already_there).unwrap();
            assert!(seen.insert(name.clone()), "{name} came up twice");
        }
        assert_eq!(seen.len(), HOW_MANY_ONE_MOMENT_MAKES as usize);
    }

    /// **A name is one path segment**, with nothing in it a filesystem reads as
    /// a folder or as a place above one — so joining it to the folder the
    /// person chose cannot land anywhere else.
    #[test]
    fn a_name_is_one_segment_and_goes_nowhere_else() {
        for already_there in [0, 1, HOW_MANY_ONE_MOMENT_MAKES - 1] {
            let name = name_for(a_moment(), already_there).unwrap();
            assert!(!name.contains('/'), "{name}");
            assert!(!name.contains('\\'), "{name}");
            assert!(!name.contains(".."), "{name}");
            assert_eq!(
                std::path::Path::new(&name).components().count(),
                1,
                "{name}"
            );
        }
    }
}
