//! **One command line that acts**: every call it makes, each counted, each
//! written down (`docs/contracts/terminal-commands.md`, sections 1, 4 and 5).
//!
//! The order is the part only this file can get wrong:
//!
//! 1. **The record is opened before anything is done**, because an act that
//!    cannot be written down does not happen.
//! 2. Each call's values are checked **before** anything is asked of the
//!    application, and a call refused there is written down with its ability
//!    and why, never its values.
//! 3. **One place to return to per command line**, asked of `alo-agentd`
//!    before the first call that changes something, and named in every entry
//!    after it. If none was kept the person is told once and the command runs
//!    anyway — ADR 0045's *a machine too full to snapshot still runs the turn*.
//! 4. Each call is carried out once, with a count of what the application
//!    sends held around it, and written down with that count.
//!
//! A line of a pipe that fails does not stop the others, and the command ends
//! with the worst of what happened.

use alo_record::SentDuring;
use alo_strings::Strings;

use crate::counted::counting;
use crate::declared::{Declared, by_name, carried, the_ability, the_call};
use crate::ended::Ended;
use crate::machine::{APlace, Knowing, Machine};
use crate::on_the_window::on_the_window;
use crate::outcome::Outcome;
use crate::pipe::{every_call, readable, which_is_the_pipe};
use crate::pressing::{pressed, to_press};
use crate::subject::Subject;
use crate::typed::{Doing, OnTheWindow};
use crate::words;

/// Whether what the command gives back is said in sentences, which is when a
/// done act is told on the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Telling {
    /// Plain sentences, on a terminal or under `--plain`.
    Sentences,
    /// One value per line, on a pipe: an act gives back nothing yet, because no
    /// ability declares what it gives.
    Values,
}

/// One call, checked and ready to carry out.
#[derive(Debug, Clone)]
enum Ready {
    /// A press.
    Press(crate::pressing::ToPress),
    /// An act on the window.
    Window(OnTheWindow),
    /// A declared ability, validated.
    Declared(Declared, alo_capability::Call),
}

impl Ready {
    /// Whether this call changes something, and so has a place kept before it.
    fn changes(&self) -> bool {
        match self {
            Self::Press(_) => true,
            Self::Window(_) => false,
            Self::Declared(declared, _) => declared.changes(),
        }
    }

    /// Whether this call is written down: every act is, and a declared read is
    /// not (the owner's answer 5).
    fn written_down(&self) -> bool {
        match self {
            Self::Press(_) | Self::Window(_) => true,
            Self::Declared(declared, _) => declared.changes(),
        }
    }
}

/// A command line that acts on `subject`.
pub struct Acting<'a> {
    /// What it acts on.
    pub subject: &'a Subject,
    /// What it knows.
    pub knowing: Knowing<'a>,
    /// How what it gives back is told.
    pub telling: Telling,
}

