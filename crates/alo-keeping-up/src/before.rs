//! The build this machine ran before the one it is running: what it was, and
//! whether it is still on the disk.
//!
//! Two witnesses answer it, and they are not the same witness twice:
//!
//! - **The base** keeps the build before as a second deployment for as long as
//!   it keeps it ([`Deployments::rollback`]). That is the only one this machine
//!   can go back to, because going back is the base starting what it kept.
//! - **The record** says what the machine last changed from, and when
//!   ([`Changed`], read by `alo-updating` out of the record's `updated` and
//!   `rolled-back` entries). It outlives the base's copy, so a build that is no
//!   longer on the disk is still named rather than forgotten.
//!
//! [`Before::of`] puts them together. The record's word is taken only when it
//! is about **the build running now** — a change to some other build is how the
//! machine once got somewhere else, not how it got here — and the base's word
//! decides whether the build is kept, because the base is what would start it.
//!
//! **When it was replaced** is the record's moment, and this crate names no
//! time: `alo-updating` carries it beside the [`Before`] it finds, from the same
//! entry that was handed in here as a [`Changed`].

use crate::deployments::{Deployments, NotRunningABuild};
use crate::digest::Digest;

/// What the record last says this machine changed: from one build to another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changed {
    /// The build it was running before the change.
    from: Digest,
    /// The build it started on.
    to: Digest,
}

impl Changed {
    /// A change the record says happened, from `from` to `to`.
    #[must_use]
    pub fn between(from: Digest, to: Digest) -> Self {
        Self { from, to }
    }

    /// The build it was running before the change.
    #[must_use]
    pub fn from(&self) -> &Digest {
        &self.from
    }

    /// The build it started on.
    #[must_use]
    pub fn to(&self) -> &Digest {
        &self.to
    }
}

/// The build this machine ran before the one running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Before {
    /// The build running now.
    running: Digest,
    /// The build before it.
    build: Digest,
    /// Whether the base still has it on the disk.
    kept: bool,
    /// Whether the record's last change is how the machine got from that build
    /// to this one.
    in_the_record: bool,
}

impl Before {
    /// The build before the one running, from what the base reports and what
    /// the record last says changed.
    ///
    /// [`None`] when neither names one: the first build this machine ran, or a
    /// machine whose record does not reach back to its last change and whose
    /// base keeps nothing earlier.
    ///
    /// # Errors
    /// [`NotRunningABuild`]: nothing is named before a build this cannot name.
    pub fn of(
        deployments: &Deployments,
        last_changed: Option<&Changed>,
    ) -> Result<Option<Self>, NotRunningABuild> {
        let running = deployments.running()?.digest().clone();
        let to_here = last_changed.filter(|changed| changed.to == running);
        let before = match (deployments.rollback(), to_here) {
            (Some(kept), Some(changed)) => Some(Self {
                in_the_record: changed.from == *kept,
                build: kept.clone(),
                kept: true,
                running,
            }),
            (Some(kept), None) => Some(Self {
                build: kept.clone(),
                kept: true,
                in_the_record: false,
                running,
            }),
            (None, Some(changed)) => Some(Self {
                build: changed.from.clone(),
                kept: false,
                in_the_record: true,
                running,
            }),
            (None, None) => None,
        };
        Ok(before)
    }

    /// The build this machine ran before.
    #[must_use]
    pub fn build(&self) -> &Digest {
        &self.build
    }

    /// The build running now, which this came before.
    #[must_use]
    pub fn running(&self) -> &Digest {
        &self.running
    }

    /// Whether the base still has it on the disk, so that going back to it is
    /// possible at all.
    #[must_use]
    pub fn is_kept(&self) -> bool {
        self.kept
    }

    /// Whether the record's last change is the one from this build to the one
    /// running — so the moment written on that change is when this build was
    /// replaced.
    ///
    /// False when the base keeps a build the record says nothing about: the
    /// record was shortened, or the machine changed some way that wrote no
    /// entry. The build is still named; when it was replaced is not known, and
    /// nothing guesses it.
    #[must_use]
    pub fn replaced_as_the_record_says(&self) -> bool {
        self.in_the_record
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::whole;

    fn changed(from: &str, to: &str) -> Changed {
        Changed::between(whole(from), whole(to))
    }

    /// **The base's kept build and the record's last change agree**: named, on
    /// the disk, and replaced when the record says.
    #[test]
    fn a_kept_build_the_record_names_is_named_kept_and_dated() {
        let deployments = Deployments::reported(Some(whole("bb")), None, Some(whole("aa")));
        let before = Before::of(&deployments, Some(&changed("aa", "bb")))
            .unwrap()
            .unwrap();
        assert_eq!(before.build(), &whole("aa"));
        assert_eq!(before.running(), &whole("bb"));
        assert!(before.is_kept());
        assert!(before.replaced_as_the_record_says());
    }

    /// **A build the base no longer keeps is still named from the record**, and
    /// said to be gone rather than left out.
    #[test]
    fn a_build_no_longer_on_the_disk_is_named_from_the_record_and_not_kept() {
        let deployments = Deployments::reported(Some(whole("bb")), None, None);
        let before = Before::of(&deployments, Some(&changed("aa", "bb")))
            .unwrap()
            .unwrap();
        assert_eq!(before.build(), &whole("aa"));
        assert!(!before.is_kept());
        assert!(before.replaced_as_the_record_says());
    }

    /// **A kept build the record says nothing about is named, and no moment is
    /// invented for it.**
    #[test]
    fn a_kept_build_the_record_does_not_reach_is_named_with_no_moment() {
        let deployments = Deployments::reported(Some(whole("bb")), None, Some(whole("aa")));
        let before = Before::of(&deployments, None).unwrap().unwrap();
        assert!(before.is_kept());
        assert!(!before.replaced_as_the_record_says());

        // A record whose last change disagrees with what the base kept names
        // the base's build, which is the one that could be started.
        let before = Before::of(&deployments, Some(&changed("cc", "bb")))
            .unwrap()
            .unwrap();
        assert_eq!(before.build(), &whole("aa"));
        assert!(!before.replaced_as_the_record_says());
    }

    /// **A change to some other build is not how the machine got here**, and is
    /// not read as the build before.
    #[test]
    fn a_change_that_did_not_end_at_the_running_build_names_nothing() {
        let deployments = Deployments::reported(Some(whole("aa")), None, None);
        assert_eq!(
            Before::of(&deployments, Some(&changed("aa", "bb"))).unwrap(),
            None
        );
        assert_eq!(Before::of(&deployments, None).unwrap(), None);
    }

    /// **Nothing is named before a build this cannot name.**
    #[test]
    fn a_machine_running_no_build_names_nothing_before_it() {
        let deployments = Deployments::reported(None, None, Some(whole("aa")));
        assert_eq!(
            Before::of(&deployments, Some(&changed("aa", "bb"))),
            Err(NotRunningABuild)
        );
    }
}
