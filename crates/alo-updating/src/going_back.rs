//! Going back to the build before, because the person approved it: read the
//! machine as it is now, note where it is going, and tell the base.
//!
//! **Called when the person approved it and at no other moment.** Like
//! [`crate::apply`], nothing in this crate calls [`go_back`] on its own; its
//! caller is whatever put `alo_keeping_up::GoingBack`'s sentence in front of
//! the person.
//!
//! # In this order, and the order is the guarantee
//!
//! 1. **The base's status is read now**, and `alo_keeping_up::Returning`
//!    refuses when the machine is no longer what the offer described. Nothing
//!    has changed, and the base is told nothing.
//! 2. **The build to go back to is noted** under `/var`, whole or not at all.
//!    If it cannot be, the base is told nothing: going back without the note
//!    would be recorded at the next start as an update, and a record that says
//!    the wrong thing happened is worse than a return that did not start.
//! 3. **The base is told**, with exactly the arguments `Returning` wrote. For
//!    *restart now* the machine restarts inside this call, which is why the
//!    note comes first.
//!
//! **A note left by a base that refused is not removed.** It names the build
//! the person chose; if the base did in fact set it before failing, the next
//! start is recorded as the return it is, and if it did not, the machine
//! starts on the same build and the note waits, harmlessly, until the next
//! change replaces or clears it ([`crate::after_a_restart`]).

use alo_keeping_up::{GoingBack, Returning, WhenItApplies};

use crate::across_restarts::AcrossRestarts;
use crate::one_build;
use crate::refusing::NotGoneBack;
use crate::status;
use crate::the_base::Base;

/// Go back to the build `offer` named, `when` the person chose.
///
/// # Errors
/// [`NotGoneBack`]. In each, the machine starts on the build it runs now.
pub fn go_back(
    base: &impl Base,
    offer: &GoingBack,
    kept: &AcrossRestarts,
    when: WhenItApplies,
) -> Result<Returning, NotGoneBack> {
    let deployments = status::deployments(base).map_err(NotGoneBack::NotRead)?;
    let returning = Returning::of(offer, &deployments, when).map_err(NotGoneBack::Refused)?;
    one_build::keep(kept.going_back_to(), returning.to()).map_err(|trouble| {
        NotGoneBack::NotNoted {
            path: trouble.path,
            why: trouble.why,
        }
    })?;
    base.asked(&returning.arguments())
        .map_err(NotGoneBack::TheBaseDidNotSetIt)?;
    Ok(returning)
}
