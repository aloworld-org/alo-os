//! Validate the opened public machine description before trusting its logins.

use std::{
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};

use crate::studio_agent_launch::{require_description, require_description_metadata};

/// Open without following a symlink or blocking on a FIFO, then check the file.
///
/// The private binary always supplies `/etc/alo/agentd.toml`; this path parameter
/// lets tests exercise real metadata and file types without changing that file.
/// No command-line or environment override selects the binary's description.
///
/// # Errors
/// Refuse untrusted metadata, nonregular files, invalid text or an invalid description.
pub fn read_description(path: &Path) -> Result<String, String> {
    let description = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(
            rustix::fs::OFlags::NOFOLLOW.bits() as i32 | rustix::fs::OFlags::NONBLOCK.bits() as i32,
        )
        .open(path)
        .map_err(|_| "the machine description could not be opened".to_owned())?;
    let metadata = description
        .metadata()
        .map_err(|_| "the machine description metadata could not be read".to_owned())?;
    require_description_metadata(
        metadata.uid(),
        metadata.mode(),
        metadata.is_file(),
        metadata.file_type().is_symlink(),
    )?;
    let mut text = String::new();
    description
        .take(65_537)
        .read_to_string(&mut text)
        .map_err(|_| "the machine description could not be read".to_owned())?;
    require_description(&text)?;
    Ok(text)
}
