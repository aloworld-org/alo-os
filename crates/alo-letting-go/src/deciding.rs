//! Which of a person's kept turns go, and in what order — decided, with
//! nothing touched.
//!
//! # The arithmetic is not here, and that is the point
//!
//! [`alo_keeping_up::HowFarBack::how_many_it_still_reaches`] already decides
//! precisely which of a machine's changing turns a window still reaches, from a
//! list of how many days ago each was, newest first. It is
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s first
//! accepted term and it is right. This module **asks** it and does no
//! arithmetic of its own about a window, because a second implementation of
//! *whichever ends first* is two answers to one question and the one a person
//! would meet is whichever ran last.
//!
//! What is decided here is the thing that was missing: the **order** the rest
//! go in, and where the line is drawn when the disk is what is short rather
//! than the window.
//!
//! # Oldest first, in one list
//!
//! ADR 0045's second term: *below a named amount of free space the oldest
//! snapshots go first.* So [`WhatGoes`] is two lists in the order the removal
//! walks them — everything the window no longer reaches, oldest first; and then
//! everything it still reaches, **also oldest first**, which is only walked at
//! all while the disk is under [`crate::THE_FLOOR`].
//!
//! # And why nothing here says how much the removal will free
//!
//! Because nothing can. A read-only snapshot on `btrfs` shares every block with
//! the subvolume it was taken from until one of them changes, so what removing
//! one frees is between nought and the whole of it and is knowable only
//! afterwards. A module that answered *removing these three will be enough*
//! would be guessing at exactly the moment a person's disk is full. So the
//! order is decided here and the *stopping* is `crate::sweeping`'s: it removes
//! one, asks the filesystem again, and stops the moment there is room.

use alo_keeping_up::HowFarBack;

/// Which of one person's kept turns go, in the order they are removed.
///
/// Positions into the list the decision was made from, which is newest first.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WhatGoes {
    /// Everything the window no longer reaches, oldest first. These go
    /// whatever the disk says.
    outside_the_window: Vec<usize>,
    /// Everything it still reaches, oldest first. These go only while the disk
    /// is below the floor, one at a time, and only for as long as it is.
    then_for_room: Vec<usize>,
}

impl WhatGoes {
    /// What goes, from the person's `window` and how many days ago each of
    /// their kept turns was, **newest first** — the order
    /// [`alo_keeping_up::HowFarBack::how_many_it_still_reaches`] asks for.
    #[must_use]
    pub fn decided(window: HowFarBack, days_ago_newest_first: &[u32]) -> Self {
        let still_reached = window.how_many_it_still_reaches(days_ago_newest_first);
        Self {
            // Oldest first is the end of the list first, so each half is walked
            // backwards from where the window drew its line.
            outside_the_window: (still_reached..days_ago_newest_first.len()).rev().collect(),
            then_for_room: (0..still_reached).rev().collect(),
        }
    }

    /// Everything the window no longer reaches, oldest first.
    #[must_use]
    pub fn outside_the_window(&self) -> &[usize] {
        &self.outside_the_window
    }

    /// Everything the window still reaches, oldest first — taken only while
    /// the disk is below the floor.
    #[must_use]
    pub fn then_for_room(&self) -> &[usize] {
        &self.then_for_room
    }

    /// Whether the window alone takes nothing.
    #[must_use]
    pub fn nothing_is_outside_the_window(&self) -> bool {
        self.outside_the_window.is_empty()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **What the window no longer reaches goes, and what it reaches stays** —
    /// read off the same answer `alo-keeping-up` gives, never worked out again
    /// here.
    #[test]
    fn what_the_window_no_longer_reaches_goes_and_the_rest_stays() {
        let window = HowFarBack::of(7, 50).unwrap();
        // Newest first: today, two days ago, eight days ago, thirty days ago.
        let goes = WhatGoes::decided(window, &[0, 2, 8, 30]);
        assert_eq!(goes.outside_the_window(), [3, 2]);
        assert_eq!(goes.then_for_room(), [1, 0]);
    }

    /// **The oldest go first**, in both halves, which is ADR 0045's second
    /// term walked rather than argued: the end of a newest-first list is the
    /// oldest thing in it.
    #[test]
    fn the_oldest_go_first_in_both_halves() {
        let window = HowFarBack::of(7, 50).unwrap();
        let goes = WhatGoes::decided(window, &[0, 1, 2, 9, 10, 11]);
        assert_eq!(goes.outside_the_window(), [5, 4, 3]);
        assert_eq!(goes.then_for_room(), [2, 1, 0]);
    }

    /// **A machine inside its window loses nothing to the window**, and what
    /// it would lose to a full disk is still the oldest first.
    #[test]
    fn a_machine_inside_its_window_loses_nothing_to_the_window() {
        let window = HowFarBack::of(7, 50).unwrap();
        let goes = WhatGoes::decided(window, &[0, 1, 2]);
        assert!(goes.nothing_is_outside_the_window());
        assert_eq!(goes.then_for_room(), [2, 1, 0]);
    }

    /// **A machine that kept nothing has nothing to let go of.**
    #[test]
    fn a_machine_that_kept_nothing_lets_nothing_go() {
        let goes = WhatGoes::decided(HowFarBack::AS_SHIPPED, &[]);
        assert_eq!(goes, WhatGoes::default());
        assert!(goes.nothing_is_outside_the_window());
        assert!(goes.then_for_room().is_empty());
    }

    /// **A wider window keeps what a narrower one would have taken.** The
    /// person's one setting, walked: the same machine, the same day, two
    /// windows, and the difference is exactly the snapshots between them.
    #[test]
    fn a_wider_window_keeps_what_a_narrower_one_would_have_taken() {
        let days_ago = [0, 2, 8, 30, 100];
        let narrow = WhatGoes::decided(HowFarBack::of(7, 50).unwrap(), &days_ago);
        let wide = WhatGoes::decided(HowFarBack::of(365, 50).unwrap(), &days_ago);

        assert_eq!(narrow.outside_the_window(), [4, 3, 2]);
        assert!(wide.nothing_is_outside_the_window());
        for kept in wide.then_for_room() {
            assert!(
                !narrow.then_for_room().contains(kept) || narrow.then_for_room().contains(kept),
                "the two halves must still name every turn between them"
            );
        }
        // Every turn is named exactly once by each decision, whichever half.
        for goes in [&narrow, &wide] {
            let mut every: Vec<usize> = goes
                .outside_the_window()
                .iter()
                .chain(goes.then_for_room())
                .copied()
                .collect();
            every.sort_unstable();
            assert_eq!(every, (0..days_ago.len()).collect::<Vec<usize>>());
        }
    }

    /// **The turns half of the window ends it too**, since it is
    /// `alo-keeping-up`'s `&&` that is being asked: a machine used hard loses
    /// the oldest to the count of changing turns rather than to the days.
    #[test]
    fn a_busy_machine_loses_the_oldest_to_the_count_of_turns() {
        let window = HowFarBack::of(7, 3).unwrap();
        let goes = WhatGoes::decided(window, &[0, 0, 0, 0, 0]);
        assert_eq!(goes.outside_the_window(), [4, 3]);
        assert_eq!(goes.then_for_room(), [2, 1, 0]);
    }
}
