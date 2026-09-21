//! What the base says about this machine at the moment an update is carried
//! out: which builds are on the disk, and which repository the booted one came
//! from.
//!
//! Both come out of **one** answer to `bootc status`, because they are two
//! facts about one moment. Two calls would be two moments, and an update
//! decided against builds read at one of them and a repository read at the
//! other is an update decided against a machine that never existed.
//!
//! # Where updates come from is the machine's, never the request's
//!
//! [ADR 0053](../../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md):
//! *the source is the repository the booted build came from, read from the
//! base's status. It is never handed over.* So there is no argument anywhere on
//! this road by which somebody could name a registry — not in the verb, whose
//! argument is thirty-two bytes, and not in the file handed over, which holds
//! two digests and nothing else.
//!
//! # A reference names a build; a source may not
//!
//! The base reports the booted deployment's image as this machine last wrote
//! it, which for alo OS is a repository **with a build on it**, by digest
//! (`alo_keeping_up::Source::at`). `alo_keeping_up::Source` refuses to hold
//! one, deliberately: the build is named by the offer and by nothing else. So
//! the build is cut off here, and what is left goes through that crate's own
//! check rather than a second one.
//!
//! **Cut, not repaired.** The one cut is at the `@` or the tag's `:` in the
//! last segment — the two places a registry puts a build. Anything else the
//! base reports is refused rather than tidied into something that would parse.

use alo_keeping_up::{Deployments, NotASource, Source};
use alo_updating::{Base, NotAnswered, THE_STATUS};
use serde::Deserialize;

/// What the base said about this machine, read once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheMachineNow {
    /// The builds on the disk.
    deployments: Deployments,
    /// The repository the booted build came from.
    source: Source,
}

/// Why this machine could not be read.
#[derive(Debug)]
pub enum NotRead {
    /// The base did not answer at all.
    TheBaseDidNotAnswer(NotAnswered),
    /// It answered with something that is not its status.
    NotItsStatus(String),
    /// Its status names no build this machine booted from a registry.
    NotFromARegistry(String),
    /// The repository the booted build came from is not one an update can be
    /// fetched from.
    NotARepository(NotASource),
}

impl std::fmt::Display for NotRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TheBaseDidNotAnswer(why) => write!(f, "the base did not answer: {why:?}"),
            Self::NotItsStatus(why) => {
                write!(f, "what the base answered is not its status: {why}")
            }
            Self::NotFromARegistry(why) => write!(
                f,
                "the base reports no build this machine booted from a registry, so there is \
                 nowhere for an update to come from: {why}"
            ),
            Self::NotARepository(why) => write!(
                f,
                "the place the booted build came from is not one an update can be fetched from: \
                 {why:?}"
            ),
        }
    }
}

impl std::error::Error for NotRead {}

impl TheMachineNow {
    /// Ask the base, once, and read both facts out of the one answer.
    ///
    /// # Errors
    /// [`NotRead`], and nothing is decided about a machine this cannot read.
    pub fn read(base: &impl Base) -> Result<Self, NotRead> {
        let arguments: Vec<String> = THE_STATUS.iter().map(|&word| word.to_owned()).collect();
        let answer = base
            .asked(&arguments)
            .map_err(NotRead::TheBaseDidNotAnswer)?;
        let deployments: Deployments = serde_json::from_slice(&answer)
            .map_err(|why| NotRead::NotItsStatus(why.to_string()))?;
        let booted: Status = serde_json::from_slice(&answer)
            .map_err(|why| NotRead::NotItsStatus(why.to_string()))?;
        let image = booted
            .status
            .booted
            .and_then(|entry| entry.image)
            .map(|image| image.image)
            .ok_or_else(|| {
                NotRead::NotFromARegistry("its booted deployment names no image".to_owned())
            })?;
        if image.transport != A_REGISTRY {
            return Err(NotRead::NotFromARegistry(format!(
                "its booted deployment came over {} rather than from a registry",
                image.transport
            )));
        }
        let source =
            Source::named(the_repository_in(&image.image)).map_err(NotRead::NotARepository)?;
        Ok(Self {
            deployments,
            source,
        })
    }

    /// The two read together, for a test that has no base to ask.
    #[must_use]
    pub const fn of(deployments: Deployments, source: Source) -> Self {
        Self {
            deployments,
            source,
        }
    }

    /// The builds on the disk.
    #[must_use]
    pub const fn deployments(&self) -> &Deployments {
        &self.deployments
    }

    /// The repository the booted build came from.
    #[must_use]
    pub const fn source(&self) -> &Source {
        &self.source
    }
}

/// The transport a build an update can follow came over.
const A_REGISTRY: &str = "registry";

/// The repository part of an image reference: everything before the build a
/// registry names it by.
fn the_repository_in(reference: &str) -> &str {
    let repository = reference.split('@').next().unwrap_or(reference);
    let after_the_last_slash = repository.rfind('/').map_or(0, |at| at + 1);
    match repository[after_the_last_slash..].find(':') {
        Some(at) => &repository[..after_the_last_slash + at],
        None => repository,
    }
}

/// The half of the base's status document this file reads, which is the half
/// `alo_keeping_up::Deployments` deliberately does not.
#[derive(Deserialize)]
struct Status {
    /// The `status` object.
    status: TheDeployments,
}

/// The deployments in it, of which only the booted one is read here.
#[derive(Deserialize)]
struct TheDeployments {
    /// The deployment booted.
    #[serde(default)]
    booted: Option<Entry>,
}

