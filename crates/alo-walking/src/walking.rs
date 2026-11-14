//! **Carrying a walk out** against a machine, and writing down what happened.
//!
//! Three rules, each the cure for something a walk by hand got wrong:
//!
//! - **A picture is taken before the keys**, at the first key after the last
//!   look, so a look compares the screen with the one the keys were pressed
//!   at — not with whatever it was a minute earlier.
//! - **The walk stops at its first failed check**, and every check after it
//!   is written down as *not reached*: a check of a screen nobody knows the
//!   walk is on would be a guess reported as a measurement.
//! - **A secret is never said.** Typing one is reported by its variable's
//!   name and its length; a character it has no key for is reported by
//!   position only.

use std::time::Duration;

use crate::frame::Frame;
use crate::keys::{self, Chord};
use crate::machine::{Machine, MachineError};
use crate::report::{Check, Entry, Report, Verdict};
use crate::settling::{self, Seen, Settling};
use crate::step::{Since, Step, Walk};
use crate::strokes::{self, Pace};
use crate::the_log;

/// How often a log check reads the log again.
const READ_THE_LOG_EVERY: Duration = Duration::from_secs(2);

/// Walk `walk` on `machine`, with `secret` answering an environment
/// variable's value (or `None` where it is not set), at `pace`.
pub fn walk(
    machine: &mut impl Machine,
    walk: &Walk,
    secret: &impl Fn(&str) -> Option<String>,
    pace: Pace,
) -> Report {
    let mut walking = Walking {
        report: Report {
            walk: walk.name().to_owned(),
            entries: Vec::new(),
        },
        before: None,
        mark: 0,
        stopped: None,
    };
    for step in walk.steps() {
        if let Some((verdict, why)) = walking.stopped.clone() {
            if let Some(check) = check_name(step) {
                walking.checked(check, verdict, why);
            }
            continue;
        }
        if let Err(MachineError(why)) = walking.step(machine, step, secret, pace) {
            walking.checked(
                "the machine answers".to_owned(),
                Verdict::Failed,
                format!("{why}, at {}", at(machine.elapsed())),
            );
            walking.stop_after_failure();
        }
    }
    walking.report
}

/// A walk under way.
struct Walking {
    /// The account so far.
    report: Report,
    /// The picture taken before the keys since the last look.
    before: Option<Frame>,
    /// Where the log was marked.
    mark: usize,
    /// Why the walk stopped, and what that makes every later check.
    stopped: Option<(Verdict, String)>,
}

impl Walking {
    /// Carry out one step.
    fn step(
        &mut self,
        machine: &mut impl Machine,
        step: &Step,
        secret: &impl Fn(&str) -> Option<String>,
        pace: Pace,
    ) -> Result<(), MachineError> {
        match step {
            Step::Press(chord) => {
                self.picture_before(machine)?;
                press(machine, chord, pace)?;
                self.did(machine.elapsed(), format!("pressed {}", chord.said()));
            }
            Step::Type(text) => {
                self.picture_before(machine)?;
                match keys::typing(text) {
                    Ok(chords) => {
                        for chord in &chords {
                            press(machine, chord, pace)?;
                        }
                        self.did(machine.elapsed(), format!("typed {text:?}"));
                    }
                    Err(keys::Untypable { at }) => {
                        self.checked(
                            format!("typing {text:?}"),
                            Verdict::Failed,
                            format!("character {at} has no key; nothing was typed"),
                        );
                        self.stop_after_failure();
                    }
                }
            }
            Step::TypeSecret(variable) => self.type_secret(machine, variable, secret, pace)?,
            Step::Look { name, rule } => self.look(machine, name, *rule)?,
            Step::Wait(for_how_long) => {
                machine.wait(*for_how_long);
                self.did(
                    machine.elapsed(),
                    format!("waited {}s", for_how_long.as_secs()),
                );
            }
            Step::Mark => {
                self.mark = machine.log()?.len();
                self.did(
                    machine.elapsed(),
                    format!("marked the log at byte {}", self.mark),
                );
            }
            Step::ExpectLine {
                check,
                phrase,
                and_says,
                since,
                within,
            } => self.expect_line(machine, check, phrase, and_says.as_deref(), *since, *within)?,
            Step::ExpectNoLine {
                check,
                phrases,
                since,
                during,
            } => self.expect_no_line(machine, check, phrases, *since, *during)?,
            Step::OnlyWith(variable) => {
                if secret(*variable).is_none_or(|it| it.is_empty()) {
                    self.stopped = Some((
                        Verdict::Skipped,
                        format!("the rest of the walk needs {variable}, which is not set"),
                    ));
                    self.did(
                        machine.elapsed(),
                        format!("{variable} is not set; the rest is skipped"),
                    );
                }
            }
        }
        Ok(())
    }

    /// Type a secret, saying only its name and length.
    fn type_secret(
        &mut self,
        machine: &mut impl Machine,
        variable: &str,
        secret: &impl Fn(&str) -> Option<String>,
        pace: Pace,
    ) -> Result<(), MachineError> {
        let Some(value) = secret(variable).filter(|it| !it.is_empty()) else {
            self.checked(
                format!("typing {variable}"),
                Verdict::Failed,
                format!("{variable} is not set"),
            );
            self.stop_after_failure();
            return Ok(());
        };
        self.picture_before(machine)?;
        match keys::typing(&value) {
            Ok(chords) => {
                for chord in &chords {
                    press(machine, chord, pace)?;
                }
                self.did(
                    machine.elapsed(),
                    format!("typed {variable}, {} characters", chords.len()),
                );
            }
            Err(keys::Untypable { at }) => {
                self.checked(
                    format!("typing {variable}"),
                    Verdict::Failed,
                    format!("character {at} of {variable} has no key; nothing was typed"),
                );
                self.stop_after_failure();
            }
        }
        Ok(())
    }

