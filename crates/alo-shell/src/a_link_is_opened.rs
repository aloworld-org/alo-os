//! The compositor's hand for `open-link`: an application started with a link
//! the person followed, as a press starts it.
//!
//! The owner, 2026-11-12: *"Yes, add it (Recommended)"* — the open-with portal
//! asks the desktop to start the application a link opens in **exactly as a
//! person's press does**. So the ask is recorded rather than sent, for the
//! reason `crate::an_application_verb_is_carried_out` gives for `open` (the
//! start waits, and this is the frame loop), on a list of its own,
//! `Server::asked_to_open_a_link`, from which `crate::an_application_is_started`
//! sends it with the link and brings its first window in front, as for a
//! press.
//!
//! **Nothing here decides what opens the link.** The portal decided that from
//! the person's default and recorded it before the line was written; this is
//! the hand, never a second judge. An application already running is started
//! again with the link, which is how a browser opens a new tab: its own
//! single-instance handling, not this compositor's.

use alo_acting::{Answer, NotActed, ToOpenALink};
use alo_dock::AppId;

use crate::Server;

impl Server {
    /// Record `asked`, to be started with its link off the input path, and
    /// answer that it was asked.
    pub(crate) fn a_link_is_asked_for(&mut self, asked: &ToOpenALink) -> Answer {
        let Ok(app) = AppId::named(asked.application().identifier()) else {
            // `Application` already refuses an empty identifier, so this is
            // unreachable from the door; an answer rather than an assumption,
            // because the two types are different crates'.
            return Answer::NotActed(NotActed::CouldNotAsk);
        };
        self.asked_to_open_a_link.push((app, asked.link().clone()));
        Answer::Acted
    }
}

#[cfg(test)]
#[path = "a_link_is_opened_tests.rs"]
mod tests;
