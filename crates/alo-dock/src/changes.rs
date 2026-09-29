//! What a person changed about their dock, which is the only part that is
//! written down.
//!
//! `alo-shortcuts`' shape and `alo-appearance`'s, for the third time and for the
//! same reason: the defaults live in the running release and this file holds the
//! difference, so a release that moves a default reaches every machine that
//! never touched it and no machine that did. An untouched machine has no
//! `dock.toml` at all ([`crate::keeping`]).
//!
//! **There is one thing to change, and it is not where the dock is.** [ADR
//! 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
//! fixes the dock along the bottom edge, so *which edge* and *which edge on this
//! screen* are not choices any more and are not stored. What is left is *whether
//! it hides when a window needs the room*.
//!
//! **A file that names an edge still reads.** `edge` and `displays` were written
//! by earlier releases, and a person's `dock.toml` is not rewritten behind them
//! — so both stay keys the file *may* have, with nothing behind them, and the
//! dock is along the bottom whatever they say. That is in [`crate::keeping`],
//! which is where the file's keys are declared; here they are simply absent from
//! the private `Written` shape below, and serde skips a key no field claims.
//!
//! *The dock's size* is still not here. `crate::layout` sizes it.

use serde::{Deserialize, Serialize};

use crate::hiding::Hiding;

/// One thing a person can change about their dock, for a settings panel that
/// offers *put it back*.
///
/// **One variant is not a mistake and not a placeholder.** It was two until ADR
/// 0076 fixed the dock to the bottom edge; an enum with one variant here is an
/// enum that gains variants additively when *the dock's size* arrives at v0.5,
/// and it keeps the shape of the question — *which setting do you mean* —
/// answerable by a panel that offers more than one row later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Setting {
    /// Whether the dock gives way when a window needs the room. There is no
    /// per-display exception for it: a dock that hid on one screen and not
    /// another would be a dock a person cannot predict, and predicting where it
    /// is, is most of what a dock is for.
    Hiding,
}

/// Everything a person has changed about their dock.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Written", into = "Written")]
pub struct Changes {
    /// Whether they asked it to give way, if they said anything about it.
    hiding: Option<Hiding>,
}

impl Changes {
    /// Nothing changed yet.
    #[must_use]
    pub fn untouched() -> Self {
        Self::default()
    }

    /// Whether nothing has been changed at all, which is what a fresh machine
    /// has, and what a missing file reads as.
    ///
    /// **A file that only names an edge is untouched.** An edge is not a change
    /// any more, so a machine whose `dock.toml` says nothing else has nothing to
    /// keep — and the next write replaces that file with a format line, which is
    /// how the dead key eventually leaves a person's folder without anybody
    /// rewriting it behind them.
    #[must_use]
    pub const fn is_untouched(&self) -> bool {
        self.hiding.is_none()
    }

    /// Say whether the dock gives way when a window needs the room.
    pub const fn set_hiding(&mut self, hiding: Hiding) {
        self.hiding = Some(hiding);
    }

    /// What they chose about hiding, if they chose.
    #[must_use]
    pub const fn hiding(&self) -> Option<Hiding> {
        self.hiding
    }

    /// Forget that this was ever changed, which puts it back to what the running
    /// release ships.
    ///
    /// Says whether there was anything to forget.
    pub const fn forget(&mut self, setting: Setting) -> bool {
        match setting {
            Setting::Hiding => self.hiding.take().is_some(),
        }
    }

    /// Forget everything, putting the dock back to what it shipped as.
    pub fn forget_everything(&mut self) {
        *self = Self::untouched();
    }
}

/// Changes as a settings file holds them: anything untouched is absent rather
/// than present and null, so an untouched machine writes no keys at all.
///
/// **There is no `edge` field and no `displays` field**, and their absence is
/// what makes a file that has them read: serde ignores a key no field claims, so
/// a `dock.toml` written by an earlier release loads with its edge dropped
/// rather than refused. [`crate::keeping`] is what keeps those two names
/// recognised, since an unrecognised key *is* refused there.
#[derive(Default, Serialize, Deserialize)]
struct Written {
    /// Whether it gives way, if anything was said about it. Absent rather than
    /// present and null, so a machine that has not chosen writes exactly the
    /// file it wrote before this key existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hiding: Option<Hiding>,
}

