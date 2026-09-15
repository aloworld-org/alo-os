//! The keyring, as the portal backend on the bus reaches it.
//!
//! `alo_portals::serving::Backend` answers `org.freedesktop.portal.Secret` and
//! cannot name this crate, which already depends on `alo-portals`; it reaches the
//! keyring through `alo_portals::KeepsSecrets`, implemented here by
//! [`TheKeyring`] and by nothing else.
//!
//! **Nothing new is decided.** A request is judged by
//! [`TheKeyring::for_the_application`], and the secret handed over is
//! `ItsOwn::the_portals_secret` — task 3's door, used as it is. What this adds is
//! only where the bytes go: into what the application handed over the bus,
//! straight from the [`crate::KeptSecret`] that holds them.

use std::io::Write;
use std::time::SystemTime;

use alo_capability::Grants;
use alo_portals::{Allowed, KeepsSecrets, NotKept, Request};

use crate::store::TheKeyring;
use crate::withheld::Withheld;

impl KeepsSecrets for TheKeyring {
    fn hand_over(
        &self,
        request: &Request,
        grants: &Grants,
        now: SystemTime,
        into: &mut dyn Write,
    ) -> Result<Allowed, NotKept> {
        let its_own = self
            .for_the_application(request, grants, now)
            .map_err(as_not_kept)?;
        let allowed = its_own.allowed().clone();
        let secret = its_own.the_portals_secret().map_err(as_not_kept)?;
        into.write_all(secret.bytes())
            .and_then(|()| into.flush())
            .map_err(|_| NotKept::NotWritten)?;
        Ok(allowed)
    }
}

/// What the backend is told when the keyring withheld the secret.
///
/// The grants' refusal travels whole, so the record names it; everything else
/// the keyring can say — locked, missing, denied, no randomness — is one thing
/// to an application on the bus: no secret, and nothing written.
/// `NotTheSecretPortal` cannot arrive from the backend, which asks only with
/// `Portal::Secret`, and is the same *no secret* if it ever did.
fn as_not_kept(withheld: Withheld) -> NotKept {
    match withheld {
        Withheld::Refused(refused) => NotKept::Refused(refused),
        Withheld::NotTheSecretPortal(_)
        | Withheld::NotAName
        | Withheld::TooLarge
        | Withheld::NoRandomness
        | Withheld::NotStored(_) => NotKept::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refusing::NotStored;
    use alo_capability::Applicant;
    use alo_portals::{Portal, Refused};

    /// **The grants' refusal reaches the backend whole**, and nothing the
    /// keyring says is mistaken for one.
    #[test]
    fn only_the_grants_refusal_is_a_refusal() {
        let refused = Refused::NothingGranted {
            application: Applicant::named("org.example.Stranger"),
            portal: Portal::Secret,
        };
        assert_eq!(
            as_not_kept(Withheld::Refused(refused.clone())),
            NotKept::Refused(refused)
        );
        for other in [
            Withheld::NotTheSecretPortal(Portal::Camera),
            Withheld::NotAName,
            Withheld::TooLarge,
            Withheld::NoRandomness,
            Withheld::NotStored(NotStored::Locked),
            Withheld::NotStored(NotStored::Denied),
        ] {
            assert_eq!(as_not_kept(other), NotKept::Unavailable);
        }
    }
}
