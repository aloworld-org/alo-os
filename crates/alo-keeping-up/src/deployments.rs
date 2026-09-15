//! What the base says this machine is running, has waiting, and could go back
//! to.
//!
//! The base keeps up to three builds of the system on the disk: the one booted,
//! one staged to boot next, and the one before (ADR 0011 — this is the rented
//! half, and nothing here changes how it keeps them). [`Deployments`] is its
//! answer to *which are they*, read from the status it reports, so that
//! **what am I running** is answered by the machine rather than remembered by
//! anything of ours.
//!
//! # Read in the base's own shape, and only the three digests out of it
//!
//! The base answers with a document about a host, and nearly all of it is the
//! base's business: which transport, which ostree checksum, which boot order.
//! What this machine decides anything by is three digests, so those are the
//! only fields read; everything else in the answer is ignored rather than
//! mirrored, and a field the base adds tomorrow changes nothing here.
//!
//! **A digest in the answer goes through [`Digest`]'s own check.** An answer
//! naming half a hash is refused whole, rather than read as a machine with
//! nothing booted.
//!
//! # A booted build with no digest is a machine this cannot update
//!
//! A deployment that was not made from a container image — installed some
//! other way, or by a base older than image digests — has no name here.
//! [`Deployments::running`] refuses it with [`NotRunningABuild`] rather than
//! inventing one, because an update is a difference between two digests and
//! there is nothing to take the difference from.

use alo_strings::{Filling, Said, Strings};
use serde::Deserialize;

use crate::digest::Digest;
use crate::standing::Running;
use crate::words;

/// The builds this machine has on its disk, as the base reports them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deployments {
    /// The build the machine booted, when it was made from an image.
    booted: Option<Digest>,
    /// The build waiting to boot next, when there is one.
    staged: Option<Digest>,
    /// The build before, when there is one.
    rollback: Option<Digest>,
}

/// The machine reports no build it booted from an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotRunningABuild;

impl Deployments {
    /// The three builds, as they were reported.
    ///
    /// For whatever reads the base's answer in some other shape, and for tests.
    #[must_use]
    pub fn reported(
        booted: Option<Digest>,
        staged: Option<Digest>,
        rollback: Option<Digest>,
    ) -> Self {
        Self {
            booted,
            staged,
            rollback,
        }
    }

    /// The build this machine is running.
    ///
    /// # Errors
    /// [`NotRunningABuild`] when the booted deployment has no image digest.
    pub fn running(&self) -> Result<Running, NotRunningABuild> {
        self.booted
            .clone()
            .map(Running::reported)
            .ok_or(NotRunningABuild)
    }

    /// The build waiting for the next restart, if any.
    #[must_use]
    pub fn staged(&self) -> Option<&Digest> {
        self.staged.as_ref()
    }

    /// The build before the one running, if the disk still has it.
    #[must_use]
    pub fn rollback(&self) -> Option<&Digest> {
        self.rollback.as_ref()
    }
}

impl NotRunningABuild {
    /// What a person reads: that which version of its system this machine
    /// runs could not be read, and so nothing was changed.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&words::RUNNING_NOT_KNOWN.key(), &Filling::nothing())
    }
}

/// The base's status document, as far as this machine reads it.
///
/// `bootc status --format json`, format version 1: `status.booted`,
/// `status.staged` and `status.rollback`, each either `null` or an entry whose
/// `image.imageDigest` names its build.
#[derive(Deserialize)]
struct Host {
    /// The half of the document about what is on the disk.
    status: Status,
}

/// `status` in the base's document.
#[derive(Deserialize)]
struct Status {
    /// The deployment booted.
    #[serde(default)]
    booted: Option<Entry>,
    /// The deployment staged for the next boot.
    #[serde(default)]
    staged: Option<Entry>,
    /// The deployment before.
    #[serde(default)]
    rollback: Option<Entry>,
}

