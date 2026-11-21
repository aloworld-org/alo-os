//! **Whether alo may work in the person's own window of one application**, as
//! one function both ends of the door call.
//!
//! `alo_capability::AloInApplications::may_work_in_their_window_of` is the rule,
//! and it takes whether the application has a copy of its own for alo, which
//! `alo_launching::has_a_copy_of_its_own` answers. Two callers putting those two
//! together each in their own way is how a desktop and a turn come to disagree.
//! So they are put together once, here:
//!
//! - the desktop answers `work-in-their-window` with it
//!   (`alo_shell::alo_at_work`), and
//! - a turn asks it **before the card** of an adapter's verb, whose message
//!   reaches the application the person has open (`alo_turn`), so a person is
//!   never asked to approve something the desktop would then refuse.

use alo_capability::AloInApplications;

/// Which applications have a copy of their own for alo, said through this
/// crate so a caller that words a refusal asks the same table the answer did.
pub use alo_launching::has_a_copy_of_its_own;

/// Whether alo may work in the person's own window of `application`, its
/// identifier, at the setting `chosen`.
#[must_use]
pub fn alo_may_work_in_their_window(chosen: AloInApplications, application: &str) -> bool {
    chosen.may_work_in_their_window_of(alo_launching::has_a_copy_of_its_own(application))
}

#[cfg(test)]
#[path = "where_alo_may_work_tests.rs"]
mod tests;
