//! **A Place remembers time** — `docs/autonomy/the-canvas-and-its-places.md`
//! task 8, on the shell's side.
//!
//! `alo_arranging` holds what each Place was and assembles the series as it
//! keeps one. This is the other half: carrying that series into the session,
//! and putting a Place back as it was through the canvas a person is looking
//! at.
//!
//! # Why this is a chord and not the ribbon the promise names
//!
//! `docs/features.md` carries **two** promises here and they are at two tiers:
//!
//! ```text
//! :490  [v0.01] A Place remembers time — drag the ribbon and the canvas is
//!               as it was on Tuesday
//! :516  [v1.1]  The time ribbon — a ribbon at the bottom edge; dragging it
//!               back fades the whole desktop into the past, with what changed
//!               glowing, and letting go restores what the person picks
//! ```
//!
//! So **the strip is not in this release and building it here would cross the
//! scope gate rather than finish a task.** What is in this release is the
//! capability, and a capability with no road is the fault
//! `the-shell-plan.md` task 19 was written about. A chord is the road tasks 3
//! and 6 of this plan both took, in their own words: *neither road is the only
//! road*.
//!
//! **One step back, never a named moment**, and that is what keeps the two
//! apart rather than blurring them. Naming *Tuesday* needs a surface to name it
//! in, and that surface is the `[v1.1]` one. Stepping back needs none, walks
//! the same series the ribbon will one day draw, and leaves the ribbon with
//! something to be built on top of rather than something to replace.
//!
//! # Going back is itself a rearrangement
//!
//! `Arrangement::as_it_was` holds the state being left before it restores the
//! one asked for, which is the acceptance's own sentence: *the person is never
//! shown a state they cannot get back from.* So pressing the chord twice walks
//! backwards rather than losing what was in between, and the canvas somebody
//! had a moment ago is never the thing that goes missing.
//!
//! # A restored place is held to the rule a dragged one is
//!
//! A frame's remembered position can be somewhere nothing could get it back
//! from — under the Dock, behind the put-aside panel, off a narrower display
//! than the one the state was held on. `crate::canvas_remembered` asks
//! `crate::canvas_never_lost`'s question of every place it offers at sign-in
//! for exactly that reason, and this asks the same question of every frame it
//! moves. **A window that fails it is left where it is rather than the restore
//! being refused**: the person asked for the canvas they had, and giving them
//! most of it with one window where they can still reach it is better than
//! giving them none of it.

use std::time::SystemTime;

use alo_arranging::{Arrangement, AsItWas};

use crate::Server;

impl Server {
    /// What this person's Places remember, told as the session keeps a layout.
    ///
    /// Told rather than read, for the reason
    /// `crate::a_chord_reaches_its_action` gives about the shortcuts: a chord
    /// is answered at the seat, where no desktop is in scope, and the file in a
    /// person's folder is the desktop's to read.
    ///
    /// **Cloned at the cadence of a save, not of a frame.**
    /// `crate::direct_desktop` calls this only where the layout actually
    /// changed, which is where it already decides to write the file.
    pub fn these_places_remember(&mut self, series: &Arrangement) {
        self.remembered = series.clone();
        // **A new series ends a walk.** The session is told one only where the
        // layout actually changed, and a layout that changed while somebody was
        // stepping backwards changed because they moved something: they are
        // arranging again, not walking, and the next press should start from
        // what is in front of them.
        self.walking_back_from = None;
    }

    /// What they remember now, for whoever has to carry it forward.
    ///
    /// The session builds each new layout from the live canvas, which has no
    /// memory, so carrying the series forward needs the previous one — and this
    /// is where the previous one is. `alo_arranging::Arrangement::following`
    /// is what does the carrying; this only hands over what it needs.
    #[must_use]
    pub fn what_these_places_remember(&self) -> &Arrangement {
        &self.remembered
    }

    /// What the Place in front has been, oldest first.
    ///
    /// Empty for a Place nobody has rearranged twice, and for every Place in a
    /// session that has not kept a layout yet. **What a ribbon will be drawn
    /// from** when there is a ribbon; until then it is what says whether
    /// stepping back has anywhere to go.
    #[must_use]
    pub fn the_earlier_states_of_this_place(&self) -> Vec<AsItWas> {
        self.remembered.earlier_on(self.the_place_now())
    }

