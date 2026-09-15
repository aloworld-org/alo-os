//! Everything the person's half asks of the daemon, as one door.
//!
//! [`crate::Changing`] revokes a row of the one list with one call whichever
//! kind of row it is, so it holds one door that can do both things a row
//! needs: knock after a grant's file is written ([`Knocking`]) and ask for a
//! pairing to be revoked ([`RevokingPairings`]). Two traits rather than one
//! wider one, because each is its own conversation with its own answers; this
//! file only says that a door is both, so nothing has to be passed twice.

use crate::knocking::Knocking;
use crate::unpairing::RevokingPairings;

/// A door the person's half can both knock on and ask to revoke a pairing.
///
/// Implemented for everything that is both — [`crate::TheDaemonsDoor`], and a
/// test's own door — so there is nothing to implement here.
pub trait Door: Knocking + RevokingPairings {}

impl<T: Knocking + RevokingPairings> Door for T {}
