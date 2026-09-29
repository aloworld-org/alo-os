//! What a person can see and change about their own machines, in one place.
//!
//! The design note asks for three things of this surface: **add a machine, see
//! what each machine may do, and revoke in one action.** This is the model
//! behind them — what the rows say and what the changes do — with no drawing and
//! no widget in it.
//!
//! # Every added machine has a row
//!
//! [`TheRows::of`] builds one row per machine the person has added, always, and
//! there is no filter. A settings page that quietly leaves out the machine it
//! could not reach is the dishonest summary [`crate::TheMachines`] was shaped to
//! make impossible, and it would be easy to reintroduce here by listing only the
//! machines a grant mentions.
//!
//! # Three states for the grant, not two
//!
//! [`WhatTheAgentHas`] separates *nothing was ever given* from *what was given
//! has ended*. A person looking at their own settings is entitled to tell those
//! apart: the first is a machine they never opened, the second is one they did
//! and which has since closed on its own, exactly as it was meant to.

use std::time::SystemTime;

use alo_nearby::MachineId;

use crate::driving::{Driving, MayDrive};
use crate::machine::{AMachine, TheName};
use crate::machines::TheMachines;
use crate::reaching::Reaching;
use crate::refusing::NotElsewhere;
use crate::words;

/// What the person's agent has on one machine, at a moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatTheAgentHas {
    /// Nothing was ever given. The ordinary state, and not a fault.
    Nothing,
    /// The whole machine, until a moment.
    ///
    /// Named for what it is. There is no narrower form of this grant, because a
    /// pointer permits everything — ADR 0079, and [`crate::WHAT_IT_CANNOT_ENUMERATE`]
    /// is the sentence saying so where the person gives it.
    TheWholeMachine {
        /// When it stops.
        until: SystemTime,
    },
    /// Something was given and has ended on its own.
    Ended,
}

impl WhatTheAgentHas {
    /// Whether the agent may drive this machine at the moment this was read.
    #[must_use]
    pub const fn stands(self) -> bool {
        matches!(self, Self::TheWholeMachine { .. })
    }

    /// The key of the sentence a person reads for this.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Nothing => words::MAY_NOT_DRIVE,
            Self::TheWholeMachine { .. } => words::MAY_DRIVE,
            Self::Ended => words::DRIVING_ENDED,
        }
    }
}

/// One machine, as the person's settings show it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ARow {
    /// What the person calls it.
    called: TheName,
    /// Which machine.
    which: MachineId,
    /// Whether it answered when this machine last asked.
    reaching: Reaching,
    /// What the agent has on it.
    agent: WhatTheAgentHas,
}

impl ARow {
    /// What the person calls it.
    #[must_use]
    pub const fn called(&self) -> &TheName {
        &self.called
    }

    /// Which machine.
    #[must_use]
    pub const fn which(&self) -> &MachineId {
        &self.which
    }

    /// Whether it answered.
    #[must_use]
    pub const fn reaching(&self) -> Reaching {
        self.reaching
    }

    /// What the agent has on it.
    #[must_use]
    pub const fn agent(&self) -> WhatTheAgentHas {
        self.agent
    }
}

/// Every machine the person has added, as their settings show them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheRows {
    /// One row per added machine, in the order they were added.
    rows: Vec<ARow>,
}

impl TheRows {
    /// Build the rows from what the machine knows, at a moment.
    ///
    /// **One row per added machine and no filter.** Reaching and the grant are
    /// looked up per machine rather than driving the list, so a machine that did
    /// not answer and a machine with no grant are both still listed.
    #[must_use]
    pub fn of(machines: &TheMachines, driving: &Driving, now: SystemTime) -> Self {
        let rows = machines
            .each()
            .map(|(machine, reaching)| ARow {
                called: machine.called().clone(),
                which: machine.which().clone(),
                reaching,
                agent: what_it_has(driving, machine.which(), now),
            })
            .collect();
        Self { rows }
    }

    /// Every row, in the order the machines were added.
    pub fn each(&self) -> impl Iterator<Item = &ARow> + '_ {
        self.rows.iter()
    }

    /// How many machines the person has.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.rows.len()
    }

    /// Whether any machine is one the agent may drive right now.
    ///
    /// What a *stop everything* control is offered against: nothing to stop is
    /// different from stopping nothing.
    #[must_use]
    pub fn any_stands(&self) -> bool {
        self.rows.iter().any(|row| row.agent.stands())
    }
}

/// What the agent has on one machine, at a moment.
fn what_it_has(driving: &Driving, which: &MachineId, now: SystemTime) -> WhatTheAgentHas {
    driving
        .each(now)
        .find(|(grant, _)| grant.which() == which)
        .map_or(WhatTheAgentHas::Nothing, |(grant, stands)| {
            if stands {
                WhatTheAgentHas::TheWholeMachine {
                    until: grant.ends(),
                }
            } else {
                WhatTheAgentHas::Ended
            }
        })
}

