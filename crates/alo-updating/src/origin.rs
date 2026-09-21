//! The build a deployment was made from, read out of the file the base writes
//! beside it.
//!
//! Every deployment on the disk has an `.origin` file next to its folder,
//! `0644 root root`, naming where that deployment came from. On an alo OS
//! machine that is a container image with its digest on it, which is exactly
//! what *which build is this machine running* means here:
//!
//! ```text
//! [origin]
//! container-image-reference=ostree-unverified-registry:ghcr.io/aloworld-org/alo-os@sha256:48bd…
//! ```
//!
//! # Read for one thing, and leniently about the rest
//!
//! The base owns this file and may put anything else in it. What is read is one
//! key and the digest on the end of its value; a line this does not recognise
//! is a line this does not read, and a key the base adds tomorrow changes
//! nothing here. That is `crate::status`'s rule about the base's status
//! document, applied to the base's other answer.
//!
//! **The transport in front of the reference is deliberately not checked.**
//! `ostree-unverified-registry:`, `ostree-remote-image:`, `ostree-image-signed:`
//! — which of those a machine carries is the base's business and the signature
//! policy's, and nothing about *which build is running* changes with it. What
//! this refuses is a reference with **no digest at all**, because a deployment
//! named by a moving tag is one no offer can be compared against.

use alo_keeping_up::Digest;

/// What the base calls the file, beside the deployment's own folder: the
/// folder's whole name with this on the end of it.
pub const BESIDE_IT: &str = ".origin";

/// The key naming where a deployment came from.
const THE_REFERENCE: &str = "container-image-reference";

/// What separates an image's name from the build it is pinned to.
const BY_ITS_CONTENT: char = '@';

/// Why the build a deployment was made from could not be read out of its
/// origin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotInTheOrigin {
    /// The file names no image: this deployment was not made from one.
    NamesNoImage,
    /// It names an image and not a build of one — a moving tag, which no offer
    /// can be compared against.
    NamesNoBuild {
        /// The reference it named.
        reference: String,
    },
    /// It names a build whose digest is not one.
    NotADigest {
        /// What was wrong with it.
        why: String,
    },
}

/// The build this origin says its deployment was made from.
///
/// # Errors
/// [`NotInTheOrigin`].
pub(crate) fn the_build(text: &str) -> Result<Digest, NotInTheOrigin> {
    let reference = reference_in(text).ok_or(NotInTheOrigin::NamesNoImage)?;
    let (_, digest) =
        reference
            .rsplit_once(BY_ITS_CONTENT)
            .ok_or_else(|| NotInTheOrigin::NamesNoBuild {
                reference: reference.to_owned(),
            })?;
    Digest::read(digest).map_err(|why| NotInTheOrigin::NotADigest {
        why: format!("{why:?}"),
    })
}

/// The value of the one key this reads, if the file carries it.
fn reference_in(text: &str) -> Option<&str> {
    text.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == THE_REFERENCE).then(|| value.trim())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The digest the pinned release is published under, whole.
    const THE_BUILD: &str =
        "sha256:48bd5f319abcecfa832eb9a5b0b2f7cd06815b1c30c43b781499500ec14c3858";

    /// The origin file of a real alo OS machine, read as uid 1000 on
    /// `alo-lane-b-bootc` and copied here byte for byte but for the digest.
    fn a_real_origin() -> String {
        format!(
            "[origin]\ncontainer-image-reference=ostree-unverified-registry:\
             ghcr.io/aloworld-org/alo-os@{THE_BUILD}\n"
        )
    }

    /// **The build is read out of the file the base actually writes.**
    #[test]
    fn the_build_is_read_out_of_the_file_the_base_writes() {
        assert_eq!(
            the_build(&a_real_origin()).map(|digest| digest.as_str().to_owned()),
            Ok(THE_BUILD.to_owned())
        );
    }

    /// **Which transport is in front of the reference changes nothing.** Which
    /// one a machine carries is the base's business and the signature policy's,
    /// and this answers the same build for each.
    #[test]
    fn the_transport_in_front_of_the_reference_changes_nothing() {
        for transport in [
            "ostree-unverified-registry:",
            "ostree-remote-image:fedora:registry:",
            "ostree-image-signed:registry:",
            "",
        ] {
            let text = format!(
                "[origin]\ncontainer-image-reference={transport}\
                 ghcr.io/aloworld-org/alo-os@{THE_BUILD}\n"
            );
            assert_eq!(
                the_build(&text).map(|digest| digest.as_str().to_owned()),
                Ok(THE_BUILD.to_owned()),
                "`{transport}` was not read"
            );
        }
    }

    /// **A deployment made from no image at all is refused**, which is what an
    /// ostree machine installed some other way reads like.
    #[test]
    fn a_deployment_made_from_no_image_is_refused() {
        let text = "[origin]\nrefspec=fedora:fedora/42/x86_64/silverblue\n";

        assert_eq!(the_build(text), Err(NotInTheOrigin::NamesNoImage));
    }

    /// **A moving tag is not a build.** An offer is a difference between two
    /// digests, and there is nothing to take the difference from.
    #[test]
    fn a_reference_with_no_digest_on_it_is_not_a_build() {
        let text = "[origin]\ncontainer-image-reference=ostree-unverified-registry:\
             ghcr.io/aloworld-org/alo-os:0.0.4\n";

        assert!(matches!(
            the_build(text),
            Err(NotInTheOrigin::NamesNoBuild { .. })
        ));
    }

    /// **Half a digest is refused rather than read**, exactly as the base's
    /// status answer is: this is the same value arriving by a second road and
    /// it goes through the same check.
    #[test]
    fn half_a_digest_is_refused_rather_than_read() {
        let text = "[origin]\ncontainer-image-reference=ghcr.io/x@sha256:beef\n";

        assert!(matches!(
            the_build(text),
            Err(NotInTheOrigin::NotADigest { .. })
        ));
    }

    /// **The key is read wherever it is in the file**, and a key that merely
    /// contains its name is not it.
    #[test]
    fn the_key_is_read_wherever_it_is_and_a_key_that_looks_like_it_is_not_it() {
        let text = format!(
            "[origin]\nunconfigured-state=\nkargs=rw\n\
             container-image-reference={}\n",
            format_args!("ghcr.io/aloworld-org/alo-os@{THE_BUILD}")
        );
        assert!(the_build(&text).is_ok());

        let looks_like_it =
            format!("[origin]\nold-container-image-reference=ghcr.io/x@{THE_BUILD}\n");
        assert_eq!(the_build(&looks_like_it), Err(NotInTheOrigin::NamesNoImage));
    }
}
