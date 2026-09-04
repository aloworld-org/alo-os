//! Where one turn may reach: several places rather than one.
//!
//! [`Place`] is one thing on a disk. This is what a turn is bound to, and it
//! holds up to [`PLACES`] of them because **one execution names more than one
//! path**: `move_file` names the file and the folder it is going into,
//! `archive_folder` names the folder and where the archive goes, and a change
//! creates a name in a folder nobody typed out.
//!
//! # The two answers in the queue were not alternatives
//!
//! Item 26b asked whether the map's value should become several places, *or*
//! the bound should be made narrower than the turn's grant and be the places
//! this execution named. They read as a choice and they are not one: a turn's
//! grants can cover several folders, and one execution can name two paths, so
//! **both answers need more than one place in the entry**. The narrower answer
//! is the one alo OS takes — least privilege, and `alo-bounding`'s `places.rs`
//! is where it is written down — and this file is the mechanism either would
//! have needed.
//!
//! # Four, and why the number is written here
//!
//! The widest call on this machine names two paths, and a change creates a name
//! in the folder each of them sits in, so four is the most a single execution
//! can need. It is a number rather than a guess, and it is in this crate for the
//! reason everything else here is: the daemon writes the entry and a program
//! inside the kernel reads it, and a width that drifted by one slot between them
//! would refuse the wrong files quietly. Changing [`PLACES`] moves both halves
//! or compiles in neither.
//!
//! A call that needed more is refused rather than bounded to the first four —
//! `alo-bounding`'s `NotBounded::TooManyPlaces` — because a bound is not a thing
//! to give somebody most of.
//!
//! # A count that cannot be read is read as fewer, never as more
//!
//! The count travels in the entry rather than being inferred from a slot that
//! looks empty, because a place made of zeroes is a real place — whichever
//! filesystem is numbered nothing — and *empty* would then be a value somebody
//! could arrange to be granted. [`Bounds::of_words`] therefore cannot fail: a
//! count larger than [`PLACES`] is read as [`PLACES`], and a count of nothing is
//! a turn bound to nowhere, which refuses every open it makes. Both directions
//! fail closed, which is what a value read out of shared memory has to do.

use crate::bound::Place;

/// The most places one turn can be bound to.
///
/// See this module's own documentation for where the number comes from and what
/// happens to a call that would need more.
pub const PLACES: usize = 4;

/// How many words of the map one bound is: the count, then two per place.
///
/// The layout is decided here and both halves go through [`Bounds::words`] and
/// [`Bounds::of_words`] rather than laying out bytes of their own.
pub const WORDS: usize = 1 + PLACES * 2;

/// A place kept in a slot nothing is looking at.
///
/// Not a sentinel: nothing compares against it, because the count is what
/// decides how far the walk looks. It is here so that two bounds holding the
/// same places are the same value whichever door they came through.
const NOWHERE: Place = Place::of(0, 0);

/// Everywhere one turn may reach.
///
/// The value of one entry in the kernel's map. Equality is about the places
/// held, which is why the slots beyond the count are cleared at both doors
/// rather than left as whatever happened to be in them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    /// The places, with everything from `how_many` onwards [`NOWHERE`].
    held: [Place; PLACES],

    /// How many of them are a turn's, which is never more than [`PLACES`].
    how_many: usize,
}

impl Bounds {
    /// A bound over the places given, in the order they were given.
    ///
    /// [`None`] when there are none of them — a boundary around nothing is not
    /// a narrower grant, it is a turn that cannot open anything — and [`None`]
    /// when there are more than [`PLACES`]. The caller is the one with words for
    /// either, so this answers with an absence rather than a reason.
    ///
    /// ```
    /// use alo_bounding_map::{Bounds, Place, PLACES};
    ///
    /// let invoices = Place::of(64, 100);
    /// let archive = Place::of(64, 200);
    /// let bound = Bounds::of(&[invoices, archive]).expect("two is not too many");
    /// assert!(bound.holds(invoices));
    /// assert!(bound.holds(archive));
    /// assert!(!bound.holds(Place::of(64, 300)));
    ///
    /// assert!(Bounds::of(&[]).is_none());
    /// assert!(Bounds::of(&[invoices; PLACES + 1]).is_none());
    /// ```
    #[must_use]
    pub fn of(places: &[Place]) -> Option<Self> {
        if places.is_empty() || places.len() > PLACES {
            return None;
        }
        let mut held = [NOWHERE; PLACES];
        for (slot, place) in held.iter_mut().zip(places) {
            *slot = *place;
        }
        Some(Self {
            held,
            how_many: places.len(),
        })
    }

