//! What a screen reader reads out for an icon, and for a window in its list.
//!
//! The owner's sentence: *a screen reader should announce the app name, how many
//! windows it has, whether a window is focused or minimized, and where a
//! selected window sits on the canvas.* Four facts, and this file is where they
//! become one sentence.
//!
//! # The same facts a person looking at the Dock can see
//!
//! **This is not a description of the Dock — it is the Dock's information.**
//! Somebody looking at an icon sees which application it is, that it has
//! windows, that one of them is in front, and that another is put away. A person
//! reading with a screen reader is owed exactly those, in their own language,
//! and nothing extra about pixels.
//!
//! What they are *not* owed is a running commentary. Nothing here announces
//! travel, or a canvas moving, because a person who asked to go somewhere knows
//! they asked.
//!
//! # Why the state is a separate word rather than four sentences
//!
//! *Focused* is added to whichever count sentence applies, rather than doubling
//! every sentence into a focused and an unfocused version. Three counts times
//! two states would be six sentences for a translator to keep consistent, and
//! the inconsistency would show up as one combination reading oddly in one
//! language — which nobody would find, because nobody tests all six by ear.
//!
//! A language that would rather inflect the whole sentence can translate the
//! count sentences and leave the joined word alone; the note on
//! [`crate::words::ANNOUNCED_FOCUSED`] says so.
//!
//! # Minimised, not put aside
//!
//! The rest of alo OS says *put aside*, which is the owner's word and the better
//! one. This says **minimised**, because it is read out to somebody who has used
//! a screen reader on another system and expects that word for that state.
//! Teaching a new vocabulary to a person who cannot see the thing being renamed
//! is a cost paid by them for a tidiness that is ours.

use alo_strings::{Filling, Said, Strings};

use crate::holding::OnTheDock;
use crate::on_the_canvas::Patch;
use crate::previews::Preview;
use crate::window::AppId;
use crate::words::{self, Word};

/// Whether one of an application's windows has the keyboard.
///
/// Named rather than a `bool` because it is read out to somebody who cannot
/// check, and a caller passing the wrong way round would be a person told the
/// opposite of what is true.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focused {
    /// One of this application's windows has the keyboard.
    OneOfItsWindowsIs,
    /// None of them does.
    No,
}

/// What is read out for one icon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Announced {
    /// The sentence carrying the counts.
    counts: Word,
    /// What fills it.
    filling: Filling,
    /// Whether *focused* is added to it.
    focused: bool,
}

impl Announced {
    /// What is read out for this icon.
    #[must_use]
    pub fn of(on_the_dock: &OnTheDock, focused: Focused) -> Self {
        let app = on_the_dock.app().name().to_owned();
        let windows = on_the_dock.how_many_windows();
        let aside = on_the_dock.how_many_put_aside();

        let (counts, filling) = if windows == 0 {
            (words::ANNOUNCED_CLOSED, Filling::of("app", app))
        } else if aside == 0 {
            (
                words::ANNOUNCED_OPEN,
                Filling::of("app", app).and("windows", windows.to_string()),
            )
        } else {
            (
                words::ANNOUNCED_SOME_PUT_ASIDE,
                Filling::of("app", app)
                    .and("windows", windows.to_string())
                    .and("aside", aside.to_string()),
            )
        };

        Self {
            counts,
            filling,
            focused: matches!(focused, Focused::OneOfItsWindowsIs),
        }
    }

    /// The counts, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.counts.key(), &self.filling)
    }

    /// The word added when one of its windows has the keyboard, if one does.
    #[must_use]
    pub fn and_focused(&self, strings: &Strings) -> Option<Said> {
        self.focused
            .then(|| strings.say(&words::ANNOUNCED_FOCUSED.key(), &Filling::nothing()))
    }

    /// Which sentence this is, for a test that wants to know which of the three
    /// was chosen without reading English.
    #[must_use]
    pub const fn which(&self) -> Word {
        self.counts
    }
}

/// Where a window sits, read out so somebody who cannot see the canvas knows
/// where choosing it would take them.
///
/// **A position, not a distance.** The numbers are the window's place on the
/// plane and may be negative, because the canvas extends in every direction —
/// *how far away is it from me* would need a view, and would change every time
/// the person moved.
#[must_use]
pub fn where_it_sits(at: Patch, strings: &Strings) -> Said {
    strings.say(
        &words::ANNOUNCED_WHERE.key(),
        &Filling::of("across", at.corner().x().to_string())
            .and("down", at.corner().y().to_string()),
    )
}

/// Where this preview's window sits.
#[must_use]
pub fn where_a_preview_sits(preview: &Preview, strings: &Strings) -> Said {
    where_it_sits(preview.at(), strings)
}

