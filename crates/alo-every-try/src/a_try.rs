//! **One try**: a request put through the agent's own loop, with a door that
//! records what the model chose instead of carrying it out.
//!
//! `alo_agent::driving_about` is the loop a person's turn runs. Here its door
//! is `Recording`: a question goes to the model; a read, a change or a
//! command is held against the next step of the road. A step taken is
//! answered as the machine would answer it and the loop goes on; the first
//! step not taken ends the try, so a try that has left the road spends no more
//! of the model's calls.

use alo_agent::{HowFar, NotAnswered, TheDoor, WhatHappened, driving_about};
use alo_capability::Verbs;
use alo_instructing::{Instructions, TheConversationSoFar, WhatItMayReach};
use alo_protocol::{FromAnAgent, ToAnAgent};
use alo_strings::Strings;

use crate::a_road::ARequest;
use crate::as_the_machine_names::AsTheMachineNames;
use crate::taking_a_step::taking;
use crate::the_machine_answers::answering;
use crate::went_wrong::WentWrong;

/// What a try is put to: the model, at a provider or in a test.
pub trait TheModel {
    /// Answer the whole text a turn shows, as the daemon would hand the
    /// answer back to the agent.
    ///
    /// # Errors
    /// What went wrong reaching it: a provider that did not answer is not a
    /// model that chose wrongly, and is counted apart.
    fn answering(&mut self, shown: &str, strings: &Strings) -> Result<ToAnAgent, String>;
}

/// How a try ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Went {
    /// It took every step of the road and then answered in words.
    TheRoad,
    /// It left the road.
    WentWrong(WentWrong),
    /// The model could not be asked, so nothing was chosen.
    NotAnswered(String),
}

/// One try: what the model chose, in order, how it ended, and how many times
/// the model was asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tried {
    /// What it chose, step by step.
    chose: Vec<String>,
    /// How it ended.
    went: Went,
    /// How many times the model was asked.
    asked: usize,
    /// The last line the model wrote.
    said_last: Option<String>,
    /// How many reads it took aside of the road.
    looked_aside: usize,
}

/// **The most reads a try may take aside of its road** — a read the machine
/// would answer, which the request declares (`ARequest::or_looking_at`). A
/// try that looks more than this has not taken the road; the summary says how
/// many passes looked at all, so a pass that needed one is seen.
pub const THE_MOST_LOOKS_ASIDE: usize = 2;

impl Tried {
    /// How many reads this try took aside of its road.
    #[must_use]
    pub const fn looked_aside(&self) -> usize {
        self.looked_aside
    }

    /// The same try, having taken `looks` reads aside of its road.
    #[must_use]
    pub const fn looking_aside(mut self, looks: usize) -> Self {
        self.looked_aside = looks;
        self
    }

    /// A try that chose `chose`, ended as `went`, and asked the model `asked`
    /// times.
    #[must_use]
    pub const fn of(chose: Vec<String>, went: Went, asked: usize) -> Self {
        Self {
            chose,
            went,
            asked,
            said_last: None,
            looked_aside: 0,
        }
    }

    /// The last line the model wrote, as it wrote it — for a try that left the
    /// road, the line that left it.
    #[must_use]
    pub fn said_last(&self) -> Option<&str> {
        self.said_last.as_deref()
    }

    /// What it chose, step by step: a verb's name, `run: …` for a command,
    /// and `an answer in words` where it answered the person.
    #[must_use]
    pub fn chose(&self) -> &[String] {
        &self.chose
    }

    /// How it ended.
    #[must_use]
    pub const fn went(&self) -> &Went {
        &self.went
    }

    /// Whether it took the road.
    #[must_use]
    pub const fn took_the_road(&self) -> bool {
        matches!(self.went, Went::TheRoad)
    }

    /// How many times the model was asked in this try.
    #[must_use]
    pub const fn asked(&self) -> usize {
        self.asked
    }
}

