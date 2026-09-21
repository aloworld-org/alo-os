//! Which build this machine is running, read off what the base has already
//! written down.
//!
//! [`crate::status::running`] asks the base's own command and is the right road
//! for anything that is going to **change** the machine, where being root is
//! correct anyway. This is the other road, and it exists because of one measured
//! fact: **`bootc status` refuses an unprivileged caller.** On a real bootc
//! machine on 2026-09-20 it answered *This command must be executed as the root
//! user* to uid 1000, in every spelling of the question, and so the one look an
//! alo OS machine takes on its way up — which runs as the person and holds
//! nothing (ADR 0018, ADR 0001 §2) — failed at every boot and kept nothing.
//! `docs/autonomy/updates/the-base-answers-only-root.md` has the whole of it.
//!
//! The answer is not to make that component root. The base has **already**
//! written down what the question needs, world-readable, in two places a person
//! can read today on a stock machine:
//!
//! 1. the words the kernel was started with name the deployment that booted
//!    ([`crate::booted`]);
//! 2. the `.origin` file beside that deployment names the image it was made
//!    from, with its digest ([`crate::origin`]).
//!
//! So this road runs **no program at all**. It reads one file, resolves one
//! path and reads one more file. Nothing on the machine is new, nothing is
//! privileged, and no grant is widened.
//!
//! # And it is the more truthful of the two roads
//!
//! Measured on the same machine: `status.booted.image.imageDigest` was
//! `sha256:2e7ecd95…` where the image reference, the spec and the origin file
//! all said `sha256:48bd5f31…`. The first is the digest the **local container
//! store** gave the image the machine was installed from; the second is the one
//! the **registry** publishes, and the registry's is the one an offer is
//! compared against. A machine running exactly the pinned build was therefore
//! told a newer version was available. The origin file carries the digest that
//! matches what the registry offers, so reading it closes that as well —
//! `docs/autonomy/updates/what-the-base-has-already-written-down.md` measures
//! both installs side by side.

use std::path::{Path, PathBuf};

use alo_keeping_up::Running;

use crate::booted::booted;
use crate::origin::{BESIDE_IT, the_build};
use crate::refusing::NotRead;

/// The root of the machine being read: `/` on a real one.
const THIS_MACHINE: &str = "/";

/// What the base has already written down about the build this machine booted.
///
/// Reads and never writes, and holds nothing across a call: the question is
/// answered from the disk every time it is asked, for [`crate::status`]'s own
/// reason — a remembered answer is wrong for exactly the moments the question
/// matters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenDown {
    /// The root every path below is read under.
    under: PathBuf,
}

impl Default for WrittenDown {
    fn default() -> Self {
        Self::on_this_machine()
    }
}

impl WrittenDown {
    /// What this machine's own base has written down.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self {
            under: PathBuf::from(THIS_MACHINE),
        }
    }

    /// The same, under some other root — a test's whole machine in a folder.
    ///
    /// **Not a setting and not a path a service is given.** There is one answer
    /// to *what this machine is running*, and it is read under `/`; this exists
    /// so that the code a real machine runs is the code a test runs, rather
    /// than a stand-in that agrees with a machine nobody owns.
    #[must_use]
    pub fn under(root: &Path) -> Self {
        Self {
            under: root.to_path_buf(),
        }
    }

    /// The root every path is read under.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.under
    }

    /// The build this machine is running.
    ///
    /// # Errors
    /// [`NotRead::NotBooted`] when the machine did not boot from a deployment
    /// or the kernel's words could not be read, and
    /// [`NotRead::NotInTheOrigin`] when the deployment that booted names no
    /// build.
    pub fn running(&self) -> Result<Running, NotRead> {
        let deployment = booted(&self.under).map_err(NotRead::NotBooted)?;
        let at = beside(&deployment);
        let text = std::fs::read_to_string(&at).map_err(|why| NotRead::NotWrittenDown {
            path: at.display().to_string(),
            why: why.to_string(),
        })?;
        the_build(&text)
            .map(Running::reported)
            .map_err(NotRead::NotInTheOrigin)
    }
}

