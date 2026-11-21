//! **A window's band found where the shell says it is drawn, and its controls
//! pressed where they are drawn** (`alo_acting::said_of_a_band`).
//!
//! The investigation of 2026-11-21 found the walk never pointed at or clicked
//! a window control — it closed everything with Alt+F4, *8 key presses and 0
//! button presses* — so a band that was not on the screen at all, as Text
//! Editor's was, was never seen. The one band the shell draws every window's
//! band through now says where each band is and where its controls are, so
//! this points at a band's middle, reads its controls, and presses each in the
//! middle of where it is drawn.
//!
//! **A line is not trusted alone.** A band said to be drawn off the screen
//! fails, as does one said and then said to be no longer drawn; and the walk
//! looks after pointing, so a band that is said and not seen changes nothing
//! on the screen and fails that look.

use std::time::Duration;

use alo_acting::said_of_a_band::{self as said, OnTheScreen, TheKind};

use super::{READ_THE_LOG_EVERY, Walking};
use crate::machine::{Machine, MachineError};
use crate::report::Verdict;
use crate::the_log;

/// How long the button is held, as a person clicks — the pointer walk's own.
const A_CLICK_HELD: Duration = Duration::from_millis(80);

/// The band the walk last pointed at, by the id its line said, and its
/// controls as the last controls line read said them.
#[derive(Debug, Default)]
pub(super) struct TheBand {
    /// The band pointed at, by its kind and the id the line said.
    pointed: Option<(TheKind, String)>,
    /// Its controls, by name, and where each is drawn.
    controls: Vec<(String, OnTheScreen)>,
}

/// The check that a band is on the screen.
pub(super) fn the_band_is_on_the_screen(kind: TheKind, id: &str) -> String {
    format!("the band of {} {id:?} is on the screen", kind.word())
}

/// The check that a band's controls are these.
pub(super) fn the_controls_are(kind: TheKind, id: &str, names: &[String]) -> String {
    format!(
        "the controls of {} {id:?} are {}",
        kind.word(),
        names.join(", ")
    )
}

/// The check that a control is pressed where it is drawn.
pub(super) fn it_is_pressed(name: &str) -> String {
    format!("{name:?} is pressed where it is drawn")
}

/// Whether a line's window, `said_kind` and `said_id`, is the one wanted:
/// the same, or — for a task window, whose number is a place in a list that
/// moves — any task window where `"task"` is wanted.
fn answers(kind: TheKind, wanted: &str, said_kind: TheKind, said_id: &str) -> bool {
    kind == said_kind
        && (said_id == wanted
            || (kind == TheKind::Task && wanted == "task" && said_id.starts_with("task ")))
}

impl Walking {
    /// Point at the middle of `kind`'s band, `id`, once the shell says where
    /// it is drawn and it is on the screen.
    pub(super) fn point_at_the_band(
        &mut self,
        machine: &mut impl Machine,
        kind: TheKind,
        id: &str,
        within: Duration,
    ) -> Result<(), MachineError> {
        let check = the_band_is_on_the_screen(kind, id);
        let phrases = [
            said::A_BAND_IS_DRAWN.to_owned(),
            said::A_BAND_IS_NO_LONGER_DRAWN.to_owned(),
        ];
        let start = machine.elapsed();
        loop {
            // **The latest line about this band decides**: drawn where, or no
            // longer drawn — read from the whole journal, not the last mark,
            // because the shell says a band once per change and a band that
            // has not moved says nothing new (run 44915943010).
            let mut latest: Option<Result<(String, OnTheScreen, String), String>> = None;
            for line in the_log::all_with(&machine.log()?, 0, &phrases) {
                if let Some((said_kind, said_id, at)) = said::read_a_band_is_drawn(&line) {
                    if answers(kind, id, said_kind, &said_id) {
                        latest = Some(Ok((said_id, at, line)));
                    }
                } else if let Some((said_kind, said_id)) =
                    said::read_a_band_is_no_longer_drawn(&line)
                    && answers(kind, id, said_kind, &said_id)
                {
                    latest = Some(Err(line));
                }
            }
            match latest {
                Some(Ok((said_id, at, line))) => {
                    let (wide, tall) = machine.frame()?.size();
                    let inside = |from: i32, across: i32, room: usize| {
                        across > 0
                            && from >= 0
                            && usize::try_from(from + across).is_ok_and(|to| to <= room)
                    };
                    if !(inside(at.x, at.w, wide) && inside(at.y, at.h, tall)) {
                        self.checked(
                            check,
                            Verdict::Failed,
                            format!("said to be drawn off the {wide}x{tall} screen: {line}"),
                        );
                        self.stop_after_failure();
                        return Ok(());
                    }
                    let (x, y) = at.middle();
                    let middle = (
                        usize::try_from(x).unwrap_or(0),
                        usize::try_from(y).unwrap_or(0),
                    );
                    machine.point_at(middle)?;
                    self.band = TheBand {
                        pointed: Some((kind, said_id)),
                        controls: Vec::new(),
                    };
                    self.checked(
                        check,
                        Verdict::Passed,
                        format!("pointed at its middle, {x},{y}: {line}"),
                    );
                    return Ok(());
                }
                Some(Err(line)) => {
                    self.checked(check, Verdict::Failed, format!("no longer drawn: {line}"));
                    self.stop_after_failure();
                    return Ok(());
                }
                None => {}
            }
            if machine.elapsed().saturating_sub(start) >= within {
                self.checked(
                    check,
                    Verdict::Failed,
                    format!(
                        "nothing said the band of {} {id:?} is drawn within {}s",
                        kind.word(),
                        within.as_secs()
                    ),
                );
                self.stop_after_failure();
                return Ok(());
            }
            machine.wait(READ_THE_LOG_EVERY);
        }
    }

