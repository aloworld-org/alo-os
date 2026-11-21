//! **One try**: a request put through the agent's own loop, with a door that
//! records what the model chose instead of carrying it out.
//!
//! `alo_agent::driving_about` is the loop a person's turn runs. Here its door
//! is `Recording`: a question goes to the model; a read, a change or a
//! command is held against the next step of the road. A step taken is
//! answered as the machine would answer it and the loop goes on; the first
//! step not taken ends the try, so a try that has left the road spends no more
//! of the model's calls.

use alo_agent::{HowFar, NotAnswered, TheDoor, WhatHappened, driving_on};
use alo_capability::Verbs;
use alo_instructing::{Instructions, TheConversationSoFar, WhatItMayReach};
use alo_protocol::{FromAnAgent, ToAPerson, ToAnAgent};
use alo_strings::Strings;

use crate::a_road::{ARequest, Step};
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
    ///
    /// `offered` is what the turn offers at that moment, which its answer is
    /// held to (`alo_instructing::the_verbs_offered`).
    fn answering(
        &mut self,
        shown: &str,
        strings: &Strings,
        offered: &Verbs,
    ) -> Result<ToAnAgent, String>;
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
    /// How many times the machine refused a step before asking the person,
    /// and the try went on from the refusal.
    refused_before_asking: usize,
    /// How many looks of the road it skipped, going straight to the act.
    skipped_looks: usize,
    /// Whether it took the request's other road (`ARequest::or_by_another_road`).
    by_the_other_road: bool,
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

    /// How many looks of its road this try skipped — a window it went
    /// straight to acting in, without reading first.
    #[must_use]
    pub const fn skipped_looks(&self) -> usize {
        self.skipped_looks
    }

    /// The same try, having skipped `looks` looks of its road.
    #[must_use]
    pub const fn skipping_looks(mut self, looks: usize) -> Self {
        self.skipped_looks = looks;
        self
    }

    /// How many times the machine refused a step of this try before asking
    /// anybody, and the model went on from what the refusal said.
    #[must_use]
    pub const fn refused_before_asking(&self) -> usize {
        self.refused_before_asking
    }

    /// The same try, having been refused `times` before anybody was asked.
    #[must_use]
    pub const fn refused_first(mut self, times: usize) -> Self {
        self.refused_before_asking = times;
        self
    }

    /// The same try, having taken `looks` reads aside of its road.
    #[must_use]
    pub const fn looking_aside(mut self, looks: usize) -> Self {
        self.looked_aside = looks;
        self
    }

    /// Whether this try took its request's other road.
    #[must_use]
    pub const fn by_the_other_road(&self) -> bool {
        self.by_the_other_road
    }

    /// The same try, having taken its request's other road.
    #[must_use]
    pub const fn the_other_way(mut self) -> Self {
        self.by_the_other_road = true;
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
            refused_before_asking: 0,
            skipped_looks: 0,
            by_the_other_road: false,
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
    one_try_at(
        TheLevel::FullAccess,
        request,
        may_reach,
        verbs,
        strings,
        model,
    )
}

/// How the scripted machine answers a change on the road.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheLevel {
    /// Carried out at once and answered as done without asking (ADR 0149).
    FullAccess,
    /// Proposed, approved once by the person, and the turn goes on from what
    /// the daemon tells them it did (`docs/contracts/asking-socket.md`, *The
    /// turn goes on after an answer*) — the step a model is shown at the
    /// default level.
    AskEachTime,
}