    /// Put the Place in front back as it was a moment ago, and say whether it
    /// moved.
    ///
    /// `false` where this Place has nothing further back to go — a canvas
    /// nobody has rearranged twice, and a walk that has reached its oldest
    /// held state. Both are ordinary things to press rather than faults,
    /// exactly as zooming in at the end of the ladder is.
    ///
    /// # Pressing it twice goes back twice, and that needed a cursor
    ///
    /// **The obvious version is a toggle and it was built before this one.**
    /// Going back is itself a rearrangement — `Arrangement::as_it_was` holds
    /// the canvas being left so it is not lost — so the state just left becomes
    /// the newest thing in the series. A second press that reached for *the
    /// newest held state* would therefore reach for the one it had this moment
    /// abandoned, and the chord would flip between two canvases however many
    /// times it was pressed. Measured, not reasoned about: the test named
    /// `the_canvas_a_person_leaves_is_itself_held` failed with the frame back
    /// where it started.
    ///
    /// So a walk keeps a cursor — **the moment this Place is currently
    /// showing** — and each press asks for the newest state *older than* it.
    /// [`Self::these_places_remember`] clears it, because being told a new
    /// series means the person moved something themselves and is no longer
    /// walking their own past.
    ///
    /// # What has no road yet, said rather than discovered
    ///
    /// **Forward.** The state a walk leaves behind is held, so nothing is
    /// destroyed and the acceptance's second clause is met — but no chord
    /// reaches it, so a person who steps back twice cannot step forward again
    /// without rearranging. Stepping forward is the ribbon's own gesture
    /// (*letting go restores what the person picks*), and the ribbon is
    /// `[v1.1]`. This is named here so the next reader finds it as a gap with a
    /// reason rather than as an oversight.
    pub fn go_back_on_this_place(&mut self, now: SystemTime) -> bool {
        let place = self.the_place_now();
        let showing = self.walking_back_from;
        let Some(when) = self
            .remembered
            .earlier_on(place)
            .into_iter()
            .map(|held| held.when())
            .rfind(|held| showing.is_none_or(|cursor| *held < cursor))
        else {
            return false;
        };
        if !self.remembered.as_it_was(place, when, now) {
            return false;
        }
        self.walking_back_from = Some(when);
        self.put_this_place_back_as_it_is_remembered(place);
        true
    }

    /// Move the frames on this Place to where the arrangement now says they
    /// were, and look where it says the person was looking.
    ///
    /// **Only this Place's frames.** A Place remembering time means the others
    /// stay where they are, which is the shape question task 8 answered from
    /// its own title: a session-wide series would make going back move every
    /// Place at once, which is the opposite of what Places are for.
    fn put_this_place_back_as_it_is_remembered(&mut self, place: alo_canvas::Place) {
        // Collected before anything moves, because placing a window borrows the
        // server that the next lookup needs.
        let moving: Vec<_> = self
            .mapped_surfaces()
            .filter(|&frame| self.the_place_of_the_window(frame) == Some(place))
            .cloned()
            .filter_map(|frame| {
                let app_id = self.the_app_id_of(&frame)?;
                let was = self.remembered.where_it_was(place, &app_id)?;
                Some((frame, was.normal().0))
            })
            .collect();
        for (frame, at) in moving {
            // The question a drag is held to, asked of a restore for the reason
            // this file's header gives. A frame that fails it stays where it is
            // and the rest of the canvas still comes back.
            if !self.show_all_would_still_reach(&frame, at) {
                continue;
            }
            let _ = self.place_window(&frame, (at.x, at.y));
        }
        if let Some(camera) = self.remembered.camera_on(place) {
            self.look_at_the_canvas(camera.at());
        }
    }
}

// The tests are `crates/alo-shell/tests/a_place_remembers_time/`, with the rest
// of this plan's: every one of them needs real frames on a real plane, and a
// unit test of the bookkeeping alone would be a test of `alo-arranging`'s
// series written a second time in the wrong crate.
