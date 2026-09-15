//! Removing an application, and everything it was granted with it.
//!
//! # One act
//!
//! [`remove`] takes the rented tool and the machine's grants together, and
//! once the tool has removed the application it ends — before returning, with
//! nothing in between that could fail — **every grant the application held**
//! and **every grant made over it**: an agent's grant to open it, focus it or
//! arrange it. What had already expired permits nothing either way; where the
//! machine's list can be swept of it (a machine with an agent) it goes too.
//!
//! The second half is `alo_applications::installed`'s security argument met
//! again. A grant is over an identifier, and an identifier removed today can be
//! installed again tomorrow from a different place by somebody else. A person
//! who let an agent open *the* text editor did not let it open whatever next
//! answers to that name; nor did a person who let an application use the camera
//! let its successor. So removing starts both over, and a reinstalled
//! application arrives granted nothing, like any other.
//!
//! If the tool does not remove it, **nothing ends**: an application still on
//! the machine keeps what its person gave it, and the refusal says why.
//!
//! # Nothing leaves, so nothing is shown
//!
//! Removing reaches no network, so it is not an `alo_egress::Errand` and puts
//! no line on the indicator. A line saying something left would be the
//! indicator saying something untrue, which is the one thing it exists never to
//! do. `tests/installing_updating_and_removing.rs` holds the indicator quiet
//! across a removal.

use std::time::SystemTime;

use alo_applications::Application;
use alo_capability::{Agent, Applicant, GrantId, Reach};
use alo_strings::{Filling, Said, Strings};

use crate::enabled::did_not_answer;
use crate::refusing::NotDone;
use crate::tool::{Failed, Tool};
use crate::words;

/// An application that has been removed, and how many grants ended with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Removed {
    /// The application.
    application: Application,
    /// How many grants ended: those it held and those over it.
    ended: usize,
}

impl Removed {
    /// The application.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// How many grants ended with it — held by it, or made over it.
    #[must_use]
    pub const fn grants_ended(&self) -> usize {
        self.ended
    }

    /// What a person is told, in the language they read.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &words::REMOVED.key(),
            &Filling::of(words::APPLICATION, self.application.identifier()),
        )
    }
}

/// Remove this application, and end every grant it held or that was made over
/// it, in the same act.
///
/// `now` is when grants are read, as everywhere in this workspace; nothing here
/// reads the clock.
///
/// # Errors
/// [`NotDone::NotInstalled`] when it is not here, and whatever the rented tool
/// refused. In both, no grant has ended.
pub fn remove(
    application: Application,
    tool: &impl Tool,
    machine: &mut Agent,
    now: SystemTime,
) -> Result<Removed, NotDone> {
    let identifier = application.identifier();
    if !tool
        .installed()
        .map_err(did_not_answer)?
        .iter()
        .any(|here| here == identifier)
    {
        return Err(NotDone::not_installed(&application));
    }
    tool.remove(&application).map_err(|failed| match failed {
        Failed::DidNotAnswer { said } => NotDone::DidNotAnswer { said },
        Failed::NotInstalled => NotDone::not_installed(&application),
        other => did_not_answer(other),
    })?;
    let ended = ending_its_grants(&application, machine, now);
    Ok(Removed { application, ended })
}

/// End everything this application held, and everything granted over it.
fn ending_its_grants(application: &Application, machine: &mut Agent, now: SystemTime) -> usize {
    let grantee = Applicant::named(application.identifier()).grantee();
    let over = Reach::Application(application.identifier().to_owned());
    match machine.grants_mut() {
        Some(grants) => {
            let held = grants.revoke_everything_for(&grantee);
            let over_it: Vec<GrantId> = grants
                .active_at(now)
                .filter(|kept| kept.grant.reach == over)
                .map(|kept| kept.id)
                .collect();
            let also = over_it.into_iter().filter(|id| grants.revoke(*id)).count();
            held.saturating_add(also)
        }
        // A machine that declined the agent holds applications' grants alone
        // (ADR 0040), so there is nothing over the application to end, and
        // what it held is revoked one handle at a time through the only door
        // such a machine has.
        None => {
            let held: Vec<GrantId> = machine
                .allowed()
                .held_by(&grantee, now)
                .map(|kept| kept.id)
                .collect();
            held.into_iter()
                .filter(|id| machine.revoke_allowed(*id))
                .count()
        }
    }
}