impl From<Written> for Changes {
    fn from(written: Written) -> Self {
        Self {
            hiding: written.hiding,
        }
    }
}

impl From<Changes> for Written {
    fn from(changes: Changes) -> Self {
        Self {
            hiding: changes.hiding,
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

    /// A change is recorded, replaced and forgotten, and *was it changed* is
    /// answerable.
    #[test]
    fn a_change_is_recorded_replaced_and_forgotten() {
        let mut changes = Changes::untouched();
        assert!(changes.is_untouched());
        assert_eq!(changes.hiding(), None);

        changes.set_hiding(Hiding::WhenAWindowNeedsTheRoom);
        changes.set_hiding(Hiding::Never);
        assert_eq!(changes.hiding(), Some(Hiding::Never));
        assert!(!changes.is_untouched());

        assert!(changes.forget(Setting::Hiding), "there was one to forget");
        assert!(!changes.forget(Setting::Hiding), "and not there twice");
        assert!(changes.is_untouched());
    }

    /// **An untouched machine writes nothing.** The file holds the difference,
    /// so a fresh machine's settings are an empty object rather than a copy of
    /// what the release ships — which is what lets a later release move the
    /// default for everybody who never touched it.
    #[test]
    fn the_file_holds_only_what_was_changed() {
        assert_eq!(serde_json::to_string(&Changes::untouched()).unwrap(), "{}");

        let mut changes = Changes::untouched();
        changes.set_hiding(Hiding::WhenAWindowNeedsTheRoom);
        let written = serde_json::to_string(&changes).unwrap();
        assert_eq!(written, r#"{"hiding":"WhenAWindowNeedsTheRoom"}"#);
        assert_eq!(serde_json::from_str::<Changes>(&written).unwrap(), changes);
    }

    /// **A file naming an edge reads, and the edge is ignored.** Every release
    /// before ADR 0076 wrote `edge` for anybody who moved their dock, and a
    /// person's own file is not rewritten behind them — so it has to load. It
    /// loads as a machine that changed nothing, because an edge is no longer a
    /// thing that can be changed.
    ///
    /// The value is not validated either: `"Middle"` was refused when there were
    /// four edges to be one of, and refusing it now would be refusing a file over
    /// a word nothing reads.
    #[test]
    fn a_file_naming_an_edge_reads_and_the_edge_is_ignored() {
        for text in [
            r#"{"edge":"Left"}"#,
            r#"{"edge":"Middle"}"#,
            r#"{"edge":3}"#,
            r#"{"displays":[["DP-3","Left"]]}"#,
        ] {
            let changes = serde_json::from_str::<Changes>(text).unwrap();
            assert!(changes.is_untouched(), "{text} was not read as untouched");
        }
    }

    /// **And a file that names an edge *and* says something about hiding keeps
    /// the half that still means something.** A person who moved their dock and
    /// also asked it to give way does not lose the second because of the first.
    #[test]
    fn the_half_of_an_old_file_that_still_means_something_survives() {
        let changes =
            serde_json::from_str::<Changes>(r#"{"edge":"Top","hiding":"WhenAWindowNeedsTheRoom"}"#)
                .unwrap();
        assert_eq!(changes.hiding(), Some(Hiding::WhenAWindowNeedsTheRoom));
        assert!(!changes.is_untouched());
    }

    /// Forgetting everything is one call, and it is the same as never having
    /// touched anything.
    #[test]
    fn everything_can_be_put_back_at_once() {
        let mut changes = Changes::untouched();
        changes.set_hiding(Hiding::WhenAWindowNeedsTheRoom);
        changes.forget_everything();
        assert!(changes.is_untouched());
        assert_eq!(changes, Changes::untouched());
    }
}
