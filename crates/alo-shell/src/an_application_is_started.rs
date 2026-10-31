//! Starting the applications a person asked for, off the input path.
//!
//! `docs/features.md` — *clicking an application … **nothing open:** it
//! starts.* `crate::a_press_on_the_dock` records the ask in
//! `Server::asked_to_start` and this is where it is sent.
//!
//! # Why it is here and not in the press
//!
//! Starting an application waits — for the bus to start one, or for the
//! person's systemd manager to take a unit — and sent from the press handler
//! that wait would hold the compositor's input loop, so every window on the
//! machine would stop drawing while one started. A person pressing an icon
//! would watch their desktop freeze.
//!
//! # Who asked is carried with the ask
//!
//! Two kinds of ask end here, and they take one road. **A person** pressing an
//! icon, pressing Enter on an application in the alo Bar, or pressing the
//! terminal chord is the choice law 2 requires — the law binds the agent, never
//! the person — so `alo-desktop` starts what they asked for the way every
//! Linux desktop does: over the bus where the entry says `DBusActivatable=true`,
//! and from the entry's own `Exec` line otherwise (`alo_launching`). **The
//! agent's** approved `open` reaches here through ADR 0108's door and is kept
//! on a list of its own, `Server::its_agent_asked_to_start`, so the sender is
//! told who asked. **Since the owner's decision of 2026-10-31 it takes the same
//! road**: the approval and the grant were the person's choice, and what runs
//! is still an installed entry's own command. It was bus activation only until
//! that day.
//!
//! *This header said until 2026-10-31 that nothing here may read an `Exec`
//! line, and that law 2 forbade the mechanism. Law 2's own text binds the
//! agent and never the person, and no ADR forbids a person's press from
//! starting an installed entry's command — so Firefox, Dolphin and LibreOffice
//! were listed on the Dock and refused to start, on a reading of the law the
//! law does not make.*
//!
//! **Nothing here starts anything.** The shell holds the asks; the sender it is
//! given does the starting, and what that sender may run is an installed
//! entry's own `Exec` line, found by its identifier — never text a caller
//! supplied, and never a line composed for a shell.
//!
//! # An application that cannot be started is said, not retried
//!
//! The sender answers `false` and that is returned rather than retried or
//! worked around, so the desktop's log names every press that started nothing.

use crate::Server;

/// Who asked for an application to be started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoAskedToStart {
    /// The person, with their own press, Enter or chord.
    ThePerson,
    /// Their agent, through the compositor's application door (ADR 0108).
    TheirAgent,
}

/// Whatever can ask an application to start, given its identifier and who
/// asked.
///
/// **A function rather than a crate**, and that is deliberate. The
/// implementation is `alo-desktop`'s — `alo_portals::activated` for the bus,
/// and the person's systemd manager for an `Exec` line — and neither belongs
/// in the compositor: `alo-portals` decides what a portal request is answered
/// with, and nothing here should be able to reach those decisions.
///
/// So the shell holds the ask and not the sending, and whoever wires the two
/// together supplies this. It is `alo_applications::Installed`'s own shape:
/// *this crate holds no opinion about where the entries come from.* It also
/// makes the decision testable without a bus, which nothing about a real
/// connection would be.
pub type AsksAnApplicationToStart<'a> = dyn Fn(&str, WhoAskedToStart) -> bool + 'a;

/// One way to start applications, owned.
///
/// The boxed, `Send` form of [`AsksAnApplicationToStart`] — what a caller hands
/// over once and this machine keeps. `Send` because a fixture runs the desktop
/// on a thread of its own, and named because the same type appears in three
/// places and three spellings of one type is three chances to change two of
/// them.
pub type AWayToStartApplications = Box<dyn Fn(&str, WhoAskedToStart) -> bool + Send>;

impl Server {
    /// Take every application anybody asked to start, and say which were not.
    ///
    /// **Every ask is taken off the list whether it succeeded or not**, which is
    /// `crate::a_press_on_an_edge`'s rule and the same reasoning: a press that
    /// failed is not retried next frame. A person pressed an icon once, and a
    /// refusal is an answer — while an ask left on the list would be a start
    /// sent on every frame, which is a loop around a bus.
    ///
    /// The person's asks are sent first and then the agent's, each in the order
    /// they were made, and each with who made it.
    ///
    /// Returns the applications that could not be reached, for a caller to
    /// record. **Not a `Result`**: one ask failing says nothing about the next,
    /// and an early return would silently drop the rest.
    pub fn start_what_was_asked_for(
        &mut self,
        ask: &AsksAnApplicationToStart<'_>,
    ) -> Vec<alo_dock::AppId> {
        let persons = std::mem::take(&mut self.asked_to_start)
            .into_iter()
            .map(|it| (it, WhoAskedToStart::ThePerson));
        let agents = std::mem::take(&mut self.its_agent_asked_to_start)
            .into_iter()
            .map(|it| (it, WhoAskedToStart::TheirAgent));
        let mut not_started = Vec::new();
        for (application, who) in persons.chain(agents) {
            if !ask(application.name(), who) {
                not_started.push(application);
            }
        }
        not_started
    }

    /// Give this machine a way to ask an application to start.
    ///
    /// Called once, by whoever stands the desktop up. See
    /// `Server::asks_an_application_to_start` for why it arrives from outside
    /// rather than being built here.
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
            let mut everything = std::mem::take(&mut self.asked_to_start);
            everything.append(&mut self.its_agent_asked_to_start);
            return everything;
        };
        let not_started = self.start_what_was_asked_for(&*ask);
        self.asks_an_application_to_start = Some(ask);
        not_started
    }

    /// How many applications are waiting to be started, whoever asked.
    ///
    /// For a test and for a caller deciding whether it needs a bus at all: a
    /// lane with nothing to start should not open a connection to discover
    /// that.
    #[must_use]
    pub fn how_many_are_waiting_to_start(&self) -> usize {
        self.asked_to_start.len() + self.its_agent_asked_to_start.len()
    }
}

#[cfg(test)]
#[path = "an_application_is_started_tests.rs"]
mod tests;