    /// Read the band's controls, once the pointer is in it, and check they
    /// are `names`, every one and no other.
    pub(super) fn the_controls_are(
        &mut self,
        machine: &mut impl Machine,
        kind: TheKind,
        id: &str,
        names: &[String],
        within: Duration,
    ) -> Result<(), MachineError> {
        let check = the_controls_are(kind, id, names);
        // The band pointed at answers by the id its own line said.
        let pointed = self
            .band
            .pointed
            .as_ref()
            .filter(|(pointed_kind, _)| *pointed_kind == kind)
            .map(|(_, said_id)| said_id.clone());
        let wanted = pointed.as_deref().unwrap_or(id);
        let phrases = [said::A_BANDS_CONTROLS_ARE_DRAWN.to_owned()];
        let start = machine.elapsed();
        loop {
            // From the whole journal too: the latest controls line is where
            // they are, however long ago it was said.
            let latest = the_log::all_with(&machine.log()?, 0, &phrases)
                .into_iter()
                .filter_map(|line| {
                    let (said_kind, said_id, controls) =
                        said::read_a_bands_controls_are_drawn(&line)?;
                    answers(kind, wanted, said_kind, &said_id).then_some((controls, line))
                })
                .next_back();
            if let Some((controls, line)) = latest {
                let mut drawn: Vec<&str> = controls.iter().map(|(name, _)| name.as_str()).collect();
                let mut expected: Vec<&str> = names.iter().map(String::as_str).collect();
                drawn.sort_unstable();
                expected.sort_unstable();
                self.band.controls = controls.clone();
                if drawn == expected {
                    self.checked(check, Verdict::Passed, line);
                } else {
                    self.checked(
                        check,
                        Verdict::Failed,
                        format!("drawn are {drawn:?}, not {expected:?}: {line}"),
                    );
                    self.stop_after_failure();
                }
                return Ok(());
            }
            if machine.elapsed().saturating_sub(start) >= within {
                self.checked(
                    check,
                    Verdict::Failed,
                    format!(
                        "nothing said where the controls of {} {id:?} are drawn within {}s",
                        kind.word(),
                        within.as_secs()
                    ),
                );
                self.stop_after_failure();
                return Ok(());
            }
            machine.wait(READ_THE_LOG_EVERY);
        }
    }

    /// Press the control called `name` in the middle of where the last
    /// controls line said it is drawn.
    pub(super) fn click_the_control(
        &mut self,
        machine: &mut impl Machine,
        name: &str,
    ) -> Result<(), MachineError> {
        let check = it_is_pressed(name);
        let Some((_, at)) = self
            .band
            .controls
            .iter()
            .find(|(drawn, _)| drawn == name)
            .cloned()
        else {
            let drawn: Vec<&str> = self
                .band
                .controls
                .iter()
                .map(|(drawn, _)| drawn.as_str())
                .collect();
            self.checked(
                check,
                Verdict::Failed,
                format!("not among the controls drawn: {drawn:?}"),
            );
            self.stop_after_failure();
            return Ok(());
        };
        let (x, y) = at.middle();
        machine.point_at((
            usize::try_from(x).unwrap_or(0),
            usize::try_from(y).unwrap_or(0),
        ))?;
        machine.button(true)?;
        machine.wait(A_CLICK_HELD);
        machine.button(false)?;
        self.checked(check, Verdict::Passed, format!("pressed at {x},{y}"));
        Ok(())
    }
}
