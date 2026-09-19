//! The two questions a place is asked, and nothing else it can be asked.
//!
//! A check is one act with two halves: **which versions of this system does
//! that place hold**, and **which build is one of them**. Neither fetches
//! anything a machine would run — the second answers with a name, and the
//! bytes the name stands for are downloaded only when a person chooses to
//! apply the update, by the base, in `alo-updating`.
//!
//! **A trait, so the acceptance is written against a place that answers
//! wrongly.** The real one is [`crate::TheRegistry`], and every refusal in
//! [`NoAnswer`] has to be reachable from a test on a machine
//! with no network at all; a check that could only be exercised against a
//! working registry would have its refusal paths tested by nobody. It is the
//! same shape `alo_updating::Base` takes for the base's own program, and for
//! the same reason.
//!
//! **A stand-in changes who answers, never what is asked.** There is no method
//! here for fetching a build, so no implementation of this trait — ours or
//! anybody's — has a way to say *and here are the bytes*.

use crate::refusing::NoAnswer;
use crate::release::Release;

/// Somewhere this machine can ask whether there is a newer version of its
/// system.
pub trait ThePlace {
    /// Every name this place holds, whether or not it is a release.
    ///
    /// The names are handed back as they were written and judged by
    /// [`Release`] afterwards, so a place holding names of other kinds — a
    /// signature's, a candidate somebody pushed — is a place that answered
    /// rather than a place that failed.
    ///
    /// # Errors
    /// [`NoAnswer`], except [`NoAnswer::NothingIsOffered`], which is
    /// [`crate::look`]'s to decide from an answer that came back with no
    /// release in it.
    fn every_name(&self) -> Result<Vec<String>, NoAnswer>;

    /// Which build this release is, as this place names it now.
    ///
    /// Text rather than a `Digest`, so that an answer naming half a build
    /// reaches the same refusal a person is shown for any other answer that
    /// could not be understood, in one place rather than in every
    /// implementation.
    ///
    /// # Errors
    /// [`NoAnswer`].
    fn the_build_of(&self, release: &Release) -> Result<String, NoAnswer>;
}