impl Acting<'_> {
    /// Carry out every call `doing` makes, and say how the command ends.
    pub fn every_call(&self, doing: &Doing, machine: &mut dyn Machine) -> Ended {
        let strings = self.knowing.strings;
        if let Err(said) = machine.the_record_is_open(strings) {
            machine.warned(said.text());
            return Ended::NotOffered;
        }
        // The values each call is made with, and how each is made ready.
        let (values, named_as): (Vec<String>, Vec<String>) = match doing {
            Doing::Press { kind, control } => (vec![kind.clone(), control.clone()], Vec::new()),
            Doing::Window(_) => (Vec::new(), Vec::new()),
            Doing::Ability { ability, given } => {
                let declared = match self.declared(ability) {
                    Ok(declared) => declared,
                    Err(refused) => return self.refused(doing.named(), &refused, machine),
                };
                match by_name(declared, given, strings) {
                    Ok(named) => named.into_iter().map(|(name, value)| (value, name)).unzip(),
                    Err(refused) => return self.refused(doing.named(), &refused, machine),
                }
            }
            Doing::Controls | Doing::Read => return Ended::Done,
        };
        let pipe = match which_is_the_pipe(values.iter().map(String::as_str)) {
            Ok(pipe) => pipe,
            Err(_) => {
                let said = strings.say(&words::TWO_PIPES.key(), &alo_strings::Filling::nothing());
                let refused = Outcome::not_asked(Ended::NotThisShape, said);
                return self.refused(doing.named(), &refused, machine);
            }
        };
        let calls = every_call(&values, pipe, || machine.next_on_the_pipe());
        let mut ended = Ended::Done;
        let mut place: Option<APlace> = None;
        for values in calls {
            let ready = match self.ready(doing, &values, &named_as, pipe) {
                Ok(ready) => ready,
                Err(refused) => {
                    ended = ended.worst(self.refused(doing.named(), &refused, machine));
                    continue;
                }
            };
            if ready.changes() && place.is_none() {
                let kept = machine.a_place_to_return_to(strings);
                if let APlace::NotKept(said) = &kept {
                    machine.warned(said);
                }
                place = Some(kept);
            }
            let (outcome, sent) = counting(self.subject, strings, machine, |machine| {
                self.carried(&ready, machine)
            });
            let kept_as = match &place {
                Some(APlace::Kept(named)) => Some(named.as_str()),
                Some(APlace::NotKept(_) | APlace::KeptUnnamed) | None => None,
            };
            ended = ended.worst(self.said(doing.named(), &outcome, &ready, sent, kept_as, machine));
        }
        ended
    }

    /// The declared ability, for an application.
    fn declared(&self, ability: &str) -> Result<Declared, Box<Outcome>> {
        let strings = self.knowing.strings;
        match self.subject.application() {
            Some(application) => the_ability(
                self.knowing.adapters,
                application.identifier(),
                ability,
                strings,
            ),
            // `alo` reaches no declared ability, which the subject already refused.
            None => Err(Outcome::refusing(
                Ended::NotOffered,
                strings.say(
                    &words::ALO_TREE_ONLY.key(),
                    &alo_strings::Filling::nothing(),
                ),
            )),
        }
    }

    /// One call's values, checked.
    fn ready(
        &self,
        doing: &Doing,
        values: &[String],
        named_as: &[String],
        pipe: Option<usize>,
    ) -> Result<Ready, Box<Outcome>> {
        let strings = self.knowing.strings;
        // **A line of the pipe that a sentence cannot hold is refused on its
        // line**, whatever it was going to be.
        if let Some(line) = pipe.and_then(|at| values.get(at))
            && !readable(line)
        {
            let said = strings.say(
                &words::UNREADABLE_VALUE.key(),
                &alo_strings::Filling::nothing(),
            );
            return Err(Outcome::refusing(Ended::ValueRefused, said));
        }
        match doing {
            Doing::Press { .. } => {
                let (Some(kind), Some(control)) = (values.first(), values.get(1)) else {
                    return Err(Outcome::refusing(
                        Ended::NotThisShape,
                        strings.say(
                            &words::PRESS_NEEDS_TWO.key(),
                            &alo_strings::Filling::nothing(),
                        ),
                    ));
                };
                to_press(kind, control, strings).map(Ready::Press)
            }
            Doing::Window(act) => Ok(Ready::Window(*act)),
            Doing::Ability { ability, .. } => {
                let declared = self.declared(ability)?;
                let named: Vec<(String, String)> = named_as
                    .iter()
                    .cloned()
                    .zip(values.iter().cloned())
                    .collect();
                let call = the_call(
                    declared,
                    &named,
                    self.knowing.standing_in,
                    self.knowing.verbs,
                    strings,
                )?;
                Ok(Ready::Declared(declared, call))
            }
            Doing::Controls | Doing::Read => Err(Outcome::refusing(
                Ended::NotThisShape,
                strings.say(
                    &words::NOTHING_ASKED.key(),
                    &alo_strings::Filling::nothing(),
                ),
            )),
        }
    }

    /// Carry one ready call out.
    fn carried(&self, ready: &Ready, machine: &mut dyn Machine) -> Outcome {
        let strings = self.knowing.strings;
        match ready {
            Ready::Press(asked) => pressed(asked, self.subject, strings, machine),
            Ready::Window(act) => match self.subject.application() {
                Some(application) => {
                    on_the_window(*act, application, &self.subject.called, strings, machine)
                }
                None => Outcome::not_asked(
                    Ended::NotOffered,
                    strings.say(
                        &words::ALO_TREE_ONLY.key(),
                        &alo_strings::Filling::nothing(),
                    ),
                ),
            },
            Ready::Declared(_, call) => carried(call, self.subject, self.knowing, machine),
        }
    }

    /// Say what came of a call, and write it down.
    fn said(
        &self,
        ability: &str,
        outcome: &Outcome,
        ready: &Ready,
        sent: SentDuring,
        kept_as: Option<&str>,
        machine: &mut dyn Machine,
    ) -> Ended {
        let strings = self.knowing.strings;
        self.spoken(outcome, machine);
        if ready.written_down() {
            let entry = outcome.entry(
                self.subject.identifier(),
                ability,
                sent,
                kept_as,
                machine.now(),
            );
            if let Err(said) = machine.keep(&entry, strings) {
                machine.warned(said.text());
            }
        }
        outcome.ended()
    }

    /// A call refused before it was ready: said, and written down.
    fn refused(&self, ability: &str, refused: &Outcome, machine: &mut dyn Machine) -> Ended {
        self.spoken(refused, machine);
        let entry = refused.entry(
            self.subject.identifier(),
            ability,
            SentDuring::Counted { bytes: 0 },
            None,
            machine.now(),
        );
        if let Err(said) = machine.keep(&entry, self.knowing.strings) {
            machine.warned(said.text());
        }
        refused.ended()
    }

    /// Tell the person what came of it: a done act on the output when the
    /// answer is sentences, and everything else on the error output.
    fn spoken(&self, outcome: &Outcome, machine: &mut dyn Machine) {
        let line = outcome.said().text();
        match (outcome.ended(), self.telling) {
            (Ended::Done, Telling::Sentences) => machine.told(line),
            (Ended::Done, Telling::Values) => {}
            _ => machine.warned(line),
        }
    }
}

/// Write down a refusal of an act that never reached a subject — an
/// application nothing answers to, say — in the words the person was told.
///
/// **Not for `front`.** The record keeps the real identifier and window, never
/// the word `front`, and a `front` that found no window has neither to keep:
/// the person is told, and nothing is written.
pub fn refused_before_a_subject(
    application: &str,
    ability: &str,
    said: &alo_strings::Said,
    strings: &Strings,
    machine: &mut dyn Machine,
) {
    machine.warned(said.text());
    if application != crate::typed::FRONT && machine.the_record_is_open(strings).is_ok() {
        let entry = alo_record::Entry::an_application_was_not_asked(
            application,
            ability,
            said.text(),
            machine.now(),
        );
        if let Err(why) = machine.keep(&entry, strings) {
            machine.warned(why.text());
        }
    }
}
