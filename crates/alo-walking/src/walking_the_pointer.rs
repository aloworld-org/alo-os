//! **The pointer checks, carried out**: is the pointer in the screen capture at
//! all, move it onto a place by reading it back off the screen, and is what is
//! drawn there alo's pointer in the look the person chose.
//!
//! Every check here keeps a picture, and every one says in its evidence what it
//! found — the shape, the colour, the size in pixels and where — so a report
//! can be checked against the picture beside it rather than believed.
//!
//! # What each check would need to pass while the pointer is broken
//!
//! - **Seen at all** passes on any small change when the pointer moves. It
//!   would pass if something else small changed at the same moment and the
//!   pointer was not in the capture — which is why it is never the check that a
//!   pointer is alo's, only the check that the others are not checks of
//!   nothing.
//! - **Point at** passes when alo's pointer is found within a few pixels of the
//!   place. It would pass on a picture of alo's pointer sitting at that place
//!   while the real one was elsewhere — which the next check rules out.
//! - **Is drawn as** passes when the shape, in the person's colour and at their
//!   size, is on the screen *and moves when the pointer is moved*. It would pass
//!   if something drew alo's exact pixels and moved them with the hand without
//!   being the pointer, which nothing on a desktop does. It does **not** see a
//!   pointer drawn by a renderer that changed its solid pixels, and then it
//!   fails, saying what it found instead.

use std::time::Duration;

use alo_cursors::{CursorIcon, EVERY_SHAPE};

use super::Walking;
use crate::a_place_on_the_screen::{self, Target};
use crate::frame::Frame;
use crate::machine::{Machine, MachineError};
use crate::report::Verdict;
use crate::steering::{self, Steered};
use crate::step::Pointing;
use crate::the_pointer_seen::{Around, Drawn, Look, Reference, found_in};

/// The display scales a first sighting tries, in per cent, most likely first.
pub const SCALES: [u16; 8] = [100, 125, 150, 175, 200, 225, 250, 300];

/// The shapes a failed check looks for to say what is drawn instead: the
/// arrow, the text cursor and the hand.
const THE_THREE: [CursorIcon; 3] = [CursorIcon::Default, CursorIcon::Text, CursorIcon::Pointer];

/// The move that shows whether the capture has the pointer in it, from a
/// third of the way across and down the screen.
const A_SMALL_MOVE: (usize, usize) = (24, 16);

/// The nudge that shows a pointer found is the one the hand moves.
const A_NUDGE: (usize, usize) = (6, 4);

/// How far around where the pointer should be a search looks first, before it
/// looks everywhere.
const NEAR: usize = 160;

/// How far a nudged pointer is looked for from where it was.
const NUDGED_REACH: usize = 48;

/// The most pixels moving the pointer may change and still be only the
/// pointer: two of the largest a walk's looks draw, 48 pixels at 300 per
/// cent, with room for a hover.
const MOST_A_POINTER_CHANGES: usize = 2 * 144 * 144 + 4096;

/// How long a click holds the button down, as a hand does.
const A_CLICK_HELD: Duration = Duration::from_millis(80);

/// How long the compositor has to draw a pointer that just changed shape.
const BEFORE_A_LOOK: Duration = Duration::from_millis(500);

/// What the walk knows about the pointer.
#[derive(Debug, Default)]
pub(super) struct ThePointer {
    /// The look the person chose, if they chose one.
    chosen: Option<Look>,
    /// The display's scale, once a pointer has been found.
    scale: Option<u16>,
    /// Where the hotspot was last found.
    at: Option<(usize, usize)>,
    /// The screen kept for a later [`Target::WhatChanged`].
    kept: Option<Frame>,
}

impl ThePointer {
    /// The look the checks expect.
    fn look(&self) -> Look {
        self.chosen.unwrap_or(Look::SHIPPED)
    }

    /// The scales to try: the one found, or all of them.
    fn scales(&self) -> Vec<u16> {
        self.scale.map_or_else(|| SCALES.to_vec(), |it| vec![it])
    }
}

impl Walking {
    /// Carry out one pointer step.
    pub(super) fn pointing(
        &mut self,
        machine: &mut impl Machine,
        pointing: &Pointing,
    ) -> Result<(), MachineError> {
        match pointing {
            Pointing::SeenAtAll { check } => self.seen_at_all(machine, check),
            Pointing::ThePersonChose(look) => {
                self.pointer.chosen = Some(*look);
                self.did(
                    machine.elapsed(),
                    format!("the person chose the {}", look.said()),
                );
                Ok(())
            }
            Pointing::Remember => {
                self.pointer.kept = Some(machine.frame()?);
                self.did(
                    machine.elapsed(),
                    "kept the screen, to find what changes".to_owned(),
                );
                Ok(())
            }
            Pointing::PointAt { check, at } => self.point_at(machine, check, *at),
            Pointing::Click => {
                machine.button(true)?;
                machine.wait(A_CLICK_HELD);
                machine.button(false)?;
                self.did(machine.elapsed(), "clicked".to_owned());
                Ok(())
            }
            Pointing::IsDrawnAs { check, icon } => self.is_drawn_as(machine, check, *icon),
        }
    }

