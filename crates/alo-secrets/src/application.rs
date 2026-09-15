//! An application's own secrets, in the person's one keyring, behind the
//! Secret portal.
//!
//! `docs/features.md`, v0.5: *secret storage — one keyring behind the Secret
//! portal, so applications stop inventing credential storage.* The keyring is
//! [`TheKeyring`], the same one a provider's key is looked up in, on the same
//! bus and under the same lock. There is no second store here to fall back to
//! and no second connection to make.
//!
//! # A grant first, and the keyring only after
//!
//! [`TheKeyring::for_the_application`] takes the request, the machine's grants
//! and the moment, and the only way to an application's secrets is through it.
//! It refuses a request to any other portal before the grants are read, then
//! judges the request with `alo_portals::Request::judged` — `alo-capability`'s
//! answer, not one restated here — and only a request the grants allowed
//! returns an [`ItsOwn`].
//!
//! **An [`ItsOwn`] does one thing and is gone.** Every operation on it takes it
//! by value, so each thing an application does to its secrets was judged
//! against the grants as they were at that request. Revoking the grant is
//! therefore felt at the next request, and nothing can hold an allowance across
//! a revocation and go on using it.
//!
//! # Whose secret it is is never the caller's to say
//!
//! Everything an application keeps is filed under the identifier of the
//! application **its grant was judged for** — [`alo_portals::Allowed::application`]
//! — and every search is made with that identifier in it. No operation here
//! takes an application's name as an argument, so there is no argument through
//! which one application could name another's secrets.
//!
//! # A collection the application's name owns, under the person's one lock
//!
//! A Secret Service *collection* is also a unit of locking: each has its own
//! password and is unlocked on its own. A collection per application in that
//! sense would be a second lock for every application, which is exactly what
//! *where a person's other secrets are, under the same lock* rules out — and on
//! a real keyring making one puts a password prompt on the screen. So an
//! application's collection is the set of items filed under its identifier and
//! [`AN_APPLICATIONS`] schema **inside the person's default collection**, the
//! one their sign-in unlocks.
//!
//! # The daemon's keys and an application's are filed apart
//!
//! A provider's key is filed under `dev.alo.Provider`, an application's under
//! [`AN_APPLICATIONS`] and its portal secret under [`THE_PORTALS`]. A Secret
//! Service search matches every attribute it is given, and every search in this
//! crate gives the schema, so a provider lookup never matches an application's
//! item and no application's request ever matches a provider's key — by what
//! is asked, not by hoping nobody picks the same name.
//!
//! # What revoking does to what was kept
//!
//! It ends the application's reach, at its next request. **It does not delete
//! the passwords.** A folder is not deleted when a grant to it is revoked, and a
//! person who revokes by mistake and grants again should find the application's
//! sign-in where it was. Removing them is a person's act on their keyring.
//!
//! # What this does not stop
//!
//! ADR 0022's limitation stands: anything running as the person can speak to
//! their keyring, and could file an item under these attributes itself. What is
//! guaranteed is what *this* door hands out, and that an application reaching
//! the keyring through the portal reaches its own and nothing else.

use std::collections::HashMap;
use std::time::SystemTime;

use alo_capability::Grants;
use alo_portals::{Allowed, Portal, Request};
use secret_service::blocking::{Collection, Item};

use crate::kept_secret::KeptSecret;
use crate::refusing::NotStored;
use crate::store::{FILED_UNDER, TheKeyring, as_not_stored};
use crate::withheld::Withheld;

/// The schema an application's own secrets are filed under.
pub const AN_APPLICATIONS: &str = "dev.alo.Application";

/// The schema an application's one portal secret is filed under.
pub const THE_PORTALS: &str = "dev.alo.Application.PortalSecret";

/// The attribute holding the identifier of the application a secret is for.
const WHOSE: &str = "application";

/// The attribute holding the name an application kept a secret under.
const NAMED: &str = "name";

/// What the keyring is told a kept secret's bytes are.
const BYTES: &str = "application/octet-stream";

/// The most bytes a name for a secret may have.
pub const LONGEST_NAME: usize = 255;

/// The most bytes one kept secret may have: a password or a key, never a file.
pub const LARGEST_SECRET: usize = 64 * 1024;

/// How many random bytes an application's portal secret is.
///
/// The Secret portal hands an application one secret to derive its own
/// encryption from: 512 bits, beyond guessing and small enough for any keyring.
pub const PORTAL_SECRET_LENGTH: usize = 64;