/// One deployment in the base's document.
#[derive(Deserialize)]
struct Entry {
    /// The image it was made from, absent when it was not made from one.
    #[serde(default)]
    image: Option<Image>,
}

/// `image` on one deployment.
#[derive(Deserialize)]
struct Image {
    /// The build, by its content.
    #[serde(rename = "imageDigest")]
    image_digest: Digest,
}

impl Entry {
    /// The digest this entry names, if it names one.
    fn digest(self) -> Option<Digest> {
        self.image.map(|image| image.image_digest)
    }
}

impl<'de> Deserialize<'de> for Deployments {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let host = Host::deserialize(deserializer)?;
        Ok(Self {
            booted: host.status.booted.and_then(Entry::digest),
            staged: host.status.staged.and_then(Entry::digest),
            rollback: host.status.rollback.and_then(Entry::digest),
        })
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

    fn whole(pair: &str) -> String {
        format!("sha256:{}", pair.repeat(32))
    }

    /// What `bootc status --format json` answers on a machine with a build
    /// booted and one waiting, trimmed of nothing the base would send.
    fn an_answer(booted: &str, staged: &str, rollback: &str) -> String {
        format!(
            r#"{{"apiVersion":"org.containers.bootc/v1","kind":"BootcHost","metadata":{{"name":"host"}},
            "spec":{{"image":{{"image":"ghcr.io/aloworld-org/alo-os","transport":"registry"}},"bootOrder":"default"}},
            "status":{{
              "staged":{staged},
              "booted":{booted},
              "rollback":{rollback},
              "rollbackQueued":false,"type":"bootcHost"}}}}"#
        )
    }

    fn entry(digest: &str) -> String {
        format!(
            r#"{{"image":{{"image":{{"image":"ghcr.io/aloworld-org/alo-os","transport":"registry"}},
            "version":"0.0.1","timestamp":null,"imageDigest":"{digest}","architecture":"amd64"}},
            "cachedUpdate":null,"incompatible":false,"pinned":false,"store":"ostreeContainer",
            "ostree":{{"checksum":"abc","deploySerial":0,"stateroot":"default"}}}}"#
        )
    }

    /// The three builds are read out of the base's answer, and the rest of it
    /// is left alone.
    #[test]
    fn the_three_builds_are_read_from_the_bases_answer() {
        let answer = an_answer(&entry(&whole("aa")), &entry(&whole("bb")), "null");
        let deployments: Deployments = serde_json::from_str(&answer).unwrap();
        assert_eq!(
            deployments.running().unwrap().digest().as_str(),
            whole("aa")
        );
        assert_eq!(deployments.staged().unwrap().as_str(), whole("bb"));
        assert_eq!(deployments.rollback(), None);
    }

    /// **A booted deployment with no image is refused, not given a name.**
    #[test]
    fn a_machine_booted_from_no_image_is_not_running_a_build_this_can_name() {
        let answer = an_answer(r#"{"image":null}"#, "null", "null");
        let deployments: Deployments = serde_json::from_str(&answer).unwrap();
        assert_eq!(deployments.running(), Err(NotRunningABuild));
        let nothing_booted: Deployments =
            serde_json::from_str(&an_answer("null", "null", "null")).unwrap();
        assert_eq!(nothing_booted.running(), Err(NotRunningABuild));
    }

    /// **Half a digest refuses the whole answer**, rather than reading as a
    /// machine with nothing booted.
    #[test]
    fn an_answer_naming_half_a_digest_is_refused_whole() {
        let answer = an_answer(&entry("sha256:beef"), "null", "null");
        assert!(serde_json::from_str::<Deployments>(&answer).is_err());
        assert!(serde_json::from_str::<Deployments>("{}").is_err());
        assert!(serde_json::from_str::<Deployments>("not json").is_err());
    }

    /// **A person reads that it could not be read**, and never a hash.
    #[test]
    fn not_running_a_build_is_said_without_the_machinery() {
        let said = NotRunningABuild.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("could not be read"), "{said}");
    }
}