    /// A bound over one place.
    ///
    /// The case that cannot fail, and it is here rather than at each caller so
    /// that nobody has to write down what to do when one place turns out to be
    /// none of them or more than [`PLACES`]. Three of the six verbs name one
    /// path, and a turn under ADR 0001 §4's grant over a single document names
    /// that document.
    #[must_use]
    pub const fn of_one(place: Place) -> Self {
        Self {
            held: [place, NOWHERE, NOWHERE, NOWHERE],
            how_many: 1,
        }
    }

    /// Whether `place` is one of the places this turn was bound to.
    ///
    /// Asked once per step of the walk in [`crate::reaches`], against every
    /// place at once: the walk is what costs, and comparing four pairs of
    /// numbers is free beside a read of kernel memory.
    #[must_use]
    pub fn holds(&self, place: Place) -> bool {
        self.each().any(|here| here == place)
    }

    /// The places, and nothing beyond the count.
    pub fn each(&self) -> impl Iterator<Item = Place> + '_ {
        self.held.iter().copied().take(self.how_many)
    }

    /// How many places this turn may reach.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.how_many
    }

    /// Whether this bound reaches nowhere at all.
    ///
    /// Never true of a bound [`Bounds::of`] made. It is true of one read back
    /// out of an entry whose count was nothing, and such a turn is refused every
    /// open it makes — which is the direction an unreadable entry has to fail
    /// in.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.how_many == 0
    }

    /// The value as the map holds it: the count, then two words per place.
    ///
    /// The order is the only thing keeping two separately compiled programs
    /// talking about the same folders, so it is decided here and nowhere else.
    #[must_use]
    pub const fn words(&self) -> [u64; WORDS] {
        let [first, second, third, fourth] = self.held;
        let [device_one, inode_one] = first.words();
        let [device_two, inode_two] = second.words();
        let [device_three, inode_three] = third.words();
        let [device_four, inode_four] = fourth.words();
        [
            self.how_many as u64,
            device_one,
            inode_one,
            device_two,
            inode_two,
            device_three,
            inode_three,
            device_four,
            inode_four,
        ]
    }

    /// A bound read back out of the map.
    ///
    /// Cannot fail, and clamps rather than refusing: a count larger than
    /// [`PLACES`] is read as [`PLACES`] and a count of nothing stays nothing, so
    /// an entry that has been read wrongly bounds a turn to *fewer* places than
    /// somebody meant and never to more. The alternative — answering [`None`] —
    /// would reach the kernel half as *this cgroup is not a turn*, which is the
    /// one answer that allows everything.
    #[must_use]
    pub const fn of_words(words: [u64; WORDS]) -> Self {
        let [
            how_many,
            device_one,
            inode_one,
            device_two,
            inode_two,
            device_three,
            inode_three,
            device_four,
            inode_four,
        ] = words;
        let how_many = if how_many < PLACES as u64 {
            how_many as usize
        } else {
            PLACES
        };
        let held = [
            take([device_one, inode_one], how_many > 0),
            take([device_two, inode_two], how_many > 1),
            take([device_three, inode_three], how_many > 2),
            take([device_four, inode_four], how_many > 3),
        ];
        Self { held, how_many }
    }
}

