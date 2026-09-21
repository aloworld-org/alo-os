//! Which build this machine is running, asked of the base at the moment it is
//! wanted.
//!
//! *What am I running* is answerable at any moment because nothing here
//! remembers it: [`deployments`] asks the base, every time, and
//! [`running`] is the booted build out of that answer. A cached answer would
//! be wrong for exactly the moments the question matters — after an update,
//! after a rollback, on a machine somebody else changed.
//!
//! # This road is root's, and there is a second one that is not
//!
//! `bootc status` **refuses an unprivileged caller** — measured on a real bootc
//! machine on 2026-09-20 — so everything here answers only a caller that is
//! already root, which every caller that is about to *change* the machine
//! already is. The one act that has to ask this and must not be root reads
//! [`crate::written_down::WrittenDown`] instead, which is the same question
//! answered out of files the base has already written.

use alo_keeping_up::{Deployments, Running};

use crate::refusing::NotRead;
use crate::the_base::Base;

/// What the base is asked: its status, as JSON, in the format version this
/// crate reads.
pub const THE_STATUS: [&str; 5] = ["status", "--format", "json", "--format-version", "1"];

/// The builds on this machine's disk, as the base reports them now.
///
/// # Errors
/// [`NotRead`].
pub fn deployments(base: &impl Base) -> Result<Deployments, NotRead> {
    let arguments: Vec<String> = THE_STATUS.iter().map(|&word| word.to_owned()).collect();
    let answer = base.asked(&arguments).map_err(NotRead::NotAnswered)?;
    serde_json::from_slice(&answer).map_err(|why| NotRead::NotUnderstood {
        why: why.to_string(),
    })
}

/// The build this machine is running, as the base reports it now.
///
/// # Errors
/// [`NotRead`], including [`NotRead::NotRunningABuild`] for a machine that
/// booted from no image.
pub fn running(base: &impl Base) -> Result<Running, NotRead> {
    deployments(base)?
        .running()
        .map_err(|_| NotRead::NotRunningABuild)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::the_base::TheBase;
    use std::path::Path;

    /// **A base that is not there is not a machine running nothing.**
    #[test]
    fn a_base_that_is_not_there_is_not_read() {
        let refused = running(&TheBase::at(Path::new("/nowhere/bootc")));
        assert!(
            matches!(refused, Err(NotRead::NotAnswered(_))),
            "{refused:?}"
        );
    }

    /// **An answer that is not the base's status is refused**, rather than
    /// read as a machine with nothing booted.
    #[cfg(unix)]
    #[test]
    fn an_answer_that_is_not_a_status_is_not_understood() {
        // `echo` answers with its arguments, which are not a status document.
        let refused = deployments(&TheBase::at(Path::new("/bin/echo")));
        assert!(
            matches!(refused, Err(NotRead::NotUnderstood { .. })),
            "{refused:?}"
        );
        // And `true` answers with nothing at all.
        let refused = running(&TheBase::at(Path::new("/bin/true")));
        assert!(
            matches!(refused, Err(NotRead::NotUnderstood { .. })),
            "{refused:?}"
        );
    }
}
