//! What a grant and a revocation each come to, when they happen at all.
//!
//! Two enums rather than one, because the two doors have different nothings.
//! A pick can come back empty — the person closed the picker — and *nothing
//! picked* must grant nothing, write nothing and knock nobody. A revocation
//! can land on a row the machine has moved past, and *already gone* must
//! change nothing, write nothing and knock nobody — the file already says
//! what the person wanted said. Each nothing is its own case so a caller
//! cannot read it as the change having happened.
//!
//! The cases that did happen each carry a [`Stood`], because by then the
//! change is on the disk and the one open question is the running daemon.

use alo_capability::GrantId;

use crate::stood::Stood;

/// What making a grant through the person's half came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Made {
    /// The picker was closed without picking. Nothing was granted, the file
    /// was not touched, and nobody was knocked — there was no change to tell
    /// anybody about.
    Nothing,
    /// The grant is on the disk, under the handle a list shows and a person
    /// revokes it by.
    Granted {
        /// The handle the machine holds the grant under.
        id: GrantId,
        /// Where the change stands with the running daemon.
        stood: Stood,
    },
}

/// What revoking through the person's half came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gone {
    /// The row was stale: the grant had expired or was already revoked.
    /// Nothing changed, the file was not touched, and nobody was knocked —
    /// `alo_granted`'s own sentence for a stale row is what a surface shows.
    AlreadyGone,
    /// The grant is out of the file.
    Revoked {
        /// Where the change stands with the running daemon.
        stood: Stood,
    },
}
