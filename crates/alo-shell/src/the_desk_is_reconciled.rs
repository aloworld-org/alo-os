//! Bring the displays this session draws to back in line with the machine.
//!
//! Called when [`crate::a_screen_arrives_or_leaves`] has said the desk changed.
//! That answer is cheap and may be over-eager; **this is the authoritative
//! one**, because it runs the same discovery the session started with.
//!
//! # Discovery names both directions, so nothing is formatted by hand
//!
//! A display's name comes from its connector —
//! `crate::output_metadata::direct_metadata` builds it and is the only place
//! that does. So this asks discovery what it reports *now* and compares names:
//! a target whose name discovery no longer returns has gone, and a name
//! discovery returns that no target holds has arrived.
//!
//! The alternative was to turn the departed connector handles into names here.
//! That would be a second place that has to agree with how a display is named,
//! and the two would drift the first time the naming changed — so the handles
//! are used only to decide *whether* to ask, never to decide *what* to retire.
//!
//! # A monitor that stayed plugged in is not touched
//!
//! Rebuilding every display from the fresh discovery would be far simpler and
//! would make the screens a person is looking at go black and come back. Only
//! the difference is acted on.
//!
//! # The loop's own display is excluded, and by name
//!
//! `run_with_input` holds one target and six `LoopInput` implementations hang
//! off that signature, so the first display belongs to the loop rather than to
//! `Desk`. It is excluded **by its name** rather than by being first in the
//! fresh discovery, and that distinction is load-bearing: discovery orders a
//! usable internal panel first and then by ascending connector, so plugging in
//! a monitor with a lower connector number than the current first would make
//! *it* first. Excluding "the first" would then retire the display the loop is
//! drawing to and adopt the new one as an extra — on a machine whose primary
//! screen never changed.
//!
//! **So a primary display being unplugged is not handled here**, and that is
//! reported rather than hidden: the loop owns that target for its whole run.
//! Task 13 is about a desk noticing a change at all; replacing the target a
//! loop was built around is a different change to a different signature.

use crate::{FrameTarget, Server};
use std::os::fd::BorrowedFd;

/// Every display this `Desk` draws to, besides the loop's own.
type Others<'a> = Vec<
    crate::direct_target::Target<
        crate::software_scanout::SoftwarePainter,
        crate::drm_inventory::Inventory<'a>,
    >,
>;

/// What changed on the desk, in words a report can use.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Reconciled {
    /// Displays now drawn to that were not before.
    pub(crate) presented: Vec<String>,
    /// Displays withdrawn because the machine no longer reports them.
    pub(crate) retired: Vec<String>,
    /// What refused, and which display it was.
    ///
    /// **A refusal does not stop the session**, which is the same choice the
    /// rest of this lane makes: a display whose painter cannot be made is left
    /// out, and the machine still has the screens it already had.
    pub(crate) refusals: Vec<String>,
}

/// Which of the drawn displays to withdraw, and which reported ones to add.
///
/// **Pure, and separated out for one reason: the rest of this file cannot be
/// tested.** `the_desk_is_reconciled` needs a device descriptor, a real DRM
/// discovery and a painter, so a test for it needs a machine with monitors on
/// it. The *decision* needs none of those — it is two lists of names — and it
/// is the part with the mistakes in it. So the decision is here, with tests,
/// and the function below is the thin part that does the input and output.
///
/// `drawn` carries [`None`] for a display that will not name itself, so this
/// can say what happens to it rather than the caller having to remember.
pub(crate) fn what_the_desk_now_holds(
    reported: &[String],
    drawn: &[Option<String>],
    the_loops_own: &str,
) -> DeskChange {
    let retire = drawn
        .iter()
        .enumerate()
        .filter_map(|(which, named)| {
            let named = named.as_ref()?;
            // **A display the machine still reports stays, untouched.**
            // Rebuilding it would make a screen nobody unplugged go black and
            // come back.
            (!reported.iter().any(|it| it == named)).then_some(which)
        })
        .collect();
    let present = reported
        .iter()
        .filter(|named| {
            // **The loop's own display, excluded by name.** Not by being first
            // in the fresh discovery: discovery orders an internal panel first
            // and then by ascending connector, so a monitor plugged into a
            // lower-numbered port becomes first. Excluding "the first" would
            // retire the display the loop is drawing to and adopt the new one
            // as an extra, on a machine whose primary screen never changed.
            *named != the_loops_own && !drawn.iter().flatten().any(|already| already == *named)
        })
        .cloned()
        .collect();
    DeskChange { retire, present }
}

/// What the desk should become: positions to withdraw, names to add.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct DeskChange {
    /// Positions in the drawn list to withdraw, ascending.
    ///
    /// Positions rather than names, because the caller holds the targets in
    /// that order and two displays could in principle answer with one name —
    /// a position names exactly one target and a string does not.
    pub(crate) retire: Vec<usize>,
    /// Displays the machine reports that nothing draws to yet.
    pub(crate) present: Vec<String>,
}

/// Make the displays drawn to match the displays the machine reports.
pub(crate) fn the_desk_is_reconciled<'a>(
    server: &mut Server,
    device: BorrowedFd<'a>,
    the_loops_own: &str,
    others: &mut Others<'a>,
) -> Reconciled {
    let mut done = Reconciled::default();
    let found = match crate::discover_every_atomic_output(device) {
        Ok(found) => found,
        // **A device that cannot be asked keeps the displays it had.** An
        // empty answer and a failed question are not the same thing: read as
        // *no displays*, one refusal would retire every screen the person is
        // looking at.
        Err(why) => {
            done.refusals
                .push(format!("the machine would not say: {why}"));
            return done;
        }
    };
    let mut reported = Vec::new();
    let mut arrivals = Vec::new();
    for output in found {
        let Ok(metadata) = crate::output_metadata::direct_metadata(&output.output) else {
            continue;
        };
        reported.push(metadata.name.clone());
        if metadata.name != the_loops_own {
            arrivals.push((metadata.name, output));
        }
    }
    let drawn: Vec<Option<String>> = others
        .iter()
        .map(|target| FrameTarget::metadata(target).map(|it| it.name).ok())
        .collect();
    let change = what_the_desk_now_holds(&reported, &drawn, the_loops_own);
    // Descending, so each removal cannot move a position still to be removed.
    for which in change.retire.iter().rev() {
        if *which >= others.len() {
            continue;
        }
        let mut target = others.remove(*which);
        let named = FrameTarget::metadata(&target)
            .map(|it| it.name)
            .unwrap_or_else(|_| format!("the display drawn {which}th"));
        if let Err(why) = target.retire() {
            done.refusals
                .push(format!("{named} would not retire: {why}"));
        }
        server.this_display_has_gone(&named);
        done.retired.push(named);
    }
    for (name, output) in arrivals {
        if !change.present.contains(&name) {
            continue;
        }
        // A painter each, because a `Target` owns its painter, and a borrow
        // each of the one descriptor — the same shape the session's own setup
        // uses, and the reason N displays are N painters and one `fd`.
        let Ok(painter) = crate::software_scanout::SoftwarePainter::new() else {
            done.refusals
                .push(format!("{name} arrived and could not be painted"));
            continue;
        };
        others.push(crate::direct_target::Target::new(
            painter,
            crate::drm_inventory::Inventory(device),
            output,
        ));
        done.presented.push(name);
    }
    done
}

#[cfg(test)]
#[path = "the_desk_is_reconciled_tests.rs"]
mod tests;
