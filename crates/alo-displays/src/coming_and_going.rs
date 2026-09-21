//! A screen unplugged and a screen plugged back in: where the windows on it
//! belong, and what happens when it returns.
//!
//! **The moving is the shell's; the decision of where is this crate's.** There
//! is not a window identifier anywhere in this file, and there is not going to
//! be one: which windows exist is `alo-desktops`' and `alo-dividing`'s, and
//! moving them is drawing. What is decided here is the only part a person
//! notices going wrong — *onto which screen*, and *do they come back*.
//!
//! # The rule, in one sentence each
//!
//! **What was on a screen that is unplugged goes to whichever screen is the
//! main one once it is gone.** Not the nearest, which changes when somebody
//! moves a monitor; not the first, which is whichever socket the machine
//! happened to enumerate first. The main screen is the one thing about an
//! arrangement a person has already decided, and it is where a window with
//! nowhere else to go belongs everywhere else in this system.
//!
//! **When it comes back, what was on it goes back to it**, at the place the
//! arrangement now gives it. A screen that is unplugged and plugged in again
//! ends where it started, which is the whole of what *hotplug* has to mean.
//!
//! **And the chain holds.** Unplug the screen the windows were moved onto, and
//! they move again with the ones that were already there — so plugging the
//! first screen back in still brings its own windows home.

use alo_strings::{Filling, Said, Strings, Word};

use crate::arrangement::NotArranged;
use crate::identity::Identity;
use crate::words;

/// Why a screen could not be unplugged or plugged in.
///
/// There is no `Display`: the only road to words is [`NotAttached::said`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAttached {
    /// No screen is in that socket.
    NotPluggedIn,
    /// A screen is already in that socket.
    AlreadyPluggedIn,
    /// It is the only screen this machine has, so what is on it has nowhere to
    /// go.
    NothingRemains,
    /// What is left is not an arrangement.
    TheyDoNotArrange(NotArranged),
}

impl NotAttached {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self {
            Self::NotPluggedIn => words::NOT_PLUGGED_IN,
            Self::AlreadyPluggedIn => words::ALREADY_PLUGGED_IN,
            Self::NothingRemains => words::NOTHING_REMAINS,
            Self::TheyDoNotArrange(why) => why.word(),
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::TheyDoNotArrange(why) => why.said(strings),
            Self::NotPluggedIn | Self::AlreadyPluggedIn | Self::NothingRemains => {
                strings.say(&self.word().key(), &Filling::nothing())
            }
        }
    }
}

/// A screen has gone, and what was on it belongs somewhere else now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moved {
    /// The screen that was unplugged.
    from: Identity,
    /// The screen what was on it belongs on now.
    onto: Identity,
    /// Every other screen whose windows were already sitting on the one that
    /// has gone, and which move with them.
    carrying: Vec<Identity>,
}

impl Moved {
    /// What was on this screen moves.
    pub(crate) const fn of(from: Identity, onto: Identity, carrying: Vec<Identity>) -> Self {
        Self {
            from,
            onto,
            carrying,
        }
    }

    /// The screen that was unplugged.
    #[must_use]
    pub const fn from(&self) -> &Identity {
        &self.from
    }

    /// The screen what was on it belongs on now.
    #[must_use]
    pub const fn onto(&self) -> &Identity {
        &self.onto
    }

    /// Every other screen whose windows were already sitting on the one that
    /// has gone, and which move with them.
    pub fn carrying(&self) -> impl Iterator<Item = &Identity> {
        self.carrying.iter()
    }

    /// The string this crate declares for what a person is told.
    #[must_use]
    pub const fn word(&self) -> Word {
        words::WINDOWS_MOVED
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &self.word().key(),
            &self.onto.named_in(
                "other",
                self.from.named_in("display", Filling::nothing(), strings),
                strings,
            ),
        )
    }
}