/// One slot, if the count says a turn was bound to it.
const fn take(words: [u64; 2], counted: bool) -> Place {
    if counted {
        Place::of_words(words)
    } else {
        NOWHERE
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The map is two programs sharing memory, so a round trip through it has to
    /// be exactly the identity — and in the order written here, because the
    /// kernel half reads the same words by position.
    #[test]
    fn a_bound_survives_the_map_unchanged() {
        let bound = Bounds::of(&[Place::of(0x0080_0002, 4_198_531), Place::of(27, 12)])
            .expect("two is not too many");
        assert_eq!(Bounds::of_words(bound.words()), bound);
        assert_eq!(
            bound.words(),
            [2, 0x0080_0002, 4_198_531, 27, 12, 0, 0, 0, 0]
        );
        assert_eq!(bound.len(), 2);
        assert!(!bound.is_empty());
    }

    /// Every width from one place up to the most there are, because the count is
    /// what the walk looks at and an off-by-one in it is a grant to somewhere
    /// nobody named.
    #[test]
    fn every_width_up_to_the_most_there_are_survives_the_map() {
        let all = [
            Place::of(64, 100),
            Place::of(64, 200),
            Place::of(65, 100),
            Place::of(66, 900),
        ];
        for width in 1..=PLACES {
            let some = all.get(..width).expect("PLACES of them were written above");
            let bound = Bounds::of(some).expect("this is not too many");
            assert_eq!(bound.len(), width);
            assert_eq!(Bounds::of_words(bound.words()), bound);
            assert_eq!(bound.each().count(), width);
            for place in some {
                assert!(bound.holds(*place), "{width}: {place:?}");
            }
            for beyond in all.iter().skip(width) {
                assert!(!bound.holds(*beyond), "{width}: {beyond:?}");
            }
        }
    }

    /// A bound around nothing is not a narrower grant, and more places than one
    /// entry holds is not a bound to give somebody most of. Both are refused
    /// here, where the daemon can say why.
    #[test]
    fn nowhere_and_too_many_are_both_refused() {
        assert!(Bounds::of(&[]).is_none());
        assert!(Bounds::of(&[Place::of(64, 100); PLACES + 1]).is_none());
        assert!(Bounds::of(&[Place::of(64, 100); PLACES]).is_some());
    }

    /// **An entry that cannot be read bounds a turn to fewer places, never to
    /// more.** A count above the width is read as the width, and one of nothing
    /// stays nothing — a turn that opens nothing at all, which is the direction
    /// this has to fail in.
    #[test]
    fn a_count_that_makes_no_sense_is_read_as_fewer_places() {
        let too_many = Bounds::of_words([99, 64, 100, 64, 200, 65, 100, 66, 900]);
        assert_eq!(too_many.len(), PLACES);
        assert!(too_many.holds(Place::of(66, 900)));

        let none_at_all = Bounds::of_words([0, 64, 100, 64, 200, 65, 100, 66, 900]);
        assert!(none_at_all.is_empty());
        assert!(
            !none_at_all.holds(Place::of(64, 100)),
            "a turn whose count was unreadable was granted a place anyway"
        );
        assert_eq!(none_at_all.each().count(), 0);
    }

    /// A slot beyond the count is nowhere at both doors, so two bounds holding
    /// the same places are the same value however they arrived.
    ///
    /// Worth asserting rather than assuming: the slot the kernel reads past the
    /// count is real memory with real numbers in it, and a comparison that
    /// included it would make one turn's bound depend on what the previous
    /// turn's entry left behind.
    #[test]
    fn what_is_past_the_count_is_not_part_of_the_bound() {
        let one = Bounds::of(&[Place::of(64, 100)]).expect("one is not too many");
        let with_rubbish_after_it = Bounds::of_words([1, 64, 100, 7, 7, 7, 7, 7, 7]);
        assert_eq!(with_rubbish_after_it, one);
        assert!(!with_rubbish_after_it.holds(Place::of(7, 7)));
        assert_eq!(with_rubbish_after_it.words(), one.words());
    }

    /// The order the places were given in is the order they are held in. Nothing
    /// depends on it today, and a value read out of shared memory that quietly
    /// reordered itself is a thing somebody would eventually depend on.
    #[test]
    fn the_places_come_back_in_the_order_they_were_given() {
        let places = [Place::of(64, 300), Place::of(64, 100), Place::of(65, 2)];
        let bound = Bounds::of(&places).expect("three is not too many");
        assert!(bound.each().eq(places));
    }

    /// One place is the same bound whichever door it came through, so there is
    /// one meaning of *a turn bounded to one folder* rather than two that
    /// currently agree — and the door that cannot fail is the one a caller with
    /// a single path uses.
    #[test]
    fn the_door_that_cannot_fail_makes_the_same_bound() {
        let place = Place::of(64, 100);
        assert_eq!(Bounds::of_one(place), Bounds::of(&[place]).expect("one"));
        assert_eq!(Bounds::of_one(place).len(), 1);
        assert!(Bounds::of_one(place).holds(place));
        assert!(!Bounds::of_one(place).holds(NOWHERE));
    }
}
