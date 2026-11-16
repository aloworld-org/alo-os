//! The road from a change the daemon is holding to the person who must answer
//! it.
//!
//! ADR 0001 §5: *any change to the machine is proposed with a sentence
//! describing it and waits for one approval. What a person approves is that
//! sentence.* Everything that sentence needs existed before this file and none
//! of it was joined: `alo_approving` held the rule, `alo_tasks::ThePermissions`
//! held the question, `alo_protocol` carried it both ways, the daemon honoured
//! it, and `alo_changing::Answering` asks. **Nothing put the question in front
//! of a person.**
//!
//! Walking a booted machine on 2026-10-25 it said *a change is waiting for you
//! to answer, as number 0* and there was nothing on the machine to answer with
//! — `docs/walks/the-first-walk.md`.
//!
//! # What it holds, and why it is one and not a list
//!
//! **One change in front of one person at a time.**
//! `alo_tasks::ThePermissions` says why in its own words: *a list of pending
//! approvals is a list a person clears by clicking, and clicking through is
//! how somebody approves the fourth one without reading it.* So this keeps the
//! first change the daemon is holding and leaves the rest where they are. They
//! are not lost — the daemon still has them, and the next ask brings the next
//! one.
//!
//! # The number is here and the sentence is in the task
//!
//! A person approves the **sentence**, so the sentence lives on the task and
//! is drawn in their conversation. The **number** is how the daemon is told,
//! and it is nobody's but this file's: a task that carried it would be a model
//! holding a protocol handle, and a surface that read it off the screen would
//! be answering something other than what it showed.
//!
//! # What it does not do
//!
//! **It does not keep *one approval is one execution*.** That is
//! `alo_capability::Approved::redeem`'s, and a second answer to the same
//! number is refused there. This can send a second answer and will be told no,
//! which is correct: the daemon is the one that counts, and a guard here would
//! be a second place to get it wrong.
//!
//! **It words nothing.** The sentence arrives already worded by whoever
//! proposed the change, and a refusal arrives already worded by the daemon.
//! Both are passed through untouched.

use alo_changing::Answering;
use alo_tasks::{ATask, Turn};

use crate::approval_keys::ApprovalKey;

/// A page of the text: the twelve lines the card shows.
#[expect(
    clippy::cast_possible_wrap,
    reason = "twelve fits in any signed number"
)]
const A_PAGE: isize = crate::the_approval_card::THE_MOST_LINES_SHOWN as isize;

/// The change a person is being asked about, the number to answer it by, and
/// which answer their keyboard is on.
///
/// **Not `Clone` and not compared whole**, because it holds the socket keeping
/// the agent's turn open, and there is one of those per question.
#[derive(Debug, Default)]
pub struct AChangeWaiting {
    /// The number the daemon answers by, while one is in front of the person.
    number: Option<u64>,
    /// Which answer is selected, and [`None`] until they move to one.
    ///
    /// **Nothing is preselected.** A question that opens with an answer under
    /// the finger is a question answered by somebody pressing Enter to make a
    /// dialogue go away, which is not an approval of anything.
    selected: Option<WhichAnswer>,
    /// The socket the request was asked on, which keeps the agent's turn — and
    /// the change in it — alive while the person reads.
    ///
    /// **Let go only after the daemon has carried the answer out**, never
    /// before: the change is carried out inside the turn, so closing this
    /// first would end the turn under the very answer it was waiting for.
    the_turn: Option<std::os::unix::net::UnixStream>,
    /// Whether the agent holding that turn goes on after the person's answer,
    /// and so is to be told what it did (`docs/contracts/asking-socket.md`).
    goes_on: bool,
    /// What the change in front of the person touches, as the daemon said —
    /// the approval card's rows naming each file (frame `549:135`). Empty
    /// when nothing is asked, and for a change that touches no file.
    touches: Vec<alo_changing::Touched>,
    /// What the change does and what it leaves alone, as its verb words them
    /// (`549:209`), already in the person's language. Empty when nothing is
    /// asked, or for a verb that says nothing.
    does: (Option<String>, Option<String>),
    /// **The text the change would type, whole** — each text it carries, as
    /// the card shows it (`alo_capability::TheWholeText`): every line, how
    /// long it is, every hidden character marked. Empty when nothing is asked
    /// and for a change that carries no text. ADR 0001 as amended 2026-11-13:
    /// no length limit, nothing cut, never run.
    texts: Vec<alo_capability::TheWholeText>,
    /// The first line of each text the card shows, from nought — the card
    /// shows twelve and then scrolls (the owner's decision of 2026-11-15).
    first_line: usize,
}

/// The two answers, in reading order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichAnswer {
    /// Allow this one action. **Not a grant, and it widens nothing.**
    AllowOnce,
    /// Say no. Nothing happens and nothing is recorded about why.
    Deny,
}

