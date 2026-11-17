//! **alo's teal edge round a window, checked on the screen**: drawn while alo
//! works in a window, and gone once its turn has ended (`crate::the_edge_seen`).
//!
//! Each check looks at a fresh picture once a second until it holds or its
//! time is up, and keeps the last picture under its name, so a report can be
//! checked against the picture beside it.
//!
//! # What each would need to pass while the product is broken
//!
//! - **Drawn** passes on four thin deep-teal sides round any rectangle on the
//!   screen. It would pass on an edge round a window other than the one alo
//!   works in — which the shell's own tests rule out (one window wears alo, the
//!   one alo acted in) and this cannot.
//! - **Drawn**, once it holds, **follows the edge** while it stays drawn, for
//!   up to [`FOLLOW_AT_MOST`], remembering each place it is seen: a window
//!   can move or take another size after alo's edge is first round it.
//! - **Gone** passes once no edge is on the screen **and** the window the edge
//!   was round is still there, by its own edge where alo's was, at any of the
//!   places the edge was seen
//!   (`crate::the_edge_seen::a_window_is_where_the_edge_was`) — not by its
//!   pixels being as they were, which a window alo works in changes. It needs an edge
//!   seen first by *drawn*, so it is never the verdict of a check that could
//!   not see one; and it does not pass on a screen gone black, locked or
//!   covered, which would be a window lost rather than an edge.

use std::time::Duration;

use super::Walking;
use crate::frame::Frame;
use crate::machine::{Machine, MachineError};
use crate::report::Verdict;
use crate::the_edge_seen::{Nearest, Ring, a_ring_in, a_window_is_where_the_edge_was};

/// How often a fresh picture is looked at.
const LOOK_EVERY: Duration = Duration::from_secs(1);

/// **How long the edge is followed once it is seen**, while it stays drawn.
///
/// The walks of 2026-11-16 on `main` at 79ae24f0, a0e6b8e9 and 19202288
/// (runs 43946984805, 43964241118 and 43967583290) first saw the edge round
/// VSCodium's window before VSCodium had drawn, at the size and place it
/// opened with and with the window's band shown; VSCodium then drew at
/// another size, and the edge went with it. A walk that remembered only the
/// first place looked for the window there once the edge had gone, and said
/// it had gone too. The turns in those walks ended about four seconds after
/// the edge was first seen; this is long enough for that and short enough
/// not to hold up a walk whose turn runs on.
pub const FOLLOW_AT_MOST: Duration = Duration::from_secs(20);

/// What the walk knows about alo's edge: every place it was seen drawn,
/// first to last.
#[derive(Debug, Default)]
pub(super) struct TheEdge {
    /// Where the edge was seen drawn, each place once, in order.
    seen: Vec<Ring>,
}

/// What one picture answered.
enum Answered {
    /// The check holds.
    Holds(String),
    /// It does not, yet.
    Not(String),
}

impl Walking {
    /// Look for the edge drawn, or gone, until it is or `within` is up.
    pub(super) fn the_edge(
        &mut self,
        machine: &mut impl Machine,
        check: &str,
        picture: &str,
        drawn: bool,
        within: Duration,
    ) -> Result<(), MachineError> {
        let start = machine.elapsed();
        // **The screen when the edge is first looked for is kept**, so a
        // failure is never judged only from the last picture, taken when the
        // time is up — by then the turn may have ended and the edge gone with
        // it, as on the walk of 2026-11-16 (run 43889781022), whose one picture
        // was taken 21 seconds after the work it was looking for had ended.
        if drawn {
            let _kept = machine.keep_picture(&format!("{picture}-when-first-looked-for"))?;
        }
        loop {
            let frame = machine.frame()?;
            let answered = if drawn {
                self.drawn_in(&frame)
            } else {
                self.gone_from(&frame)
            };
            let took = machine.elapsed().saturating_sub(start);
            match answered {
                Answered::Holds(evidence) => {
                    let kept = machine.keep_picture(picture)?;
                    let evidence = if drawn {
                        format!("{evidence}{}", self.follow(machine, picture)?)
                    } else {
                        evidence
                    };
                    self.checked(
                        check.to_owned(),
                        Verdict::Passed,
                        format!("after {}s, {evidence}; kept as {kept}", took.as_secs()),
                    );
                    return Ok(());
                }
                Answered::Not(why) if took >= within => {
                    let kept = machine.keep_picture(picture)?;
                    self.checked(
                        check.to_owned(),
                        Verdict::Failed,
                        format!("after {}s, {why}; kept as {kept}", took.as_secs()),
                    );
                    self.stop_after_failure();
                    return Ok(());
                }
                Answered::Not(_) => machine.wait(LOOK_EVERY),
            }
        }
    }