    /// Move the pointer a little: the capture must change, and only by a
    /// pointer's worth.
    fn seen_at_all(&mut self, machine: &mut impl Machine, check: &str) -> Result<(), MachineError> {
        let (width, height) = machine.frame()?.size();
        let from = (width / 3, height / 3);
        let to = (from.0 + A_SMALL_MOVE.0, from.1 + A_SMALL_MOVE.1);
        steering::put(machine, from)?;
        let before = machine.frame()?;
        steering::put(machine, to)?;
        let after = machine.frame()?;
        let kept = machine.keep_picture(&picture_name(check))?;
        let changed = after.differs_from(&before);
        let (verdict, evidence) = if changed == 0 {
            (
                Verdict::Failed,
                format!(
                    "moving the pointer from {from:?} to {to:?} changed no pixel of the screen \
                     capture: the capture does not show the pointer, or the pointer did not \
                     move, and every pointer check after this would be a check of nothing; \
                     kept as {kept}"
                ),
            )
        } else if changed > MOST_A_POINTER_CHANGES {
            (
                Verdict::Failed,
                format!(
                    "moving the pointer from {from:?} to {to:?} changed {changed} pixels, more than \
                     a pointer's {MOST_A_POINTER_CHANGES}: something else changed too, so this \
                     cannot say the pointer is in the capture; kept as {kept}"
                ),
            )
        } else {
            (
                Verdict::Passed,
                format!(
                    "moving the pointer from {from:?} to {to:?} changed {changed} pixels; kept as \
                     {kept}"
                ),
            )
        };
        self.checked(check.to_owned(), verdict, evidence);
        if verdict == Verdict::Failed {
            self.stop_after_failure();
        }
        Ok(())
    }

    /// Steer the pointer onto `target`.
    fn point_at(
        &mut self,
        machine: &mut impl Machine,
        check: &str,
        target: Target,
    ) -> Result<(), MachineError> {
        let now = machine.frame()?;
        let Some(to) = a_place_on_the_screen::where_is(target, &now, self.pointer.kept.as_ref())
        else {
            let kept = machine.keep_picture(&picture_name(check))?;
            self.failed_and_stopped(
                check,
                format!("{} is not on the screen; kept as {kept}", target.said()),
            );
            return Ok(());
        };
        let Some((from, drawn)) = self.find(&now, None) else {
            let kept = machine.keep_picture(&picture_name(check))?;
            let instead = self.instead(&now);
            self.failed_and_stopped(
                check,
                format!(
                    "alo's pointer, the {}, is nowhere on the screen to be moved; {instead}; \
                     kept as {kept}",
                    self.pointer.look().said(),
                ),
            );
            return Ok(());
        };
        let references = references(drawn.look, drawn.scale, &EVERY_SHAPE);
        let steered = steering::steer(machine, from, to, |frame, near| {
            first_found(frame, &references, Some(near)).map(|(_, place)| place)
        })?;
        match steered {
            Steered::There { at, rounds } => {
                self.pointer.at = Some(at);
                self.checked(
                    check.to_owned(),
                    Verdict::Passed,
                    format!(
                        "from {from:?} to {at:?}, aiming at {to:?} on {}, in {rounds} rounds",
                        target.said()
                    ),
                );
            }
            Steered::Lost { last, rounds } => {
                let kept = machine.keep_picture(&picture_name(check))?;
                self.failed_and_stopped(
                    check,
                    format!(
                        "the pointer was lost in round {rounds}, last seen at {last:?}, aiming at \
                         {to:?} on {}; kept as {kept}",
                        target.said()
                    ),
                );
            }
            Steered::NotThere { at } => {
                let kept = machine.keep_picture(&picture_name(check))?;
                self.failed_and_stopped(
                    check,
                    format!(
                        "after {} rounds the pointer is at {at:?}, not at {to:?} on {}; kept as \
                         {kept}",
                        steering::ROUNDS,
                        target.said()
                    ),
                );
            }
        }
        Ok(())
    }