/// A screen is back, and what was on it goes back to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameBack {
    /// The screen that is back.
    back: Identity,
    /// Where what was on it has been sitting, when that screen is still
    /// attached to say so.
    were_on: Option<Identity>,
}

impl CameBack {
    /// This screen is back, from there.
    pub(crate) const fn of(back: Identity, were_on: Option<Identity>) -> Self {
        Self { back, were_on }
    }

    /// The screen that is back.
    #[must_use]
    pub const fn back(&self) -> &Identity {
        &self.back
    }

    /// Where what was on it has been sitting, when that screen is still
    /// attached to say so.
    #[must_use]
    pub const fn were_on(&self) -> Option<&Identity> {
        self.were_on.as_ref()
    }

    /// Whether anything at all goes back to it.
    ///
    /// A screen plugged in for the first time in this session carries nothing
    /// home, and the shell has nothing to move.
    #[must_use]
    pub const fn anything_goes_back(&self) -> bool {
        self.were_on.is_some()
    }

    /// The string this crate declares for what a person is told, where there is
    /// anything to tell them.
    #[must_use]
    pub const fn word(&self) -> Option<Word> {
        if self.were_on.is_some() {
            Some(words::WINDOWS_CAME_BACK)
        } else {
            None
        }
    }

    /// What this says, in the language the person reads — or nothing, for a
    /// screen that carries nothing home. Never fails and never panics.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Option<Said> {
        self.word().map(|word| {
            strings.say(
                &word.key(),
                &self.back.named_in("display", Filling::nothing(), strings),
            )
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, the_home_screen, the_laptop, the_office_screen};

    /// **What a person is told when a screen goes names both screens**, which
    /// is the whole of what they need in order to go and find their window.
    #[test]
    fn what_a_person_is_told_when_a_screen_goes_names_both_screens() {
        let strings = in_english();
        let moved = Moved::of(the_office_screen(), the_laptop(), vec![the_home_screen()]);
        let said = moved.said(&strings);
        assert!(said.text().contains("Dell U2720Q"), "{said}");
        assert!(said.text().contains("Built-in screen"), "{said}");
        assert!(!said.text().contains("eDP-1"), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert_eq!(moved.from(), &the_office_screen());
        assert_eq!(moved.onto(), &the_laptop());
        assert_eq!(moved.carrying().count(), 1);
    }

    /// **A screen that carries nothing home says nothing**, because a sentence
    /// every time anybody plugs a screen in is a sentence nobody reads.
    #[test]
    fn a_screen_that_carries_nothing_home_says_nothing() {
        let strings = in_english();
        let first_time = CameBack::of(the_office_screen(), None);
        assert!(!first_time.anything_goes_back());
        assert_eq!(first_time.word(), None);
        assert_eq!(first_time.said(&strings), None);

        let returning = CameBack::of(the_office_screen(), Some(the_laptop()));
        assert!(returning.anything_goes_back());
        assert_eq!(returning.were_on(), Some(&the_laptop()));
        let said = returning.said(&strings).unwrap();
        assert!(said.text().contains("Dell U2720Q"), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
    }

    /// Every refusal says a declared sentence, and the one that wraps an
    /// arrangement's refusal says the arrangement's.
    #[test]
    fn every_refusal_says_a_declared_sentence() {
        let strings = in_english();
        for refused in [
            NotAttached::NotPluggedIn,
            NotAttached::AlreadyPluggedIn,
            NotAttached::NothingRemains,
            NotAttached::TheyDoNotArrange(NotArranged::NoMainScreen),
        ] {
            let said = refused.said(&strings);
            assert!(words::EVERY_WORD.contains(&refused.word()), "{refused:?}");
            assert!(said.unfilled().is_empty(), "{refused:?}: {said}");
            assert!(!said.is_a_bug(), "{refused:?}: {said}");
        }
        assert_eq!(
            NotAttached::TheyDoNotArrange(NotArranged::NoMainScreen).word(),
            NotArranged::NoMainScreen.word()
        );
    }
}
