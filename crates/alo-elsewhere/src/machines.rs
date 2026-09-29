//! The machines a person has added, each with whether it answered.
//!
//! # The one design decision in this file
//!
//! **There is no way to ask this type for its machines without also being told
//! how each one is reaching.** [`TheMachines::each`] yields pairs, and nothing
//! yields an `AMachine` alone.
//!
//! The design note this comes from says a machine that cannot be reached is a
//! state and is shown, never an omission — and the way to make that true is not
//! a rule somebody remembers at the drawing end. A list of machines and a
//! separate *and here is which ones are down* is a shape where the second half
//! can be forgotten, filtered, or arrive empty because a lookup failed, and the
//! result reads as *all your machines are fine*.
//!
//! A dashboard that is only truthful when every machine is reachable is the
//! failure named in the design note. This type cannot produce one.

use alo_nearby::MachineId;

use crate::machine::AMachine;
use crate::reaching::Reaching;
use crate::refusing::NotElsewhere;

/// The person's own machines, in the order they added them.
///
/// Added order, not answering order: a list that reorders itself when a machine
/// goes quiet moves the thing a person is reaching for at the moment they are
/// reaching for it. The dock learnt the same lesson — using a window may not
/// reorder the icons.
#[derive(Debug, Default, Clone)]
pub struct TheMachines {
    /// Each machine beside how it is reaching, in the order they were added.
    ///
    /// **One list, not two.** A `Vec<AMachine>` with a separate map of states
    /// is the shape where the states can be missing for a machine, and a
    /// machine with no state reads as a machine with nothing wrong.
    added: Vec<(AMachine, Reaching)>,
}

impl TheMachines {
    /// No machines yet.
    #[must_use]
    pub const fn none_yet() -> Self {
        Self { added: Vec::new() }
    }

    /// Add a machine.
    ///
    /// It starts at [`Reaching::NotAskedYet`], which is the truth: adding a
    /// machine is not asking it anything.
    ///
    /// # Errors
    ///
    /// [`NotElsewhere::AlreadyAdded`] if that identity is already on the list.
    /// Adding it twice would give the person two rows for one machine, which
    /// they would then have to keep in step by hand.
    pub fn add(&mut self, machine: AMachine) -> Result<(), NotElsewhere> {
        if self.holds(machine.which()) {
            return Err(NotElsewhere::AlreadyAdded);
        }
        self.added.push((machine, Reaching::NotAskedYet));
        Ok(())
    }

    /// Forget a machine, and say whether there was one to forget.
    pub fn forget(&mut self, which: &MachineId) -> bool {
        let was = self.added.len();
        self.added.retain(|(machine, _)| machine.which() != which);
        self.added.len() != was
    }

    /// Whether this identity is already on the list.
    #[must_use]
    pub fn holds(&self, which: &MachineId) -> bool {
        self.added
            .iter()
            .any(|(machine, _)| machine.which() == which)
    }

