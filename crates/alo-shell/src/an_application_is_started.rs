//! Starting the applications a person asked for, off the input path.
//!
//! `docs/features.md` [v0.01] — *clicking an application … **nothing open:** it
//! starts.* `crate::a_press_on_the_dock` records the ask in
//! `Server::asked_to_start` and this is where it is sent.
//!
//! # Why it is here and not in the press
//!
//! `alo_portals::activated` sends `Activate` to the application's own bus name
//! and **the bus starts it if it is not running** — which is the correct
//! mechanism and is not a fast one. Sent from the press handler it would hold
//! the compositor's input loop until the application answered, so every window
//! on the machine would stop drawing while one started. A person pressing an
//! icon would watch their desktop freeze.
//!
//! # What this is not
//!
//! **It is not a launcher.** `alo_portals::opening`'s header has refused that
//! since it was written, and the refusal is repeated at `activated` itself: a
//! message to an application, never a command run. Nothing reads an `Exec`
//! line, starts a process, or passes anything to a shell. The destination is a
//! validated identifier turned into a path by one character substitution, the
//! method is a literal, and the only argument is an empty map.
//!
//! That is what makes this permissible under the second law at all. *No code
//! runs unless the person chose it* — pressing one's own Dock icon **is** that
//! choice, so starting the application is allowed. What the law forbids is the
//! mechanism, and anything assembling a command line hands over the one thing
//! the capability model exists to withhold.
//!
//! # An application that cannot be reached this way is not started
//!
//! No name on the bus and nothing to start one answers `false`, and that is
//! returned rather than retried or worked around. `alo_portals::opening` makes
//! the same refusal for `Open` and names the alternative it will not take:
//! *starting such an application from its `Exec` line is a launcher's work.*

use crate::Server;

/// Whatever can ask an application to start, given its identifier.
///
/// **A function rather than a crate**, and that is deliberate.
/// `alo_portals::activated` is the implementation — D-Bus activation, the same
/// call as `Open` with no file — and `alo-portals` is a **dev-dependency** of
/// this crate rather than a real one, because the compositor has no business
/// depending on portal *policy*: that crate decides what a portal request is
/// answered with, and nothing here should be able to reach those decisions.
///
/// The two ways out of that were both worse. Promoting the dependency puts
/// every portal decision inside the compositor's reach to satisfy one twenty-
/// line call. Copying the call here makes **two** implementations of D-Bus
/// activation, and this repository has learned what two implementations of one
/// question cost — a press reaching a control that is not the one that lit up.
///
/// So the shell holds the ask and not the sending, and whoever wires the two
/// together supplies this. It is `alo_applications::Installed`'s own shape:
/// *this crate holds no opinion about where the entries come from.* It also
/// makes the decision testable without a bus, which nothing about a real
/// connection would be.
pub type AsksAnApplicationToStart<'a> = dyn Fn(&str) -> bool + 'a;

/// One way to start applications, owned.
///
/// The boxed, `Send` form of [`AsksAnApplicationToStart`] — what a caller hands
/// over once and this machine keeps. `Send` because a fixture runs the desktop
/// on a thread of its own, and named because the same type appears in three
/// places and three spellings of one type is three chances to change two of
/// them.
pub type AWayToStartApplications = Box<dyn Fn(&str) -> bool + Send>;

impl Server {
    /// Take every application a person asked to start, and say which were not.
    ///
    /// **Every ask is taken off the list whether it succeeded or not**, which is
    /// `crate::a_press_on_an_edge`'s rule and the same reasoning: a press that
    /// failed is not retried next frame. A person pressed an icon once, and a
    /// refusal is an answer — while an ask left on the list would be an
    /// `Activate` sent on every frame, which is a loop around a bus.
    ///
    /// Returns the applications that could not be reached, for a caller to
    /// record. **Not a `Result`**: one ask failing says nothing about the next,
    /// and an early return would silently drop the rest.
    pub fn start_what_was_asked_for(
        &mut self,
        ask: &AsksAnApplicationToStart<'_>,
    ) -> Vec<alo_dock::AppId> {
        let asked = std::mem::take(&mut self.asked_to_start);
        let mut not_started = Vec::new();
        for application in asked {
            if !ask(application.name()) {
                not_started.push(application);
            }
        }
        not_started
    }

    /// Give this machine a way to ask an application to start.
    ///
    /// Called once, by whoever stands the desktop up and can reach
    /// `alo_portals::activated`. See `Server::asks_an_application_to_start`
    /// for why it arrives from outside rather than being built here.
    pub fn it_can_start_applications_with(&mut self, ask: AWayToStartApplications) {
        self.asks_an_application_to_start = Some(ask);
    }

    /// Start what was asked for, through the sender this machine was given.
    ///
    /// Returns what could not be started, for a caller to record — and
    /// **everything, when this machine was never given a sender**, which is the
    /// honest answer rather than a silent success.
    pub fn start_what_was_asked_for_now(&mut self) -> Vec<alo_dock::AppId> {
        // Taken out and put back so the asks list can be drained mutably while
        // the sender is called. Clunkier than a borrow and it is the borrow
        // checker's correct objection: the sender could, in principle, ask this
        // machine for something.
        let Some(ask) = self.asks_an_application_to_start.take() else {
            return std::mem::take(&mut self.asked_to_start);
        };
        let not_started = self.start_what_was_asked_for(&*ask);
        self.asks_an_application_to_start = Some(ask);
        not_started
    }

    /// How many applications are waiting to be started.
    ///
    /// For a test and for a caller deciding whether it needs a bus at all: a
    /// lane with nothing to start should not open a connection to discover
    /// that.
    #[must_use]
    pub fn how_many_are_waiting_to_start(&self) -> usize {
        self.asked_to_start.len()
    }
}

#[cfg(test)]
#[path = "an_application_is_started_tests.rs"]
mod tests;