/// [`one_try`] at a level: at [`TheLevel::AskEachTime`] every change on the
/// road waits, is approved once, and the agent's turn goes on through
/// `alo_agent::WhatHappened::going_on_after`, as `alo-agent`'s own loop does.
#[must_use]
pub fn one_try_at(
    level: TheLevel,
    request: &ARequest,
    may_reach: &WhatItMayReach,
    verbs: &Verbs,
    strings: &Strings,
    model: &mut dyn TheModel,
) -> Tried {
    let mut door = Recording {
        level,
        approved: None,
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
        refused_before_asking: 0,
        skipped_looks: 0,
        on_the_other: None,
        the_screen_road_open: false,
    };
    let mut after = Vec::new();
    let happened = loop {
        let happened = driving_on(
            &TheConversationSoFar::nothing(),
            request.window(),
            request.words(),
            verbs,
            may_reach,
            &mut door,
            HowFar::THE_ORDINARY,
            Instructions::SHOWN_TO_A_TURN,
            &|| false,
            std::mem::take(&mut after),
        );
        // **The person approves the change, once**, and the turn goes on
        // from what the daemon told them it did.
        match door
            .approved
            .take()
            .and_then(|did| happened.going_on_after(&did))
        {
            Some(steps) => after = steps,
            None => break happened,
        }
    };
    let of = door.the_road().len();
    let done = door.taken;
    let by_the_other_road = door.on_the_other.is_some();
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
            // **Never here**: a try drives with `HowFar::THE_ORDINARY`, which
            // does not pause for an application. Said, rather than assumed away.
            WhatHappened::AnApplicationWasRefused { refusal, .. } => {
                Went::WentWrong(WentWrong::EndedOtherwise {
                    done,
                    of,
                    why: format!("it paused for an application: {refusal}"),
                })
            }
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
        refused_before_asking: door.refused_before_asking,
        skipped_looks: door.skipped_looks,
        by_the_other_road,
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
    /// How a change on the road is answered.
    level: TheLevel,
    /// What the daemon would tell the person the change waiting did, once
    /// they approve it, at [`TheLevel::AskEachTime`].
    approved: Option<ToAPerson>,
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
    /// How many steps the machine refused before asking anybody.
    refused_before_asking: usize,
    /// How many looks of the road the try skipped.
    skipped_looks: usize,
    /// Which of the request's other roads the try is held to, from where it
    /// parted from the road; [`None`] on the request's own road.
    on_the_other: Option<usize>,
    /// Whether a read in this try was answered with windows that describe
    /// nothing, which opens the screen road for the rest of it.
    the_screen_road_open: bool,
}

/// Why the loop is stopped once the road is left: the try is over.
const THE_TRY_IS_OVER: &str = "the try left the road, so nothing more is asked";

impl TheDoor for Recording<'_> {
    fn asking(&mut self, request: &FromAnAgent) -> Result<ToAnAgent, NotAnswered> {
        let answer = self.answering_one(request);
        // **A read of windows that describe nothing opens the screen road**,
        // as the agent reads it off the same answer.
        if let Ok(ToAnAgent::Did(done)) = &answer
            && done.a_window_that_describes_nothing()
        {
            self.the_screen_road_open = true;
        }
        answer
    }
}

