//! **Publishing: the command written, run once, and read back.**
//!
//! The order is `alo-turn`'s to hold (`publishing_a_folder`): the authority is
//! checked; the rule about what may leave this machine is asked of
//! [`the_departure()`] — what is about to leave, and where — and a rule that
//! says no refuses with nothing sent; only then is [`publish()`] called, with
//! the turn's own [`AroundACommand`], which starts the command inside a control
//! group of its own and shows on the indicator what it sends, as for every
//! command alo runs (ADR 0148, as amended 2026-11-13).

use alo_capability::{AroundACommand, Grantee};
use alo_egress::{Destination, DestinationError, Leaving, Why};
use alo_strings::{Filling, Strings};

use crate::not_published::NotPublished;
use crate::publication::{Publication, Published};
use crate::runs::{Runs, Through};
use crate::the_line::the_line;
use crate::what_came_of_it::what_came_of_it;
use crate::words;

/// Where a publication goes, as a rule about what may leave is asked about it.
pub const WHERE_IT_GOES: &str = "github.com";

/// What leaves while `agent` publishes — something sent to github.com — for the
/// rule in force to say whether it may.
///
/// # Errors
/// [`DestinationError`], which [`WHERE_IT_GOES`] cannot cause.
pub fn the_departure(agent: &Grantee) -> Result<Leaving, DestinationError> {
    Ok(Leaving::because(
        agent,
        Why::Sending,
        Destination::at(WHERE_IT_GOES)?,
    ))
}

/// Publish `publication` through `through`, run by `runs` and started and
/// watched by `around` — once.
///
/// # Errors
/// [`NotPublished`], naming what stopped it.
pub fn publish(
    publication: &Publication,
    through: &Through,
    runs: &dyn Runs,
    around: &mut dyn AroundACommand,
    strings: &Strings,
) -> Result<Published, NotPublished> {
    let message = strings.say(&words::THE_COMMIT_SAYS.key(), &Filling::nothing());
    let line = the_line(publication, through, message.text())?;
    let did = runs.run(&line, around).map_err(NotPublished::DidNotStart)?;
    what_came_of_it(publication, &did)
}