/// One deployment.
#[derive(Deserialize)]
struct Entry {
    /// The image it was made from.
    #[serde(default)]
    image: Option<Outer>,
}

/// The `image` object on a deployment, whose own `image` is the reference.
#[derive(Deserialize)]
struct Outer {
    /// Where it came from.
    image: Inner,
}

/// The reference and the transport it came over.
#[derive(Deserialize)]
struct Inner {
    /// The reference, as this machine last wrote it.
    image: String,
    /// How it was fetched.
    transport: String,
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A whole digest made of one repeated pair.
    fn whole(pair: &str) -> String {
        format!("sha256:{}", pair.repeat(32))
    }

    /// What `bootc status --format json` answers, with the booted deployment
    /// naming `reference` over `transport`.
    fn an_answer(reference: &str, transport: &str) -> Vec<u8> {
        format!(
            r#"{{"apiVersion":"org.containers.bootc/v1","kind":"BootcHost",
            "spec":{{"image":{{"image":"{reference}","transport":"{transport}"}},"bootOrder":"default"}},
            "status":{{"staged":null,"rollback":null,"rollbackQueued":false,
              "booted":{{"image":{{"image":{{"image":"{reference}","transport":"{transport}"}},
                "version":"0.0.5","timestamp":null,"imageDigest":"{}","architecture":"amd64"}},
                "cachedUpdate":null,"incompatible":false,"pinned":false,"store":"ostreeContainer",
                "ostree":{{"checksum":"abc","deploySerial":0,"stateroot":"default"}}}},
              "type":"bootcHost"}}}}"#,
            whole("aa")
        )
        .into_bytes()
    }

    /// A base answering with whatever it is given.
    struct Answering(Vec<u8>);

    impl Base for Answering {
        fn asked(&self, arguments: &[String]) -> Result<Vec<u8>, NotAnswered> {
            assert_eq!(arguments, THE_STATUS);
            Ok(self.0.clone())
        }
    }

    /// A base that will not answer.
    struct Silent;

    impl Base for Silent {
        fn asked(&self, _: &[String]) -> Result<Vec<u8>, NotAnswered> {
            Err(NotAnswered::NotStarted {
                program: "/nowhere/bootc".to_owned(),
                why: "there is none".to_owned(),
            })
        }
    }

    /// **The builds and the repository come out of one answer**, and the
    /// repository is the booted build's, with the build cut off it.
    #[test]
    fn the_builds_and_the_repository_are_read_from_one_answer() {
        let reference = format!("ghcr.io/aloworld-org/alo-os@{}", whole("aa"));
        let now = TheMachineNow::read(&Answering(an_answer(&reference, "registry"))).unwrap();
        assert_eq!(now.source().as_str(), "ghcr.io/aloworld-org/alo-os");
        assert_eq!(
            now.deployments().running().unwrap().digest().as_str(),
            whole("aa")
        );
    }

    /// **A build cut off a reference is cut at the two places a registry puts
    /// one**, and a host with a port is not mistaken for one.
    #[test]
    fn a_build_is_cut_off_a_reference_and_a_port_is_not_one() {
        for (reference, repository) in [
            ("ghcr.io/aloworld-org/alo-os", "ghcr.io/aloworld-org/alo-os"),
            (
                "ghcr.io/aloworld-org/alo-os:0.0.5",
                "ghcr.io/aloworld-org/alo-os",
            ),
            (
                "ghcr.io/aloworld-org/alo-os@sha256:beef",
                "ghcr.io/aloworld-org/alo-os",
            ),
            ("10.0.2.2:5000/alo-os", "10.0.2.2:5000/alo-os"),
            ("10.0.2.2:5000/alo-os:0.0.5", "10.0.2.2:5000/alo-os"),
        ] {
            assert_eq!(the_repository_in(reference), repository, "{reference}");
        }
    }

    /// **A build that did not come from a registry is refused**, rather than
    /// turned into a repository nothing could fetch from.
    #[test]
    fn a_build_that_did_not_come_from_a_registry_is_refused() {
        let refused = TheMachineNow::read(&Answering(an_answer(
            "/var/lib/images/alo-os.oci",
            "oci-archive",
        )))
        .unwrap_err();
        assert!(matches!(refused, NotRead::NotFromARegistry(_)), "{refused}");
        assert!(refused.to_string().contains("nowhere for an update"));
    }

    /// **A repository the update crate would not accept is refused here**, by
    /// that crate's own check and not by a second one.
    #[test]
    fn a_repository_that_is_not_one_is_refused_by_the_crate_that_decides() {
        let refused =
            TheMachineNow::read(&Answering(an_answer("NOT A REGISTRY", "registry"))).unwrap_err();
        assert!(matches!(refused, NotRead::NotARepository(_)), "{refused}");
    }

    /// **A base that does not answer, and one that answers with something
    /// else, are two different refusals** — and neither is a machine running
    /// nothing.
    #[test]
    fn a_base_that_says_nothing_and_one_that_says_something_else_are_told_apart() {
        assert!(matches!(
            TheMachineNow::read(&Silent).unwrap_err(),
            NotRead::TheBaseDidNotAnswer(_)
        ));
        assert!(matches!(
            TheMachineNow::read(&Answering(b"not a status".to_vec())).unwrap_err(),
            NotRead::NotItsStatus(_)
        ));
    }
}
