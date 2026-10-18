//! The two ways asking the record fails, and only one of them is a fault.
//!
//! `crate::went_wrong_looking` beside it, with the difference stated:
//! **this read can be stopped by the world.** The catalogue is a string
//! compiled into the binary; the record is a file on a disk, and a disk can
//! refuse. So one of these two is a thing that can really happen on a working
//! machine, and the other is alo OS contradicting itself.
//!
//! - **the record could not be read back** — the file is gone, the disk
//!   refused, or a line in it is not a line this version understands. A real
//!   outcome, reported as one. It is not *nothing touched the file*, and the
//!   difference matters enough that these are two types rather than an empty
//!   answer;
//! - an authority for another verb, or a malformed one, reached this executor
//!   — which is this machine routing a call to the wrong place.
//!
//! Both are kept in English, for `alo-software`'s stated reason about the
//! second: *its reader is fixing that executor.* The sentence the person reads
//! is [`crate::words::COULD_NOT_ASK_THE_RECORD`] and carries neither, which is
//! `NOT_BOUNDED`'s rule met again — what a person can act on is that nothing
//! was answered and that the machine is what has to be looked at.

//! # Why the first arm is not a `source`
//!
//! `alo_keeping::NotKept` is **worded through the vocabulary, not through
//! `Display`** — this workspace's rule, so that a person reads a refusal in
//! their own language rather than in a library author's English. So it is not
//! an `Error`, it cannot be a `#[from]` source, and the English here renders
//! it with `Debug`. That is the right trade for a sentence only an
//! administrator reads: the person's sentence is
//! [`crate::words::COULD_NOT_ASK_THE_RECORD`], translated like every other.

use alo_keeping::NotKept;
use alo_record::NotAsked;

/// A question of the record alo OS could not answer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WentWrongAsking {
    /// The record could not be read back to be asked anything.
    ///
    /// **The one here that is not a fault in alo OS.** A record kept on a disk
    /// is a file, and a file can be unreadable — which is why this is reported
    /// rather than folded into an answer of *nothing*.
    #[error("the record could not be read back: {0:?}")]
    NotReadBack(NotKept),
    /// An authority that is not a question of the record reached this
    /// executor.
    #[error("the record's executor was handed an authority that is not its question: {0}")]
    NotItsQuestion(#[from] NotAsked),
}

impl From<NotKept> for WentWrongAsking {
    fn from(why: NotKept) -> Self {
        Self::NotReadBack(why)
    }
}