    /// Is the pointer drawn as `icon` — or as any of alo's shapes — in the
    /// person's look, and does it move with the hand?
    fn is_drawn_as(
        &mut self,
        machine: &mut impl Machine,
        check: &str,
        icon: Option<CursorIcon>,
    ) -> Result<(), MachineError> {
        machine.wait(BEFORE_A_LOOK);
        let now = machine.frame()?;
        let look = self.pointer.look();
        let icons: Vec<CursorIcon> = icon.map_or_else(|| EVERY_SHAPE.to_vec(), |it| vec![it]);
        let expected = match icon {
            Some(icon) => Drawn {
                icon,
                look,
                scale: self.pointer.scale.unwrap_or(100),
            }
            .said(),
            None => format!("any of alo's shapes in the {}", look.said()),
        };
        let mut seen = None;
        for scale in self.pointer.scales() {
            let these = references(look, scale, &icons);
            if let Some((reference, place)) = first_found(&now, &these, self.pointer.at) {
                seen = Some((reference.clone(), place));
                break;
            }
        }
        let kept = machine.keep_picture(&picture_name(check))?;
        let Some((reference, place)) = seen else {
            let instead = self.instead(&now);
            self.checked(
                check.to_owned(),
                Verdict::Failed,
                format!("{expected} is not on the screen; {instead}; kept as {kept}"),
            );
            return Ok(());
        };
        self.pointer.scale = Some(reference.drawn.scale);
        steering::put(machine, (place.0 + A_NUDGE.0, place.1 + A_NUDGE.1))?;
        let after = machine.frame()?;
        let around = Around {
            x: place.0,
            y: place.1,
            reach: NUDGED_REACH,
        };
        let went_to = found_in(&after, &reference, Some(around))
            .into_iter()
            .find(|it| *it != place);
        match went_to {
            Some(to) => {
                self.pointer.at = Some(to);
                self.checked(
                    check.to_owned(),
                    Verdict::Passed,
                    format!(
                        "{} at {place:?}, and at {to:?} once the pointer was moved; kept as {kept}",
                        reference.drawn.said()
                    ),
                );
            }
            None => self.checked(
                check.to_owned(),
                Verdict::Failed,
                format!(
                    "{} at {place:?}, but it was not found moved when the pointer was: a \
                     picture of a pointer rather than the pointer, or one that changed shape \
                     as it moved; kept as {kept}",
                    reference.drawn.said()
                ),
            ),
        }
        Ok(())
    }

    /// alo's pointer in the person's look, in any shape, and where: near where
    /// it was last, else anywhere — finding the display's scale the first time.
    fn find(
        &mut self,
        frame: &Frame,
        near: Option<(usize, usize)>,
    ) -> Option<((usize, usize), Drawn)> {
        let near = near.or(self.pointer.at);
        for scale in self.pointer.scales() {
            let these = references(self.pointer.look(), scale, &EVERY_SHAPE);
            if let Some((reference, place)) = first_found(frame, &these, near) {
                self.pointer.scale = Some(scale);
                return Some((place, reference.drawn));
            }
        }
        None
    }

    /// What is drawn where the expected pointer is not: the arrow, the text
    /// cursor or the hand in any look, near where the pointer was last or
    /// anywhere, said with where — or that there is none.
    fn instead(&self, frame: &Frame) -> String {
        let icons = &THE_THREE[..];
        let around = self.pointer.at;
        let scale = self.pointer.scale.unwrap_or(100);
        let mut seen = Vec::new();
        for look in Look::every() {
            for reference in references(look, scale, icons) {
                let reach = around.map(|(x, y)| Around { x, y, reach: NEAR });
                for place in found_in(frame, &reference, reach) {
                    seen.push(format!("{} at {place:?}", reference.drawn.said()));
                }
            }
        }
        seen.dedup();
        if seen.is_empty() {
            "no pointer alo draws, in any shape, colour or size, is there".to_owned()
        } else {
            format!("what is drawn instead: {}", seen.join("; "))
        }
    }

    /// Write down a failure, and stop.
    fn failed_and_stopped(&mut self, check: &str, evidence: String) {
        self.checked(check.to_owned(), Verdict::Failed, evidence);
        self.stop_after_failure();
    }
}

/// Every cursor in `icons` that can be looked for, in `look` at `scale`.
fn references(look: Look, scale: u16, icons: &[CursorIcon]) -> Vec<Reference> {
    icons
        .iter()
        .filter_map(|&icon| Reference::of(Drawn { icon, look, scale }).ok())
        .collect()
}

/// The first of `references` found on the screen, and where: every one near a
/// place first, then every one anywhere.
fn first_found<'a>(
    frame: &Frame,
    references: &'a [Reference],
    near: Option<(usize, usize)>,
) -> Option<(&'a Reference, (usize, usize))> {
    let found_around = |around: Option<Around>| {
        references.iter().find_map(|reference| {
            found_in(frame, reference, around)
                .first()
                .map(|place| (reference, *place))
        })
    };
    near.and_then(|(x, y)| found_around(Some(Around { x, y, reach: NEAR })))
        .or_else(|| found_around(None))
}

/// A check's name as a picture's: lowercase words joined by hyphens.
fn picture_name(check: &str) -> String {
    let words: Vec<String> = check
        .split(|it: char| !it.is_ascii_alphanumeric())
        .filter(|it| !it.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    format!("pointer-{}", words.join("-"))
}

#[cfg(test)]
#[path = "walking_the_pointer_tests.rs"]
mod tests;
