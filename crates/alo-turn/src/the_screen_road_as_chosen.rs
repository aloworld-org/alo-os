//! **The person's two choices for the screen road**, read at the moment of each
//! picture ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md)):
//! which applications alo may use pictures and keys in, and who reads the
//! pictures.
//!
//! A trait, as `the_windows_on_the_screen` is, because both are kept in the
//! person's own file (`docs/contracts/how-code-runs-file.md`) and this crate
//! touches no disk: `alo-agentd` holds the one that reads it, **each time it is
//! asked**, so a switch turned off between two pictures stops the second.
//!
//! **A file that will not read is no application and no one**, never a guess at
//! what it meant — `alo_capability::PicturesAndKeys::as_read` and
//! `alo_capability::WhoReadsPictures::as_read` are that rule, and an
//! implementation answers through them.

use alo_capability::{PicturesAndKeys, WhoReadsPictures};

/// What the person chose for the screen road, as their file says now.
pub trait TheScreenRoadAsChosen {
    /// The applications alo may use pictures and keys in.
    fn pictures_and_keys(&self) -> PicturesAndKeys;

    /// Who reads the pictures alo takes.
    fn who_reads_pictures(&self) -> WhoReadsPictures;
}
