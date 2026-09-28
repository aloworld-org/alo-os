//! What a person changed about their dock, which is the only part that is
//! written down.
//!
//! `alo-shortcuts`' shape and `alo-appearance`'s, for the third time and for the
//! same reason: the defaults live in the running release and this file holds the
//! difference, so a release that moves a default reaches every machine that
//! never touched it and no machine that did. An untouched machine has no
//! `dock.toml` at all ([`crate::keeping`]).
//!
//! **There was one thing to change at v0.01, and that was not a mistake.** *The
//! dock's size*, *whether it hides when a window needs the room* and *one dock
//! per display* are all v0.5 in `docs/features.md`. A file with one key in it
//! then was a file that gains keys additively later; a file with four keys in it
//! then, three of which nothing read, would have been three settings somebody had
//! to keep working for a release that had not been designed.
//!
//! **The second of those keys arrived on 2026-09-27, and additively as promised.**
//! *Per display, so the dock can sit along the bottom of the laptop and down the
//! side of the external screen* is the `[v0.5]` promise, and it is a display
//! **singled out as an exception to the edge** — the third time this file takes
//! `alo-appearance`'s shape, because that crate already made a display an
//! exception to a background and two answers in this repository about what *per
//! display* means would be one too many. A machine that never singled one out
//! writes no such key and reads exactly as it did before.
//!
//! *The dock's size* and *whether it hides when a window needs the room* are
//! still not here. `crate::layout` sizes it; the hiding has no code at all and
//! says so where it would go.

use serde::{Deserialize, Serialize};

use alo_appearance::DisplayId;

use crate::edge::Edge;
use crate::hiding::Hiding;

/// One thing a person can change about their dock, for a settings panel that
/// offers *put it back*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Setting {
    /// Which edge of the screen the dock is on, everywhere. Per-display
    /// exceptions are their own, and [`Changes::forget_display`] is how one of
    /// those goes back — `alo-appearance` says the same about a background, for
    /// the same reason: *put the edge back* and *stop singling this screen out*
    /// are two different things a person means.
    Edge,
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
    /// Which edge they moved it to, on every display they have not singled out.
    edge: Option<Edge>,
    /// Whether they asked it to give way, if they said anything about it.
    hiding: Option<Hiding>,
    /// The displays they singled out, oldest first, one entry each.
    displays: Vec<(DisplayId, Edge)>,
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
    /// **A display singled out counts.** A machine whose only change is one
    /// screen's own edge has been changed, and a reader that said otherwise would
    /// write no file for it and lose the change at the next sign-in.
    #[must_use]
    pub fn is_untouched(&self) -> bool {
        self.edge.is_none() && self.hiding.is_none() && self.displays.is_empty()
    }

    /// Put the dock on this edge, on every display not singled out.
    pub fn set_edge(&mut self, edge: Edge) {
        self.edge = Some(edge);
    }

    /// Say whether the dock gives way when a window needs the room.
    pub fn set_hiding(&mut self, hiding: Hiding) {
        self.hiding = Some(hiding);
    }

    /// What they chose about hiding, if they chose.
    #[must_use]
    pub const fn hiding(&self) -> Option<Hiding> {
        self.hiding
    }

    /// Which edge they moved it to, if they moved it.
    #[must_use]
    pub const fn edge(&self) -> Option<Edge> {
        self.edge
    }

    /// Single this display out, putting the dock on this edge of it alone.
    ///
    /// Replaces the exception if there is one, so a display appears once;
    /// otherwise it goes on the end, which is what keeps [`Self::displays`]
    /// oldest first.
    pub fn set_edge_on(&mut self, display: DisplayId, edge: Edge) {
        self.displays.retain(|(named, _)| *named != display);
        self.displays.push((display, edge));
    }

    /// The edge they singled this display out for, if they singled it out.
    #[must_use]
    pub fn edge_on(&self, display: &DisplayId) -> Option<Edge> {
        self.displays
            .iter()
            .find(|(named, _)| named == display)
            .map(|(_, edge)| *edge)
    }

    /// Every display they singled out, oldest first.
    pub fn displays(&self) -> impl Iterator<Item = (&DisplayId, Edge)> {
        self.displays.iter().map(|(named, edge)| (named, *edge))
    }

    /// Stop singling this display out, putting it back to the edge they chose
    /// for everywhere.
    ///
    /// Says whether there was anything to put back.
    pub fn forget_display(&mut self, display: &DisplayId) -> bool {
        let before = self.displays.len();
        self.displays.retain(|(named, _)| named != display);
        self.displays.len() != before
    }

    /// Forget that this was ever changed, which puts it back to what the running
    /// release ships.
    ///
    /// Says whether there was anything to forget.
    pub fn forget(&mut self, setting: Setting) -> bool {
        match setting {
            Setting::Edge => self.edge.take().is_some(),
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
#[derive(Default, Serialize, Deserialize)]
struct Written {
    /// The edge, if it was moved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    edge: Option<Edge>,
    /// Whether it gives way, if anything was said about it. Absent rather than
    /// present and null, like every other key here, so a machine that has not
    /// chosen writes exactly the file it wrote before this key existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hiding: Option<Hiding>,
    /// The displays singled out, if any were. Absent rather than an empty list,
    /// so a machine that singled none out writes exactly the file it wrote
    /// before this key existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    displays: Vec<(DisplayId, Edge)>,
}

impl From<Written> for Changes {
    fn from(written: Written) -> Self {
        Self {
            edge: written.edge,
            hiding: written.hiding,
            displays: written.displays,
        }
    }
}

impl From<Changes> for Written {
    fn from(changes: Changes) -> Self {
        Self {
            edge: changes.edge,
            hiding: changes.hiding,
            displays: changes.displays,
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