impl TheKeyring {
    /// An application's own secrets, if the grants allow this request now.
    ///
    /// # Errors
    /// [`Withheld::NotTheSecretPortal`] for a request to any other portal,
    /// before the grants are read; and [`Withheld::Refused`], carrying the
    /// grants' own refusal, when nothing covers this request at `now` — an
    /// application granted nothing, granted something else, or whose grant
    /// expired or was revoked.
    pub fn for_the_application(
        &self,
        request: &Request,
        grants: &Grants,
        now: SystemTime,
    ) -> Result<ItsOwn<'_>, Withheld> {
        if request.portal() != Portal::Secret {
            return Err(Withheld::NotTheSecretPortal(request.portal()));
        }
        let allowed = request.judged(grants, now).map_err(Withheld::Refused)?;
        Ok(ItsOwn {
            keyring: self,
            allowed,
        })
    }
}

/// One allowed request to an application's own secrets.
///
/// Made only by [`TheKeyring::for_the_application`], and spent by the one
/// operation it is used for.
pub struct ItsOwn<'keyring> {
    /// The keyring, which is the machine's one.
    keyring: &'keyring TheKeyring,
    /// What the grants allowed, and whom.
    allowed: Allowed,
}

impl<'keyring> ItsOwn<'keyring> {
    /// What the grants allowed, and against which grant — what a record of
    /// this request names.
    #[must_use]
    pub const fn allowed(&self) -> &Allowed {
        &self.allowed
    }

    /// Keep `secret` under `named`, replacing whatever this application kept
    /// under that name before.
    ///
    /// # Errors
    /// [`Withheld::NotAName`], [`Withheld::TooLarge`], and
    /// [`Withheld::NotStored`] — [`NotStored::Locked`] when the person's
    /// keyring is locked, **which is never unlocked here**.
    pub fn keep(self, named: &str, secret: &[u8]) -> Result<(), Withheld> {
        let named = a_name(named)?;
        if secret.len() > LARGEST_SECRET {
            return Err(Withheld::TooLarge);
        }
        let collection = self.the_collection()?;
        let mut attributes = self.filed(AN_APPLICATIONS);
        attributes.insert(NAMED, named);
        let label = format!("{}/{named}", self.whose());
        collection
            .create_item(&label, attributes, secret, true, BYTES)
            .map_err(as_not_stored)?;
        Ok(())
    }

    /// What this application kept under `named`.
    ///
    /// # Errors
    /// [`Withheld::NotAName`], and [`Withheld::NotStored`] —
    /// [`NotStored::Missing`] when this application kept nothing under that
    /// name, **including when another application did**.
    pub fn kept(self, named: &str) -> Result<KeptSecret, Withheld> {
        let named = a_name(named)?;
        let collection = self.the_collection()?;
        let mut attributes = self.filed(AN_APPLICATIONS);
        attributes.insert(NAMED, named);
        let found = collection.search_items(attributes).map_err(as_not_stored)?;
        let item = earliest(found)?.ok_or(NotStored::Missing)?;
        Ok(KeptSecret::holding(
            item.get_secret().map_err(as_not_stored)?,
        ))
    }

    /// Remove what this application kept under `named`.
    ///
    /// # Errors
    /// [`Withheld::NotAName`], and [`Withheld::NotStored`] —
    /// [`NotStored::Missing`] when there was nothing of this application's
    /// under that name to remove.
    pub fn forget(self, named: &str) -> Result<(), Withheld> {
        let named = a_name(named)?;
        let collection = self.the_collection()?;
        let mut attributes = self.filed(AN_APPLICATIONS);
        attributes.insert(NAMED, named);
        let found = collection.search_items(attributes).map_err(as_not_stored)?;
        if found.is_empty() {
            return Err(Withheld::NotStored(NotStored::Missing));
        }
        for item in found {
            item.delete().map_err(as_not_stored)?;
        }
        Ok(())
    }

