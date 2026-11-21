//! **The person's session bus, opened the first time an adapter's message
//! goes**, and kept for the daemon's life — so a daemon whose person has no
//! session bus yet starts all the same, and an adapter's approved call is
//! refused in words (`NotDelivered::NotThere`) rather than the daemon failing.

use std::cell::OnceCell;

use alo_adapters::{Delivers, Message, NotDelivered, SessionBus};

/// The session bus of the person this daemon runs as.
#[derive(Default)]
pub(crate) struct ThePersonsSessionBus {
    /// Opened on the first message: `None` inside when there was none to open.
    opened: OnceCell<Option<SessionBus>>,
}

impl Delivers for ThePersonsSessionBus {
    fn deliver(&self, message: &Message) -> Result<(), NotDelivered> {
        match self.opened.get_or_init(|| SessionBus::this_persons().ok()) {
            Some(bus) => bus.deliver(message),
            None => Err(NotDelivered::NotThere),
        }
    }
}
