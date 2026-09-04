//! The boundary this service puts around a turn's work, and the only real one
//! there is.
//!
//! `alo-turn` says what a boundary has to do and cannot say how, because it is
//! portable and a boundary is a kernel. `alo-bounding` says how and does not
//! know what a turn is asking for. This file is the one place both are held, and
//! it is here rather than in either of them for the reason every other join in
//! this crate is: **the daemon is the process**, and everything else in this
//! workspace is a decision it makes in order.
//!
//! # What one turn's execution costs
//!
//! A control group made beside `home`, one entry written into the kernel's map,
//! this thread moved in by writing a byte, the verb, the thread moved back out
//! through a descriptor opened when the service started, the entry removed and
//! the control group taken away. `alo_bounding::Turns::doing` is all of that in
//! order and this file adds two things to it: the name, and everywhere the
//! execution may reach.
//!
//! # The name is a number, and it is this service's
//!
//! A turn's control group is one component under the service's own, and it has
//! to be a name no other turn is using. It is a count kept here rather than
//! anything about the turn: a name made out of the agent, the person or the verb
//! would put something an agent influences into a path on the filesystem, and a
//! name made out of a clock would collide on a machine whose clock moved. The
//! count is `u64` and saturates, which is a machine that has run out of names
//! after eighteen quintillion turns and is a refusal rather than a reused one.
//!
//! # Everything a refusal here means is *nothing happened*
//!
//! Except one, and that one is why `alo_turn::NoBoundary` has a question on it:
//! a thread that went into a boundary and could not be brought back is still in
//! there, refused everything outside a grant that has ended. The service stops
//! over it (`crate::serving`), and the reason it can is that the fact travels as
//! an answerable question rather than as an English sentence somebody would have
//! had to match on.

use alo_bounding::{Boundary, NotBounded, Turns, places_of};
use alo_files::Reaching;
use alo_turn::{Bounding, Doing, Done, NoBoundary};

/// The boundary this machine's kernel imposes, for as long as this is held.
///
/// Made once, when the service starts, and given back when it stops: making it
/// moves this process into a control group of its own, and the programme it
/// loads is attached to `file_open` for every process on the machine until it
/// is dropped.
#[derive(Debug)]
pub struct ByTheKernel {
    /// Where this service's turns are made, and the way back out of one.
    turns: Turns,

    /// The programme in the kernel, and the map it decides from.
    boundary: Boundary,

    /// How many turns have been carried out, which is where the next name
    /// comes from.
    named: u64,
}

impl ByTheKernel {
    /// Load the programme, attach it, and make this service's subtree.
    ///
    /// In that order, and the order is `alo-bounding`'s: a subtree with nothing
    /// attached is a service whose turns would be bounded by nobody, and it is
    /// the cheaper of the two to give back if the other fails.
    ///
    /// # Errors
    /// [`NotBounded`], and ADR 0015's rule is that this is the end of it: a
    /// service that cannot impose a boundary cannot bound a turn, and a turn
    /// that cannot be bounded does not run — so what a caller does with one of
    /// these is not start.
    pub fn imposed() -> Result<Self, NotBounded> {
        let boundary = Boundary::imposed()?;
        let turns = Turns::of_this_service()?;
        Ok(Self {
            turns,
            boundary,
            named: 0,
        })
    }

    /// Put this service back where it was and take its subtree away.
    ///
    /// Separate from dropping the value for `alo_bounding::Turns::given_back`'s
    /// reason: moving a process between control groups can fail, and a `Drop`
    /// that swallowed it would leave a machine filling with the remains of
    /// daemons with nothing saying so. Dropping the programme *is* the right
    /// shape — a service that has stopped stops enforcing — so that half is a
    /// `Drop` and this half is not.
    ///
    /// # Errors
    /// [`NotBounded`] if this process could not be put back or the subtree could
    /// not be removed. What is left on the machine is what was there before.
    pub fn given_back(self) -> Result<(), NotBounded> {
        self.turns.given_back()
    }
}

impl Bounding for ByTheKernel {
    /// Carry one execution out inside a boundary, and nothing else inside it.
    ///
    /// The places are made from the reach rather than from the grants, which is
    /// item 26b's decision: `alo_bounding::places_of` looks each one up and
    /// refuses a path that is not there rather than turning it into a place made
    /// of zeroes.
    fn carrying_out(&mut self, reaching: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary> {
        let places: Vec<&std::path::Path> = reaching.places().collect();
        let granted = places_of(&places).map_err(as_no_boundary)?;
        let named = next_name(&mut self.named);
        self.turns
            .doing(&mut self.boundary, &named, granted, || doing.done())
            .map_err(as_no_boundary)
    }
}

/// The name the next turn's control group is made under.
///
/// A free function rather than a method so that a machine with no kernel can
/// still be asked what it would call things — which is the only part of this
/// file that is arithmetic rather than a syscall.
fn next_name(named: &mut u64) -> String {
    *named = named.saturating_add(1);
    format!("turn-{named}")
}

/// What a turn is told about a boundary that could not be imposed.
///
/// The reason keeps its English and goes to the service log; the sentence the
/// person reads is `alo-turn`'s, and this carries neither into the other. The
/// one thing that crosses as a fact rather than as text is whether a thread is
/// still inside, because that is what a service decides on.
fn as_no_boundary(why: NotBounded) -> NoBoundary {
    if why.a_thread_is_still_inside() {
        return NoBoundary::with_a_thread_still_inside(why.to_string());
    }
    NoBoundary::because(why.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A reason about the kernel reaches the service log and nothing else.**
    /// The conversion is the whole seam between the two audiences `failing.rs`
    /// and `alo_turn::unbounded` each describe from their own side.
    #[test]
    fn the_reason_crosses_as_english_for_whoever_looks_after_the_machine() {
        let why = NotBounded::NothingToBound;
        let said = why.to_string();
        let crossed = as_no_boundary(why);

        assert_eq!(crossed.why(), said);
        assert!(!crossed.a_thread_is_still_inside());
    }

    /// **And a thread left inside crosses as a question**, which is the one
    /// thing a service has to be able to ask without reading a sentence.
    #[test]
    fn a_thread_left_inside_crosses_as_something_a_service_can_ask() {
        let crossed = as_no_boundary(NotBounded::NotBroughtBack {
            why: std::io::Error::other("the control group is gone"),
        });

        assert!(crossed.a_thread_is_still_inside());
        assert!(crossed.why().contains("could not be brought back"));
    }

    /// **Every turn is made under a name of its own**, and the name is a count
    /// of this service's turns rather than anything an agent can influence: a
    /// name made out of the agent, the person or the verb would put something a
    /// model can reach into a path on the filesystem.
    #[test]
    fn every_turn_is_named_once_and_after_this_services_own_count() {
        let mut named = 0;
        assert_eq!(next_name(&mut named), "turn-1");
        assert_eq!(next_name(&mut named), "turn-2");

        // A service that has run out of names says the last one again rather
        // than starting over at one. It saturates because the alternative is a
        // count that wraps back through names, and it is safe either way for a
        // reason worth writing down: a turn's control group is taken away when
        // the turn ends, and this boundary is held exclusively, so there are
        // never two turns on one service to collide.
        let mut most = u64::MAX - 1;
        assert_eq!(next_name(&mut most), format!("turn-{}", u64::MAX));
        assert_eq!(next_name(&mut most), format!("turn-{}", u64::MAX));
    }
}
