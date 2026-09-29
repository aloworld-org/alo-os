//! Work sent to another of the person's machines.
//!
//! # A goal crosses, not a command
//!
//! `docs/design/a-persons-other-machines.md`: *the asking machine says what it
//! wants done; the working machine decides how, under its own rules, its own
//! capability model and its own person's grants. A command would make the second
//! machine an extension of the first, and then the first machine's compromise is
//! every machine's compromise.*
//!
//! So [`AGoal`] is words and nothing else. **There is no program name here, no
//! arguments, no path, and no verb** — not as a private field, not as a
//! constructor a caller could reach. That absence is the design; a struct with
//! somewhere to put an argument list is a command protocol whatever its
//! documentation says.
//!
//! It also keeps [ADR 0009](../../../docs/decisions/0009-a-good-computer-without-the-agent.md)'s
//! shape: the working machine's manual path and agent path are both still its
//! own.
//!
//! # The result lands where it was asked for
//!
//! [`TheWork`] is the asking machine's own record, so a result comes back here
//! because **there is nowhere else it could be addressed**. Nothing in this file
//! takes a destination. The design note's answer — *where the person asked, not
//! where the work ran* — is kept by there being no way to express the
//! alternative.
//!
//! # Stopping reaches work already running
//!
//! [`TheWork::stop`] stops something that is working, and
//! [`TheWork::stop_everything_on`] stops everything on one machine at once,
//! which is what a revoked grant has to be able to do. ADR 0031 already makes
//! revocation immediate for proofs; work in flight has to honour the same word
//! or *immediately* means two different things in one product.
//!
//! And a result arriving after the person stopped something is **refused**, not
//! kept. Keeping it would undo the stop quietly, which is the one outcome
//! somebody pressing stop has ruled out.

use alo_nearby::MachineId;

use crate::refusing::NotElsewhere;
use crate::words;

/// The longest a goal may be.
///
/// Long enough for a sentence or two saying what is wanted, short enough that
/// what crosses stays something a person could read back. Not a limit on
/// ambition: a goal is what somebody wants done, not the doing of it.
pub const AT_MOST_A_GOAL: usize = 500;

/// What the person wants done, in words.
///
/// **Words, deliberately.** See this file's own note: there is no room here for
/// a program, an argument or a path, because the machine doing the work decides
/// how under its own rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AGoal(String);

impl AGoal {
    /// A goal, as the person or their agent said it.
    ///
    /// # Errors
    ///
    /// [`NotElsewhere::NoGoal`] when nothing is left after trimming, and
    /// [`NotElsewhere::GoalTooLong`] past [`AT_MOST_A_GOAL`].
    pub fn said(words: &str) -> Result<Self, NotElsewhere> {
        let trimmed = words.trim();
        if trimmed.is_empty() {
            return Err(NotElsewhere::NoGoal);
        }
        let how_long = trimmed.chars().count();
        if how_long > AT_MOST_A_GOAL {
            return Err(NotElsewhere::GoalTooLong {
                how_long,
                at_most: AT_MOST_A_GOAL,
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// The goal, as it was said.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Which piece of work, on the machine that asked for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkId(u64);

impl WorkId {
    /// The number this reads as, for a caller that has to write it down.
    #[must_use]
    pub const fn as_number(self) -> u64 {
        self.0
    }
}

/// How a piece of work is going.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HowItIsGoing {
    /// Sent, and the other machine has not said it has started.
    Sent,
    /// The other machine is working on it.
    Working,
    /// It finished and the result is here.
    CameBack,
    /// The person stopped it.
    Stopped,
    /// It never reached the other machine.
    ///
    /// **A state, not a silence.** Work that could not be sent stays in the
    /// list saying so, for the reason the rest of this crate keeps: a list that
    /// drops what went wrong is a list that reads as *everything is fine*.
    CouldNotBeSent,
}

impl HowItIsGoing {
    /// Every state, so a caller drawing them cannot quietly handle four of five.
    pub const ALL: &'static [Self] = &[
        Self::Sent,
        Self::Working,
        Self::CameBack,
        Self::Stopped,
        Self::CouldNotBeSent,
    ];

    /// Whether this is still going, and therefore still stoppable.
    #[must_use]
    pub const fn is_still_going(self) -> bool {
        matches!(self, Self::Sent | Self::Working)
    }

    /// The key of the sentence a person reads for this state.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Sent => words::WORK_SENT,
            Self::Working => words::WORK_WORKING,
            Self::CameBack => words::WORK_CAME_BACK,
            Self::Stopped => words::WORK_STOPPED,
            Self::CouldNotBeSent => words::WORK_COULD_NOT_BE_SENT,
        }
    }
}

/// One piece of work, on the machine that asked for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct APieceOfWork {
    /// Which piece.
    id: WorkId,
    /// What was wanted.
    goal: AGoal,
    /// Which machine was asked. One machine — work is not broadcast.
    to: MachineId,
    /// How it is going.
    how: HowItIsGoing,
}

impl APieceOfWork {
    /// Which piece.
    #[must_use]
    pub const fn id(&self) -> WorkId {
        self.id
    }

