//! Whether the download is a genuine alo OS, before anything is written.
//!
//! [ADR 0023](../../../docs/decisions/0023-installed-from-the-machine-it-replaces.md)
//! §3: *verifying signatures before writing.*
//! [ADR 0036](../../../docs/decisions/0036-the-image-is-signed-by-a-key-a-person-holds.md):
//! the owner signs **by digest**, with a key whose public half ships inside
//! the installer, and without a transparency log.
//!
//! # Two layers, and both are checked here
//!
//! The digest is pinned in the environment itself, so what is asked about is
//! one set of bytes that nobody can move after the owner signed them. The
//! signature is then checked against the pinned key, and the checker's own
//! answer is read back: it must name **that digest** and nothing else. A
//! checker that exited successfully while describing some other manifest is not
//! a pass.
//!
//! The writer then pulls by the same digest. A registry that served other bytes
//! the second time would be serving bytes with a different address, which the
//! pull refuses on its own — so there is no window between the check and the
//! write for a different release to arrive through.
//!
//! # Not reached is not the same as not genuine
//!
//! Both write nothing. They are said differently because they ask different
//! things of the person: a cable, or nothing at all. The checker's complaint is
//! read only to tell those two apart, and anything this cannot recognise as the
//! network failing is **not genuine** — the one of the two that offers no way
//! past it.

use std::path::{Path, PathBuf};

use alo_image::ThePin;
use serde::Deserialize;

use crate::program::Ran;

/// What the checker prints when the network is what failed.
///
/// Go's own words for a dial, a lookup, a timeout and a broken connection,
/// lower-cased. Matched only on a failed run.
const THE_NETWORK_FAILED: [&str; 9] = [
    "dial tcp",
    "no such host",
    "i/o timeout",
    "network is unreachable",
    "connection refused",
    "connection reset",
    "tls handshake timeout",
    "server misbehaving",
    "temporary failure in name resolution",
];

/// The check, with everything it is given already checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verifying {
    /// The public half of the owner's key, on the environment.
    key: PathBuf,
    /// The release, by digest.
    reference: String,
    /// The digest alone, which the answer must name.
    digest: String,
}

/// What the check found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verified {
    /// Signed by the pinned key, for the pinned digest.
    Genuine,
    /// Reached, and not shown to be the owner's.
    NotGenuine,
    /// Not reached.
    NotReachable,
}

/// One claim the checker verified, as much of it as is read.
#[derive(Debug, Deserialize)]
struct Claim {
    /// The part of the signed payload that names what was signed.
    critical: Critical,
}

/// What was signed.
#[derive(Debug, Deserialize)]
struct Critical {
    /// The image the signature is over.
    image: Signed,
}

/// The image a signature is over.
#[derive(Debug, Deserialize)]
struct Signed {
    /// Its digest.
    #[serde(rename = "docker-manifest-digest")]
    digest: String,
}

impl Verifying {
    /// The check of the pinned release against the key at this path.
    #[must_use]
    pub fn of(pin: &ThePin, key: &Path) -> Self {
        Self {
            key: key.to_path_buf(),
            reference: pin.reference(),
            digest: pin.digest().to_owned(),
        }
    }

    /// The checker's arguments.
    ///
    /// `--insecure-ignore-tlog=true` is the flag for *there is no log entry to
    /// check*, which is true of every alo OS signature by decision (ADR 0036);
    /// the key is still checked, and this is the invocation `docs/booting.md`
    /// gives and the owner ran on 2026-09-15.
    #[must_use]
    pub fn arguments(&self) -> Vec<String> {
        vec![
            "verify".to_owned(),
            "--key".to_owned(),
            self.key.display().to_string(),
            "--insecure-ignore-tlog=true".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
            self.reference.clone(),
        ]
    }

