//! Why the boundary was not imposed on this machine.
//!
//! Two of these are about how the loader was *started* and the third is
//! everything `alo-bounding` can refuse. That division is the whole shape of
//! this crate: it makes no decision about what to load, so the only mistakes
//! that are its own are mistakes in the unit file that runs it.
//!
//! # These keep their English, and that is a decision rather than an omission
//!
//! `CLAUDE.md` calls hardcoded English a bug, and the 9-series moved every
//! sentence in this workspace onto `alo-strings`. This type is in the same
//! category as `alo-bounding`'s [`NotBounded`] and `alo-capability`'s
//! `VerbError`: it is read by whoever is standing a machine up, in a service
//! log, before anybody has signed in. Nobody using alo OS ever sees one — a
//! person whose machine has no boundary reads `alo-turn`'s sentence about their
//! agent not running, in their own language, and this is what the log says
//! underneath it.

use alo_bounding::NotBounded;

/// Why this machine has no boundary.
#[derive(Debug, thiserror::Error)]
pub enum NotLoaded {
    /// The loader is not running as root.
    ///
    /// Loading a BPF LSM programme needs `CAP_BPF` and `CAP_SYS_ADMIN`, and
    /// ADR 0018 gives them to this and to nothing else. Refused here rather than
    /// left to the verifier, because what comes back from the kernel is a
    /// permission error on a syscall and what is actually wrong is a unit file.
    #[error(
        "alo-boundaryd is running as {uid} and imposes the boundary as root: it is the one \
         privileged component alo OS has (ADR 0018), and its unit file is what says so"
    )]
    NotAsRoot {
        /// Who it is running as.
        uid: u32,
    },

    /// Its own group is root's, so the map would be given to nobody.
    ///
    /// The pinned map of turns is handed to this process's own group, and that
    /// is how `alo-agentd` — which holds no capability — is let in to write it.
    /// A loader running in root's group would pin a map only root could write,
    /// which is a machine whose boundary is loaded and whose daemon can never
    /// bound a turn. It is a wrong `Group=` line, and it says so.
    #[error(
        "alo-boundaryd is running in root's group, so the map of turns would be pinned where no \
         agent daemon can write it: its unit file names the agent's group in `Group=`"
    )]
    TheRootsGroup,

    /// The kernel, the filesystem, or a boundary that is already there.
    ///
    /// Everything `alo-bounding` refuses, carried whole. Nothing is left pinned
    /// on any of these roads: `alo_bounding::Imposed::once` takes its own pins
    /// away, and the two this crate makes before it are removed here.
    #[error(transparent)]
    NotImposed(#[from] NotBounded),
}
