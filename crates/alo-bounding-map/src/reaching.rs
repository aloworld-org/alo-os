//! Whether the file being opened is inside one of the places a turn was
//! granted.
//!
//! This is the decision ADR 0015 asks the kernel to make, and it is deliberately
//! the smallest thing that can carry it: walk from the file being opened up
//! through the directories it sits in, and answer yes the moment one of them is
//! a granted place.
//!
//! # One walk, and every granted place asked at each step
//!
//! A turn is bound to several places ([`Bounds`]), and there are two ways to ask
//! about them: walk once per place, or walk once and compare each step against
//! all of them. It is the second, and the reason is what the two halves cost.
//! **A step is four reads of kernel memory; a comparison is two numbers.** So a
//! second walk would multiply the expensive half by the number of places, and
//! asking all of them at each step adds nothing the verifier has to think about
//! — the loop is still [`DEPTH`] steps long, and the inner one is [`Bounds`]'
//! own fixed width.
//!
//! # Three rules, and each of them fails closed
//!
//! **The file itself counts.** ADR 0001 §4 has a grant over a single document —
//! the one a person had open when they invoked the agent — and for that grant
//! *inside* is not the question. Starting the walk at the file rather than at
//! its folder answers both shapes of grant with one rule.
//!
//! **A step that cannot be taken refuses.** The walk stops at the top of a
//! filesystem, and it stops if the kernel would not answer. Both arrive here as
//! [`None`], and both mean no — a bound that guessed when it could not see
//! would be a bound that opens under exactly the conditions somebody would
//! choose to create.
//!
//! **The walk is bounded.** [`DEPTH`] directories and no further. A program the
//! kernel's verifier cannot prove terminates is a program the kernel will not
//! load, so the bound is not a safety net around an unbounded loop — it is the
//! reason there is a program at all. A path deeper than [`DEPTH`] below its
//! grant is refused rather than allowed, which is the direction that costs
//! somebody an error message instead of costing them a file.

use crate::bound::Place;
use crate::bounds::Bounds;

/// How far up the tree the kernel looks before it gives up and refuses.
///
/// Thirty-two directories is deeper than any granted folder a person will
/// arrange by hand, and shallow enough that the verifier can unroll the walk.
/// It is a limit on the *grant*, not on the filesystem: a file thirty-three
/// directories under a granted folder is refused, and the person's answer is to
/// grant something closer to it.
pub const DEPTH: usize = 32;

