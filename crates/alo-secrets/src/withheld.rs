//! Why an application was not given what it asked the keyring for.
//!
//! Six ways, in the order they are reached. **Which portal** comes first,
//! before the grants are looked at: a request to any other portal has no
//! business with the keyring. **The grants' answer** comes next — `alo-portals`'
//! own refusal, carried whole, so that a request refused here and one refused
//! anywhere else read the same. Only a request the grants allowed ever reaches
//! the name, the size, the generator or the store.
//!
//! **No `Display`, and no words yet**, for [`NotStored`]'s reason one file over:
//! the sentences a person reads about a refused portal request belong beside
//! the backend that says them and writes them into the record (task 5 of the
//! applications plan), and [`Withheld::Refused`] already carries the words the
//! grants refuse with. Nothing here carries a secret, or what the store said.

use alo_portals::{Portal, Refused};

use crate::refusing::NotStored;

/// Why an application's request to its own secrets was not answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Withheld {
    /// The request was to a portal that is not the Secret portal, so neither
    /// the grants nor the keyring were asked anything.
    NotTheSecretPortal(Portal),

    /// The grants refused it: nothing was granted to this application, or not
    /// this, or the grant has expired or been revoked. The keyring was not
    /// opened for it.
    Refused(Refused),

    /// The name a secret was to be kept or found under is empty, too long, or
    /// has a control character in it.
    NotAName,

    /// The secret is larger than [`crate::LARGEST_SECRET`]: a keyring keeps
    /// passwords and keys, not files.
    TooLarge,

    /// The kernel would not give this process random bytes for an
    /// application's portal secret, so none was made — and none was made up.
    NoRandomness,

    /// The keyring itself would not: it is unavailable, locked, has nothing
    /// under that name, or refused this caller.
    NotStored(NotStored),
}

impl From<NotStored> for Withheld {
    fn from(why: NotStored) -> Self {
        Self::NotStored(why)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The store's four states arrive as themselves, told apart.
    #[test]
    fn the_stores_refusals_arrive_as_themselves() {
        for why in [
            NotStored::Unavailable,
            NotStored::Locked,
            NotStored::Missing,
            NotStored::Denied,
        ] {
            assert_eq!(Withheld::from(why), Withheld::NotStored(why));
        }
    }
}
