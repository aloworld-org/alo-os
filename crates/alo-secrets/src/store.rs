//! The Secret Service, reached over a connection this crate made.
//!
//! ADR 0022, amended 2026-09-08: **the client is given the connection.** That is
//! the whole reason this is not `libsecret` — no libsecret API accepts one, so a
//! daemon using it reaches whichever bus `DBUS_SESSION_BUS_ADDRESS` names, which
//! is the default connection the ADR exists to refuse.
//!
//! Here the address comes from [`crate::TheBus`], which comes from a uid, and
//! `SecretService::connect_with_existing` is handed the connection built from
//! it. Nothing in this file calls `Connection::session()` or
//! `SecretService::connect`, and both of those are exactly the environment-chosen
//! default.
//!
//! # The session is encrypted, and that is not a default
//!
//! [`EncryptionType::Dh`], never `Plain`. A secret crossing the bus in clear is
//! a secret available to whatever else can read that bus, and *it made the tests
//! simpler* is not a reason anybody should find in this file later.

use std::collections::HashMap;

use alo_models::{Secret, SecretRef};
use secret_service::EncryptionType;
use secret_service::blocking::SecretService;

use crate::bus::TheBus;
use crate::refusing::NotStored;

/// What every item alo OS keeps is filed under, so that a search cannot match
/// somebody else's entry that happens to share a name.
const OURS: &str = "xdg:schema";

/// The value of [`OURS`], which is this product's own reverse-domain name.
const ALO: &str = "dev.alo.Provider";

/// What the reference itself is filed under.
const REFERRED_TO_AS: &str = "reference";

/// The person's keyring, open on a bus this crate chose.
pub struct TheKeyring {
    /// The service, holding the connection it was handed.
    service: SecretService<'static>,
}

impl TheKeyring {
    /// Open the keyring on this bus.
    ///
    /// The connection is built from [`TheBus::as_an_address`] and handed to the
    /// client, so the bus reached is the one this crate derived and not one an
    /// environment named.
    ///
    /// # Errors
    /// [`NotStored::Unavailable`] when nothing answers on that bus, or when what
    /// answers is not a Secret Service.
    pub fn opened(bus: &TheBus) -> Result<Self, NotStored> {
        let connection = zbus::blocking::connection::Builder::address(bus.as_an_address().as_str())
            .map_err(|_| NotStored::Unavailable)?
            .build()
            .map_err(|_| NotStored::Unavailable)?;
        let service = SecretService::connect_with_existing(EncryptionType::Dh, connection)
            .map_err(|_| NotStored::Unavailable)?;

        // **And it is asked something.** Building the client is a proxy and a
        // session handshake, and on a bus with no Secret Service on it that can
        // still come back holding nothing — measured, in
        // `a_bus_with_no_keyring_on_it_is_unavailable`, where `opened` returned
        // a keyring nobody was serving. So the collections are asked for, which
        // is the cheapest question only a real service can answer, and a store
        // that cannot answer it is `Unavailable` here rather than at the first
        // key somebody wanted.
        service
            .get_all_collections()
            .map_err(|_| NotStored::Unavailable)?;
        Ok(Self { service })
    }

    /// The key this reference names, if the keyring will give it up.
    ///
    /// # Errors
    /// [`NotStored::Locked`] when the entry is there behind a locked collection —
    /// **and nothing is unlocked here**, because unlocking is a person answering
    /// a prompt and not a daemon deciding for them.
    /// [`NotStored::Missing`] when nothing is filed under this reference.
    /// [`NotStored::Denied`] when the service refuses this caller, and
    /// [`NotStored::Unavailable`] when it stops answering.
    pub fn look_up(&self, named: &SecretRef) -> Result<Secret, NotStored> {
        let mut asked = HashMap::new();
        asked.insert(OURS, ALO);
        asked.insert(REFERRED_TO_AS, named.as_str());

        let found = self.service.search_items(asked).map_err(as_not_stored)?;

        // Locked before missing: an entry that is there and will not open is a
        // different thing for a person to do about than one that was never
        // added, and answering `Missing` for it would send them to add a key
        // they already have.
        let Some(item) = found.unlocked.first() else {
            if found.locked.is_empty() {
                return Err(NotStored::Missing);
            }
            return Err(NotStored::Locked);
        };

        let held = item.get_secret().map_err(as_not_stored)?;
        // The bytes reach `Secret` and nothing else, and this is the only place
        // in alo OS where a key exists outside the request that carries it. It
        // is not logged, not returned on the error path, and not kept.
        let said = std::str::from_utf8(&held).map_err(|_| NotStored::Missing)?;
        Secret::typed(said).map_err(|_| NotStored::Missing)
    }
}

/// What the service said, as one of the four — and **never carrying what it
/// said**.
///
/// A keyring's own message is not in anybody's language and may quote what it
/// was asked about. `alo_choosing::NotToml` is the same argument one file to the
/// left, and `41c9f1e` is the day it stopped being theoretical.
fn as_not_stored(why: secret_service::Error) -> NotStored {
    match why {
        // The prompt was dismissed, or the service would not have this caller.
        secret_service::Error::Prompt | secret_service::Error::Locked => NotStored::Locked,
        secret_service::Error::NoResult => NotStored::Missing,
        // Anything else is the bus or the service, which is one thing to a
        // person: there is nothing here to ask.
        _ => NotStored::Unavailable,
    }
}