    /// How many machines the person has added.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.added.len()
    }

    /// Every machine, each with how it is reaching.
    ///
    /// **The only way to read this list.** See the note at the top of this file
    /// for why there is no method giving the machines alone.
    pub fn each(&self) -> impl Iterator<Item = (&AMachine, Reaching)> + '_ {
        self.added
            .iter()
            .map(|(machine, reaching)| (machine, *reaching))
    }

    /// Write down what happened when this machine last asked one of them.
    ///
    /// Answers whether that machine is on the list; an answer about a machine
    /// the person has forgotten is dropped rather than re-adding it.
    pub fn now_reaching(&mut self, which: &MachineId, reaching: Reaching) -> bool {
        for (machine, held) in &mut self.added {
            if machine.which() == which {
                *held = reaching;
                return true;
            }
        }
        false
    }

    /// How a named machine is reaching, if the person has added it.
    #[must_use]
    pub fn reaching(&self, which: &MachineId) -> Option<Reaching> {
        self.each()
            .find(|(machine, _)| machine.which() == which)
            .map(|(_, reaching)| reaching)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::machine::TheName;

    fn identity(last: char) -> MachineId {
        let mut said = "0123456789abcdef0123456789abcde".to_owned();
        said.push(last);
        MachineId::read(&said).expect("thirty-two hexadecimal characters")
    }

    fn machine(last: char, called: &str) -> AMachine {
        AMachine::added(identity(last), TheName::given(called).expect("a name"))
    }

    fn two() -> TheMachines {
        let mut machines = TheMachines::none_yet();
        machines.add(machine('a', "in the studio")).expect("added");
        machines.add(machine('b', "at home")).expect("added");
        machines
    }

    /// **The whole point of the crate.** A machine that did not answer is still
    /// in the list, still named, and carrying the state that says so.
    #[test]
    fn a_machine_that_did_not_answer_is_still_on_the_list() {
        let mut machines = two();
        machines.now_reaching(&identity('a'), Reaching::Answered);
        machines.now_reaching(&identity('b'), Reaching::DidNotAnswer);

        let listed: Vec<_> = machines.each().collect();
        assert_eq!(listed.len(), 2, "a machine was left out for not answering");

        let quiet = listed
            .iter()
            .find(|(machine, _)| machine.called().as_str() == "at home")
            .expect("the quiet machine is still listed");
        assert_eq!(quiet.1, Reaching::DidNotAnswer);
    }

    /// **Adding a machine is not asking it anything**, so it starts at
    /// *not asked yet* rather than at either verdict.
    #[test]
    fn a_machine_starts_unasked_rather_than_failing() {
        let machines = two();
        for (_, reaching) in machines.each() {
            assert_eq!(reaching, Reaching::NotAskedYet);
        }
    }

    /// Two rows for one machine would have to be kept in step by hand.
    #[test]
    fn the_same_machine_cannot_be_added_twice() {
        let mut machines = two();
        assert_eq!(
            machines.add(machine('a', "a second name for it")),
            Err(NotElsewhere::AlreadyAdded)
        );
        assert_eq!(machines.how_many(), 2);
    }

    /// The order is the order they were added, and a machine going quiet does
    /// not move it under the person's hand.
    #[test]
    fn going_quiet_does_not_reorder_the_list() {
        let mut machines = two();
        let before: Vec<String> = machines
            .each()
            .map(|(machine, _)| machine.called().as_str().to_owned())
            .collect();

        machines.now_reaching(&identity('a'), Reaching::DidNotAnswer);
        machines.now_reaching(&identity('b'), Reaching::Answered);

        let after: Vec<String> = machines
            .each()
            .map(|(machine, _)| machine.called().as_str().to_owned())
            .collect();
        assert_eq!(before, after, "the list reordered itself");
    }

    /// An answer about a machine the person has forgotten is dropped, not
    /// treated as a reason to put it back.
    #[test]
    fn an_answer_about_a_forgotten_machine_does_not_re_add_it() {
        let mut machines = two();
        assert!(machines.forget(&identity('b')));
        assert!(!machines.now_reaching(&identity('b'), Reaching::Answered));
        assert_eq!(machines.how_many(), 1);
        assert!(!machines.holds(&identity('b')));
    }

    /// Forgetting says whether there was anything to forget, so a caller can
    /// tell *done* from *there was nothing there*.
    #[test]
    fn forgetting_says_whether_there_was_one() {
        let mut machines = two();
        assert!(machines.forget(&identity('a')));
        assert!(!machines.forget(&identity('a')));
    }

    /// Asking about a machine that was never added answers nothing rather than
    /// guessing at a state.
    #[test]
    fn a_machine_nobody_added_has_no_state() {
        let machines = two();
        assert_eq!(machines.reaching(&identity('c')), None);
        assert_eq!(
            machines.reaching(&identity('a')),
            Some(Reaching::NotAskedYet)
        );
    }
}
