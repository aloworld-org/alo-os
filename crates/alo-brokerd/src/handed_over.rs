//! What a person hands the broker, believed only as a plain file of their own.
//!
//! ADR 0049 §3: the broker's door takes no text, so a person's choice is a file
//! in `/run/alo-broker/wanted` — a folder this process makes `0770` in the
//! person's group — and the request carries the digest of exactly those bytes.
//! Two files are handed over that way now, the proxy and the password it signs
//! in with (ADR 0060 §3), and this is the reading both of them are believed
//! through.
//!
//! **It is one file because the checks are the thing.** A second copy of *open
//! it without following a link, look at what was actually opened, refuse
//! anything that is not a plain file of the person's, and never read more than
//! it should be* is a second place for one of those four to be left out.
//!
//! Each of them is a way somebody who is not the person could otherwise have
//! this process read something for them: a link pointing at a file of root's, a
//! file the agent's own login left in a folder it shares a group with, or
//! something enormous that a privileged process reads into memory.

use std::fs::File;
use std::io::Read as _;
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

use alo_broker::NotCarried;

/// The bytes of a file a person handed over, or [`None`] where they handed over
/// nothing.
///
/// `longest` is the most this file may be, which each caller knows for its own
/// kind of thing. Nothing longer is read: the length is asked of the descriptor
/// that was opened, and the read is capped again over the same number, so a
/// file that grows between the two still cannot be read past it.
///
/// # Errors
/// [`NotCarried`], saying which of the four it was, and nothing is read.
pub fn bytes(at: &Path, person: u32, longest: u64, what: &str) -> Result<Vec<u8>, NotCarried> {
    let refused = |why: &str| NotCarried(format!("the {what} handed over {why}"));
    let opened = rustix::fs::open(
        at,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::NONBLOCK
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map_err(|why| refused(&format!("could not be opened: {why}")))?;
    let file = File::from(opened);
    let about = file
        .metadata()
        .map_err(|why| refused(&format!("could not be looked at: {why}")))?;
    if !about.file_type().is_file() {
        return Err(refused("is not a plain file"));
    }
    if about.uid() != person {
        return Err(refused(&format!(
            "is owned by user {}, not by the person",
            about.uid()
        )));
    }
    if about.len() > longest {
        return Err(refused("is longer than it can be"));
    }
    let mut bytes = Vec::new();
    file.take(longest)
        .read_to_end(&mut bytes)
        .map_err(|why| refused(&format!("could not be read: {why}")))?;
    Ok(bytes)
}

/// Whether nothing at all was handed over here.
///
/// Told apart from *something was handed over and it is wrong*: a proxy set
/// without a password is an ordinary thing a person does, and a password file
/// that is there and unreadable is not.
/// A link left where a file should be is **something**, not nothing: it is
/// refused by the reading above rather than read as an empty hand-over.
#[must_use]
pub fn nothing_is_there(at: &Path) -> bool {
    matches!(at.symlink_metadata(), Err(why) if why.kind() == std::io::ErrorKind::NotFound)
}
