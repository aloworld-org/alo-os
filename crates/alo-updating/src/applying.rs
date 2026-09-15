//! Applying an update the person chose: read the machine as it is now, decide
//! the instruction, and hand it to the base.
//!
//! **Called when the person chose it and at no other moment.** Nothing in this
//! crate calls [`apply`] on its own, and there is no schedule, retry or
//! background road to it: it is one function whose caller is whatever put the
//! choice in front of the person.
//!
//! **Nothing on the machine changes before the base is told**, and the base
//! changes nothing the person keeps: it stages the build beside the one
//! running (ADR 0011). So each failure below leaves the machine exactly as it
//! was, and the next restart starts the build that is running now — which is
//! what [`crate::NotApplied::said`] tells the person.

use alo_keeping_up::{Ready, Source, Staging, WhenItApplies};

use crate::refusing::NotApplied;
use crate::status;
use crate::the_base::Base;

/// Stage `ready`'s offered build from `source`, to apply `when` the person
/// chose.
///
/// Reads the base's status first, so the decision is about the machine as it
/// is at this moment: a machine that moved on since the update was found, or
/// already has this build waiting, is refused before the base is told
/// anything. For [`WhenItApplies::NowBecauseThePersonAsked`] the base restarts
/// the machine once the build is staged, as the person asked.
///
/// # Errors
/// [`NotApplied`].
pub fn apply(
    base: &impl Base,
    ready: &Ready,
    source: &Source,
    when: WhenItApplies,
) -> Result<Staging, NotApplied> {
    let deployments = status::deployments(base).map_err(NotApplied::NotRead)?;
    let staging = Staging::of(ready, &deployments, source, when).map_err(NotApplied::Refused)?;
    base.asked(&staging.arguments())
        .map_err(NotApplied::TheBaseDidNotStageIt)?;
    Ok(staging)
}
