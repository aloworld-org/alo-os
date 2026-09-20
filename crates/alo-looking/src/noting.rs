//! Where a check's one departure is written down afterwards.
//!
//! Law 1 is two halves and this crate only had one of them. A check was shown
//! while it happened ([`crate::look`] puts it on the indicator before it asks
//! anything) and then it was gone: nothing in this repository wrote it into
//! `alo-record`, so a person who looked at their machine an hour later found no
//! trace of the one errand alo OS makes on its own on the way up. *Every
//! network egress an agent causes is visible at the moment it happens **and
//! afterwards in a record*** — the second half is this file.
//!
//! # Why it is a trait rather than a dependency on the record
//!
//! `alo_record::Entry::left_on_its_own` takes an `alo_egress::Underway`, and
//! the indicator is the only thing that makes one — which is what stops an
//! errand nobody showed from being written down as one. [`crate::look`] holds
//! that `Underway` and ends it, so the writing has to happen inside the check.
//!
//! It could have happened by this crate taking `alo-record` and `alo-keeping`
//! as dependencies and opening the machine's record itself. It does not,
//! because *where a record is kept and how long it is kept for* is a decision
//! `alo-keeping` owns and an organisation sets (ADR 0004), and a crate that
//! asks a question of a registry has no business holding the file everything
//! else on the machine writes to. So the check is handed the thing that writes,
//! and the one caller on a booted machine is what knows which file that is.
//!
//! # It cannot refuse
//!
//! [`Noting::the_check_left`] answers nothing. A record that could decline to
//! keep something would be a record with a way to be silent about exactly the
//! entry that mattered, which is `alo_record::Record::keep`'s own argument —
//! and a check that abandoned its answer because a disk was full would be a
//! machine that stops finding out about updates when its record stops being
//! written. An implementation that meets a failure **keeps it** and whoever
//! holds it says so afterwards; that is what
//! `alo_looking_once::IntoTheRecord` does.

use alo_egress::Underway;

/// What a check's departure is written into.
///
/// One method, called once per check — on every road out of it, including
/// every refusal, because what the indicator showed is what the record has to
/// be able to account for afterwards.
pub trait Noting {
    /// alo OS reached the network to ask whether there is an update.
    ///
    /// Called while the errand is still underway, because the entry is made
    /// from it and the indicator takes it back at the end of the check. The
    /// moment in the entry is the moment the errand went on the indicator, so
    /// what a person saw and what the record says cannot disagree about when.
    fn the_check_left(&mut self, underway: &Underway);
}