impl Recording<'_> {
    /// One request answered, before what it opens is noted.
    fn answering_one(&mut self, request: &FromAnAgent) -> Result<ToAnAgent, NotAnswered> {
        if let FromAnAgent::Ask { question, .. } = request {
            // **As the daemon shows it**: trimmed, and passed on unwrapped
            // because it already begins with the product's own words
            // (`alo_turn::next_request`'s `Held::shown`).
            self.asked += 1;
            // **Held to what the turn offers now**, by the agent's own reading
            // of the steps so far (`alo_agent::the_screen_road_is_open`).
            let offered = alo_instructing::the_verbs_offered(self.verbs, self.the_screen_road_open);
            let answered = self
                .model
                .answering(question.trim(), self.strings, &offered);
            if let Ok(ToAnAgent::Answered { text, .. }) = &answered {
                self.said_last = Some(text.clone());
            }
            return answered.map_err(|why| {
                self.unanswered = Some(why.clone());
                NotAnswered::NotReached { why }
            });
        }
        // **At full access, a path through `/home` is where its link leads**,
        // as the machine resolves it (`crate::through_the_link`).
        let resolved;
        let request = if self.level == TheLevel::FullAccess {
            resolved = crate::through_the_link::through_the_link(request, self.verbs);
            &resolved
        } else {
            request
        };
        self.chose.push(match request {
            FromAnAgent::Run { command } => format!("run: {command}"),
            other => other.verb().unwrap_or_default().to_owned(),
        });
        let road = self.the_road();
        let asked = request.verb().unwrap_or_default().trim();
        // **A look taken again**, or a step the road lets a try take as often
        // as it asks, is answered again — a change as the level answers one —
        // and the road stays where it was.
        if let Some(looked) = self
            .taken
            .checked_sub(1)
            .and_then(|last| road.get(last))
            .filter(|last| {
                (last.is_a_look() || last.is_taken_as_often_as_asked()) && last.verb() == asked
            })
        {
            return match taking(
                looked,
                self.taken,
                request,
                self.verbs,
                &self.machine,
                self.strings,
            ) {
                Ok(call) => {
                    let answered = answering(looked.answered(), &call, self.strings);
                    Ok(self.as_the_level_answers(answered, &call))
                }
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
            self.skipped_looks += 1;
        }
        // **The other road, from where the two part** (`ARequest::or_by_another_road`):
        // a try that asks for the other road's next step where this road's
        // next step is something else is held to the other road from here.
        // **Undone where the step that parted is refused before anybody is
        // asked**: a refused call changed nothing, so the try is still where
        // it was, on the road it was on.
        let was_on = self.on_the_other;
        self.parting_for(asked);
        let road = self.the_road();
        // **A read aside of the road**, which the request declares and the
        // machine would answer: answered, and the road stays where it was —
        // at most `THE_MOST_LOOKS_ASIDE` a try. On the other road, never
        // instead of its own next step when that is what was asked: its last
        // read is what shows the person's thing reached.
        let the_next_step_is_this = self.on_the_other.is_some()
            && road
                .get(self.taken)
                .is_some_and(|next| next.verb() == asked);
        if let Some(answer) = (!the_next_step_is_this)
            .then(|| self.aside_of_the_road(request, asked))
            .flatten()
        {
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
                let answered = answering(step.answered(), &call, self.strings);
                Ok(self.as_the_level_answers(answered, &call))
            }
            Err(wrong) => {
                // **As the machine answers a step it refuses before asking
                // anybody**: with the refusal, which the model reads and may go
                // on from — counted, so a pass that needed one is seen.
                if let WentWrong::RefusedBeforeAsking { said, .. } = &wrong {
                    self.refused_before_asking += 1;
                    self.on_the_other = was_on;
                    return Ok(ToAnAgent::refused(said));
                }
                // **As the machine answers a call that does not form**: turned
                // away before anything else, at both levels, in its own words
                // — whatever verb it named (`crate::turned_away`).
                if let Some(refused) = crate::turned_away::turned_away(
                    request,
                    self.verbs,
                    &self.machine,
                    self.strings,
                ) {
                    self.refused_before_asking += 1;
                    self.on_the_other = was_on;
                    return Ok(refused);
                }
                // **As the machine answers a file named where there is none**:
                // refused, with where one of that name is in the home itself
                // (`alo_files::Touching::of_looking_in`), as often as it is
                // named wrongly — the machine sets no limit, and the turn's own
                // bound on rounds is what ends a model that never listens.
                if let Some(refused) = self.where_it_is(&wrong) {
                    self.on_the_other = was_on;
                    return Ok(refused);
                }
                // **As the machine answers a slider the window does not show**:
                // refused before the card, at both levels, and counted as such
                // (`crate::no_such_control`).
                if let Some(refused) = crate::no_such_control::no_such_control(
                    step,
                    &wrong,
                    request,
                    self.verbs,
                    &self.machine,
                    self.strings,
                ) {
                    self.refused_before_asking += 1;
                    return Ok(refused);
                }
                // **As the machine answers a path where nothing is**, where the
                // request says what is there: refused before anybody is asked,
                // and counted as such (`crate::nothing_is_there`).
                if let Some(refused) = crate::nothing_is_there::nothing_is_at(
                    self.request.there(),
                    request,
                    self.verbs,
                    &self.machine,
                    self.strings,
                ) {
                    self.refused_before_asking += 1;
                    self.on_the_other = was_on;
                    return Ok(refused);
                }
                self.wrong = Some(wrong);
                Err(NotAnswered::NotReached {
                    why: THE_TRY_IS_OVER.to_owned(),
                })
            }
        }
    }
}