/// The name of an application, which is never translated.
///
/// Here so that a caller reaching for an application's name in a sentence finds
/// the rule beside it rather than having to remember it.
#[must_use]
pub fn the_name_of(app: &AppId) -> Filling {
    Filling::of("app", app.name().to_owned())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::holding::Holding;
    use crate::on_the_canvas::Spot;
    use crate::testing::{in_english, translated};
    use crate::window::{HowItSits, Window, WindowId};
    use crate::windows::Windows;

    fn app(name: &str) -> AppId {
        AppId::named(name).unwrap()
    }

    fn window(number: u64, of: &str, sits: HowItSits) -> Window {
        Window::of(
            WindowId::numbered(number),
            Some(app(of)),
            "A window",
            Patch::of(Spot::at(0, 0), 800, 600).unwrap(),
            sits,
        )
    }

    /// One icon's row, for an application with this many windows, this many of
    /// them put aside.
    fn an_icon(open: usize, aside: usize) -> OnTheDock {
        let mut holding = Holding::nothing();
        holding.pin(app("Browser"));
        let mut windows = Windows::none();
        for number in 0..open {
            let sits = if number < aside {
                HowItSits::PutAside
            } else {
                HowItSits::OnTheCanvas
            };
            windows.opened(window(number as u64 + 1, "Browser", sits));
        }
        holding.showing(&windows).into_iter().next().unwrap()
    }

    /// **The owner's example, read out:** *Browser, three windows open, one
    /// minimised.*
    #[test]
    fn the_owners_example_is_what_is_read_out() {
        let strings = in_english();
        let said = Announced::of(&an_icon(3, 1), Focused::No).said(&strings);
        assert_eq!(said.text(), "Browser, 3 open, 1 minimised");
        assert!(said.unfilled().is_empty());
    }

    /// **Three sentences, and which one is chosen is a fact about the counts.**
    /// Asked without reading English, so a translation cannot change which
    /// sentence a state produces.
    #[test]
    fn the_sentence_chosen_follows_the_counts() {
        assert_eq!(
            Announced::of(&an_icon(0, 0), Focused::No).which().key(),
            words::ANNOUNCED_CLOSED.key()
        );
        assert_eq!(
            Announced::of(&an_icon(3, 0), Focused::No).which().key(),
            words::ANNOUNCED_OPEN.key()
        );
        assert_eq!(
            Announced::of(&an_icon(3, 2), Focused::No).which().key(),
            words::ANNOUNCED_SOME_PUT_ASIDE.key()
        );
    }

    /// **A pinned application that is closed is announced as not open**, rather
    /// than as zero windows — a count of nothing is a sentence nobody says.
    #[test]
    fn a_closed_application_is_not_announced_as_zero() {
        let strings = in_english();
        let said = Announced::of(&an_icon(0, 0), Focused::No).said(&strings);
        assert_eq!(said.text(), "Browser, not open");
        assert!(!said.text().contains('0'));
    }

    /// **Focused is added to whichever sentence applies**, rather than doubling
    /// every sentence — three counts times two states would be six for a
    /// translator to keep consistent.
    #[test]
    fn focused_is_one_word_added_to_any_of_them() {
        let strings = in_english();
        for (open, aside) in [(0, 0), (3, 0), (3, 1)] {
            let announced = Announced::of(&an_icon(open, aside), Focused::OneOfItsWindowsIs);
            assert_eq!(
                announced.and_focused(&strings).unwrap().text(),
                "focused",
                "{open} open, {aside} aside"
            );
        }
        assert_eq!(
            Announced::of(&an_icon(3, 0), Focused::No).and_focused(&strings),
            None
        );
    }

    /// **All of it is read in the person's own language**, including the word
    /// joined on the end.
    #[test]
    fn every_part_of_it_is_read_in_the_persons_language() {
        let strings = translated(&[
            (
                words::ANNOUNCED_SOME_PUT_ASIDE,
                "{app}, {windows} offen, {aside} minimiert",
            ),
            (words::ANNOUNCED_FOCUSED, "aktiv"),
        ]);
        let announced = Announced::of(&an_icon(3, 1), Focused::OneOfItsWindowsIs);

        let said = announced.said(&strings);
        assert_eq!(said.text(), "Browser, 3 offen, 1 minimiert");
        assert!(said.is_translated());
        assert!(said.unfilled().is_empty());

        let focused = announced.and_focused(&strings).unwrap();
        assert_eq!(focused.text(), "aktiv");
        assert!(focused.is_translated());
    }

    /// **The application's name is never translated**, because it came off this
    /// machine rather than out of a vocabulary.
    #[test]
    fn the_application_name_is_not_translated() {
        let strings = translated(&[(words::ANNOUNCED_OPEN, "{app}, {windows} offen")]);
        let said = Announced::of(&an_icon(2, 0), Focused::No).said(&strings);
        assert!(said.text().starts_with("Browser,"), "{said}");

        // A filling can only be observed by filling something with it, which is
        // the point: it is not a bag of strings to be read back, it is what a
        // sentence is completed from.
        let named = strings.say(
            &words::ANNOUNCED_CLOSED.key(),
            &the_name_of(&app("Blender")),
        );
        assert_eq!(named.text(), "Blender, not open");
        assert!(named.unfilled().is_empty());
    }

    /// **Where a window sits is a position and can be negative**, because the
    /// canvas extends in every direction from its origin.
    #[test]
    fn where_a_window_sits_is_a_position_that_can_be_negative() {
        let strings = in_english();
        let at = Patch::of(Spot::at(-4_200, 1_800), 800, 600).unwrap();
        let said = where_it_sits(at, &strings);
        assert_eq!(said.text(), "on the canvas, -4200 across and 1800 down");
        assert!(said.unfilled().is_empty());
    }

    /// Every sentence this file reaches for is one the crate declares, so none
    /// of them can reach a person as a key.
    #[test]
    fn every_sentence_it_says_is_in_the_vocabulary() {
        let vocabulary = crate::words::dock_words().unwrap();
        for word in [
            words::ANNOUNCED_CLOSED,
            words::ANNOUNCED_OPEN,
            words::ANNOUNCED_SOME_PUT_ASIDE,
            words::ANNOUNCED_FOCUSED,
            words::ANNOUNCED_WHERE,
        ] {
            assert!(
                vocabulary.phrase(&word.key()).is_some(),
                "{} is not collected",
                word.named()
            );
            assert!(word.note().is_some(), "{} has no note", word.named());
        }
    }
}