    /// What was wanted.
    #[must_use]
    pub const fn goal(&self) -> &AGoal {
        &self.goal
    }

    /// Which machine was asked.
    #[must_use]
    pub const fn to(&self) -> &MachineId {
        &self.to
    }

    /// How it is going.
    #[must_use]
    pub const fn how(&self) -> HowItIsGoing {
        self.how
    }
}

/// Everything this machine has asked another of the person's machines to do.
///
/// Held here, on the machine that asked, which is what makes *the result lands
/// where it was asked for* a fact about the type rather than a rule somebody
/// follows.
#[derive(Debug, Default, Clone)]
pub struct TheWork {
    /// Every piece, in the order it was sent.
    sent: Vec<APieceOfWork>,
    /// The number the next piece gets.
    next: u64,
}

impl TheWork {
    /// Nothing sent yet.
    #[must_use]
    pub const fn none_sent() -> Self {
        Self {
            sent: Vec::new(),
            next: 1,
        }
    }

    /// Ask a machine for something, and say which piece of work that is.
    pub fn send(&mut self, goal: AGoal, to: MachineId) -> WorkId {
        let id = WorkId(self.next);
        self.next = self.next.saturating_add(1);
        self.sent.push(APieceOfWork {
            id,
            goal,
            to,
            how: HowItIsGoing::Sent,
        });
        id
    }

    /// The other machine has started.
    ///
    /// Answers whether anything changed: starting something already stopped, or
    /// already back, changes nothing and says so.
    pub fn now_working(&mut self, id: WorkId) -> bool {
        self.change(id, HowItIsGoing::Sent, HowItIsGoing::Working)
    }

    /// It never reached the other machine.
    pub fn could_not_be_sent(&mut self, id: WorkId) -> bool {
        self.change(id, HowItIsGoing::Sent, HowItIsGoing::CouldNotBeSent)
    }

    /// The result is here.
    ///
    /// # Errors
    ///
    /// [`NotElsewhere::ItWasStopped`] if the person stopped this piece before
    /// the result arrived. **Keeping it would undo the stop quietly**, which is
    /// the one outcome somebody pressing stop has ruled out.
    pub fn came_back(&mut self, id: WorkId) -> Result<bool, NotElsewhere> {
        let Some(piece) = self.sent.iter_mut().find(|piece| piece.id == id) else {
            return Ok(false);
        };
        if piece.how == HowItIsGoing::Stopped {
            return Err(NotElsewhere::ItWasStopped);
        }
        if !piece.how.is_still_going() {
            return Ok(false);
        }
        piece.how = HowItIsGoing::CameBack;
        Ok(true)
    }

    /// Stop one piece of work, and say whether there was one still going.
    ///
    /// Reaches work that is already running, not only work not yet started.
    pub fn stop(&mut self, id: WorkId) -> bool {
        let Some(piece) = self.sent.iter_mut().find(|piece| piece.id == id) else {
            return false;
        };
        if !piece.how.is_still_going() {
            return false;
        }
        piece.how = HowItIsGoing::Stopped;
        true
    }

    /// Stop everything still going on one machine, and say how many that was.
    ///
    /// **What a revoked grant needs.** ADR 0031 makes revocation immediate;
    /// work in flight has to honour the same word or *immediately* means two
    /// different things in one product.
    pub fn stop_everything_on(&mut self, machine: &MachineId) -> usize {
        let mut stopped: usize = 0;
        for piece in &mut self.sent {
            if &piece.to == machine && piece.how.is_still_going() {
                piece.how = HowItIsGoing::Stopped;
                stopped = stopped.saturating_add(1);
            }
        }
        stopped
    }

    /// How one piece is going, if this machine asked for it.
    #[must_use]
    pub fn how_is(&self, id: WorkId) -> Option<HowItIsGoing> {
        self.sent
            .iter()
            .find(|piece| piece.id == id)
            .map(APieceOfWork::how)
    }