/// What a press on a waiting question did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatThePressDid {
    /// The selection moved, so the screen is drawn again.
    ItMoved,
    /// An answer was given, and this is the one answer to send the daemon.
    ///
    /// **Not sent here**: the daemon may carry the change out by asking this
    /// shell's own door, so the answer is carried off the frame loop
    /// (`crate::an_answer_goes_off_the_frame`).
    ItWasAnswered(TheAnswerGiven),
    /// Nothing happened, which includes Enter with nothing selected.
    Nothing,
}

/// The one answer a press gave: which change, and whether it was allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheAnswerGiven {
    /// The number the daemon answers by.
    pub number: u64,
    /// Allow once, or Deny.
    pub allowed: bool,
}

/// What putting a question in front of a person did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatArrived {
    /// A change is now in front of them, drawn in their conversation.
    AQuestion,
    /// The daemon is holding nothing, or said nothing at all. A surface does
    /// the same for both: there is no question to draw.
    Nothing,
    /// A question was already in front of them and stays there.
    ///
    /// **Not replaced.** A question that changed under somebody about to
    /// answer it is how a person approves one thing believing it was another.
    TheOneAlreadyThere,
}

impl AChangeWaiting {
    /// Nothing is being asked.
    #[must_use]
    pub const fn nothing_yet() -> Self {
        Self {
            number: None,
            // **Nothing preselected**, here as well as after every answer.
            selected: None,
            the_turn: None,
            goes_on: false,
            touches: Vec::new(),
            does: (None, None),
            texts: Vec::new(),
            first_line: 0,
        }
    }

    /// The question now in front of the person is held open by this socket.
    ///
    /// Kept until they answer — see the field — so the agent that proposed it
    /// keeps its turn, and with it the change, for as long as it is on screen.
    pub(crate) fn and_its_turn_is_held_by(&mut self, the_turn: std::os::unix::net::UnixStream) {
        self.the_turn = Some(the_turn);
        self.goes_on = false;
    }

    /// The agent holding the turn paused its reply for the answer, and goes on
    /// after it. Nothing when no turn is held.
    pub(crate) fn and_its_turn_goes_on_after_the_answer(&mut self) {
        self.goes_on = self.the_turn.is_some();
    }

    /// Whether the agent is to be told what the answer did, asked once with
    /// [`Self::the_turn_it_held`].
    pub(crate) fn the_turn_goes_on(&mut self) -> bool {
        std::mem::take(&mut self.goes_on)
    }

    /// Whether a change is in front of the person now.
    #[must_use]
    pub const fn is_asking(&self) -> bool {
        self.number.is_some()
    }

    /// Ask the daemon what it is holding, and put the first one in front of
    /// this person.
    ///
    /// The sentence goes into the conversation as
    /// [`alo_tasks::Turn::alo_proposing`] **and** onto the task's permissions,
    /// which are the two halves of one thing: what they read, and what they
    /// answer.
    pub fn arrived(&mut self, task: &mut ATask, daemon: &dyn Answering) -> WhatArrived {
        self.arrived_after(None, task, daemon)
    }

    /// What the agent said, and then whatever the daemon is holding.
    ///
    /// **The agent's words go into the conversation before the card, unless
    /// all they do is announce the card.** `alo-agent` reports a change it left
    /// waiting as *a change is waiting for you to answer, as number 0:* and then
    /// the sentence — the daemon's own number, which means nothing to a person,
    /// above the sentence the card is about to show in full. The design's
    /// approval card *names the action*, so the conversation says it once, on
    /// the card. Words whose last line is anything else are alo's and are kept.
    pub fn arrived_after(
        &mut self,
        said: Option<String>,
        task: &mut ATask,
        daemon: &dyn Answering,
    ) -> WhatArrived {
        let say = |task: &mut ATask, said: Option<String>| {
            if let Some(said) = said {
                task.alo_says(Turn::from_alo(said));
            }
        };
        if self.number.is_some() {
            say(task, said);
            return WhatArrived::TheOneAlreadyThere;
        }
        let changes = daemon.waiting().unwrap_or_default();
        let Some(first) = changes.first() else {
            say(task, said);
            return WhatArrived::Nothing;
        };
        let sentence = first.sentence().text().to_owned();
        say(
            task,
            said.filter(|said| !it_only_announces(said, &sentence)),
        );
        self.number = Some(first.number());
        self.touches = first.touches().to_vec();
        self.texts = first.texts().iter().map(|text| text.shown()).collect();
        self.first_line = 0;
        self.does = (
            first.changes().map(|it| it.text().to_owned()),
            first.leaves().map(|it| it.text().to_owned()),
        );
        task.is_asking(sentence.clone());
        task.alo_says(Turn::alo_proposing(sentence));
        WhatArrived::AQuestion
    }