/// What a person can change from this surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Add one of their machines, under a name they will recognise.
    Add {
        /// Which machine.
        which: MachineId,
        /// What they call it.
        called: TheName,
    },
    /// Remove a machine from their list.
    ///
    /// **This also takes back anything the agent had on it.** See
    /// [`carry_out`].
    Forget {
        /// Which machine.
        which: MachineId,
    },
    /// Let the agent drive one machine, until a moment.
    LetTheAgentDrive {
        /// Which machine.
        which: MachineId,
        /// When it stops.
        until: SystemTime,
    },
    /// Take back what the agent has on one machine.
    StopTheAgentOn {
        /// Which machine.
        which: MachineId,
    },
    /// Take back everything the agent has, everywhere, in one action.
    StopTheAgentEverywhere,
}

/// Carry out one change, and say what it did.
///
/// # Forgetting a machine takes its grant with it
///
/// **The one decision in this file that is not obvious.** Removing a machine
/// leaves no row, so a grant over it would become a permission the person can
/// neither see nor revoke — and if they added the same machine again it would
/// silently arrive already driveable. The widest grant in this product must
/// never be something a person gets back without giving it.
///
/// # Errors
///
/// Whatever the underlying act refuses: [`NotElsewhere::AlreadyAdded`],
/// [`NotElsewhere::EndsBeforeItBegins`] or [`NotElsewhere::AlreadyDriving`].
pub fn carry_out(
    change: Change,
    machines: &mut TheMachines,
    driving: &mut Driving,
    now: SystemTime,
) -> Result<usize, NotElsewhere> {
    match change {
        Change::Add { which, called } => {
            machines.add(AMachine::added(which, called))?;
            Ok(1)
        }
        Change::Forget { which } => {
            let forgotten = usize::from(machines.forget(&which));
            // Takes the grant with it. See this function's own note.
            driving.revoke(&which);
            Ok(forgotten)
        }
        Change::LetTheAgentDrive { which, until } => {
            driving.give(MayDrive::given_until(which, now, until)?)?;
            Ok(1)
        }
        Change::StopTheAgentOn { which } => Ok(usize::from(driving.revoke(&which))),
        Change::StopTheAgentEverywhere => Ok(driving.revoke_everything()),
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn identity(last: char) -> MachineId {
        let mut said = "0123456789abcdef0123456789abcde".to_owned();
        said.push(last);
        MachineId::read(&said).expect("thirty-two hexadecimal characters")
    }

    fn a_name(called: &str) -> TheName {
        TheName::given(called).expect("a name")
    }

    fn at(seconds: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn two_added() -> (TheMachines, Driving) {
        let mut machines = TheMachines::none_yet();
        machines
            .add(AMachine::added(identity('a'), a_name("in the studio")))
            .expect("added");
        machines
            .add(AMachine::added(identity('b'), a_name("at home")))
            .expect("added");
        (machines, Driving::none_given())
    }

    /// **Every added machine has a row**, including one that did not answer and
    /// one the agent has nothing on. A settings page that left either out would
    /// be the dishonest summary this crate is shaped to prevent.
    #[test]
    fn every_machine_has_a_row_whatever_its_state() {
        let (mut machines, driving) = two_added();
        machines.now_reaching(&identity('b'), Reaching::DidNotAnswer);

        let rows = TheRows::of(&machines, &driving, at(1_000));
        assert_eq!(rows.how_many(), 2, "a machine was left out of the list");

        let quiet = rows
            .each()
            .find(|row| row.called().as_str() == "at home")
            .expect("the quiet machine is listed");
        assert_eq!(quiet.reaching(), Reaching::DidNotAnswer);
        assert_eq!(quiet.agent(), WhatTheAgentHas::Nothing);
    }

    /// **Never given and has ended are different facts**, and a person looking
    /// at their own settings is entitled to tell them apart.
    #[test]
    fn a_grant_that_ended_does_not_read_as_one_never_given() {
        let (machines, mut driving) = two_added();
        carry_out(
            Change::LetTheAgentDrive {
                which: identity('a'),
                until: at(2_000),
            },
            &mut TheMachines::none_yet(),
            &mut driving,
            at(1_000),
        )
        .expect("given");

        let standing = TheRows::of(&machines, &driving, at(1_500));
        let row = standing.each().next().expect("a row");
        assert_eq!(
            row.agent(),
            WhatTheAgentHas::TheWholeMachine { until: at(2_000) }
        );
        assert!(row.agent().stands());

        let after = TheRows::of(&machines, &driving, at(9_999));
        let row = after.each().next().expect("a row");
        assert_eq!(row.agent(), WhatTheAgentHas::Ended);
        assert!(!row.agent().stands());
        assert_ne!(row.agent(), WhatTheAgentHas::Nothing);
        assert_ne!(
            WhatTheAgentHas::Ended.word(),
            WhatTheAgentHas::Nothing.word(),
            "two different facts read as one sentence"
        );
    }

    /// **Forgetting a machine takes its grant with it.** Otherwise the widest
    /// grant in the product becomes a permission the person can neither see nor
    /// revoke, and a re-added machine arrives already driveable.
    #[test]
    fn forgetting_a_machine_takes_back_what_the_agent_had_on_it() {
        let (mut machines, mut driving) = two_added();
        carry_out(
            Change::LetTheAgentDrive {
                which: identity('a'),
                until: at(9_000),
            },
            &mut machines,
            &mut driving,
            at(1_000),
        )
        .expect("given");
        assert!(driving.may_drive(&identity('a'), at(1_500)));

        carry_out(
            Change::Forget {
                which: identity('a'),
            },
            &mut machines,
            &mut driving,
            at(1_500),
        )
        .expect("forgotten");

        assert!(!driving.may_drive(&identity('a'), at(1_500)));
        assert!(
            !driving.holds(&identity('a')),
            "a grant nobody can see remains"
        );

        // And adding it again does not bring the grant back.
        carry_out(
            Change::Add {
                which: identity('a'),
                called: a_name("in the studio"),
            },
            &mut machines,
            &mut driving,
            at(2_000),
        )
        .expect("added again");
        assert!(
            !driving.may_drive(&identity('a'), at(2_000)),
            "a re-added machine arrived already driveable"
        );
    }

    /// **Revoke in one action**, for every machine at once, and it says how many
    /// it took back.
    #[test]
    fn everything_the_agent_has_can_be_taken_back_at_once() {
        let (machines, mut driving) = two_added();
        for machine in [identity('a'), identity('b')] {
            carry_out(
                Change::LetTheAgentDrive {
                    which: machine,
                    until: at(9_000),
                },
                &mut TheMachines::none_yet(),
                &mut driving,
                at(1_000),
            )
            .expect("given");
        }
        assert!(TheRows::of(&machines, &driving, at(1_500)).any_stands());

        let taken = carry_out(
            Change::StopTheAgentEverywhere,
            &mut TheMachines::none_yet(),
            &mut driving,
            at(1_500),
        )
        .expect("stopped");
        assert_eq!(taken, 2);
        assert!(!TheRows::of(&machines, &driving, at(1_500)).any_stands());
    }

    /// Stopping one machine leaves the others, and says whether there was
    /// anything to stop.
    #[test]
    fn stopping_one_machine_leaves_the_others() {
        let (machines, mut driving) = two_added();
        for machine in [identity('a'), identity('b')] {
            carry_out(
                Change::LetTheAgentDrive {
                    which: machine,
                    until: at(9_000),
                },
                &mut TheMachines::none_yet(),
                &mut driving,
                at(1_000),
            )
            .expect("given");
        }

        let taken = carry_out(
            Change::StopTheAgentOn {
                which: identity('a'),
            },
            &mut TheMachines::none_yet(),
            &mut driving,
            at(1_500),
        )
        .expect("stopped");
        assert_eq!(taken, 1);
        assert!(!driving.may_drive(&identity('a'), at(1_500)));
        assert!(driving.may_drive(&identity('b'), at(1_500)));

        let again = carry_out(
            Change::StopTheAgentOn {
                which: identity('a'),
            },
            &mut TheMachines::none_yet(),
            &mut driving,
            at(1_500),
        )
        .expect("asked");
        assert_eq!(again, 0, "it reported stopping something twice");
        let _ = machines;
    }

    /// The refusals come through this surface rather than being swallowed by it.
    #[test]
    fn a_refused_change_says_why() {
        let (mut machines, mut driving) = two_added();
        assert_eq!(
            carry_out(
                Change::Add {
                    which: identity('a'),
                    called: a_name("a second name for it"),
                },
                &mut machines,
                &mut driving,
                at(1_000),
            ),
            Err(NotElsewhere::AlreadyAdded)
        );
        assert_eq!(
            carry_out(
                Change::LetTheAgentDrive {
                    which: identity('a'),
                    until: at(500),
                },
                &mut machines,
                &mut driving,
                at(1_000),
            ),
            Err(NotElsewhere::EndsBeforeItBegins)
        );
    }

    /// Nothing to stop is different from stopping nothing, so the control has
    /// something to be offered against.
    #[test]
    fn a_person_with_no_grants_has_nothing_standing() {
        let (machines, driving) = two_added();
        assert!(!TheRows::of(&machines, &driving, at(1_000)).any_stands());
    }
}
