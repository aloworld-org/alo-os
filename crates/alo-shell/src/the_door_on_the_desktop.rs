//! **The application door, served by the desktop a person is signed in to.**
//!
//! `docs/contracts/application-door.md` publishes the door and
//! [`crate::the_application_door`] answers it, and until 2026-10-31 **nothing
//! on a real machine served it**: only an example did. So every approved
//! `open_application`, `focus_application` and `close_application` an agent
//! made knocked at `/run/alo/<uid>/shell.sock`, found nothing, and the person
//! was told *this machine could not reach* the application. That made the
//! owner's decision of 2026-10-31 — an approved `open` starts an application
//! exactly as a Dock press does — unreachable, so serving it is part of that
//! task rather than a second one.
//!
//! # The directory is the daemon's, so this asks again
//!
//! `/run/alo/<uid>` is made by `alo-agentd` when the person's session starts,
//! and the desktop may come up first. The contract's rule is that a compositor
//! finding it missing **serves no door and says which directory is missing** —
//! so this says so once, and looks again every [`HOW_OFTEN`] frames until it
//! is there. A door is never made by this file; only the socket in it.
//!
//! The socket goes when this does, which is the contract's *removed when it
//! stops*.

use crate::Server;
use crate::the_application_door::TheApplicationDoor;
use crate::the_application_doors_place::ThePlace;

/// How many frames pass between two looks for the daemon's directory — two
/// seconds at the desktop's sixty a second.
pub(crate) const HOW_OFTEN: u32 = 120;

/// The door, once it can be served, and when to look again until then.
pub(crate) struct TheDoorOnTheDesktop {
    /// Where the door goes: this person's own directory.
    place: ThePlace,
    /// Whose door it is, by the kernel's own number for them.
    person: u32,
    /// The door, once served.
    served: Option<TheApplicationDoor>,
    /// Frames until the next look, while it is not served.
    until_the_next_look: u32,
    /// Whether the reason it is not served has been said.
    said: bool,
}

impl TheDoorOnTheDesktop {
    /// The door for the person this process runs as, not yet served.
    pub(crate) fn for_this_person() -> Self {
        let person = rustix::process::geteuid().as_raw();
        Self::at(ThePlace::for_person(person), person)
    }

    /// The door at this place, for this person — the road a test takes.
    pub(crate) const fn at(place: ThePlace, person: u32) -> Self {
        Self {
            place,
            person,
            served: None,
            until_the_next_look: 0,
            said: false,
        }
    }

    /// Answer whoever knocked, serving the door first if it can be.
    ///
    /// **Never waits**, which is the door's own rule: a frame loop calls this.
    pub(crate) fn answered(&mut self, server: &mut Server) -> usize {
        if self.served.is_none() {
            self.looked_for();
        }
        self.served.as_mut().map_or(0, |door| door.answered(server))
    }

    /// Whether the door is served — for a test; the desktop only answers.
    #[cfg(test)]
    pub(crate) const fn is_served(&self) -> bool {
        self.served.is_some()
    }

    /// Try to serve the door, if it is time to look.
    fn looked_for(&mut self) {
        if self.until_the_next_look > 0 {
            self.until_the_next_look -= 1;
            return;
        }
        match TheApplicationDoor::served(self.place.clone(), self.person) {
            Ok(door) => {
                eprintln!(
                    "alo-shell: the application door is served at {}",
                    door.socket().display()
                );
                self.served = Some(door);
            }
            Err(why) => {
                if !self.said {
                    eprintln!(
                        "alo-shell: no application door yet, so an agent's approved open, focus \
                         or close cannot reach this desktop: {why}"
                    );
                    self.said = true;
                }
                self.until_the_next_look = HOW_OFTEN;
            }
        }
    }
}

impl Drop for TheDoorOnTheDesktop {
    fn drop(&mut self) {
        if let Some(door) = &self.served {
            door.taken_away();
        }
    }
}

#[cfg(test)]
#[path = "the_door_on_the_desktop_tests.rs"]
mod tests;
