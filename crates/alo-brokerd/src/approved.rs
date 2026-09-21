//! What a person approved about an update, as bytes the two sides of the door
//! both digest.
//!
//! The broker's door takes no text (`docs/contracts/agent-verbs.md`), and an
//! update is two builds. So an update crosses the way a proxy does
//! ([ADR 0049](../../../docs/decisions/0049-the-network-is-changed-through-the-broker-and-its-password-never-reaches-the-agent.md)
//! §3): the side that asks writes the two builds into a file the person owns,
//! and the verb's argument is the SHA-256 of exactly those bytes.
//! [ADR 0053](../../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)
//! §B is the decision, and `docs/contracts/machine-update-file.md` is the
//! agreement both sides are held to.
//!
//! | verb | what is handed over | what the identity is the digest of |
//! |---|---|---|
//! | `updates.apply` | [`AnUpdate`], at `/run/alo-broker/wanted/update.json` | those bytes |
//! | `updates.roll-back` | nothing at all | [`GoingBackApproved`]'s bytes, which the unit writes again for itself |
//!
//! # Going back hands nothing over, and that is the point
//!
//! Going back has no argument the machine did not already know: the build
//! before, the build running, and the update that would be set aside are all
//! in the base's own status and the machine's record. So nothing is handed
//! over and nothing could be swapped — the unit decides
//! `alo_keeping_up::GoingBack` again for itself and refuses unless what it
//! decided digests to the identity the person approved.
//!
//! # Rebuilt identically, or not read at all
//!
//! [`AnUpdate::read`] refuses bytes that do not come back exactly as they were
//! written, which is `alo_networks::proxy_file::rechecked`'s rule for the same
//! reason: a file that can be spelt two ways is a file whose digest stands for
//! less than it appears to.

use alo_broker::Identity;
use alo_keeping_up::{Digest, GoingBack};
use serde::{Deserialize, Serialize};

/// The most bytes a handed-over update may be.
///
/// Two digests, two field names and the punctuation between them is under two
/// hundred; this leaves room for nothing anybody needs and refuses anything a
/// privileged process should not be reading into memory.
pub const LONGEST_HANDED_OVER: u64 = 1024;

/// Why some bytes are not an update a person approved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAnUpdate {
    /// They are not this file at all: not JSON, not these two fields, or a
    /// build named by something that is not a whole digest.
    NotThisFile(String),
    /// They are this file written some other way, and a file that can be spelt
    /// twice is not one a digest can stand for.
    NotWrittenAsThisMachineWritesIt,
}

impl std::fmt::Display for NotAnUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotThisFile(why) => write!(
                f,
                "the update handed over is not a pair of builds this machine wrote: {why}"
            ),
            Self::NotWrittenAsThisMachineWritesIt => write!(
                f,
                "the update handed over is written differently from the way this machine writes \
                 one, so it was not read"
            ),
        }
    }
}

/// The two builds an approved update is between.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnUpdate {
    /// The build the machine was running when the person was told.
    from: Digest,
    /// The build they approved changing to.
    to: Digest,
}

/// Going back, as the person approved it — never handed over, always written
/// again by whoever needs to know what its identity is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoingBackApproved {
    /// The build being left.
    from: Digest,
    /// The build returned to.
    to: Digest,
    /// The update waiting for the next restart that going back sets aside.
    sets_aside: Option<Digest>,
}

impl AnUpdate {
    /// The update between these two builds.
    #[must_use]
    pub const fn between(from: Digest, to: Digest) -> Self {
        Self { from, to }
    }

    /// The bytes a person hands over, and what their digest is taken of.
    #[must_use]
    pub fn written(&self) -> Vec<u8> {
        written(self)
    }

    /// The identity `updates.apply` is approved under for this update.
    #[must_use]
    pub fn identity(&self) -> Identity {
        Identity::of_what_was_reported(&self.written())
    }

    /// Some bytes, read as an update — and only if they are exactly the bytes
    /// this machine would have written for it.
    ///
    /// # Errors
    /// [`NotAnUpdate`].
    pub fn read(bytes: &[u8]) -> Result<Self, NotAnUpdate> {
        let read: Self = serde_json::from_slice(bytes)
            .map_err(|why| NotAnUpdate::NotThisFile(why.to_string()))?;
        if read.written() == bytes {
            Ok(read)
        } else {
            Err(NotAnUpdate::NotWrittenAsThisMachineWritesIt)
        }
    }

    /// The build the machine was running when the person was told.
    #[must_use]
    pub const fn from(&self) -> &Digest {
        &self.from
    }

    /// The build they approved changing to.
    #[must_use]
    pub const fn to(&self) -> &Digest {
        &self.to
    }
}

