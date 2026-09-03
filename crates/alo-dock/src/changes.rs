//! What a person changed about their dock, which is the only part that is
//! written down.
//!
//! `alo-shortcuts`' shape and `alo-appearance`'s, for the third time and for the
//! same reason: the defaults live in the running release and this file holds the
//! difference, so a release that moves a default reaches every machine that
//! never touched it and no machine that did. An untouched machine writes `{}`.
//!
//! **There is one thing to change at v0.01, and that is not a mistake.** *The
//! dock's size*, *whether it hides when a window needs the room* and *one dock
//! per display* are all v0.5 in `docs/features.md`. A file with one key in it
//! now is a file that gains keys additively later; a file with four keys in it
//! now, three of which nothing reads, is three settings somebody has to keep
//! working for a release that has not been designed.

use serde::{Deserialize, Serialize};

use crate::edge::Edge;

/// One thing a person can change about their dock, for a settings panel that
/// offers *put it back*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Setting {
    /// Which edge of the screen the dock is on.
    Edge,
}

/// Everything a person has changed about their dock.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Written", into = "Written")]
pub struct Changes {
    /// Which edge they moved it to.
    edge: Option<Edge>,
}

impl Changes {
    /// Nothing changed yet.
    #[must_use]
    pub fn untouched() -> Self {
        Self::default()
    }

    /// Whether nothing has been changed at all, which is what a fresh machine
    /// writes to its settings file.
    #[must_use]
    pub const fn is_untouched(&self) -> bool {
        self.edge.is_none()
    }

    /// Put the dock on this edge.
    pub fn set_edge(&mut self, edge: Edge) {
        self.edge = Some(edge);
    }

    /// Which edge they moved it to, if they moved it.
    #[must_use]
    pub const fn edge(&self) -> Option<Edge> {
        self.edge
    }

    /// Forget that this was ever changed, which puts it back to what the running
    /// release ships.
    ///
    /// Says whether there was anything to forget.
    pub fn forget(&mut self, setting: Setting) -> bool {
        match setting {
            Setting::Edge => self.edge.take().is_some(),
        }
    }

    /// Forget everything, putting the dock back to what it shipped as.
    pub fn forget_everything(&mut self) {
        *self = Self::untouched();
    }
}

/// Changes as a settings file holds them: anything untouched is absent rather
/// than present and null, so an untouched machine writes `{}`.
#[derive(Default, Serialize, Deserialize)]
struct Written {
    /// The edge, if it was moved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    edge: Option<Edge>,
}

impl From<Written> for Changes {
    fn from(written: Written) -> Self {
        Self { edge: written.edge }
    }
}

impl From<Changes> for Written {
    fn from(changes: Changes) -> Self {
        Self { edge: changes.edge }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The edge is recorded, replaced and forgotten, and *was it changed* is
    /// answerable.
    #[test]
    fn a_change_is_recorded_replaced_and_forgotten() {
        let mut changes = Changes::untouched();
        assert!(changes.is_untouched());
        assert_eq!(changes.edge(), None);

        changes.set_edge(Edge::Left);
        changes.set_edge(Edge::Top);
        assert_eq!(changes.edge(), Some(Edge::Top));
        assert!(!changes.is_untouched());

        assert!(changes.forget(Setting::Edge), "there was one to forget");
        assert!(!changes.forget(Setting::Edge), "and not there twice");
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
        changes.set_edge(Edge::Right);
        let written = serde_json::to_string(&changes).unwrap();
        assert_eq!(written, r#"{"edge":"Right"}"#);
        assert_eq!(serde_json::from_str::<Changes>(&written).unwrap(), changes);
    }

    /// A hand-edited file naming an edge that does not exist is refused where it
    /// is read, rather than becoming a dock nobody can find.
    #[test]
    fn a_file_cannot_name_an_edge_there_is_not() {
        assert!(serde_json::from_str::<Changes>(r#"{"edge":"Middle"}"#).is_err());
        assert!(serde_json::from_str::<Changes>(r#"{"edge":3}"#).is_err());
        assert_eq!(
            serde_json::from_str::<Changes>(r#"{"edge":"Bottom"}"#)
                .unwrap()
                .edge(),
            Some(Edge::Bottom)
        );
    }

    /// Forgetting everything is one call, and it is the same as never having
    /// touched anything.
    #[test]
    fn everything_can_be_put_back_at_once() {
        let mut changes = Changes::untouched();
        changes.set_edge(Edge::Left);
        changes.forget_everything();
        assert!(changes.is_untouched());
        assert_eq!(changes, Changes::untouched());
    }
}
