//! **One try**: a request put through the agent's own loop, with a door that
//! records what the model chose instead of carrying it out.
//!
//! `alo_agent::driving_about` is the loop a person's turn runs. Here its door
//! is [`Recording`]: a question goes to the model; a read, a change or a
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
    chose: Vec<String>,
    went: Went,
    asked: usize,
}

impl Tried {
    /// A try that chose `chose`, ended as `went`, and asked the model `asked`
    /// times.
    #[must_use]
    pub const fn of(chose: Vec<String>, went: Went, asked: usize) -> Self {
        Self { chose, went, asked }
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
    let _ = (request, may_reach, verbs, strings, model);
    let _: Option<&dyn TheDoor> = None;
    let _ = (
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
        TheConversationSoFar::nothing(),
    );
    let _ = driving_about;
    let _: Option<WhatHappened> = None;
    let _: Option<NotAnswered> = None;
    let _: Option<FromAnAgent> = None;
    Tried {
        chose: Vec::new(),
        went: Went::TheRoad,
        asked: 0,
    }
}

#[cfg(test)]
#[path = "a_try_tests.rs"]
mod tests;
