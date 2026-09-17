//! **What a call application may see, and for how long.**
//!
//! Task 5 of `docs/autonomy/v0-5-capture-and-the-room-plan.md`. Somebody in a
//! meeting shares their screen. The network half of the call is the
//! application's; **what it may see, and showing that it is seeing it, is alo
//! OS's.**
//!
//! # The picking cannot be pre-answered
//!
//! There is no *always allow this application to see my screen*. A grant lets an
//! application **ask**; every share is a fresh choice of what it gets, made at
//! the moment of sharing, by the person.
//!
//! The reason is that the thing being shared is not a capability, it is a
//! moment: the screen at half past three, with whatever happens to be on it.
//! A person who agreed once agreed to what was on the screen then. [`Shared`]
//! therefore has no constructor that takes a remembered answer, and
//! [`Shared::nothing`] is what a call gets until somebody picks.
//!
//! # A window is not the screen, and the indicator says which
//!
//! *Sharing* is not one fact. Sharing one window is a promise that the rest of
//! the screen — the other meeting in another window, the mail, the file
//! somebody's name is in — is not travelling. So [`Shared::what_it_can_see`] is
//! what the indicator shows, and it is the picked thing rather than the word
//! *screen*.
//!
//! # Notifications do not arrive on a shared screen
//!
//! While a screen is shared, a notification on it is somebody else's message
//! read aloud to a meeting — and that somebody is not in the meeting, did not
//! agree, and will never know it happened. The same shape as the microphone in
//! the room, and it is held here by [`Shared::notifications_may_be_shown`]
//! answering **false** for as long as the share lasts.
//!
//! A person sharing one window is protected too: notifications are drawn by the
//! shell over whatever is there, so *only a window is shared* is not a defence.

use crate::what::What;
use crate::words::{self, Word};

/// **What a call application can see right now.**
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Shared {
    /// Nothing. Where every share begins, and where it returns when it ends.
    #[default]
    Nothing,
    /// Exactly what the person picked, this time.
    This(What),
}

impl Shared {
    /// Nothing is shared.
    #[must_use]
    pub const fn nothing() -> Self {
        Self::Nothing
    }

    /// **The person picked this, now.**
    ///
    /// The only way to a share. It takes what was picked and nothing else — no
    /// application, no remembered answer, no *as before*: whoever calls this
    /// has just watched somebody choose.
    #[must_use]
    pub const fn the_person_picked(what: What) -> Self {
        Self::This(what)
    }

    /// **What the application can see**, or [`None`] while nothing is shared.
    ///
    /// What the indicator names. Not *the screen*: the thing that was picked,
    /// because sharing one window is a promise about everything else.
    #[must_use]
    pub const fn what_it_can_see(&self) -> Option<&What> {
        match self {
            Self::Nothing => None,
            Self::This(what) => Some(what),
        }
    }

    /// Whether anything is being shared at all.
    #[must_use]
    pub const fn is_sharing(&self) -> bool {
        matches!(self, Self::This(_))
    }

    /// **Whether a notification may be drawn while this is shared.**
    ///
    /// False for the whole of a share, whatever was picked. A notification is
    /// somebody else's message, and a meeting is not who they sent it to.
    #[must_use]
    pub const fn notifications_may_be_shown(&self) -> bool {
        !self.is_sharing()
    }

    /// **End it.** One act, and what the application can see is nothing.
    #[must_use]
    pub fn ended(self) -> Self {
        Self::Nothing
    }

    /// What a person reads about the state of it.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self {
            Self::Nothing => words::SHARING_NOTHING,
            Self::This(What::TheWholeScreen) => words::SHARING_THE_WHOLE_SCREEN,
            Self::This(What::OneWindow(_)) => words::SHARING_ONE_WINDOW,
            Self::This(What::APartOfIt(_)) => words::SHARING_A_PART_OF_THE_SCREEN,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::region::Region;
    use crate::window::{Window, WindowId};

    fn a_window() -> Window {
        Window::of(
            WindowId::recorded(3),
            &crate::testing::anna(),
            Region::of(0, 0, 800, 600).expect("a region"),
        )
    }

    /// **A call sees nothing until somebody picks**, and there is no way to
    /// begin a share from a remembered answer.
    #[test]
    fn a_call_sees_nothing_until_a_person_picks_something() {
        let before = Shared::default();
        assert_eq!(before, Shared::nothing());
        assert!(!before.is_sharing());
        assert_eq!(before.what_it_can_see(), None);
    }

    /// **What the indicator names is what was picked**, not the word *screen*.
    #[test]
    fn the_indicator_names_the_thing_that_was_picked() {
        let one_window = Shared::the_person_picked(What::OneWindow(a_window()));
        assert!(matches!(
            one_window.what_it_can_see(),
            Some(What::OneWindow(_))
        ));
        assert_eq!(one_window.word().key(), words::SHARING_ONE_WINDOW.key());

        let all_of_it = Shared::the_person_picked(What::TheWholeScreen);
        assert_eq!(
            all_of_it.word().key(),
            words::SHARING_THE_WHOLE_SCREEN.key(),
            "sharing a window and sharing everything read the same, and they are not the same \
             promise"
        );

        let part = Shared::the_person_picked(What::APartOfIt(
            Region::of(0, 0, 400, 300).expect("a region"),
        ));
        assert_eq!(part.word().key(), words::SHARING_A_PART_OF_THE_SCREEN.key());
    }

    /// **No notification is drawn while anything is shared** — whatever was
    /// picked, and including one window.
    #[test]
    fn notifications_do_not_arrive_on_a_shared_screen() {
        assert!(Shared::nothing().notifications_may_be_shown());
        for shared in [
            Shared::the_person_picked(What::TheWholeScreen),
            Shared::the_person_picked(What::OneWindow(a_window())),
            Shared::the_person_picked(What::APartOfIt(
                Region::of(0, 0, 100, 100).expect("a region"),
            )),
        ] {
            assert!(
                !shared.notifications_may_be_shown(),
                "{shared:?}: somebody else's message would be drawn in front of a meeting"
            );
        }
    }

    /// **Ending it is one act**, and afterwards the call sees nothing.
    #[test]
    fn ending_a_share_leaves_the_call_seeing_nothing() {
        let sharing = Shared::the_person_picked(What::TheWholeScreen);
        let ended = sharing.ended();
        assert_eq!(ended, Shared::nothing());
        assert_eq!(ended.what_it_can_see(), None);
        assert!(
            ended.notifications_may_be_shown(),
            "notifications are still held back after the share ended"
        );
    }
}
