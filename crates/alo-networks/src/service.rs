//! Asking the network manager what there is, and telling it the three changes
//! the broker makes.
//!
//! Two traits, because the two sides need different things. The person's side —
//! listing networks in Settings, saying what a proposed change would do to the
//! connection a conversation is using — only asks ([`Networks`]). The broker
//! also changes ([`NetworkService`]), and only after its door has decided a
//! change is exactly one a person approved. A trait rather than the client
//! itself so that which network is acted on is tested against every shape of
//! answer; `crate::network_manager` is the one a machine runs.
//!
//! The failures carry English for a service log and a record's reason, never
//! for a person: what a person reads is said by the surface that asked, from
//! the broker's one-word answer.

use crate::reported::{Saved, TheNetworks, Visible};

/// The network manager could not be asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotAnswering(pub String);

impl std::fmt::Display for NotAnswering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the network manager could not be asked: {}", self.0)
    }
}

impl std::error::Error for NotAnswering {}

/// The network manager did not make a change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotDone(pub String);

impl std::fmt::Display for NotDone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the network manager did not make the change: {}", self.0)
    }
}

impl std::error::Error for NotDone {}

/// What there is now.
pub trait Networks {
    /// Everything the network manager reports at this moment.
    ///
    /// # Errors
    /// [`NotAnswering`].
    fn now(&self) -> Result<TheNetworks, NotAnswering>;
}

/// What there is now, and the three changes the broker makes.
pub trait NetworkService: Networks {
    /// Join this visible network, and answer once it is joined or has failed.
    ///
    /// No password is handed over here, because there is nowhere to put one:
    /// a protected network's password is asked of the person by the network
    /// manager, through the agent in their own session.
    ///
    /// # Errors
    /// [`NotDone`].
    fn join(&self, network: &Visible) -> Result<(), NotDone>;

    /// Forget this saved network.
    ///
    /// # Errors
    /// [`NotDone`].
    fn forget(&self, network: &Saved) -> Result<(), NotDone>;

    /// Turn the wireless radio on, or off.
    ///
    /// # Errors
    /// [`NotDone`].
    fn switch_wireless(&self, on: bool) -> Result<(), NotDone>;
}
