//! The network's three verbs, carried out.
//!
//! By the time a verb reaches here the door has decided it is exactly one a
//! person approved, once, and written that down. What is left is to do it to the
//! right network and to no other — and the broker was never told which network
//! in any form it could act on. It was told thirty-two bytes. So each verb asks
//! the network manager what it has **now**, digests what it reported for each
//! network the way the side that asked digested it, and acts on the one that
//! matches:
//!
//! | verb | asks | acts with |
//! |---|---|---|
//! | `network.join` | the networks the machine can see | `NetworkService::join` |
//! | `network.forget` | the networks this machine has saved | `NetworkService::forget` |
//! | `network.radio` | nothing: on or off is the whole argument | `NetworkService::switch_wireless` |
//!
//! **No match is no change**, and so are two: a network that went out of range
//! between the approval and now is not replaced by whichever is nearest, and an
//! identity two networks share names neither. **A network asking for an
//! organisation's sign-in is not joined** in v0.5, and the answer says so.
//!
//! The network manager decides what a network is and whether joining it worked.
//! Nothing here decides either, and nothing here reads a network's name: only
//! what it was reported under, digested.

use alo_broker::{Identity, NotCarried, Switch};
use alo_networks::{NetworkService, Protection, Saved, Visible};

/// What carries the network's verbs out on this machine.
#[derive(Debug)]
pub struct Network<S> {
    /// The network manager.
    service: S,
}

impl<S: NetworkService> Network<S> {
    /// Carry the network's verbs out against this network manager.
    #[must_use]
    pub const fn against(service: S) -> Self {
        Self { service }
    }

    /// The network manager, for a test to look at what it was asked.
    #[must_use]
    pub const fn service(&self) -> &S {
        &self.service
    }

    /// `network.join`: join the one visible network reported under this
    /// identity.
    ///
    /// # Errors
    /// [`NotCarried`], and nothing was joined.
    pub fn join(&self, identity: Identity) -> Result<(), NotCarried> {
        let now = self
            .service
            .now()
            .map_err(|why| NotCarried(why.to_string()))?;
        let network = the_one(&now.visible, identity, Visible::as_reported)?;
        if network.protection() == Protection::Enterprise {
            return Err(NotCarried(
                "the network approved asks for an organisation's sign-in, which this machine \
                 does not set up in v0.5, so nothing was changed"
                    .to_owned(),
            ));
        }
        self.service
            .join(network)
            .map_err(|why| NotCarried(why.to_string()))
    }

    /// `network.forget`: forget the one saved network reported under this
    /// identity.
    ///
    /// # Errors
    /// [`NotCarried`], and nothing was forgotten.
    pub fn forget(&self, identity: Identity) -> Result<(), NotCarried> {
        let now = self
            .service
            .now()
            .map_err(|why| NotCarried(why.to_string()))?;
        let network = the_one(&now.saved, identity, Saved::as_reported)?;
        self.service
            .forget(network)
            .map_err(|why| NotCarried(why.to_string()))
    }

    /// `network.radio`: turn the wireless radio on or off.
    ///
    /// # Errors
    /// [`NotCarried`], and the radio is as it was.
    pub fn radio(&self, switch: Switch) -> Result<(), NotCarried> {
        self.service
            .switch_wireless(switch == Switch::On)
            .map_err(|why| NotCarried(why.to_string()))
    }
}

/// The one network reported under this identity, or no change.
fn the_one<T>(
    among: &[T],
    identity: Identity,
    reported: impl Fn(&T) -> Vec<u8>,
) -> Result<&T, NotCarried> {
    let mut matching = among
        .iter()
        .filter(|network| Identity::of_what_was_reported(&reported(network)) == identity);
    match (matching.next(), matching.next()) {
        (Some(one), None) => Ok(one),
        (None, _) => Err(NotCarried(
            "the network manager reports no network with the identity that was approved, so \
             nothing was changed"
                .to_owned(),
        )),
        (Some(_), Some(_)) => Err(NotCarried(
            "the network manager reports more than one network with the identity that was \
             approved, so nothing was changed"
                .to_owned(),
        )),
    }
}
