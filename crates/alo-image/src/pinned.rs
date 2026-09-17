//! The one published image an installer pulls, as `image/pinned.toml` pins it.
//!
//! [ADR 0033](../../../docs/decisions/0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)
//! §3 publishes alo OS to `ghcr.io/aloworld-org/alo-os` and pins, in this
//! repository, the digest an installer pulls.
//! [ADR 0036](../../../docs/decisions/0036-the-image-is-signed-by-a-key-a-person-holds.md)
//! says what makes a digest worth pinning: the owner signed it with the private
//! half of the key whose public half the pin names, and the signature verified.
//!
//! # Why one file, and why it is read here
//!
//! The digest is written in exactly one place so that nothing — the installer,
//! the boot environment, `docs/booting.md`, a Release's notes — can carry a
//! second spelling of it that drifts. This crate reads it because this crate is
//! already where the recipe and the document are held to each other, and the
//! pin is a sentence with its other half in the recipe: *release `0.0.2`* is
//! *these bytes*. `crate::publishing` holds the two halves together.
//!
//! # Read strictly, unlike the recipe
//!
//! `crate::version` and `crate::disk` read leniently because a recipe missing a
//! label is a recipe `crate::checking` has sentences about. A pin is different:
//! a digest that is not a digest, a version that moves or a key path that leaves
//! the image is not a pin with something wrong with it, it is not a pin, and an
//! installer handed one would pull whatever it could make of it. So
//! [`ThePin::read`] refuses those, and what is left for `crate::publishing` is
//! whether a well-formed pin agrees with everything else.

use std::path::{Component, Path};

use serde::Deserialize;

use crate::refusing::NotPinned;
use crate::version::is_a_release;

/// Where the pin is, beneath the image's directory.
pub const THE_PIN: &str = "pinned.toml";

/// The registry alo OS is published to (ADR 0033 §3).
pub const THE_REGISTRY: &str = "ghcr.io/aloworld-org/alo-os";

/// What a digest begins with. SHA-256 is the only algorithm a pin accepts,
/// because it is the only one a registry reports for this image.
const SHA256: &str = "sha256:";

/// How many hexadecimal characters a SHA-256 digest has.
const A_WHOLE_DIGEST: usize = 64;

/// How many hexadecimal characters a whole git commit name has.
const A_WHOLE_REVISION: usize = 40;

/// The published release an installer pulls, as the pin states it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThePin {
    /// The repository in the registry, with no tag and no digest on it.
    registry: String,
    /// The release that was signed, as `MAJOR.MINOR.PATCH`.
    version: String,
    /// The content address of that release, `sha256:` and 64 lowercase hex.
    digest: String,
    /// The commit the release was built from, in full.
    revision: String,
    /// The public half of the signing key, relative to the image's directory.
    key: String,
    /// The release the recipe is building towards, where one has been declared
    /// and not yet pinned.
    #[serde(default)]
    next: Option<String>,
}

impl ThePin {
    /// The pin this file states.
    ///
    /// # Errors
    ///
    /// [`NotPinned`] when the file is not TOML of exactly this shape, when the
    /// version or the next release is not `MAJOR.MINOR.PATCH`, when the digest is
    /// not a whole SHA-256, when the revision is not a whole commit name, or
    /// when the key is not a path inside the image's directory.
    pub fn read(pinned: &str) -> Result<Self, NotPinned> {
        let pin: Self = toml::from_str(pinned).map_err(|why| NotPinned::NotToml {
            why: why.message().to_owned(),
        })?;

        for release in std::iter::once(&pin.version).chain(pin.next.as_ref()) {
            if !is_a_release(release) {
                return Err(NotPinned::NotARelease {
                    stated: release.clone(),
                });
            }
        }
        if !is_a_digest(&pin.digest) {
            return Err(NotPinned::NotADigest {
                stated: pin.digest.clone(),
            });
        }
        if !is_lowercase_hex(&pin.revision, A_WHOLE_REVISION) {
            return Err(NotPinned::NotARevision {
                stated: pin.revision.clone(),
            });
        }
        if !stays_inside(&pin.key) {
            return Err(NotPinned::KeyOutsideTheImage {
                stated: pin.key.clone(),
            });
        }
        Ok(pin)
    }