/// Put `request` to `model` once, as a turn would on a machine that may reach
/// `may_reach`, and hold what it chose against the request's road.
#[must_use]
pub fn one_try(
    request: &ARequest,
    may_reach: &WhatItMayReach,
    verbs: &Verbs,
    strings: &Strings,
    model: &mut dyn TheModel,
) -> Tried {
    let mut door = Recording {
        request,
        verbs,
        machine: AsTheMachineNames::of(may_reach),
        strings,
        model,
        taken: 0,
        chose: Vec::new(),
        wrong: None,
        unanswered: None,
        asked: 0,
        said_last: None,
        looked_aside: 0,
    };
    let happened = driving_about(
        &TheConversationSoFar::nothing(),
        None,
        request.words(),
        verbs,
        may_reach,
        &mut door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
    );
    let of = request.road().len();
    let done = door.taken;
    let mut chose = door.chose;
    let went = if let Some(why) = door.unanswered {
        Went::NotAnswered(why)
    } else if let Some(wrong) = door.wrong {
        Went::WentWrong(wrong)
    } else {
        match happened {
            WhatHappened::ItWasAnsweredInWords { said, .. } => {
                chose.push(AN_ANSWER.to_owned());
                if done == of {
                    Went::TheRoad
                } else {
                    Went::WentWrong(WentWrong::StoppedShort {
                        done,
                        of,
                        said: the_start_of(&said),
                    })
                }
            }
            WhatHappened::ItRanOutOfRounds { .. } => {
                Went::WentWrong(WentWrong::RanOutOfRounds { done, of })
            }
            WhatHappened::AChangeIsWaiting { sentence, .. } => {
                Went::WentWrong(WentWrong::EndedOtherwise {
                    done,
                    of,
                    why: format!("a change waited for the person: {sentence}"),
                })
            }
            WhatHappened::ItWasRefused { said, .. } => Went::WentWrong(WentWrong::EndedOtherwise {
                done,
                of,
                why: format!("it was refused: {said}"),
            }),
            WhatHappened::ItWasStopped { .. } => Went::WentWrong(WentWrong::EndedOtherwise {
                done,
                of,
                why: "it was stopped".to_owned(),
            }),
            WhatHappened::TheMachineCouldNotBeAsked { why, .. } => {
                Went::WentWrong(WentWrong::EndedOtherwise { done, of, why })
            }
        }
    };
    Tried {
        chose,
        went,
        asked: door.asked,
        said_last: door.said_last,
        looked_aside: door.looked_aside,
    }
}

/// What a try records where the model answered the person.
const AN_ANSWER: &str = "an answer in words";

/// The most of what a model said that a summary shows.
const AS_MUCH_AS_A_SUMMARY_SHOWS: usize = 160;

/// The start of `said`, on one line.
fn the_start_of(said: &str) -> String {
    let one_line = said.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut start: String = one_line.chars().take(AS_MUCH_AS_A_SUMMARY_SHOWS).collect();
    if one_line.chars().count() > AS_MUCH_AS_A_SUMMARY_SHOWS {
        start.push('…');
    }
    start
}

/// The door a try's loop asks: questions go to the model, and everything else
/// is held against the road.
struct Recording<'a> {
    /// The request being tried.
    request: &'a ARequest,
    /// The machine's verbs, which a step is validated against.
    verbs: &'a Verbs,
    /// How the machine names an application a step gives by its name.
    machine: AsTheMachineNames,
    /// The words the answers are said in.
    strings: &'a Strings,
    /// The model a question goes to.
    model: &'a mut dyn TheModel,
    /// How many steps have been taken.
    taken: usize,
    /// What the model chose, in order.
    chose: Vec<String>,
    /// How the try left the road, once it has.
    wrong: Option<WentWrong>,
    /// Why the model could not be asked, if it could not.
    unanswered: Option<String>,
    /// How many times the model was asked.
    asked: usize,
    /// The last line the model wrote.
    said_last: Option<String>,
    /// How many reads the try took aside of the road.
    looked_aside: usize,
}

/// Why the loop is stopped once the road is left: the try is over.
const THE_TRY_IS_OVER: &str = "the try left the road, so nothing more is asked";

