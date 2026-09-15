//! The name of one build of the operating system.
//!
//! The image this machine boots is a container image (ADR 0011), and the only
//! name for one that cannot drift is its content digest: a tag can be moved to
//! another build, a digest cannot. So *what is running* and *what is offered*
//! are both a [`Digest`], and whether an update exists is whether two of them
//! differ.
//!
//! **Checked when it is made, and read back the same way.** A digest arrives in
//! an answer from somewhere else, and half of one is not half a name: it is a
//! name that stopped saying which build it is. The same rule `alo-image` holds
//! its pins to — whole, `sha256`, lowercase hexadecimal — is held here, and a
//! digest read back off a disk goes through the same door.
//!
//! **Never shown to a person.** A person is told that an update is ready, not
//! which hash it has (`docs/features.md`: *a person never learns the name of
//! anything we rented*), so there is no `Display` here and no word for a
//! digest. [`NotADigest::said`] is what a person reads when an answer could not
//! be understood, and it names no hash either.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Deserializer, Serialize};

use crate::words;

/// The one algorithm a digest here may name.
const THE_ALGORITHM: &str = "sha256:";

/// How many hexadecimal characters a sha256 digest has.
const A_WHOLE_DIGEST: usize = 64;

/// One build of the operating system, named by its content.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct Digest(String);

/// Why some text is not a digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotADigest {
    /// It does not begin `sha256:`. Another algorithm is not refused because it
    /// is weaker but because nothing this machine pins uses it, and an answer
    /// naming one is an answer about some other image.
    NotSha256,
    /// The hexadecimal part is not sixty-four characters long.
    NotWhole {
        /// How many characters it had.
        length: usize,
    },
    /// A character that is not a lowercase hexadecimal digit.
    NotLowercaseHex,
}

impl Digest {
    /// Some text, read as a digest.
    ///
    /// # Errors
    /// [`NotADigest`] naming the first thing wrong with it. Nothing is trimmed
    /// or lowered on the way in: a digest with a stray space or a capital in it
    /// did not come from anything that names images, and guessing what it meant
    /// is how a machine ends up comparing against the wrong build.
    pub fn read(text: &str) -> Result<Self, NotADigest> {
        let Some(hex) = text.strip_prefix(THE_ALGORITHM) else {
            return Err(NotADigest::NotSha256);
        };
        if hex.len() != A_WHOLE_DIGEST {
            return Err(NotADigest::NotWhole { length: hex.len() });
        }
        if !hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
            return Err(NotADigest::NotLowercaseHex);
        }
        Ok(Self(text.to_owned()))
    }

    /// The digest as it is written, `sha256:` and all.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::read(&text)
            .map_err(|why| serde::de::Error::custom(format!("{text:?} is not a digest: {why:?}")))
    }
}

impl NotADigest {
    /// What a person reads when the answer about updates named no build this
    /// machine can recognise.
    ///
    /// One sentence for every reason, because the difference between them is
    /// for whoever runs the place updates come from, and what the person needs
    /// to know is the same each time: nothing on their machine changed.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&words::ANSWER_NOT_UNDERSTOOD.key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// A whole digest, from one repeated pair.
    fn whole(pair: &str) -> String {
        format!("sha256:{}", pair.repeat(32))
    }

    /// A whole, lowercase sha256 digest is a digest, and is written back
    /// exactly as it was read.
    #[test]
    fn a_whole_digest_is_read_and_written_back_unchanged() {
        let digest = Digest::read(&whole("ab")).unwrap();
        assert_eq!(digest.as_str(), whole("ab"));
    }

    /// **Half a digest, another algorithm and a capital are each refused**, and
    /// each for its own reason.
    #[test]
    fn text_that_is_not_a_whole_sha256_digest_is_refused() {
        assert_eq!(Digest::read(&"ab".repeat(32)), Err(NotADigest::NotSha256));
        assert_eq!(
            Digest::read(&format!("sha512:{}", "ab".repeat(64))),
            Err(NotADigest::NotSha256)
        );
        assert_eq!(
            Digest::read(&format!("sha256:{}", "ab".repeat(16))),
            Err(NotADigest::NotWhole { length: 32 })
        );
        assert_eq!(
            Digest::read("sha256:"),
            Err(NotADigest::NotWhole { length: 0 })
        );
        assert_eq!(Digest::read(&whole("AB")), Err(NotADigest::NotLowercaseHex));
        assert_eq!(Digest::read(&whole("zz")), Err(NotADigest::NotLowercaseHex));
    }

    /// **Nothing is trimmed on the way in.** A stray space is not forgiven,
    /// because what sent it was not something that names images.
    #[test]
    fn a_digest_with_space_around_it_is_refused_rather_than_trimmed() {
        assert!(Digest::read(&format!(" {}", whole("ab"))).is_err());
        assert!(Digest::read(&format!("{}\n", whole("ab"))).is_err());
    }

    /// **Read back through the same door.** A digest survives being written
    /// down, and a bad one written down by something else does not become one
    /// by being deserialised.
    #[test]
    fn a_digest_read_back_is_checked_the_way_a_new_one_is() {
        let digest = Digest::read(&whole("0f")).unwrap();
        let written = serde_json::to_string(&digest).unwrap();
        assert_eq!(serde_json::from_str::<Digest>(&written).unwrap(), digest);
        assert!(serde_json::from_str::<Digest>("\"sha256:beef\"").is_err());
        assert!(serde_json::from_str::<Digest>("\"latest\"").is_err());
    }

    /// **A person reads that nothing changed, and never a hash.**
    #[test]
    fn an_answer_that_was_not_a_digest_is_said_without_naming_one() {
        let said = NotADigest::NotLowercaseHex.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("nothing on this machine"), "{said}");
        assert!(!said.text().contains("sha256"), "{said}");
    }
}