    /// The repository in the registry this pin names.
    #[must_use]
    pub fn registry(&self) -> &str {
        &self.registry
    }

    /// The release that was signed.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The content address an installer pulls.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// The commit that release was built from.
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// The public half of the signing key, relative to the image's directory.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The release declared as being built next, where there is one.
    #[must_use]
    pub fn next(&self) -> Option<&str> {
        self.next.as_deref()
    }

    /// What an installer pulls: the registry, by digest.
    ///
    /// Never the tag. A tag is a name somebody can move, and the one thing a pin
    /// exists for is that what is pulled cannot be moved after it was signed.
    #[must_use]
    pub fn reference(&self) -> String {
        format!("{}@{}", self.registry, self.digest)
    }
}

/// Whether this is `sha256:` and a whole digest, in lowercase.
fn is_a_digest(stated: &str) -> bool {
    stated
        .strip_prefix(SHA256)
        .is_some_and(|hex| is_lowercase_hex(hex, A_WHOLE_DIGEST))
}

/// Whether this is exactly this many lowercase hexadecimal characters.
///
/// Lowercase only, because the OCI specification writes digests that way and a
/// registry compares them as strings.
fn is_lowercase_hex(stated: &str, length: usize) -> bool {
    stated.len() == length
        && stated
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Whether this path names something beneath the directory it is relative to.
fn stays_inside(key: &str) -> bool {
    let path = Path::new(key);
    !key.is_empty()
        && !key.starts_with('/')
        && !key.starts_with('\\')
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The pin this repository ships, as text.
    fn the_pin() -> String {
        std::fs::read_to_string(Path::new(crate::THE_IMAGE).join(THE_PIN)).unwrap()
    }

    /// A pin with one line replaced, from the one this repository ships.
    fn the_pin_with(from: &str, to: &str) -> String {
        let shipped = the_pin();
        assert!(shipped.contains(from), "the pin does not contain `{from}`");
        shipped.replace(from, to)
    }

    /// **The pin this repository ships reads**, and says what the owner
    /// signed on 2026-09-15 — and, since 2026-09-17, which release is being
    /// prepared.
    ///
    /// The two are deliberately separate facts. `version` and `digest` are what
    /// an installer pulls **today**, and they do not move because a newer
    /// release is being worked on; `next` is the candidate the recipe now
    /// names, and it becomes the pin only when a digest signed against the key
    /// below is written here. This reading is therefore what a release in
    /// This reads the pin between releases: what was signed is what the
    /// recipe names, and no candidate is declared.
    #[test]
    fn the_shipped_pin_names_the_release_the_owner_signed() {
        let pin = ThePin::read(&the_pin()).unwrap();

        assert_eq!(pin.registry(), THE_REGISTRY);
        assert_eq!(pin.version(), "0.0.2");
        assert_eq!(
            pin.reference(),
            "ghcr.io/aloworld-org/alo-os@sha256:8f9c36e0d608eb13d8ba7746b9c549438a939bcbd51e90e2b5fcd5103be90bf9"
        );
        assert_eq!(pin.revision(), "8d2619daeb07b4b0ebbed59ded8caee722103b65");
        assert_eq!(pin.key(), "signing/alo-os.pub");
        assert_eq!(pin.next(), None);
    }

    /// **A digest that is not a whole SHA-256 is not a pin**: a short one, an
    /// uppercase one, another algorithm, and a tag where the digest goes.
    #[test]
    fn a_digest_that_is_not_one_is_refused() {
        let digest = "sha256:8f9c36e0d608eb13d8ba7746b9c549438a939bcbd51e90e2b5fcd5103be90bf9";
        for instead in [
            "sha256:d3f05b60",
            "sha256:D3F05B60975EDCFF51A44C1F21E764A32B286677E306BA24631BAD6A00B6A13C",
            "sha512:8f9c36e0d608eb13d8ba7746b9c549438a939bcbd51e90e2b5fcd5103be90bf9",
            "8f9c36e0d608eb13d8ba7746b9c549438a939bcbd51e90e2b5fcd5103be90bf9",
            "latest",
            "",
        ] {
            let refused = ThePin::read(&the_pin_with(digest, instead)).unwrap_err();
            assert!(
                matches!(refused, NotPinned::NotADigest { .. }),
                "`{instead}`: {refused}"
            );
        }
    }

    /// **A version that moves is not a pin**, and neither is a next release
    /// that moves.
    #[test]
    fn a_release_that_moves_is_refused() {
        let refused =
            ThePin::read(&the_pin_with("version = \"0.0.2\"", "version = \"latest\"")).unwrap_err();
        assert!(
            matches!(refused, NotPinned::NotARelease { .. }),
            "{refused}"
        );

        // Between releases the shipped pin declares no candidate, so one is
        // added here rather than changed. While a release is in flight the pin
        // does carry `next`, and this fixture would have to replace it instead
        // — two `next` keys are refused by the parser before this rule is
        // reached, which would fail for the wrong reason.
        let refused = ThePin::read(&the_pin_with(
            "version = \"0.0.2\"",
            "version = \"0.0.2\"
next = \"dev\"",
        ))
        .unwrap_err();
        assert!(
            matches!(refused, NotPinned::NotARelease { .. }),
            "{refused}"
        );
    }

    /// **A revision that is not a whole commit is not a pin.** An abbreviated
    /// one is a name that can become ambiguous later.
    #[test]
    fn a_revision_that_is_not_a_whole_commit_is_refused() {
        let refused = ThePin::read(&the_pin_with(
            "revision = \"8d2619daeb07b4b0ebbed59ded8caee722103b65\"",
            "revision = \"2501af5\"",
        ))
        .unwrap_err();
        assert!(
            matches!(refused, NotPinned::NotARevision { .. }),
            "{refused}"
        );
    }

    /// **A key path that leaves the image is refused**, so a pin cannot point
    /// verification at a file somebody else placed.
    #[test]
    fn a_key_outside_the_image_is_refused() {
        for instead in [
            "../../somewhere.pub",
            "/etc/alo-os.pub",
            "",
            "signing/../x.pub",
        ] {
            let refused = ThePin::read(&the_pin_with(
                "key = \"signing/alo-os.pub\"",
                &format!("key = \"{instead}\""),
            ))
            .unwrap_err();
            assert!(
                matches!(refused, NotPinned::KeyOutsideTheImage { .. }),
                "`{instead}`: {refused}"
            );
        }
    }

    /// **A key nobody declared, or one missing, is not a pin.** A misspelt
    /// `next` read as absent would be a candidate nobody could see.
    #[test]
    fn a_pin_of_another_shape_is_refused() {
        for pinned in [
            the_pin_with(
                "key = \"signing/alo-os.pub\"",
                "key = \"signing/alo-os.pub\"\nnxet = \"0.0.2\"",
            ),
            the_pin_with("key = \"signing/alo-os.pub\"", ""),
            "not toml at all".to_owned(),
        ] {
            let refused = ThePin::read(&pinned).unwrap_err();
            assert!(matches!(refused, NotPinned::NotToml { .. }), "{refused}");
        }
    }

    /// **What is pulled is the digest, never the tag.**
    #[test]
    fn the_reference_is_by_digest() {
        let pin = ThePin::read(&the_pin()).unwrap();
        assert!(pin.reference().contains('@'));
        assert!(!pin.reference().contains(":0.0.2"));
    }
}