    /// They allowed the one action being asked about: the answer to send, and
    /// [`None`] when nothing was being asked.
    ///
    /// **The agent's turn is kept** — `Self::the_turn_it_held` hands it to
    /// whatever carries the answer, which lets it go only after the daemon has
    /// carried the change out inside it.
    pub fn allow_once(&mut self, task: &mut ATask) -> Option<TheAnswerGiven> {
        let number = self.number.take()?;
        self.selected = None;
        self.touches.clear();
        self.does = (None, None);
        self.texts.clear();
        self.first_line = 0;
        task.allow_once();
        Some(TheAnswerGiven {
            number,
            allowed: true,
        })
    }

    /// They said no. Nothing is carried about why, because nothing was asked.
    pub fn deny(&mut self, task: &mut ATask) -> Option<TheAnswerGiven> {
        let number = self.number.take()?;
        self.selected = None;
        self.touches.clear();
        self.does = (None, None);
        self.texts.clear();
        self.first_line = 0;
        task.deny();
        Some(TheAnswerGiven {
            number,
            allowed: false,
        })
    }

    /// The socket holding the agent's turn, handed to whatever carries the
    /// answer, which drops it once the daemon has answered.
    pub(crate) fn the_turn_it_held(&mut self) -> Option<std::os::unix::net::UnixStream> {
        self.the_turn.take()
    }

    /// **Let the change go unanswered**, because the person pressed Stop
    /// (ADR 0139): it is neither approved nor declined, the task's question
    /// lapses, and the connection holding the agent's turn is handed back so
    /// the Stop can travel on it. Nothing is sent to the daemon from here.
    pub(crate) fn let_go_unanswered(
        &mut self,
        task: &mut ATask,
    ) -> Option<std::os::unix::net::UnixStream> {
        self.number.take()?;
        self.selected = None;
        self.touches.clear();
        self.does = (None, None);
        self.texts.clear();
        self.first_line = 0;
        task.the_question_lapsed();
        self.the_turn.take()
    }

    /// What the change in front of the person does, and what it leaves alone,
    /// as its verb words them — nothing for either when no change is asked or
    /// its verb says nothing.
    #[must_use]
    pub fn does(&self) -> (Option<&str>, Option<&str>) {
        (self.does.0.as_deref(), self.does.1.as_deref())
    }

    /// What the change in front of the person touches, and nothing when no
    /// change is.
    #[must_use]
    pub fn touches(&self) -> &[alo_changing::Touched] {
        &self.touches
    }

    /// The text the change in front of the person would type, whole, and
    /// nothing when no change is or it carries none.
    #[must_use]
    pub fn texts(&self) -> &[alo_capability::TheWholeText] {
        &self.texts
    }

    /// How many characters of text the card shows, all of them, across every
    /// text the change carries — nought when it carries none.
    #[must_use]
    pub fn characters_of_text(&self) -> usize {
        self.texts
            .iter()
            .map(|text| text.as_given().chars().count())
            .sum()
    }

    /// The first line the card shows of each text, from nought.
    #[must_use]
    pub const fn first_line_shown(&self) -> usize {
        self.first_line
    }

    /// Scroll the text by `lines`, back when negative, and stop at either
    /// end: the last line it can start at leaves the last twelve showing.
    fn scrolled(&mut self, lines: isize) -> WhatThePressDid {
        let furthest = self
            .texts
            .iter()
            .map(|text| {
                text.how_many_lines()
                    .saturating_sub(crate::the_approval_card::THE_MOST_LINES_SHOWN)
            })
            .max()
            .unwrap_or(0);
        let now = self.first_line.saturating_add_signed(lines).min(furthest);
        if now == self.first_line {
            return WhatThePressDid::Nothing;
        }
        self.first_line = now;
        WhatThePressDid::ItMoved
    }

    /// Which answer their keyboard is on, and [`None`] until they move to one.
    #[must_use]
    pub const fn selected(&self) -> Option<WhichAnswer> {
        self.selected
    }

