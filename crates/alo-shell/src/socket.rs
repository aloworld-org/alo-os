//! Own a fresh private socket directory without replacing another session.

use std::{
    fs::{self, DirBuilder},
    io,
    os::unix::fs::{DirBuilderExt, MetadataExt},
    path::{Path, PathBuf},
};

use smithay::reexports::wayland_server::{BindError, ListeningSocket};

/// Why a private compositor socket could not be opened.
#[derive(Debug, thiserror::Error)]
pub enum SocketError {
    /// Runtime directory must be absolute, owned by this user and mode 0700.
    #[error("runtime directory must be a real, private directory owned by this user")]
    UnsafeRuntime,
    /// Session names contain only ASCII letters, digits, '-' and '_'.
    #[error("invalid compositor session name")]
    InvalidName,
    /// A session directory already exists; it is never removed or reused.
    #[error("compositor session name is already in use")]
    InUse,
    /// Filesystem or display creation failed.
    #[error("compositor socket I/O: {0}")]
    Io(#[from] io::Error),
    /// Wayland could not create the socket in the fresh private directory.
    #[error("Wayland socket: {0}")]
    Bind(#[from] BindError),
}

/// Drop the listener before removing only the directory we created.
pub(crate) struct Socket {
    /// Owned listener; taken before directory cleanup.
    pub(crate) listener: Option<ListeningSocket>,
    /// Fresh private directory, never an existing session's directory.
    directory: PathBuf,
    /// Absolute Wayland socket path suitable for a child's WAYLAND_DISPLAY.
    pub(crate) path: PathBuf,
}

impl Socket {
    /// Create a new named session inside a trusted runtime directory.
    pub(crate) fn bind(runtime: &Path, name: &str) -> Result<Self, SocketError> {
        if name.is_empty()
            || name.len() > 48
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(SocketError::InvalidName);
        }
        let metadata = fs::symlink_metadata(runtime)?;
        if !runtime.is_absolute()
            || !metadata.is_dir()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o777 != 0o700
        {
            return Err(SocketError::UnsafeRuntime);
        }
        let directory = runtime.join(name);
        DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .map_err(|error| {
                if error.kind() == io::ErrorKind::AlreadyExists {
                    SocketError::InUse
                } else {
                    SocketError::Io(error)
                }
            })?;
        let mut socket = Self {
            path: directory.join("wayland"),
            directory,
            listener: None,
        };
        socket.listener = Some(ListeningSocket::bind_absolute(socket.path.clone())?);
        Ok(socket)
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        if self.listener.is_none() {
            // A failed bind may have created its lock before the socket failed.
            let _ = fs::remove_file(self.path.with_extension("lock"));
        }
        drop(self.listener.take());
        // No recursive cleanup: unexpected contents belong to whoever put them there.
        let _ = fs::remove_dir(&self.directory);
    }
}