    /// Look, keep the picture, and fail if the screen did not change.
    fn look(
        &mut self,
        machine: &mut impl Machine,
        name: &str,
        rule: Settling,
    ) -> Result<(), MachineError> {
        let before = self.before.take();
        let looked = settling::look(machine, before.as_ref(), rule)?;
        let kept = machine.keep_picture(name)?;
        let verdict = match looked.seen {
            Seen::Unchanged => Verdict::Failed,
            _ => Verdict::Passed,
        };
        self.checked(
            format!("the screen {name}"),
            verdict,
            format!(
                "{} after {}s, kept as {kept}",
                looked.seen.said(),
                looked.took.as_secs()
            ),
        );
        if verdict == Verdict::Failed {
            self.stop_after_failure();
        }
        Ok(())
    }

    /// Wait for a phrase in the log.
    fn expect_line(
        &mut self,
        machine: &mut impl Machine,
        check: &str,
        phrase: &str,
        and_says: Option<&str>,
        since: Since,
        within: Duration,
    ) -> Result<(), MachineError> {
        let from = self.from(since);
        let start = machine.elapsed();
        loop {
            if let Some(line) = the_log::first_with(&machine.log()?, from, phrase) {
                let holds = and_says.is_none_or(|also| line.contains(also));
                let verdict = if holds {
                    Verdict::Passed
                } else {
                    Verdict::Failed
                };
                let took = machine.elapsed().saturating_sub(start).as_secs();
                let evidence = match and_says {
                    Some(also) if !holds => {
                        format!("after {took}s the line does not say {also:?}: {line}")
                    }
                    _ => format!("after {took}s: {line}"),
                };
                self.checked(check.to_owned(), verdict, evidence);
                if !holds {
                    self.stop_after_failure();
                }
                return Ok(());
            }
            if machine.elapsed().saturating_sub(start) >= within {
                self.checked(
                    check.to_owned(),
                    Verdict::Failed,
                    format!("{phrase:?} was not said within {}s", within.as_secs()),
                );
                self.stop_after_failure();
                return Ok(());
            }
            machine.wait(READ_THE_LOG_EVERY);
        }
    }

    /// Watch the log for a whole time, and fail on any line with a phrase.
    fn expect_no_line(
        &mut self,
        machine: &mut impl Machine,
        check: &str,
        phrases: &[String],
        since: Since,
        during: Duration,
    ) -> Result<(), MachineError> {
        let from = self.from(since);
        let start = machine.elapsed();
        while machine.elapsed().saturating_sub(start) < during {
            machine.wait(READ_THE_LOG_EVERY);
        }
        let found = the_log::all_with(&machine.log()?, from, phrases);
        if found.is_empty() {
            self.checked(
                check.to_owned(),
                Verdict::Passed,
                format!("none of {phrases:?} in {}s", during.as_secs()),
            );
        } else {
            let shown: Vec<&str> = found.iter().take(5).map(String::as_str).collect();
            self.checked(
                check.to_owned(),
                Verdict::Failed,
                format!(
                    "{} lines in {}s, the first: {}",
                    found.len(),
                    during.as_secs(),
                    shown.join(" / ")
                ),
            );
            self.stop_after_failure();
        }
        Ok(())
    }

    /// The picture before the keys, once per look.
    fn picture_before(&mut self, machine: &mut impl Machine) -> Result<(), MachineError> {
        if self.before.is_none() {
            self.before = Some(machine.frame()?);
        }
        Ok(())
    }

    /// Where a check reads from.
    fn from(&self, since: Since) -> usize {
        match since {
            Since::Boot => 0,
            Since::Mark => self.mark,
        }
    }

    /// Write down something done.
    fn did(&mut self, elapsed: Duration, what: String) {
        self.report
            .entries
            .push(Entry::Did(format!("{} {what}", at(elapsed))));
    }

    /// Write down a check.
    fn checked(&mut self, name: String, verdict: Verdict, evidence: String) {
        self.report.entries.push(Entry::Checked(Check {
            name,
            verdict,
            evidence,
        }));
    }

    /// Stop: every later check is not reached.
    fn stop_after_failure(&mut self) {
        self.stopped = Some((Verdict::NotReached, "an earlier check failed".to_owned()));
    }
}

/// Press one chord.
fn press(machine: &mut impl Machine, chord: &Chord, pace: Pace) -> Result<(), MachineError> {
    for stroke in strokes::of(chord, pace) {
        machine.stroke(&stroke)?;
        machine.wait(stroke.then);
    }
    Ok(())
}

/// The name a step's check would have, for a step that is a check.
fn check_name(step: &Step) -> Option<String> {
    match step {
        Step::Look { name, .. } => Some(format!("the screen {name}")),
        Step::ExpectLine { check, .. } | Step::ExpectNoLine { check, .. } => Some(check.clone()),
        _ => None,
    }
}

/// The machine's time, as a report writes it.
fn at(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs();
    format!("[{:>2}:{:02}]", seconds / 60, seconds % 60)
}

#[cfg(test)]
#[path = "walking_tests.rs"]
mod tests;
