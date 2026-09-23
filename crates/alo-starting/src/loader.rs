//! The loader's own two files, as whoever changes the default reaches them —
//! and the only two things alo OS ever asks of them.
//!
//! [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
//! term 3 puts the last choice in the loader's own saved default and nowhere
//! else; that file is root's, and
//! [ADR 0066](../../../docs/decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md)
//! decided that a person in Settings reaches it through a verb on the broker's
//! list. This trait is what the side holding the privilege is handed, and it is
//! deliberately three methods over **bytes**: everything that knows what those
//! bytes mean is [`crate::EnvironmentBlock`], on this side of the door, so the
//! privileged side neither parses nor invents a file.
//!
//! # What is not on it
//!
//! There is **no method that writes the menu**, adds an entry to it or removes
//! one. The menu is generated whole by [`crate::Menu`] when a machine is
//! installed, and a road that could rewrite it from a verb would be a road to
//! *which programs this computer can start at all* — which is not what a person
//! in Settings asked for. What is here reads it, and only to answer *does this
//! machine offer Windows*.

use crate::systems::System;

/// One of the loader's files could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct NotRead(pub String);

/// The loader's saved default could not be written.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct NotWritten(pub String);

/// The loader's files on the machine alo OS is running on.
pub trait TheLoader {
    /// Which systems the menu on this machine offers.
    ///
    /// alo OS is always one of them, because a machine reading this is running
    /// it. Windows is one only on a machine installed beside a Windows.
    ///
    /// # Errors
    /// [`NotRead`] when the menu could not be read at all — which is not the
    /// same answer as *there is no Windows here* and is never given as one.
    fn offering(&self) -> Result<Vec<System>, NotRead>;

    /// The bytes of the file the last choice is kept in, exactly as they are.
    ///
    /// # Errors
    /// [`NotRead`] when the file could not be read.
    fn saved(&self) -> Result<Vec<u8>, NotRead>;

    /// Put these bytes back, as the whole of that file.
    ///
    /// What is handed over is always a block written at the length it was read
    /// at ([`crate::EnvironmentBlock::written`]), because the loader writes
    /// that file in place and a file of another size is one it can no longer
    /// save into.
    ///
    /// # Errors
    /// [`NotWritten`] when the file would not be written. Nothing partial is
    /// ever left: an implementation that cannot write the whole of it writes
    /// none of it.
    fn save(&self, bytes: &[u8]) -> Result<(), NotWritten>;
}