    /// One key press on the question in front of them.
    ///
    /// **Enter with nothing selected does nothing**, which is *nothing
    /// proceeds on silence* in the one place it can be got wrong: a person who
    /// presses Enter to dismiss something has not answered it.
    ///
    /// **Escape says no** whatever is selected and whether or not anything is,
    /// because leaving a question unanswered is a refusal rather than a maybe.
    pub fn pressed(&mut self, key: ApprovalKey, task: &mut ATask) -> WhatThePressDid {
        if !self.is_asking() {
            return WhatThePressDid::Nothing;
        }
        match key {
            // **Two answers, so next and previous are the same move.** Written
            // as one arm rather than two identical ones: a third answer would
            // make them differ, and then they would be written differently.
            ApprovalKey::NextAnswer | ApprovalKey::PreviousAnswer => {
                self.selected = Some(match self.selected {
                    None | Some(WhichAnswer::Deny) => WhichAnswer::AllowOnce,
                    Some(WhichAnswer::AllowOnce) => WhichAnswer::Deny,
                });
                WhatThePressDid::ItMoved
            }
            ApprovalKey::Choose => match self.selected {
                Some(WhichAnswer::AllowOnce) => self
                    .allow_once(task)
                    .map_or(WhatThePressDid::Nothing, WhatThePressDid::ItWasAnswered),
                Some(WhichAnswer::Deny) => self
                    .deny(task)
                    .map_or(WhatThePressDid::Nothing, WhatThePressDid::ItWasAnswered),
                None => WhatThePressDid::Nothing,
            },
            ApprovalKey::Decline => self
                .deny(task)
                .map_or(WhatThePressDid::Nothing, WhatThePressDid::ItWasAnswered),
            // **Scrolling the text answers nothing** and moves no answer: it
            // is reading, and a person reads before they answer.
            ApprovalKey::EarlierLines => self.scrolled(-1),
            ApprovalKey::LaterLines => self.scrolled(1),
            ApprovalKey::AnEarlierPage => self.scrolled(-A_PAGE),
            ApprovalKey::ALaterPage => self.scrolled(A_PAGE),
            ApprovalKey::Nothing => WhatThePressDid::Nothing,
        }
    }
}

impl crate::Server {
    /// What the agent said came back to a task window: put it in the
    /// conversation, and put in front of the person whatever the daemon is
    /// now holding for them.
    ///
    /// **Here and not in the frame loop**, so that a test can prove what it
    /// never does: **it never touches the keyboard.** A question arriving is
    /// not a reason to move it — see `crate::the_tasks_seat`.
    ///
    /// Asked when an answer arrives rather than once a frame: a turn ends by
    /// answering or by leaving a change waiting, so this is the moment one can
    /// exist. A machine never told where its daemon is asks nothing.
    pub(crate) fn an_answer_came_back(
        &mut self,
        which: crate::the_tasks_a_person_opened::WhichTask,
        said: String,
        the_machines_fault: bool,
        the_turn: Option<std::os::unix::net::UnixStream>,
        daemon: Option<&dyn Answering>,
        strings: &alo_strings::Strings,
    ) {
        // **The fields, not the methods**, so the compiler can see the question
        // and the task are different parts of the server.
        let waiting = &mut self.the_change_waiting;
        let Some(task) = self.the_tasks.the_mut(which) else {
            return;
        };
        // The machine failing and the agent answering are told apart, because a
        // person reads them differently: an answer belongs in the conversation
        // and *this machine could not be asked* belongs beside it.
        let words = if the_machines_fault {
            task.and_then(alo_tasks::Act::done(said));
            // **And the turn ends in the conversation too.** Until 2026-10-27
            // the reason went beside the conversation and nothing went in it, so
            // the person's request stayed the last thing said and the window
            // read *alo is working* for as long as it was open.
            task.alo_says(Turn::from_alo(
                strings
                    .say(
                        &crate::the_cards_words::IT_STOPPED_WITHOUT_FINISHING.key(),
                        &alo_strings::Filling::nothing(),
                    )
                    .text()
                    .to_owned(),
            ));
            None
        } else {
            Some(said)
        };
        let arrived = match daemon {
            Some(daemon) => waiting.arrived_after(words, task, daemon),
            None => {
                if let Some(words) = words {
                    task.alo_says(Turn::from_alo(words));
                }
                WhatArrived::Nothing
            }
        };
        // **A question that arrived keeps the agent's turn open** until the
        // person answers it. Anything else — an answer in words, a refusal, a
        // question already standing, no daemon — lets the agent go now, by
        // dropping the socket here.
        if arrived == WhatArrived::AQuestion
            && let Some(the_turn) = the_turn
        {
            waiting.and_its_turn_is_held_by(the_turn);
        }
        // **Said on the journal when a card goes up**, with how much text it
        // shows and never the text, so a walk can tell the card was put in
        // front of the person and carried the whole of what alo would type.
        if arrived == WhatArrived::AQuestion {
            eprintln!(
                "{}",
                alo_acting::said_on_the_log::a_card_is_up(waiting.characters_of_text())
            );
        }
        // **And a reply the person sent while alo was answering goes now**,
        // with the conversation as it stands — unless what just arrived is a
        // change waiting for them, which keeps it held (ADR 0138).
        self.send_what_was_held(which, strings);
    }
}

/// Whether the agent's words only announce the change about to be shown:
/// their last line is that change's own sentence.
fn it_only_announces(said: &str, sentence: &str) -> bool {
    said.lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .is_some_and(|last| last == sentence.trim())
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on an answer that will not read is the failure being reported"
)]
#[path = "a_change_waiting_reaches_the_person_tests.rs"]
mod tests;