    /// What the checker's run says.
    #[must_use]
    pub fn answer(&self, ran: &Ran) -> Verified {
        if !ran.succeeded {
            let complained = ran.complained.to_lowercase();
            return if THE_NETWORK_FAILED
                .iter()
                .any(|failed| complained.contains(failed))
            {
                Verified::NotReachable
            } else {
                Verified::NotGenuine
            };
        }
        match serde_json::from_str::<Vec<Claim>>(ran.printed.trim()) {
            Ok(claims)
                if !claims.is_empty()
                    && claims
                        .iter()
                        .all(|claim| claim.critical.image.digest == self.digest) =>
            {
                Verified::Genuine
            }
            _ => Verified::NotGenuine,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The pin this repository ships.
    fn the_pin() -> ThePin {
        ThePin::read(
            &std::fs::read_to_string(Path::new(alo_image::THE_IMAGE).join(alo_image::THE_PIN))
                .unwrap(),
        )
        .unwrap()
    }

    /// The check of the shipped pin.
    fn the_check() -> Verifying {
        Verifying::of(
            &the_pin(),
            Path::new("/usr/lib/alo/installing/signing/alo-os.pub"),
        )
    }

    /// What the checker printed inside the environment on 2026-09-15, for the
    /// owner's release and the owner's key.
    fn what_it_printed(digest: &str) -> String {
        format!(
            r#"[{{"critical":{{"identity":{{"docker-reference":"ghcr.io/aloworld-org/alo-os@{digest}"}},"image":{{"docker-manifest-digest":"{digest}"}},"type":"https://sigstore.dev/cosign/sign/v1"}},"optional":{{}}}}]"#
        )
    }

    /// The check asks about the pinned digest, with the pinned key, and no log.
    #[test]
    fn the_check_asks_about_the_pinned_digest_with_the_pinned_key() {
        let pin = the_pin();
        assert_eq!(
            the_check().arguments(),
            [
                "verify",
                "--key",
                "/usr/lib/alo/installing/signing/alo-os.pub",
                "--insecure-ignore-tlog=true",
                "--output",
                "json",
                &format!("ghcr.io/aloworld-org/alo-os@{}", pin.digest()),
            ]
        );
    }

    /// The owner's signature over the pinned digest is genuine.
    #[test]
    fn the_owners_signature_over_the_pinned_digest_is_genuine() {
        let ran = Ran {
            succeeded: true,
            printed: what_it_printed(the_pin().digest()),
            complained: String::new(),
        };
        assert_eq!(the_check().answer(&ran), Verified::Genuine);
    }

    /// **A signature the key does not match is not genuine.** The checker's own
    /// words on 2026-09-15, for a different key.
    #[test]
    fn a_signature_by_another_key_is_not_genuine() {
        let ran = Ran {
            succeeded: false,
            printed: String::new(),
            complained: "Error: no matching signatures: error verifying bundle: failed to verify \
                         signature\nmain.go:74: error during command execution: no matching \
                         signatures"
                .to_owned(),
        };
        assert_eq!(the_check().answer(&ran), Verified::NotGenuine);
    }

    /// **A success that names another digest is not genuine**, and neither is a
    /// success that names nothing.
    #[test]
    fn a_success_about_something_else_is_not_genuine() {
        let other = format!("sha256:{}", "0".repeat(64));
        for printed in [
            what_it_printed(&other),
            String::new(),
            "[]".to_owned(),
            "Verification for ghcr.io/aloworld-org/alo-os".to_owned(),
            format!(
                "[{}, {}]",
                what_it_printed(the_pin().digest()).trim_matches(['[', ']']),
                what_it_printed(&other).trim_matches(['[', ']'])
            ),
        ] {
            let ran = Ran {
                succeeded: true,
                printed: printed.clone(),
                complained: String::new(),
            };
            assert_eq!(the_check().answer(&ran), Verified::NotGenuine, "{printed}");
        }
    }

    /// **The network failing is not reached, not not genuine** — the checker's
    /// words inside the environment with no name server, 2026-09-15.
    #[test]
    fn the_network_failing_is_not_reached() {
        let ran = Ran {
            succeeded: false,
            printed: String::new(),
            complained:
                "Error: Get \"https://ghcr.io/v2/\": dial tcp: lookup ghcr.io on [::1]:53: \
                         read udp [::1]:38334->[::1]:53: read: connection refused"
                    .to_owned(),
        };
        assert_eq!(the_check().answer(&ran), Verified::NotReachable);
    }

    /// A failure the network words do not describe is not genuine, and a
    /// success is never read for the network's words.
    #[test]
    fn anything_else_that_fails_is_not_genuine() {
        let ran = Ran {
            succeeded: false,
            printed: String::new(),
            complained: "Error: GET https://ghcr.io/v2/aloworld-org/alo-os/manifests/sha256:...: \
                         MANIFEST_UNKNOWN"
                .to_owned(),
        };
        assert_eq!(the_check().answer(&ran), Verified::NotGenuine);

        let ran = Ran {
            succeeded: true,
            printed: String::new(),
            complained: "dial tcp".to_owned(),
        };
        assert_eq!(the_check().answer(&ran), Verified::NotGenuine);
    }
}