    /// Every piece of work, in the order it was sent.
    ///
    /// Each one carries its own state, so there is no list of work without the
    /// states beside it — the same shape as [`crate::TheMachines::each`], for
    /// the same reason.
    pub fn each(&self) -> impl Iterator<Item = &APieceOfWork> + '_ {
        self.sent.iter()
    }

    /// How many pieces this machine has asked for.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.sent.len()
    }

    /// Move one piece from one state to another, and say whether it moved.
    fn change(&mut self, id: WorkId, from: HowItIsGoing, to: HowItIsGoing) -> bool {
        let Some(piece) = self.sent.iter_mut().find(|piece| piece.id == id) else {
            return false;
        };
        if piece.how != from {
            return false;
        }
        piece.how = to;
        true
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn identity(last: char) -> MachineId {
        let mut said = "0123456789abcdef0123456789abcde".to_owned();
        said.push(last);
        MachineId::read(&said).expect("thirty-two hexadecimal characters")
    }

    fn goal(said: &str) -> AGoal {
        AGoal::said(said).expect("a goal")
    }

    /// **A goal is words, kept as words.** Nothing splits it, parses it or
    /// reads a program out of the front of it — which is the observable half of
    /// *a goal crosses, not a command*.
    #[test]
    fn a_goal_is_kept_as_the_person_said_it() {
        let said = "render tonight's footage and put it on the shared disk";
        assert_eq!(goal(said).as_str(), said);
    }

    /// The spaces either side are a typing accident; an empty goal is not a
    /// goal at all.
    #[test]
    fn an_empty_goal_is_refused() {
        assert_eq!(AGoal::said("   "), Err(NotElsewhere::NoGoal));
        assert_eq!(
            AGoal::said(&"a".repeat(AT_MOST_A_GOAL + 1)),
            Err(NotElsewhere::GoalTooLong {
                how_long: AT_MOST_A_GOAL + 1,
                at_most: AT_MOST_A_GOAL,
            })
        );
        assert!(AGoal::said(&"a".repeat(AT_MOST_A_GOAL)).is_ok());
    }

    /// **Stopping reaches work already running**, not only work not yet
    /// started.
    #[test]
    fn stopping_reaches_work_that_is_already_running() {
        let mut work = TheWork::none_sent();
        let id = work.send(goal("render the footage"), identity('a'));
        assert!(work.now_working(id));
        assert_eq!(work.how_is(id), Some(HowItIsGoing::Working));

        assert!(work.stop(id), "work already running could not be stopped");
        assert_eq!(work.how_is(id), Some(HowItIsGoing::Stopped));
    }

    /// **A result arriving after a stop is refused, not kept.** Keeping it
    /// would undo the stop quietly, which is what pressing stop ruled out.
    #[test]
    fn a_result_that_arrives_after_a_stop_does_not_undo_it() {
        let mut work = TheWork::none_sent();
        let id = work.send(goal("render the footage"), identity('a'));
        work.now_working(id);
        assert!(work.stop(id));

        assert_eq!(work.came_back(id), Err(NotElsewhere::ItWasStopped));
        assert_eq!(
            work.how_is(id),
            Some(HowItIsGoing::Stopped),
            "the stop was undone by a late result"
        );
    }

    /// One action stops everything on a machine, which is what a revoked grant
    /// needs — and it leaves other machines alone.
    #[test]
    fn everything_on_one_machine_stops_at_once() {
        let mut work = TheWork::none_sent();
        let first = work.send(goal("render the footage"), identity('a'));
        let second = work.send(goal("pack the archive"), identity('a'));
        let elsewhere = work.send(goal("check the mail"), identity('b'));
        work.now_working(first);

        assert_eq!(work.stop_everything_on(&identity('a')), 2);
        assert_eq!(work.how_is(first), Some(HowItIsGoing::Stopped));
        assert_eq!(work.how_is(second), Some(HowItIsGoing::Stopped));
        assert_eq!(
            work.how_is(elsewhere),
            Some(HowItIsGoing::Sent),
            "work on another machine was stopped too"
        );
    }

    /// Stopping something that has already come back changes nothing and says
    /// so, rather than reporting a stop that did not happen.
    #[test]
    fn stopping_something_finished_says_there_was_nothing_to_stop() {
        let mut work = TheWork::none_sent();
        let id = work.send(goal("pack the archive"), identity('a'));
        assert!(work.came_back(id).expect("not stopped"));

        assert!(!work.stop(id));
        assert_eq!(work.how_is(id), Some(HowItIsGoing::CameBack));
    }

    /// **Work that never arrived is a state, not a silence.** It stays in the
    /// list saying so.
    #[test]
    fn work_that_could_not_be_sent_stays_in_the_list() {
        let mut work = TheWork::none_sent();
        let id = work.send(goal("render the footage"), identity('a'));
        assert!(work.could_not_be_sent(id));

        assert_eq!(work.how_many(), 1, "it was dropped instead of kept");
        assert_eq!(work.how_is(id), Some(HowItIsGoing::CouldNotBeSent));
        assert!(!work.how_is(id).expect("a state").is_still_going());
    }

    /// Every piece gets its own number, so two goals with the same words are
    /// still two pieces of work.
    #[test]
    fn two_pieces_of_work_are_two_even_with_the_same_words() {
        let mut work = TheWork::none_sent();
        let first = work.send(goal("render the footage"), identity('a'));
        let second = work.send(goal("render the footage"), identity('a'));
        assert_ne!(first, second);
        assert_eq!(work.how_many(), 2);
    }

    /// Asking about work this machine never sent answers nothing rather than
    /// guessing at a state.
    #[test]
    fn work_nobody_sent_has_no_state() {
        let work = TheWork::none_sent();
        assert_eq!(work.how_is(WorkId(7)), None);
    }

    /// Every state has its own sentence, and no two share one.
    #[test]
    fn each_state_says_something_different() {
        let mut seen = Vec::new();
        for state in HowItIsGoing::ALL {
            let word = state.word();
            assert!(!seen.contains(&word), "two states share {word}");
            seen.push(word);
        }
        assert_eq!(seen.len(), 5, "a state was added without a sentence");
    }
}