impl<'a> Recording<'a> {
    /// The road the try is held to now: the request's, or the other road it
    /// took.
    fn the_road(&self) -> &'a [Step] {
        let request: &'a ARequest = self.request;
        self.on_the_other
            .and_then(|at| request.other_roads().get(at))
            .map_or_else(|| request.road(), Vec::as_slice)
    }

    /// **Take another road here**, where the road the try is on does not go on
    /// with `asked` and another road does — one whose steps up to here are,
    /// verb for verb, the ones this road took. The first declared is taken
    /// where two would do. The request's own road is never gone back to.
    fn parting_for(&mut self, asked: &str) {
        let road = self.the_road();
        let goes_on = |road: &[Step]| {
            road.get(self.taken)
                .is_some_and(|next| next.verb() == asked)
        };
        if goes_on(road) {
            return;
        }
        let shares = |other: &[Step]| {
            road.get(..self.taken).is_some_and(|taken| {
                other.get(..self.taken).is_some_and(|theirs| {
                    taken
                        .iter()
                        .zip(theirs)
                        .all(|(one, two)| one.verb() == two.verb())
                })
            })
        };
        if let Some(at) = self
            .request
            .other_roads()
            .iter()
            .position(|other| shares(other) && goes_on(other))
        {
            self.on_the_other = Some(at);
        }
    }
}

impl Recording<'_> {
    /// At [`TheLevel::AskEachTime`], a change done without asking is instead
    /// left waiting under the road's step number, on its own sentence, and what
    /// it did is kept for the approval. Everything else is answered as it was.
    fn as_the_level_answers(
        &mut self,
        answered: ToAnAgent,
        call: &alo_capability::Call,
    ) -> ToAnAgent {
        let (TheLevel::AskEachTime, ToAnAgent::DoneWithoutAsking { did, .. }) =
            (self.level, &answered)
        else {
            return answered;
        };
        let waiting = serde_json::json!({
            "format": 1,
            "tells": {"proposed": {
                "number": self.taken,
                "sentence": {"text": call.sentence(self.strings).into_text(), "came_from": "the-source"},
            }},
        });
        match ToAnAgent::read(&waiting.to_string()) {
            Ok(proposed) => {
                self.approved = Some(ToAPerson::Did(did.clone()));
                proposed
            }
            Err(_) => answered,
        }
    }

    /// The machine's refusal for a path named where there is nothing, saying
    /// where one of that name is — where the road expected a file of the same
    /// name directly in the person's home. The set's machine is at full
    /// access, so every path is one the grants permit, as on that machine.
    /// [`None`] for any other way of leaving the road.
    fn where_it_is(&self, wrong: &WentWrong) -> Option<ToAnAgent> {
        let WentWrong::AnotherValue {
            expected, given, ..
        } = wrong
        else {
            return None;
        };
        let (expected_at, given_at) = (std::path::Path::new(expected), std::path::Path::new(given));
        let same_name =
            expected_at.file_name().is_some() && expected_at.file_name() == given_at.file_name();
        let in_the_home = expected_at.parent() == Some(std::path::Path::new(crate::THE_HOME));
        if !same_name || !in_the_home {
            return None;
        }
        let said = self.strings.say(
            &alo_files::words::NOTHING_THERE_BUT_ONE_IS.key(),
            &alo_strings::Filling::of("path", given.clone()).and("found", expected.clone()),
        );
        Some(ToAnAgent::refused(&said))
    }

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
                    // **A read aside is not a step of the road**, and a refusal
                    // of it is not reported, so it carries no step's number.
                    taking(aside, 0, request, self.verbs, &self.machine, self.strings).ok()
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
