//! What a person changed about their dock, which is the only part that is
//! written down.
//!
//! `alo-shortcuts`' shape and `alo-appearance`'s, for the third time and for the
//! same reason: the defaults live in the running release and this file holds the
//! difference, so a release that moves a default reaches every machine that
//! never touched it and no machine that did. An untouched machine has no
//! `dock.toml` at all ([`crate::keeping`]).
//!
//! **Two things to change: where the dock goes, and whether it hides.**
//!
//! *Where* was withdrawn by [ADR
//! 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
//! on 2026-09-29 and **the owner reversed that within a day** — *bottom should
//! be the default; the person can choose bottom, left, right, or top.* Their
//! order of work of 2026-10-04 then held the setting back while only two of the
//! four edges could be laid out: *nonfunctional edge choices are not to be
//! exposed as finished settings.*
//!
//! **That condition is met, which is why the field is here now.** All four lay
//! out — the owner's ruling of 2026-10-10 gave a side dock's names a tooltip
//! beside the icon, which was the one missing measurement — and all four draw,
//! on their own edge, with the corner following. The header of this file argued
//! from the withdrawn decision until today, eleven days after it was reversed.
//!
//! **`displays` still reads and still means nothing.** It was written by earlier
//! releases for per-display exceptions, which `docs/features.md` keeps at
//! **[v0.5]**; a person's `dock.toml` is not rewritten behind them, so it stays
//! a key the file *may* have with nothing behind it. That is in
//! [`crate::keeping`], where the file's keys are declared; here it is simply
//! absent from the private `Written` shape below, and serde skips a key no field
//! claims. **`edge` is no longer one of those** — it is read, kept and honoured.
//!
//! *The dock's size* is still not here. `crate::layout` sizes it.

use serde::{Deserialize, Serialize};

use crate::edge::Edge;
use crate::hiding::Hiding;

/// One thing a person can change about their dock, for a settings panel that
/// offers *put it back*.
///
/// **Two variants, which is what it was before ADR 0076 and is again.** It
/// gains more additively when *the dock's size* arrives at v0.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Setting {
    /// Whether the dock gives way when a window needs the room. There is no
    /// per-display exception for it: a dock that hid on one screen and not
    /// another would be a dock a person cannot predict, and predicting where it
    /// is, is most of what a dock is for.
    Hiding,
    /// Which edge of the screen the dock is on.
    ///
    /// **One edge for every display, at this release.** *Per display, so the
    /// dock can sit along the bottom of the laptop and down the side of the
    /// external screen* is a separate promise at **[v0.5]** in
    /// `docs/features.md`, and the `displays` key that earlier releases wrote
    /// for it is still read and still means nothing.
    WhereItGoes,
}

/// Everything a person has changed about their dock.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Written", into = "Written")]
pub struct Changes {
    /// Whether they asked it to give way, if they said anything about it.
    hiding: Option<Hiding>,
    /// Which edge they put it on, if they said anything about it.
    ///
    /// **An `Option`, like the one above, and for the release-coupling reason
    /// this file exists for**: absent means *whatever this release ships*, so a
    /// release that moved the default edge would reach every machine that never
    /// chose and no machine that did. `Edge::Bottom` stored here is a person
    /// having picked the bottom on purpose, which is not the same fact as never
    /// having opened the setting.
    edge: Option<Edge>,
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
        self.hiding.is_none() && self.edge.is_none()
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

    /// Say which edge of the screen the dock is on.
    pub const fn set_edge(&mut self, edge: Edge) {
        self.edge = Some(edge);
    }

    /// Which edge they put it on, if they chose.
    #[must_use]
    pub const fn edge(&self) -> Option<Edge> {
        self.edge
    }