/// Whether an opened file lies at or under any of the places `granted` holds.
///
/// `step` is asked for the file itself first, then for the directory it is in,
/// then for that directory's directory, and so on upwards. It answers [`None`]
/// when there is nothing above — the top of a filesystem — or when the kernel
/// could not be read, and either of those ends the walk with a refusal.
///
/// ```
/// use alo_bounding_map::{Bounds, Place, reaches};
///
/// let invoices = Place::of(64, 100);
/// let archive = Place::of(64, 700);
/// let granted = Bounds::of(&[invoices, archive]).expect("two is not too many");
///
/// let mut chain = [Place::of(64, 300), invoices].into_iter();
/// assert!(reaches(granted, || chain.next()));
///
/// // The second place a call named answers the same walk.
/// let mut into = [Place::of(64, 301), archive].into_iter();
/// assert!(reaches(granted, || into.next()));
///
/// let mut elsewhere = [Place::of(64, 300), Place::of(64, 2)].into_iter();
/// assert!(!reaches(granted, || elsewhere.next()));
/// ```
pub fn reaches(granted: Bounds, mut step: impl FnMut() -> Option<Place>) -> bool {
    let mut taken = 0;
    while taken < DEPTH {
        match step() {
            Some(here) if granted.holds(here) => return true,
            Some(_) => taken += 1,
            None => return false,
        }
    }
    false
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::bounds::PLACES;

    /// Walks the chain a real open produces: the file, then each directory
    /// above it, ending at the top of the filesystem.
    fn walking(chain: &[Place]) -> impl FnMut() -> Option<Place> + '_ {
        let mut steps = chain.iter().copied();
        move || steps.next()
    }

    /// A turn bound to one place, which is what most of these tests are about.
    fn only(place: Place) -> Bounds {
        Bounds::of(&[place]).expect("one place is a bound")
    }

    /// The ordinary case, and the one the allow test on a real kernel is.
    #[test]
    fn a_file_under_the_granted_folder_is_inside_it() {
        let granted = Place::of(64, 100);
        let chain = [Place::of(64, 501), Place::of(64, 400), granted];
        assert!(reaches(only(granted), walking(&chain)));
    }

    /// A grant over one document — what a person had open when they invoked the
    /// agent. *Inside* is not the question, and the same walk answers it.
    #[test]
    fn the_granted_file_itself_is_inside_the_grant() {
        let granted = Place::of(64, 501);
        let chain = [granted, Place::of(64, 400)];
        assert!(reaches(only(granted), walking(&chain)));
    }

    /// The refusal this whole mechanism exists for: a file whose directories
    /// never include the granted one.
    #[test]
    fn a_file_the_walk_never_meets_is_refused() {
        let granted = Place::of(64, 100);
        let chain = [Place::of(64, 900), Place::of(64, 800), Place::of(64, 2)];
        assert!(!reaches(only(granted), walking(&chain)));
    }

    /// **The second place a call named answers the same walk**, and so does the
    /// last of them. `move_file` names a file and the folder it is going into,
    /// and the two live in different parts of the tree — so a walk that only
    /// ever met the first would refuse half of every move.
    #[test]
    fn any_of_the_granted_places_ends_the_walk() {
        let granted = [
            Place::of(64, 100),
            Place::of(64, 200),
            Place::of(65, 100),
            Place::of(66, 900),
        ];
        let bound = Bounds::of(&granted).expect("four is not too many");
        for place in granted {
            let chain = [Place::of(64, 999), Place::of(64, 998), place];
            assert!(reaches(bound, walking(&chain)), "{place:?}");
        }
        let nowhere_near = [Place::of(64, 999), Place::of(66, 901)];
        assert!(!reaches(bound, walking(&nowhere_near)));
    }

    /// **A turn bound to nowhere reaches nothing**, which is what an entry whose
    /// count could not be read comes to. It is the direction the clamp in
    /// [`Bounds::of_words`] has to fail in, asserted at the walk rather than at
    /// the value.
    #[test]
    fn a_turn_bound_to_nowhere_opens_nothing() {
        let mut words = [0_u64; crate::bounds::WORDS];
        words[1] = 64;
        words[2] = 100;
        let nowhere = Bounds::of_words(words);
        let chain = [Place::of(64, 501), Place::of(64, 100)];
        assert!(!reaches(nowhere, walking(&chain)));
    }

    /// A folder on another filesystem with the granted folder's inode number is
    /// somewhere else, and the walk has to say so — including when the turn is
    /// bound to several places and one of them is on that filesystem.
    #[test]
    fn the_granted_inode_on_another_filesystem_is_not_the_grant() {
        let granted = Place::of(64, 100);
        let chain = [Place::of(65, 501), Place::of(65, 100)];
        assert!(!reaches(only(granted), walking(&chain)));

        let both = Bounds::of(&[granted, Place::of(65, 700)]).expect("two is not too many");
        assert!(!reaches(both, walking(&chain)));
    }

    /// A kernel that will not answer, and a filesystem whose top has been
    /// reached, arrive as the same `None` — and both refuse. Guessing here
    /// would open the bound under precisely the conditions somebody would
    /// arrange on purpose.
    #[test]
    fn a_walk_that_cannot_be_taken_refuses() {
        let granted = only(Place::of(64, 100));
        assert!(!reaches(granted, || None));
        let mut given = 0;
        assert!(!reaches(granted, || {
            given += 1;
            (given < 3).then(|| Place::of(64, 900))
        }));
    }

    /// Deeper than the walk goes is refused, not allowed. The person's answer
    /// is a grant closer to the file; the alternative answer is a loop the
    /// kernel's verifier will not load.
    ///
    /// The bound is the widest there is, because [`DEPTH`] is a limit on the
    /// steps rather than on the comparisons: a turn bound to [`PLACES`] places
    /// looks exactly as far up the tree as one bound to a single place.
    #[test]
    fn deeper_than_the_walk_goes_is_refused() {
        let met = Place::of(64, 100);
        let granted = Bounds::of(&[met; PLACES]).expect("PLACES is not too many");
        let mut given: u64 = 0;
        let deep = reaches(granted, || {
            given += 1;
            Some(if given > DEPTH as u64 {
                met
            } else {
                Place::of(64, 900 + given)
            })
        });
        assert!(!deep);

        let mut given: u64 = 0;
        let just_inside = reaches(granted, || {
            given += 1;
            Some(if given == DEPTH as u64 {
                met
            } else {
                Place::of(64, 900 + given)
            })
        });
        assert!(just_inside);
    }
}
