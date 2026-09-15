//! What changed since this machine last started: the same build, an update, or
//! going back to the build before.
//!
//! The base boots whichever build is staged, and says afterwards which one it
//! booted; it does not say that this start is the first on that build. So the
//! doing crate keeps one fact across a restart — the build this machine was
//! running the last time it looked — and [`Since::between`] compares it with
//! what the base reports now.
//!
//! **Decided from two digests and nothing else.** No clock, and no reading of
//! the record: an update is exactly a different build booting than the one
//! last known, and what the record is then told is [`Since::Updated`]'s two
//! digests, from which and to which.
//!
//! **A return is told apart from an update, not guessed at.** Going back
//! is also a different build booting. What makes it [`Since::RolledBack`] is the
//! second fact the doing crate keeps: the build the person chose to go back to,
//! noted before the base was told. Nothing is inferred from which build is
//! older, because digests have no order.
//!
//! **A machine with nothing known is not an update.** The first start after
//! installing, or after the fact kept across restarts was lost, has no *from*;
//! it answers [`Since::FirstKnown`], and inventing a *from* would be writing a
//! change into the record that nobody saw happen.

use crate::deployments::{Deployments, NotRunningABuild};
use crate::digest::Digest;

/// What changed since this machine last looked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Since {
    /// Nothing was known before: this is the first build this machine knows it
    /// ran.
    FirstKnown(Digest),
    /// The same build as last time.
    Unchanged(Digest),
    /// A different build booted, and it was not one the person chose to go back
    /// to.
    Updated {
        /// The build this machine was running before the restart.
        from: Digest,
        /// The build it is running now.
        to: Digest,
    },
    /// The build the person chose to go back to booted.
    RolledBack {
        /// The build this machine was running before the restart.
        from: Digest,
        /// The earlier build it went back to, and is running now.
        to: Digest,
    },
}

impl Since {
    /// Compare the build last known with what the base reports now.
    ///
    /// `going_back_to` is the build the person chose to go back to, noted before
    /// the base was told, if one is noted. A different build booting is a return
    /// exactly when it is that build, and an update otherwise — told apart by
    /// what the person asked for, never by guessing from the builds.
    ///
    /// # Errors
    /// [`NotRunningABuild`] when the base names no booted build: nothing is
    /// decided about a machine this cannot name.
    pub fn between(
        last_known: Option<&Digest>,
        going_back_to: Option<&Digest>,
        deployments: &Deployments,
    ) -> Result<Self, NotRunningABuild> {
        let now = deployments.running()?.digest().clone();
        Ok(match last_known {
            None => Self::FirstKnown(now),
            Some(before) if *before == now => Self::Unchanged(now),
            Some(before) if going_back_to == Some(&now) => Self::RolledBack {
                from: before.clone(),
                to: now,
            },
            Some(before) => Self::Updated {
                from: before.clone(),
                to: now,
            },
        })
    }

    /// The build running now, whichever this was.
    #[must_use]
    pub fn now(&self) -> &Digest {
        match self {
            Self::FirstKnown(now)
            | Self::Unchanged(now)
            | Self::Updated { to: now, .. }
            | Self::RolledBack { to: now, .. } => now,
        }
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

    fn booted(pair: &str) -> Deployments {
        Deployments::reported(Some(whole(pair)), None, Some(whole("ff")))
    }

    /// A different build booting than the one last known is an update, from
    /// that one to this.
    #[test]
    fn a_different_build_booting_is_an_update_from_the_last_known_to_this() {
        assert_eq!(
            Since::between(Some(&whole("aa")), None, &booted("bb")),
            Ok(Since::Updated {
                from: whole("aa"),
                to: whole("bb"),
            })
        );
    }

    /// **The same build is not an update**, however many restarts.
    #[test]
    fn the_same_build_again_is_unchanged() {
        assert_eq!(
            Since::between(Some(&whole("aa")), None, &booted("aa")),
            Ok(Since::Unchanged(whole("aa")))
        );
    }

    /// **Nothing known is not an update**: no *from* is invented, and the
    /// build the base keeps as the one before is not read as one either.
    #[test]
    fn nothing_known_before_is_not_an_update() {
        let since = Since::between(None, None, &booted("bb")).unwrap();
        assert_eq!(since, Since::FirstKnown(whole("bb")));
        assert_eq!(since.now(), &whole("bb"));
    }

    /// **The build the person chose to go back to booting is a return**, from the
    /// build last known to it — and a different build booting while a return was
    /// noted is still an update, because it is not the one chosen.
    #[test]
    fn the_build_chosen_to_go_back_to_booting_is_a_return_and_any_other_is_an_update() {
        assert_eq!(
            Since::between(Some(&whole("bb")), Some(&whole("aa")), &booted("aa")),
            Ok(Since::RolledBack {
                from: whole("bb"),
                to: whole("aa"),
            })
        );
        assert_eq!(
            Since::between(Some(&whole("bb")), Some(&whole("aa")), &booted("cc")),
            Ok(Since::Updated {
                from: whole("bb"),
                to: whole("cc"),
            })
        );
        // Not restarted yet: still the same build, whatever was noted.
        assert_eq!(
            Since::between(Some(&whole("bb")), Some(&whole("aa")), &booted("bb")),
            Ok(Since::Unchanged(whole("bb")))
        );
        // Nothing known before is not a return either: there is no from.
        assert_eq!(
            Since::between(None, Some(&whole("aa")), &booted("aa")),
            Ok(Since::FirstKnown(whole("aa")))
        );
    }

    /// **A machine the base names no build for decides nothing.**
    #[test]
    fn a_machine_running_no_build_decides_nothing() {
        assert_eq!(
            Since::between(
                Some(&whole("aa")),
                None,
                &Deployments::reported(None, None, None)
            ),
            Err(NotRunningABuild)
        );
    }
}