impl GoingBackApproved {
    /// What a person approves when they are offered going back.
    #[must_use]
    pub fn offered(offer: &GoingBack) -> Self {
        Self {
            from: offer.from().clone(),
            to: offer.to().clone(),
            sets_aside: offer.sets_aside().cloned(),
        }
    }

    /// The bytes its identity is the digest of. Nothing is ever handed over:
    /// both sides write these for themselves.
    #[must_use]
    pub fn written(&self) -> Vec<u8> {
        written(self)
    }

    /// The identity `updates.roll-back` is approved under for this offer.
    #[must_use]
    pub fn identity(&self) -> Identity {
        Identity::of_what_was_reported(&self.written())
    }
}

/// One shape, written one way: the fields in the order they are declared, and
/// a newline, so a file a person can look at ends like every other.
fn written<T: Serialize>(what: &T) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(what).unwrap_or_default();
    bytes.push(b'\n');
    bytes
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A whole digest made of one repeated pair.
    fn whole(pair: &str) -> Digest {
        Digest::read(&format!("sha256:{}", pair.repeat(32))).unwrap()
    }

    /// **An update reads back from exactly the bytes it writes**, and its
    /// identity is the digest of those bytes.
    #[test]
    fn an_update_reads_back_from_the_bytes_it_writes() {
        let update = AnUpdate::between(whole("aa"), whole("bb"));
        let bytes = update.written();
        assert_eq!(AnUpdate::read(&bytes), Ok(update.clone()));
        assert_eq!(
            update.identity(),
            Identity::of_what_was_reported(&bytes),
            "the identity is not the digest of the bytes handed over"
        );
        assert!(bytes.ends_with(b"\n"));
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            format!(
                "{{\"from\":\"{}\",\"to\":\"{}\"}}\n",
                whole("aa").as_str(),
                whole("bb").as_str()
            )
        );
    }

    /// **Bytes that are not this file are refused**, and so are this file's
    /// two builds written as anything but whole digests.
    #[test]
    fn bytes_that_are_not_a_pair_of_whole_builds_are_refused() {
        for (what, bytes) in [
            ("nothing at all", b"".as_slice()),
            ("not json", b"switch me".as_slice()),
            ("no builds", b"{}".as_slice()),
            (
                "half a digest",
                br#"{"from":"sha256:beef","to":"sha256:beef"}"#.as_slice(),
            ),
            (
                "a field nobody wrote",
                br#"{"from":"sha256:aa","to":"sha256:bb","apply":true}"#.as_slice(),
            ),
        ] {
            assert!(
                matches!(AnUpdate::read(bytes), Err(NotAnUpdate::NotThisFile(_))),
                "{what} was read as an update"
            );
        }
    }

    /// **The same two builds written another way is not the same file**, so a
    /// digest stands for one spelling and not for a family of them.
    #[test]
    fn the_same_builds_spelt_another_way_are_not_read() {
        let spelt_out = format!(
            "{{ \"from\": \"{}\", \"to\": \"{}\" }}\n",
            whole("aa").as_str(),
            whole("bb").as_str()
        );
        assert_eq!(
            AnUpdate::read(spelt_out.as_bytes()),
            Err(NotAnUpdate::NotWrittenAsThisMachineWritesIt)
        );
        let reordered = format!(
            "{{\"to\":\"{}\",\"from\":\"{}\"}}\n",
            whole("bb").as_str(),
            whole("aa").as_str()
        );
        assert_eq!(
            AnUpdate::read(reordered.as_bytes()),
            Err(NotAnUpdate::NotWrittenAsThisMachineWritesIt)
        );
    }

    /// **Two different updates are two different identities**, including the
    /// same pair the wrong way round.
    #[test]
    fn no_two_updates_share_an_identity() {
        let forwards = AnUpdate::between(whole("aa"), whole("bb"));
        let backwards = AnUpdate::between(whole("bb"), whole("aa"));
        let elsewhere = AnUpdate::between(whole("aa"), whole("cc"));
        assert_ne!(forwards.identity(), backwards.identity());
        assert_ne!(forwards.identity(), elsewhere.identity());
    }

    /// **An offer to go back that sets an update aside is a different identity
    /// from the same offer that does not** — because it is a different thing
    /// to approve.
    #[test]
    fn going_back_that_sets_an_update_aside_is_approved_as_its_own_thing() {
        let plain = GoingBackApproved {
            from: whole("bb"),
            to: whole("aa"),
            sets_aside: None,
        };
        let aside = GoingBackApproved {
            sets_aside: Some(whole("cc")),
            ..plain.clone()
        };
        assert_ne!(plain.identity(), aside.identity());
        assert_eq!(
            String::from_utf8(plain.written()).unwrap(),
            format!(
                "{{\"from\":\"{}\",\"to\":\"{}\",\"sets_aside\":null}}\n",
                whole("bb").as_str(),
                whole("aa").as_str()
            )
        );
    }
}