/// The origin file beside a deployment's folder.
fn beside(deployment: &Path) -> PathBuf {
    let mut name = deployment.as_os_str().to_owned();
    name.push(BESIDE_IT);
    PathBuf::from(name)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::booted::THE_KERNELS_WORDS;

    /// The digest the pinned release is published under, whole.
    const THE_BUILD: &str =
        "sha256:48bd5f319abcecfa832eb9a5b0b2f7cd06815b1c30c43b781499500ec14c3858";

    /// A machine that booted a deployment made from `reference`, laid out the
    /// way a real one is.
    fn a_machine(named: &str, deployment: &str, reference: Option<&str>) -> PathBuf {
        let folder = std::env::temp_dir().join(format!(
            "alo-updating-written-down-{named}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&folder);
        let deployed = folder.join("ostree/deploy/default/deploy");
        std::fs::create_dir_all(deployed.join(deployment)).unwrap();
        std::fs::create_dir_all(folder.join("proc")).unwrap();
        std::fs::write(
            folder.join(THE_KERNELS_WORDS),
            format!("rw ostree=/ostree/deploy/default/deploy/{deployment}\n"),
        )
        .unwrap();
        if let Some(reference) = reference {
            std::fs::write(
                deployed.join(format!("{deployment}.origin")),
                format!("[origin]\ncontainer-image-reference={reference}\n"),
            )
            .unwrap();
        }
        folder
    }

    /// **The build this machine is running is the one the base wrote down**,
    /// with no program run and nothing privileged read — the whole road, laid
    /// out as a real machine lays it out.
    #[test]
    fn the_build_running_is_the_one_the_base_wrote_down() {
        let machine = a_machine(
            "running",
            "f9ce6166.0",
            Some(&format!(
                "ostree-unverified-registry:ghcr.io/aloworld-org/alo-os@{THE_BUILD}"
            )),
        );

        let running = WrittenDown::under(&machine).running().unwrap();

        assert_eq!(running.digest().as_str(), THE_BUILD);
    }

    /// **A machine that booted no deployment is refused**, and says which of
    /// the two roads it failed on: this is every machine in this repository
    /// that is not an alo OS machine.
    #[test]
    fn a_machine_that_booted_no_deployment_is_refused() {
        let nowhere = std::env::temp_dir().join(format!(
            "alo-updating-written-down-nowhere-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&nowhere);
        std::fs::create_dir_all(&nowhere).unwrap();

        let refused = WrittenDown::under(&nowhere).running();

        assert!(matches!(refused, Err(NotRead::NotBooted(_))), "{refused:?}");
    }

    /// **A deployment whose origin is not there is refused**, rather than read
    /// as a machine running nothing: the deployment booted and this road could
    /// not say which build it is.
    #[test]
    fn a_deployment_with_no_origin_beside_it_is_refused() {
        let machine = a_machine("no-origin", "f9ce6166.0", None);

        let refused = WrittenDown::under(&machine).running();

        assert!(
            matches!(refused, Err(NotRead::NotWrittenDown { .. })),
            "{refused:?}"
        );
    }

    /// **An origin naming a moving tag is refused**, because an offer is a
    /// difference between two digests.
    #[test]
    fn an_origin_naming_no_build_is_refused() {
        let machine = a_machine(
            "no-build",
            "f9ce6166.0",
            Some("ostree-unverified-registry:ghcr.io/aloworld-org/alo-os:0.0.4"),
        );

        let refused = WrittenDown::under(&machine).running();

        assert!(
            matches!(refused, Err(NotRead::NotInTheOrigin(_))),
            "{refused:?}"
        );
    }

    /// **This machine's own road is read under its own root**, and nothing
    /// about it is a setting.
    #[test]
    fn this_machines_own_road_is_read_under_its_own_root() {
        assert_eq!(WrittenDown::on_this_machine().root(), Path::new("/"));
        assert_eq!(WrittenDown::default(), WrittenDown::on_this_machine());
    }
}
