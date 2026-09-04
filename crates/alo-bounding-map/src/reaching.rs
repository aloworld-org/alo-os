//! Whether the file being opened is inside the place a turn was granted.
//!
//! This is the decision ADR 0015 asks the kernel to make, and it is deliberately
//! the smallest thing that can carry it: walk from the file being opened up
//! through the directories it sits in, and answer yes the moment one of them is
//! the granted place.
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

/// How far up the tree the kernel looks before it gives up and refuses.
///
/// Thirty-two directories is deeper than any granted folder a person will
/// arrange by hand, and shallow enough that the verifier can unroll the walk.
/// It is a limit on the *grant*, not on the filesystem: a file thirty-three
/// directories under a granted folder is refused, and the person's answer is to
/// grant something closer to it.
pub const DEPTH: usize = 32;

/// Whether an opened file lies at or under `granted`.
///
/// `step` is asked for the file itself first, then for the directory it is in,
/// then for that directory's directory, and so on upwards. It answers [`None`]
/// when there is nothing above — the top of a filesystem — or when the kernel
/// could not be read, and either of those ends the walk with a refusal.
///
/// ```
/// use alo_bounding_map::{Place, reaches};
///
/// let granted = Place::of(64, 100);
/// let mut chain = [Place::of(64, 300), Place::of(64, 100)].into_iter();
/// assert!(reaches(granted, || chain.next()));
///
/// let mut elsewhere = [Place::of(64, 300), Place::of(64, 2)].into_iter();
/// assert!(!reaches(granted, || elsewhere.next()));
/// ```
pub fn reaches(granted: Place, mut step: impl FnMut() -> Option<Place>) -> bool {
    let mut taken = 0;
    while taken < DEPTH {
        match step() {
            Some(here) if here == granted => return true,
            Some(_) => taken += 1,
            None => return false,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Walks the chain a real open produces: the file, then each directory
    /// above it, ending at the top of the filesystem.
    fn walking(chain: &[Place]) -> impl FnMut() -> Option<Place> + '_ {
        let mut steps = chain.iter().copied();
        move || steps.next()
    }

    /// The ordinary case, and the one the allow test on a real kernel is.
    #[test]
    fn a_file_under_the_granted_folder_is_inside_it() {
        let granted = Place::of(64, 100);
        let chain = [Place::of(64, 501), Place::of(64, 400), granted];
        assert!(reaches(granted, walking(&chain)));
    }

    /// A grant over one document — what a person had open when they invoked the
    /// agent. *Inside* is not the question, and the same walk answers it.
    #[test]
    fn the_granted_file_itself_is_inside_the_grant() {
        let granted = Place::of(64, 501);
        let chain = [granted, Place::of(64, 400)];
        assert!(reaches(granted, walking(&chain)));
    }

    /// The refusal this whole mechanism exists for: a file whose directories
    /// never include the granted one.
    #[test]
    fn a_file_the_walk_never_meets_is_refused() {
        let granted = Place::of(64, 100);
        let chain = [Place::of(64, 900), Place::of(64, 800), Place::of(64, 2)];
        assert!(!reaches(granted, walking(&chain)));
    }

    /// A folder on another filesystem with the granted folder's inode number is
    /// somewhere else, and the walk has to say so.
    #[test]
    fn the_granted_inode_on_another_filesystem_is_not_the_grant() {
        let granted = Place::of(64, 100);
        let chain = [Place::of(65, 501), Place::of(65, 100)];
        assert!(!reaches(granted, walking(&chain)));
    }

    /// A kernel that will not answer, and a filesystem whose top has been
    /// reached, arrive as the same `None` — and both refuse. Guessing here
    /// would open the bound under precisely the conditions somebody would
    /// arrange on purpose.
    #[test]
    fn a_walk_that_cannot_be_taken_refuses() {
        let granted = Place::of(64, 100);
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
    #[test]
    fn deeper_than_the_walk_goes_is_refused() {
        let granted = Place::of(64, 100);
        let mut given: u64 = 0;
        let deep = reaches(granted, || {
            given += 1;
            Some(if given > DEPTH as u64 {
                granted
            } else {
                Place::of(64, 900 + given)
            })
        });
        assert!(!deep);

        let mut given: u64 = 0;
        let just_inside = reaches(granted, || {
            given += 1;
            Some(if given == DEPTH as u64 {
                granted
            } else {
                Place::of(64, 900 + given)
            })
        });
        assert!(just_inside);
    }
}
