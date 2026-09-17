//! How the approving key reaches the turn that issues tokens, and nobody else.
//!
//! The broker makes a fresh key every time it starts, so nothing issued under a
//! previous one is genuine (`crate::spent`). The turn in `alo-agentd` must hold
//! the same key to issue tokens the broker believes, and the two are different
//! processes run by different users: the broker as root, the turn as the
//! person. So the broker writes the key's bytes into one file, and the turn
//! reads that file **each time it issues a token** — which is what makes a
//! restarted broker's new key the one used next, with nothing to reconnect.
//!
//! # Who can read it is the whole of the argument
//!
//! The file is `0440`, owned by root, in the broker's group — which the unit
//! makes the person's own group — inside a directory that is `0750` in the same
//! group. **The agent's own login is not in that group** (the person is in the
//! agent's, not the other way round: `image/usr/lib/sysusers.d/alo.conf`), so a
//! model driving the agent can neither read the key nor mint a token with it.
//! What can read it is any program of the person's own, which is the limit the
//! broker's first report states in full: law 2 binds the agent, never the
//! person, and the person changing their own network is ADR 0009.
//!
//! # And it is believed only in that shape
//!
//! [`the_key_handed_over`] refuses a key anybody but its owner could write, one
//! anybody outside its group could read, one that is not a plain file, one owned
//! by anybody but the user it is expected from, and one that is not exactly a
//! key long. A key somebody else planted is a key whose tokens somebody else
//! can issue, and a turn that used one would be proving approvals to whoever
//! planted it.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt as _, OpenOptionsExt as _, PermissionsExt as _};
use std::path::Path;

use crate::approving::{ApprovingKey, NoRandomness, fresh_bytes};

/// How many bytes a key is.
const KEY_BYTES: usize = 32;

/// How far a key is read: one byte past a key, so a longer file is seen to be
/// longer without reading all of it.
const LONGER_THAN_A_KEY: u64 = 33;

/// The mode the key is written with: its owner and its group may read it.
const THE_KEYS_MODE: u32 = 0o440;

/// The bits a key must not have: anybody but its owner writing, anybody
/// outside its group reading, and anybody executing.
const NEVER: u32 = 0o137;

/// Why no key was handed over, or none can be believed.
#[derive(Debug)]
pub enum NotHandedOver {
    /// There was no randomness to make a key from.
    NoRandomness(NoRandomness),
    /// The key could not be written, or read.
    Unreadable(std::io::Error),
    /// The key is not a plain file.
    NotAFile,
    /// Somebody other than its owner can write it, or somebody outside its group
    /// can read it.
    OpenToOthers {
        /// Its mode.
        mode: u32,
    },
    /// It is owned by somebody other than the user it is expected from.
    NotTheBrokers {
        /// Who owns it.
        owner: u32,
    },
    /// It is not exactly a key long.
    NotAKey,
}

impl std::fmt::Display for NotHandedOver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRandomness(why) => write!(f, "{why}"),
            Self::Unreadable(why) => {
                write!(
                    f,
                    "the broker's approving key could not be handed over: {why}"
                )
            }
            Self::NotAFile => write!(
                f,
                "the broker's approving key is not a plain file, so it is not believed"
            ),
            Self::OpenToOthers { mode } => write!(
                f,
                "the broker's approving key is mode {mode:o}, which somebody other than the \
                 broker could write or somebody outside its group could read, so it is not \
                 believed"
            ),
            Self::NotTheBrokers { owner } => write!(
                f,
                "the broker's approving key is owned by user {owner}, not by the broker, so it \
                 is not believed"
            ),
            Self::NotAKey => write!(
                f,
                "the broker's approving key is not {KEY_BYTES} bytes long, so it is not believed"
            ),
        }
    }
}

impl std::error::Error for NotHandedOver {}

/// Make a fresh key, write it at `at` for this group to read, and hold it.
///
/// Written beside `at` and renamed into place, so a turn reading the file never
/// reads half of a key, and a key that was not wholly written never replaces
/// the one before it.
///
/// # Errors
/// [`NotHandedOver`], and then the broker has no key and must not open its door.
pub fn hand_over_a_fresh_key(at: &Path, group: u32) -> Result<ApprovingKey, NotHandedOver> {
    let bytes = fresh_bytes().map_err(NotHandedOver::NoRandomness)?;
    let beside = at.with_extension("handing-over");
    drop(fs::remove_file(&beside));
    let written = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(&beside)
        .and_then(|mut file| {
            file.write_all(&bytes)?;
            file.sync_all()
        })
        .and_then(|()| fs::set_permissions(&beside, fs::Permissions::from_mode(THE_KEYS_MODE)))
        .and_then(|()| crate::unix::give_to_group(&beside, group))
        .and_then(|()| fs::rename(&beside, at));
    match written {
        Ok(()) => Ok(ApprovingKey::of(&bytes)),
        Err(why) => {
            drop(fs::remove_file(&beside));
            Err(NotHandedOver::Unreadable(why))
        }
    }
}

/// The key the broker handed over at `at`, believed only if the user `from`
/// wrote it and nobody else could have.
///
/// `from` is root on a machine; a test hands over and reads back as itself.
///
/// # Errors
/// [`NotHandedOver`], and then no token is issued.
pub fn the_key_handed_over(at: &Path, from: u32) -> Result<ApprovingKey, NotHandedOver> {
    let about = fs::symlink_metadata(at).map_err(NotHandedOver::Unreadable)?;
    if !about.file_type().is_file() {
        return Err(NotHandedOver::NotAFile);
    }
    let mode = about.permissions().mode() & 0o7777;
    if mode & NEVER != 0 || mode & 0o7000 != 0 {
        return Err(NotHandedOver::OpenToOthers { mode });
    }
    if about.uid() != from {
        return Err(NotHandedOver::NotTheBrokers { owner: about.uid() });
    }
    let mut read = Vec::with_capacity(KEY_BYTES + 1);
    fs::File::open(at)
        .and_then(|file| file.take(LONGER_THAN_A_KEY).read_to_end(&mut read))
        .map_err(NotHandedOver::Unreadable)?;
    let bytes: [u8; KEY_BYTES] = read.try_into().map_err(|_| NotHandedOver::NotAKey)?;
    Ok(ApprovingKey::of(&bytes))
}