impl TheDoor for Recording<'_> {
    fn asking(&mut self, request: &FromAnAgent) -> Result<ToAnAgent, NotAnswered> {
        if let FromAnAgent::Ask { question, .. } = request {
            // **As the daemon shows it**: trimmed, and passed on unwrapped
            // because it already begins with the product's own words
            // (`alo_turn::next_request`'s `Held::shown`).
            self.asked += 1;
            let answered = self.model.answering(question.trim(), self.strings);
            if let Ok(ToAnAgent::Answered { text, .. }) = &answered {
                self.said_last = Some(text.clone());
            }
            return answered.map_err(|why| {
                self.unanswered = Some(why.clone());
                NotAnswered::NotReached { why }
            });
        }
        self.chose.push(match request {
            FromAnAgent::Run { command } => format!("run: {command}"),
            other => other.verb().unwrap_or_default().to_owned(),
        });
        let road = self.request.road();
        let asked = request.verb().unwrap_or_default().trim();
        // **A look taken again** is answered again, and the road stays where
        // it was.
        if let Some(looked) = self
            .taken
            .checked_sub(1)
            .and_then(|last| road.get(last))
            .filter(|last| last.is_a_look() && last.verb() == asked)
        {
            return match taking(
                looked,
                self.taken,
                request,
                self.verbs,
                &self.machine,
                self.strings,
            ) {
                Ok(call) => Ok(answering(looked.answered(), &call, self.strings)),
                Err(wrong) => {
                    self.wrong = Some(wrong);
                    Err(NotAnswered::NotReached {
                        why: THE_TRY_IS_OVER.to_owned(),
                    })
                }
            };
        }
        // **A look skipped**: the model went straight on, so the step after it
        // is the one this is held to.
        if road
            .get(self.taken)
            .is_some_and(|next| next.is_a_look() && next.verb() != asked)
        {
            self.taken += 1;
        }
        // **A read aside of the road**, which the request declares and the
        // machine would answer: answered, and the road stays where it was —
        // at most `THE_MOST_LOOKS_ASIDE` a try.
        if let Some(answer) = self.aside_of_the_road(request, asked) {
            return Ok(answer);
        }
        let Some(step) = road.get(self.taken) else {
            self.wrong = Some(WentWrong::WentOn {
                asked: self.chose.last().cloned().unwrap_or_default(),
            });
            return Err(NotAnswered::NotReached {
                why: THE_TRY_IS_OVER.to_owned(),
            });
        };
        match taking(
            step,
            self.taken + 1,
            request,
            self.verbs,
            &self.machine,
            self.strings,
        ) {
            Ok(call) => {
                self.taken += 1;
                Ok(answering(step.answered(), &call, self.strings))
            }
            Err(wrong) => {
                self.wrong = Some(wrong);
                Err(NotAnswered::NotReached {
                    why: THE_TRY_IS_OVER.to_owned(),
                })
            }
        }
    }
}

impl Recording<'_> {
    /// The machine's answer to `request`, a read of `asked` the request
    /// declares aside of its road, while this try has looks left; [`None`]
    /// otherwise, and the road decides as it always did.
    fn aside_of_the_road(&mut self, request: &FromAnAgent, asked: &str) -> Option<ToAnAgent> {
        if !matches!(request, FromAnAgent::Read { .. }) || self.looked_aside >= THE_MOST_LOOKS_ASIDE
        {
            return None;
        }
        let (aside, call) = self.request.aside().iter().find_map(|aside| {
            (aside.verb() == asked)
                .then(|| {
                    taking(
                        aside,
                        self.taken + 1,
                        request,
                        self.verbs,
                        &self.machine,
                        self.strings,
                    )
                    .ok()
                })
                .flatten()
                .map(|call| (aside, call))
        })?;
        self.looked_aside += 1;
        Some(answering(aside.answered(), &call, self.strings))
    }
}

#[cfg(test)]
#[path = "a_try_tests.rs"]
mod tests;
