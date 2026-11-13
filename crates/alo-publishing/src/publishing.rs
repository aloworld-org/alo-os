//! **Publishing, once the line on the indicator is up**: the command written,
//! run once, and read back.
//!
//! The order is `alo-turn`'s to hold (`publishing_a_folder`): the authority is
//! checked, the line goes up on the indicator with [`the_departure()`], and only
//! then is [`publish()`] called — with the [`Departing`] that proves it — and
//! the line comes down whatever happened. Nothing here can be reached without
//! that proof, because [`crate::Runs::run`] will not take a command without it.

use alo_capability::Grantee;
use alo_egress::{Departing, Destination, DestinationError, Leaving, Why};
use alo_strings::{Filling, Strings};

use crate::not_published::NotPublished;
use crate::publication::{Publication, Published};
use crate::runs::{Runs, Through};
use crate::the_line::the_line;
use crate::what_came_of_it::what_came_of_it;
use crate::words;

/// Where a publication goes, as the indicator names it.
pub const WHERE_IT_GOES: &str = "github.com";

/// What the indicator shows while `agent` publishes: *sending something to
/// github.com*.
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

/// Publish `publication` through `through`, run by `runs`, while `leaving` is
/// on the indicator — once.
///
/// # Errors
/// [`NotPublished`], naming what stopped it.
pub fn publish(
    leaving: &Departing,
    publication: &Publication,
    through: &Through,
    runs: &dyn Runs,
    strings: &Strings,
) -> Result<Published, NotPublished> {
    let message = strings.say(&words::THE_COMMIT_SAYS.key(), &Filling::nothing());
    let line = the_line(publication, through, message.text())?;
    let did = runs
        .run(&line, leaving)
        .map_err(NotPublished::DidNotStart)?;
    what_came_of_it(publication, &did)
}
