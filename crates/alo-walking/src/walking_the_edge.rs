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
//! - **Gone** passes once no edge is on the screen **and** the window the edge
//!   was round is still there, by its own edge where alo's was
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

/// What the walk knows about alo's edge: where it was last seen, and the
/// picture it was seen in.
#[derive(Debug, Default)]
pub(super) struct TheEdge {
    /// Where the edge was last seen drawn.
    seen: Option<Ring>,
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
                self.edge.seen = Some(ring);
                Answered::Holds(said)
            }
            Err(nearest) => Answered::Not(no_edge(nearest)),
        }
    }

    /// Whether the edge seen before is gone from this picture, with its window
    /// still there.
    fn gone_from(&self, frame: &Frame) -> Answered {
        let Some(ring) = self.edge.seen else {
            return Answered::Not(
                "no edge was seen before, so its going cannot be told from never being there"
                    .to_owned(),
            );
        };
        if let Ok(still) = a_ring_in(frame) {
            return Answered::Not(format!("the edge is still drawn {}", where_it_is(still)));
        }
        if !a_window_is_where_the_edge_was(ring, frame) {
            return Answered::Not(format!(
                "no edge, and the window it was round {} is not there either",
                where_it_is(ring)
            ));
        }
        Answered::Holds(format!(
            "no edge, and the window it was round {} is still there",
            where_it_is(ring)
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