    /// Forget that this was ever changed, which puts it back to what the running
    /// release ships.
    ///
    /// Says whether there was anything to forget.
    pub const fn forget(&mut self, setting: Setting) -> bool {
        match setting {
            Setting::Hiding => self.hiding.take().is_some(),
            Setting::WhereItGoes => self.edge.take().is_some(),
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
/// **There is no `displays` field**, and its absence is what makes a file that
/// has one read: serde ignores a key no field claims, so a `dock.toml` written
/// by an earlier release loads with its per-display exceptions dropped rather
/// than refused. [`crate::keeping`] is what keeps that name recognised, since an
/// unrecognised key *is* refused there.
///
/// **`edge` is a field now**, so a file naming one is honoured rather than
/// ignored. A file from a release that wrote an edge this one does not know —
/// there is no fifth edge, but a future one is not this file's to rule out —
/// fails to deserialize and is reported through `crate::keeping` like any other
/// unreadable file, rather than being silently read as the bottom.
#[derive(Default, Serialize, Deserialize)]
struct Written {
    /// Whether it gives way, if anything was said about it. Absent rather than
    /// present and null, so a machine that has not chosen writes exactly the
    /// file it wrote before this key existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hiding: Option<Hiding>,
    /// Which edge it is on, if anything was said about it. Absent rather than
    /// present and null, for the reason above.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    edge: Option<Edge>,
}

impl From<Written> for Changes {
    fn from(written: Written) -> Self {
        Self {
            hiding: written.hiding,
            edge: written.edge,
        }
    }
}

impl From<Changes> for Written {
    fn from(changes: Changes) -> Self {
        Self {
            hiding: changes.hiding,
            edge: changes.edge,
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

    /// **A file naming an edge is honoured.** This test asserted the opposite
    /// until 2026-10-10, and task 11 of `the-smallest-canvas-worth-showing.md`
    /// named it as the thing that changes when this is done: *a `dock.toml` that
    /// names an edge is **honoured** rather than ignored, and the two tests
    /// asserting the opposite are replaced in the same change.*
    ///
    /// **So a person who moved their dock before ADR 0076 gets it back.** Every
    /// release before that wrote `edge`, those files were read with the edge
    /// dropped for eleven days, and a person's own file is not rewritten behind
    /// them — so the choice was still sitting there waiting to mean something
    /// again.
    #[test]
    fn a_file_naming_an_edge_is_honoured() {
        for (text, wanted) in [
            (r#"{"edge":"Bottom"}"#, Edge::Bottom),
            (r#"{"edge":"Left"}"#, Edge::Left),
            (r#"{"edge":"Right"}"#, Edge::Right),
            (r#"{"edge":"Top"}"#, Edge::Top),
        ] {
            let changes = serde_json::from_str::<Changes>(text).unwrap();
            assert_eq!(changes.edge(), Some(wanted), "{text} was not honoured");
            assert!(
                !changes.is_untouched(),
                "{text} names a choice, so the machine is not untouched"
            );
        }
    }

    /// **An edge that is not one is refused, and that is a change in behaviour
    /// worth naming.**
    ///
    /// While nothing read the key, `"Middle"` and `3` were tolerated — refusing
    /// a file over a word nothing reads would have been refusing it for nothing.
    /// Now the key means something, so a value that is not an edge is treated
    /// exactly as a `hiding` that is not a hiding: the file does not read, and
    /// `crate::keeping` reports it as not understood rather than guessing.
    ///
    /// **Nobody's file says `Middle`.** No release ever wrote anything but the
    /// four names, so this changes what happens to a typo and to nothing else.
    #[test]
    fn an_edge_that_is_not_one_is_refused_rather_than_guessed_at() {
        for text in [r#"{"edge":"Middle"}"#, r#"{"edge":3}"#] {
            assert!(
                serde_json::from_str::<Changes>(text).is_err(),
                "{text} was read as something rather than refused"
            );
        }
        // **But an explicit `null` is not a bad value, it is an absence.**
        // `#[serde(default)]` reads it as *nothing was said*, which is the right
        // answer: a key present and null means the same as a key missing, and
        // refusing it would refuse a file that says nothing wrong. Asserted
        // rather than left to be discovered, because it was in the list above
        // until this test ran.
        let nothing = serde_json::from_str::<Changes>(r#"{"edge":null}"#).unwrap();
        assert_eq!(nothing.edge(), None);
        assert!(nothing.is_untouched());
    }

    /// **`displays` still reads and still means nothing.** It held per-display
    /// exceptions, which `docs/features.md` keeps at **[v0.5]**, and a file that
    /// has one loads as a machine that changed nothing.
    #[test]
    fn a_file_naming_displays_reads_and_they_are_ignored() {
        let changes = serde_json::from_str::<Changes>(r#"{"displays":[["DP-3","Left"]]}"#).unwrap();
        assert!(changes.is_untouched());
    }

    /// **A file that names an edge *and* says something about hiding keeps
    /// both.** It kept only the second until today.
    #[test]
    fn a_file_that_names_both_keeps_both() {
        let changes =
            serde_json::from_str::<Changes>(r#"{"edge":"Top","hiding":"WhenAWindowNeedsTheRoom"}"#)
                .unwrap();
        assert_eq!(changes.hiding(), Some(Hiding::WhenAWindowNeedsTheRoom));
        assert_eq!(changes.edge(), Some(Edge::Top));
        assert!(!changes.is_untouched());
    }

    /// **An untouched machine still writes no edge**, which is the whole of the
    /// release-coupling: a file with an `edge` key in it is a person's choice,
    /// and a file without one follows the release.
    #[test]
    fn choosing_only_hiding_writes_no_edge_key() {
        let mut changes = Changes::untouched();
        changes.set_hiding(Hiding::WhenAWindowNeedsTheRoom);
        let written = serde_json::to_string(&changes).unwrap();
        assert!(
            !written.contains("edge"),
            "a machine that never chose an edge wrote one: {written}"
        );
        changes.set_edge(Edge::Left);
        let written = serde_json::to_string(&changes).unwrap();
        assert!(written.contains(r#""edge":"Left""#), "{written}");
        assert_eq!(serde_json::from_str::<Changes>(&written).unwrap(), changes);
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