    /// Whether the edge is drawn in this picture, remembering where.
    fn drawn_in(&mut self, frame: &Frame) -> Answered {
        match a_ring_in(frame) {
            Ok(ring) => {
                let said = format!("the edge {}", where_it_is(ring));
                self.edge.seen = vec![ring];
                Answered::Holds(said)
            }
            Err(nearest) => Answered::Not(no_edge(nearest)),
        }
    }

    /// **Follow the edge while it stays drawn**, for up to [`FOLLOW_AT_MOST`],
    /// remembering each new place it is seen; the last picture it is seen in
    /// is kept when it moved. Says, for the report, where it went.
    fn follow(
        &mut self,
        machine: &mut impl Machine,
        picture: &str,
    ) -> Result<String, MachineError> {
        let start = machine.elapsed();
        let mut moved = false;
        while machine.elapsed().saturating_sub(start) < FOLLOW_AT_MOST {
            machine.wait(LOOK_EVERY);
            let Ok(ring) = a_ring_in(&machine.frame()?) else {
                break;
            };
            if !self.edge.seen.contains(&ring) {
                self.edge.seen.push(ring);
                moved = true;
                let _kept = machine.keep_picture(&format!("{picture}-where-it-went"))?;
            }
        }
        let followed = machine.elapsed().saturating_sub(start).as_secs();
        Ok(match self.edge.seen.last() {
            Some(last) if moved => format!(
                "; followed for {followed}s, it went with its window to {}",
                where_it_is(*last)
            ),
            _ => format!("; followed for {followed}s, it stayed there"),
        })
    }

    /// Whether the edge seen before is gone from this picture, with its window
    /// still there — at the last place the edge was seen, or any before it.
    fn gone_from(&self, frame: &Frame) -> Answered {
        let Some(&last) = self.edge.seen.last() else {
            return Answered::Not(
                "no edge was seen before, so its going cannot be told from never being there"
                    .to_owned(),
            );
        };
        if let Ok(still) = a_ring_in(frame) {
            return Answered::Not(format!("the edge is still drawn {}", where_it_is(still)));
        }
        let Some(ring) = self
            .edge
            .seen
            .iter()
            .rev()
            .find(|ring| a_window_is_where_the_edge_was(**ring, frame))
        else {
            let places = self.edge.seen.len();
            return Answered::Not(format!(
                "no edge, and the window it was round {} is not there either{}",
                where_it_is(last),
                if places > 1 {
                    format!(", nor at the {} places it was before", places - 1)
                } else {
                    String::new()
                }
            ));
        };
        Answered::Holds(format!(
            "no edge, and the window it was round {} is still there",
            where_it_is(*ring)
        ))
    }
}

/// Where an edge is, as a report says it.
fn where_it_is(ring: Ring) -> String {
    format!(
        "from ({}, {}) to ({}, {})",
        ring.left, ring.top, ring.right, ring.bottom
    )
}

/// What was found instead of an edge.
fn no_edge(nearest: Nearest) -> String {
    format!(
        "no edge: the longest teal line across is {} pixels and down {}",
        nearest.across, nearest.down
    )
}