    /// The application's one portal secret — what `RetrieveSecret` on
    /// `org.freedesktop.portal.Secret` answers — made the first time it is
    /// asked for, and the same bytes every time after.
    ///
    /// Two first requests at once can both make one. Both then read back the
    /// one the keyring holds as the earliest, so both are answered with the
    /// same bytes rather than each with its own.
    ///
    /// # Errors
    /// [`Withheld::NoRandomness`] when the kernel will not give random bytes,
    /// and [`Withheld::NotStored`] from the keyring.
    pub fn the_portals_secret(self) -> Result<KeptSecret, Withheld> {
        let collection = self.the_collection()?;
        let attributes = self.filed(THE_PORTALS);
        let found = collection
            .search_items(attributes.clone())
            .map_err(as_not_stored)?;
        if let Some(item) = earliest(found)? {
            return Ok(KeptSecret::holding(
                item.get_secret().map_err(as_not_stored)?,
            ));
        }

        let mut fresh = vec![0_u8; PORTAL_SECRET_LENGTH];
        getrandom::fill(&mut fresh).map_err(|_| Withheld::NoRandomness)?;
        collection
            .create_item(self.whose(), attributes.clone(), &fresh, false, BYTES)
            .map_err(as_not_stored)?;

        let found = collection.search_items(attributes).map_err(as_not_stored)?;
        let item = earliest(found)?.ok_or(NotStored::Missing)?;
        Ok(KeptSecret::holding(
            item.get_secret().map_err(as_not_stored)?,
        ))
    }

    /// The identifier of the application the grants allowed.
    fn whose(&self) -> &str {
        self.allowed.application().as_str()
    }

    /// The attributes every item of this application's under `schema` has.
    fn filed(&self, schema: &'static str) -> HashMap<&'static str, &str> {
        let mut attributes = HashMap::new();
        attributes.insert(FILED_UNDER, schema);
        attributes.insert(WHOSE, self.whose());
        attributes
    }

    /// The person's default collection, open — or why not.
    ///
    /// A locked collection is [`NotStored::Locked`] and stays locked:
    /// unlocking is the person answering a prompt, not a portal deciding for
    /// them.
    fn the_collection(&self) -> Result<Collection<'keyring>, Withheld> {
        let collection = self
            .keyring
            .service()
            .get_default_collection()
            .map_err(|why| match as_not_stored(why) {
                // No default collection is nowhere to keep anything.
                NotStored::Missing => NotStored::Unavailable,
                other => other,
            })?;
        if collection.is_locked().map_err(as_not_stored)? {
            return Err(Withheld::NotStored(NotStored::Locked));
        }
        Ok(collection)
    }
}

/// The item made first, when a search found more than one — so that whoever
/// asks is answered with the same one.
fn earliest(found: Vec<Item<'_>>) -> Result<Option<Item<'_>>, NotStored> {
    let mut dated = Vec::with_capacity(found.len());
    for item in found {
        let created = item.get_created().map_err(as_not_stored)?;
        dated.push((created, item.item_path.as_str().to_owned(), item));
    }
    dated.sort_by(|one, other| (one.0, &one.1).cmp(&(other.0, &other.1)));
    Ok(dated.into_iter().next().map(|(_, _, item)| item))
}

/// A name a secret may be kept under, or why not.
fn a_name(named: &str) -> Result<&str, Withheld> {
    if named.is_empty() || named.len() > LONGEST_NAME || named.chars().any(char::is_control) {
        return Err(Withheld::NotAName);
    }
    Ok(named)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A name is checked before the keyring is asked anything.**
    #[test]
    fn a_name_that_is_not_one_is_refused() {
        assert_eq!(a_name(""), Err(Withheld::NotAName));
        assert_eq!(a_name("token\n"), Err(Withheld::NotAName));
        assert_eq!(a_name("a\u{0}b"), Err(Withheld::NotAName));
        assert_eq!(
            a_name(&"n".repeat(LONGEST_NAME + 1)),
            Err(Withheld::NotAName)
        );
        assert_eq!(a_name("matrix access token"), Ok("matrix access token"));
        let longest = "n".repeat(LONGEST_NAME);
        assert_eq!(a_name(&longest), Ok(longest.as_str()));
    }

    /// **The three schemas are three**, so a search for one never matches the
    /// others — the separation between the daemon's keys and an application's
    /// is in what is asked.
    #[test]
    fn the_daemons_keys_and_an_applications_are_filed_apart() {
        let provider = crate::store::PROVIDERS;
        assert_ne!(AN_APPLICATIONS, provider);
        assert_ne!(THE_PORTALS, provider);
        assert_ne!(AN_APPLICATIONS, THE_PORTALS);
    }
}
