//! The desk a machine wakes up at.
//!
//! A laptop suspended at home and opened at the office is the commonest thing
//! a resume has to get right, and it is the one road through it that the
//! events cannot carry. `alo_displays::Attached` learns about screens one
//! cable at a time — one unplugged, one plugged in — and **a machine that was
//! asleep saw none of them.** It wakes holding the set it went to sleep with:
//! two screens that are no longer plugged into anything, or one screen where
//! there are now three.
//!
//! So [`Woke::the_desk`] asks the whole question again from what is reported
//! **now**. It is asked here, at the resume, rather than left to every caller
//! to remember — a machine whose screens are right only when the shell thought
//! to ask is a machine whose screens are wrong.
//!
//! # Nothing about screens is decided in this crate
//!
//! Which screens these are, where each goes, how large each draws and where
//! what was open on a screen that has gone belongs are all
//! `alo_displays::Attached::resumed_to`'s, and that crate is read and never
//! edited. What is this crate's is the **when**: at a wake, once, from the set
//! the caller was handed.
//!
//! # Nothing polls and nothing watches
//!
//! The screens are read once, at the resume, out of what the caller was given.
//! There is no thread here, no timer and nothing listening: a machine that
//! keeps asking what is plugged into it is a machine reporting on the person
//! using it.

use alo_displays::{Attached, NotArranged, Reported, Resumed};

use crate::waking::Woke;

impl<N> Woke<N> {
    /// The screens in front of the person now, from the whole set the machine
    /// reports — never a cable at a time, because a machine that was asleep
    /// saw no cable move.
    ///
    /// `attached` is the set the machine went to sleep with, and it is brought
    /// up to what is reported now. The answer says which screens have gone and
    /// where what was open on each belongs, which have come back, and whether
    /// the person is told anything at all — a machine that wakes at the same
    /// desk moves nothing and says nothing.
    ///
    /// # Errors
    /// [`NotArranged::NoScreens`] when the machine wakes with nothing plugged
    /// into it, and then `attached` is left exactly as it was: the screens it
    /// went to sleep with are what the next resume is compared against, and a
    /// dock that has not woken yet must not cost the person their arrangement.
    pub fn the_desk(
        &self,
        attached: &mut Attached,
        reported: Vec<Reported>,
        remembered: &alo_displays::Changes,
    ) -> Result<Resumed, NotArranged> {
        attached.resumed_to(reported, remembered)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_displays::{Changes as Screens, Note, Panel, Socket, Support};
    use alo_locking::Seat;
    use alo_overlay::Summoning;

    use super::*;
    use crate::changes::Settings;
    use crate::deciding::{Decided, Why, asked};
    use crate::going::Slept;
    use crate::holding::Holding;
    use crate::lid::Displays;
    use crate::testing::{Holds, TheMachine, anna, hour, noon};

    /// The laptop's own panel, which says nothing about itself.
    fn a_laptop() -> Reported {
        Reported::of(
            Socket::named("eDP-1").unwrap(),
            None,
            (1920, 1080),
            Some((294, 165)),
        )
        .unwrap()
    }

    /// The screen at the office, which says what it is.
    fn an_office_screen() -> Reported {
        Reported::of(
            Socket::named("DP-1").unwrap(),
            Some(Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap()),
            (3840, 2160),
            Some((596, 336)),
        )
        .unwrap()
    }

    /// Close the lid on Anna's open seat and let the machine sleep, then wake
    /// it a quarter of an hour later.
    fn slept_and_woke() -> Woke<String> {
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let mut holding: Holding<crate::testing::Hold> = Holding::none();
        let Decided::Sleeps(going) = asked(
            Why::LidClosed,
            &Settings::shipped(),
            Displays::OnlyItsOwn,
            &mut holding,
            &alo_capability::Grants::default(),
            noon(),
        ) else {
            unreachable!("a closed lid with nothing attached sleeps");
        };
        let Slept::Asleep(asleep) = going.carried_out(
            Seat::opened(anna()),
            &mut Summoning::closed(),
            &mut logind,
            noon(),
        ) else {
            unreachable!("the machine sleeps");
        };
        asleep.woke(noon() + hour() / 4)
    }

    /// **A resume is where the desk is asked.** The machine slept with only
    /// its own screen and woke with another one plugged in, and the answer
    /// comes back from the wake rather than from anything the caller had to
    /// remember to do.
    #[test]
    fn the_desk_is_asked_at_the_resume() {
        let woke = slept_and_woke();
        let remembered = Screens::untouched();
        let mut attached =
            Attached::now(vec![a_laptop()], &remembered, Support::Fractional).unwrap();

        let resumed = woke
            .the_desk(
                &mut attached,
                vec![a_laptop(), an_office_screen()],
                &remembered,
            )
            .unwrap();

        assert!(!resumed.the_same_desk());
        assert_eq!(resumed.note(), Some(&Note::TheDeskChanged));
        assert_eq!(attached.each().count(), 2);
        assert!(woke.seat().is_locked(), "and it is still locked");
    }

    /// **A machine that wakes with nothing plugged in is refused**, and keeps
    /// the screens it went to sleep with.
    #[test]
    fn a_resume_with_nothing_plugged_in_is_refused_and_keeps_the_screens() {
        let woke = slept_and_woke();
        let remembered = Screens::untouched();
        let mut attached = Attached::now(
            vec![a_laptop(), an_office_screen()],
            &remembered,
            Support::Fractional,
        )
        .unwrap();

        assert_eq!(
            woke.the_desk(&mut attached, Vec::new(), &remembered),
            Err(NotArranged::NoScreens)
        );
        assert_eq!(attached.each().count(), 2);
        assert!(!attached.notes().contains(&Note::TheDeskChanged));
    }
}
