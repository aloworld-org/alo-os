//! **What the service told between two frames, so that no departure it told
//! is missed by every frame.**
//!
//! `alo-agentd` tells the desktop its whole list each time it changes (ADR
//! 0134), and the desktop takes what arrived once a frame. Taking only the
//! newest list is right for what is leaving *now*, and wrong for what left
//! *in between*: a question that went out and came back between two frames is
//! told as a lit list and then an empty one, and the newest-only reading draws
//! neither. On a machine that draws sixty frames a second that gap is a few
//! milliseconds. On a slow one it is not — the twelfth Demo 1 walk, in a
//! virtual machine whose input trailed by seconds, took twenty-five pictures
//! while questions were away and found the indicator on none of them.
//!
//! So a line told and taken back before the next frame is still drawn on that
//! frame, beside whatever the newest list holds, and the newest list alone is
//! drawn on the frame after. **This is the one-frame trailing ADR 0134 already
//! accepts**, applied to the end of a departure as well as its start; it adds no
//! delay to a line going up.
//!
//! # What it does not do
//!
//! It holds a missed line for **one frame**, not for a length of time a person
//! could be sure to see. How long a line that left and returned in a moment
//! should stay on the screen is not settled anywhere in this repository, and a
//! duration chosen here would be a promise nobody made.

use alo_egress::Reported;

use crate::told::{FromTheDaemon, ToldByTheDaemon};

/// What was told since the last frame and has not yet been drawn.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BetweenFrames {
    /// The newest list, held back one frame because a frame before it had to
    /// draw lines it no longer holds.
    owed: Option<FromTheDaemon>,
}

impl BetweenFrames {
    /// Nothing told yet.
    #[must_use]
    pub const fn nothing_owed() -> Self {
        Self { owed: None }
    }

    /// What this frame draws, given everything told since the last one, oldest
    /// first.
    ///
    /// `None` when nothing was told and nothing is owed: the frame draws what
    /// the last one did. Otherwise the newest list — **unless** a list told
    /// before it held lines the newest does not, in which case this frame draws
    /// those lines followed by the newest list's, and the newest list alone is
    /// owed to the next frame.
    ///
    /// When the newest is *cannot tell* and lines were missed, this frame draws
    /// the missed lines and the next one says it cannot tell. A line that left
    /// is drawn before an absence of knowledge is, because the line is what law
    /// 1 is about.
    pub fn this_frame(
        &mut self,
        told: impl IntoIterator<Item = FromTheDaemon>,
    ) -> Option<FromTheDaemon> {
        let mut told: Vec<FromTheDaemon> = self.owed.take().into_iter().chain(told).collect();
        let newest = told.pop()?;
        let mut missed: Vec<Reported> = Vec::new();
        for line in told.iter().flat_map(FromTheDaemon::lines) {
            if !newest.lines().contains(line) && !missed.contains(line) {
                missed.push(line.clone());
            }
        }
        if missed.is_empty() {
            return Some(newest);
        }
        missed.extend(newest.lines().iter().cloned());
        self.owed = Some(newest);
        Some(FromTheDaemon::Told(ToldByTheDaemon::heard(missed)))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{asking_a_provider, fetching, noon};
    use alo_egress::{EgressPolicy, Indicator};

    /// The service's list while `leaving` is away, as the socket would carry it.
    fn told_while(leaving: &[alo_egress::Leaving]) -> FromTheDaemon {
        let mut indicator = Indicator::default();
        let mut away = Vec::new();
        for one in leaving {
            away.push(
                indicator
                    .beginning(&EgressPolicy::Anywhere, one.clone(), noon())
                    .unwrap(),
            );
        }
        let lines = indicator.showing().iter().map(Reported::of).collect();
        for departing in away {
            assert!(indicator.ended(departing));
        }
        FromTheDaemon::Told(ToldByTheDaemon::heard(lines))
    }

    /// The service's list with nothing away.
    fn told_nothing() -> FromTheDaemon {
        FromTheDaemon::Told(ToldByTheDaemon::heard(Vec::new()))
    }

    /// **A question that went and came back between two frames is drawn on
    /// the next one**, and the frame after that is dark. Read newest-only, both
    /// frames were dark and the question was never on the screen at all.
    #[test]
    fn a_departure_told_and_taken_back_between_frames_is_drawn_once() {
        let mut between = BetweenFrames::nothing_owed();
        let away = told_while(&[asking_a_provider()]);

        let drawn = between.this_frame([away.clone(), told_nothing()]).unwrap();
        assert_eq!(drawn.lines(), away.lines());

        let after = between.this_frame([]).unwrap();
        assert_eq!(after, told_nothing());
        assert_eq!(between.this_frame([]), None, "nothing is owed any longer");
    }

    /// **With nothing missed, the newest list is drawn and nothing is owed**:
    /// the ordinary case adds no frame of delay.
    #[test]
    fn the_newest_list_is_drawn_when_it_holds_everything_told_before_it() {
        let mut between = BetweenFrames::nothing_owed();
        let one = told_while(&[asking_a_provider()]);
        let both = told_while(&[asking_a_provider(), fetching()]);

        assert_eq!(between.this_frame([one, both.clone()]), Some(both));
        assert_eq!(between.this_frame([]), None);
    }

    /// **Nothing told and nothing owed changes nothing**, so the frame keeps
    /// what it showed.
    #[test]
    fn a_frame_with_nothing_told_changes_nothing() {
        assert_eq!(BetweenFrames::nothing_owed().this_frame([]), None);
    }

    /// **A missed line is drawn before *cannot tell* is**, and *cannot tell*
    /// follows on the next frame rather than being lost.
    #[test]
    fn a_line_missed_before_the_service_went_quiet_is_drawn_and_then_cannot_tell() {
        let mut between = BetweenFrames::nothing_owed();
        let away = told_while(&[fetching()]);

        let drawn = between
            .this_frame([away.clone(), FromTheDaemon::Unreachable])
            .unwrap();
        assert_eq!(drawn.lines(), away.lines());
        assert!(!drawn.cannot_tell());

        assert_eq!(between.this_frame([]), Some(FromTheDaemon::Unreachable));
    }

    /// **What arrives while a list is owed supersedes it**, and the owed list's
    /// lines are not lost if the newer one dropped them.
    #[test]
    fn a_newer_list_replaces_what_was_owed() {
        let mut between = BetweenFrames::nothing_owed();
        let asking = told_while(&[asking_a_provider()]);
        let fetching_too = told_while(&[asking_a_provider(), fetching()]);

        // `fetching` went and came back between frames, beside a question
        // still away: drawn with it, and the question alone is owed.
        let drawn = between
            .this_frame([fetching_too.clone(), asking.clone()])
            .unwrap();
        assert_eq!(drawn.lines().len(), 2);

        // The question is still away on the next frame: owed and newest agree.
        assert_eq!(between.this_frame([asking.clone()]), Some(asking));
        assert_eq!(between.this_frame([]), None);
    }
}
