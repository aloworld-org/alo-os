//! **What `alo-agentd`'s indicator shows, as this machine's desktop was told
//! it** — or that it was not told at all.
//!
//! [`crate::Lamp`] deliberately offers no way to be built from a reported
//! number, because a number anybody could hand in is a light anybody could
//! switch off. This is the other road its own documentation promised: **a
//! different type, with the report in its name.** A [`ToldByTheDaemon`] has no
//! public constructor. `crate::watching` makes one, from a line read off the
//! person's own socket after the kernel said who was listening there, and
//! nothing else in any crate can.
//!
//! # Not told is never dark
//!
//! [`FromTheDaemon::NotYetHeard`] and [`FromTheDaemon::Unreachable`] both draw
//! as *alo OS cannot tell* — never as *nothing is leaving*. A quiet indicator
//! that might be wrong is the silent failure law 1 exists to forbid; a screen
//! that does not know says so.

use alo_egress::Reported;

/// What the service's indicator showed when it last told the desktop.
///
/// **No public constructor**, so the only lines in one are lines read off the
/// service's own socket by `crate::watching`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToldByTheDaemon {
    /// What it showed, oldest first.
    lines: Vec<Reported>,
}

impl ToldByTheDaemon {
    /// What the service said it is showing. Only `crate::watching` calls this.
    pub(crate) const fn heard(lines: Vec<Reported>) -> Self {
        Self { lines }
    }

    /// What it showed, oldest first.
    #[must_use]
    pub fn lines(&self) -> &[Reported] {
        &self.lines
    }
}

/// What the desktop knows about what the service is sending.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FromTheDaemon {
    /// Nothing heard yet: the session has just started. Drawn as *cannot tell*.
    #[default]
    NotYetHeard,
    /// The service is not answering, or stopped answering. Drawn as *cannot
    /// tell* — **never** as the last list it gave, which may no longer be true.
    Unreachable,
    /// What it said, most recently.
    Told(ToldByTheDaemon),
}

impl FromTheDaemon {
    /// The lines it is showing, when it said.
    #[must_use]
    pub fn lines(&self) -> &[Reported] {
        match self {
            Self::Told(told) => told.lines(),
            Self::NotYetHeard | Self::Unreachable => &[],
        }
    }

    /// Whether the desktop cannot say what the service is sending.
    #[must_use]
    pub const fn cannot_tell(&self) -> bool {
        !matches!(self, Self::Told(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Never heard and not answering both mean *cannot tell***, and say so
    /// with no lines rather than with a list that might be stale.
    #[test]
    fn not_being_told_is_not_being_told_nothing() {
        for not_told in [FromTheDaemon::NotYetHeard, FromTheDaemon::Unreachable] {
            assert!(not_told.cannot_tell(), "{not_told:?}");
            assert!(not_told.lines().is_empty());
        }
        let told = FromTheDaemon::Told(ToldByTheDaemon::heard(Vec::new()));
        assert!(!told.cannot_tell());
    }
}
