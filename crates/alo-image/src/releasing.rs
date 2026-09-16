//! The workflow that builds the installer a person downloads, and the things it
//! may never do.
//!
//! The installer plan's task 5 asks for a workflow that builds `alo-installer`
//! for Windows on a tag, signs the executable, and publishes it as a Release
//! asset with a checksum beside it.
//! [ADR 0046](../../../docs/decisions/0046-the-installer-is-signed-by-a-certificate-a-person-holds.md)
//! then says which half of that a machine may do: it builds, checksums and
//! publishes a **draft**, and **a person signs**. That is
//! [ADR 0036](../../../docs/decisions/0036-the-image-is-signed-by-a-key-a-person-holds.md)
//! applied to the second artefact, and for the stronger reason — this is the
//! program that repartitions somebody's only computer.
//!
//! A workflow is the file in this repository nobody reads twice, and every rule
//! above is one step away from being undone in it. So this reads it, and
//! `tests/how_the_installer_is_released.rs` holds the shipped one to each.
//!
//! # Not a YAML reader, and not a second one either
//!
//! It reads lines through `crate::workflow`'s reader, which already knows how
//! to find the triggers and skip the comments. Two readers of GitHub's file
//! format in one crate would be exactly the drift this crate exists to catch.

use crate::notes::THE_NOTES;
use crate::workflow::TheWorkflow;

/// Where the release's workflow is, in this repository.
pub const THE_RELEASE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../.github/workflows/release.yml"
);

/// The only trigger it may have: a tag somebody pushed.
const PUSHED: &str = "push";

/// What says the push it runs on is a tag's.
const A_TAG: &str = "tags:";

/// Every spelling of signing this workflow could reach for: the image's
/// checker, and the three ways a Windows executable is signed.
const SIGNING: [&str; 5] = [
    "cosign sign",
    "COSIGN_",
    "signtool",
    "osslsigncode",
    "Set-AuthenticodeSignature",
];

/// What a workflow says to reach for something the repository does not hold —
/// a certificate, a key, a password. `github.token` is not one of these: it is
/// issued to the run and expires with it.
const A_SECRET: &str = "secrets.";

/// What makes the Release.
const PUBLISH: &str = "gh release create";

/// What keeps it off the download page until a person has signed what is in it.
const A_DRAFT: &str = "--draft";

/// What takes the notes from a file rather than from a sentence in the workflow.
const NOTES_FROM_A_FILE: &str = "--notes-file";

/// What types them instead.
const NOTES_TYPED: &str = "--notes ";

/// The file the checksum of every asset is written to.
const THE_CHECKSUMS: &str = "SHA256SUMS";

/// The variable the boot environment's list is compiled into the installer as.
const THE_ENVIRONMENTS_LIST: &str = "ALO_INSTALLER_ENVIRONMENT_SHA256";

/// The pin the tag is held to before anything is built.
const THE_PIN: &str = "image/pinned.toml";

/// What builds the installer.
const THE_BUILD: &str = "cargo build";

/// The words the reason it does not sign is recorded under.
const THE_REASON: &str = "Why it does not sign";

/// What the release's workflow says, as far as the rules above need it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TheRelease {
    /// The file, read the way every workflow in this repository is read.
    read: TheWorkflow,
}

impl TheRelease {
    /// What this workflow file says, read off its text.
    #[must_use]
    pub fn read(workflow: &str) -> Self {
        Self {
            read: TheWorkflow::read(workflow),
        }
    }

    /// Whether it runs only on a tag somebody pushed.
    ///
    /// Never on a commit landing on a branch: a Release is a thing a person
    /// decides to make, and a workflow that made one on every push would put an
    /// unsigned installer on the download page by accident.
    #[must_use]
    pub fn runs_only_on_a_tag(&self) -> bool {
        self.read.triggers() == [PUSHED] && self.live().any(|line| line.contains(A_TAG))
    }

    /// Whether anything it runs signs — the image's checker, or any of the ways
    /// a Windows executable is signed.
    #[must_use]
    pub fn signs(&self) -> bool {
        self.live()
            .any(|line| SIGNING.iter().any(|signing| line.contains(signing)))
    }

    /// Whether it reaches for a certificate, a key or a password the repository
    /// does not hold.
    ///
    /// A separate question from [`Self::signs`] on purpose: the step that would
    /// undo ADR 0046 arrives as a secret before it arrives as a command, and the
    /// secret is the half a reviewer can see in the file.
    #[must_use]
    pub fn reaches_for_a_secret(&self) -> bool {
        self.live().any(|line| line.contains(A_SECRET))
    }

    /// Whether the Release it makes is a draft.
    #[must_use]
    pub fn publishes_a_draft(&self) -> bool {
        self.publishing().is_some_and(|it| it.contains(A_DRAFT))
    }

    /// Whether the notes it publishes are the committed ones, and not a
    /// sentence written in the workflow.
    #[must_use]
    pub fn takes_its_notes_from_the_committed_file(&self) -> bool {
        self.publishing().is_some_and(|it| {
            it.contains(&format!("{NOTES_FROM_A_FILE} image/{THE_NOTES}"))
                && !it.contains(NOTES_TYPED)
        })
    }

    /// Whether the checksum of every asset is written before the Release is
    /// made, and published with it.
    #[must_use]
    pub fn writes_the_checksum_before_it_publishes(&self) -> bool {
        let written = matches!(
            (self.first(THE_CHECKSUMS), self.first(PUBLISH)),
            (Some(checksum), Some(publish)) if checksum < publish
        );
        written
            && self
                .publishing()
                .is_some_and(|it| it.contains(THE_CHECKSUMS))
    }

    /// Whether the installer it builds carries the environment it stages.
    ///
    /// A build with no list has nothing to hold a download to and refuses every
    /// one of them (`alo_installer::Released`), so a Release made without it
    /// would be a download that says it is not genuine.
    #[must_use]
    pub fn holds_the_build_to_the_environment(&self) -> bool {
        matches!(
            (self.first(THE_ENVIRONMENTS_LIST), self.first(THE_BUILD)),
            (Some(list), Some(build)) if list < build
        )
    }

    /// Whether it refuses a tag the pin does not name, before anything is built.
    #[must_use]
    pub fn refuses_a_tag_the_pin_does_not_name(&self) -> bool {
        matches!(
            (self.first(THE_PIN), self.first(THE_BUILD)),
            (Some(pin), Some(build)) if pin < build
        )
    }

    /// Whether it records why it does not sign, so that nobody adds the step
    /// without reading why it is not there.
    #[must_use]
    pub fn says_why_it_does_not_sign(&self) -> bool {
        self.read.comments().contains(THE_REASON)
    }

    /// Every line that is not a comment, in order.
    fn live(&self) -> impl Iterator<Item = &String> {
        self.read.live().iter()
    }

    /// Where this is first said, among the lines that run.
    fn first(&self, what: &str) -> Option<usize> {
        self.live().position(|line| line.contains(what))
    }

    /// The line that makes the Release, where there is one.
    fn publishing(&self) -> Option<&String> {
        self.live().find(|line| line.contains(PUBLISH))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The workflow this repository ships, as text.
    fn shipped() -> String {
        std::fs::read_to_string(THE_RELEASE).unwrap()
    }

    /// The shipped workflow with one thing changed.
    fn with(from: &str, to: &str) -> TheRelease {
        let text = shipped();
        assert!(
            text.contains(from),
            "the workflow does not contain `{from}`"
        );
        TheRelease::read(&text.replace(from, to))
    }

    /// The line that makes the Release, as the shipped workflow writes it.
    fn the_publish() -> String {
        let text = shipped();
        text.lines()
            .find(|line| line.contains(PUBLISH))
            .unwrap()
            .to_owned()
    }

    /// **The shipped workflow keeps every rule**, so the refusals below are
    /// about the rules rather than a reader that never worked.
    #[test]
    fn the_shipped_workflow_builds_checksums_and_drafts() {
        let read = TheRelease::read(&shipped());

        assert!(read.runs_only_on_a_tag());
        assert!(!read.signs());
        assert!(!read.reaches_for_a_secret());
        assert!(read.publishes_a_draft());
        assert!(read.takes_its_notes_from_the_committed_file());
        assert!(read.writes_the_checksum_before_it_publishes());
        assert!(read.holds_the_build_to_the_environment());
        assert!(read.refuses_a_tag_the_pin_does_not_name());
        assert!(read.says_why_it_does_not_sign());
    }

    /// **A workflow that signs is caught**, however it spells it — and the
    /// comment saying a person signs is not signing.
    #[test]
    fn a_workflow_that_signs_is_caught() {
        for signing in [
            "signtool sign /fd sha256 alo-installer.exe",
            "Set-AuthenticodeSignature alo-installer.exe $cert",
            "osslsigncode sign -in alo-installer.exe",
            "cosign sign --key cosign.key alo-installer.exe",
        ] {
            let read = with(
                "          Get-Content SHA256SUMS",
                &format!("          Get-Content SHA256SUMS\n          {signing}"),
            );
            assert!(read.signs(), "{signing}");
        }
        assert!(!TheRelease::read(&shipped()).signs());
    }

    /// **A workflow reaching for a certificate is caught before it signs with
    /// it**, which is the half a reviewer can see.
    #[test]
    fn a_workflow_reaching_for_a_secret_is_caught() {
        let read = with(
            "          GH_TOKEN: ",
            "          CERTIFICATE: ${{ secrets.WINDOWS_CERTIFICATE }}\n          GH_TOKEN: ",
        );
        assert!(read.reaches_for_a_secret());
    }

    /// **A workflow that runs on anything but a tag is caught**: a branch, a
    /// person asking beside it, a bare push.
    #[test]
    fn a_workflow_that_runs_without_a_tag_is_caught() {
        let on = "on:\n  push:\n    tags:\n      - 'v*'\n";
        for to in [
            "on:\n  push:\n    branches: [main]\n",
            "on:\n  push:\n    tags:\n      - 'v*'\n  workflow_dispatch:\n",
            "on: push\n",
        ] {
            let read = with(on, to);
            assert!(!read.runs_only_on_a_tag(), "{to}");
        }
    }

    /// **A workflow that publishes rather than drafts is caught**: a Release
    /// that exists is a Release somebody can download, and nobody has signed
    /// what is in it yet.
    #[test]
    fn a_workflow_that_publishes_outright_is_caught() {
        let read = with(" --draft", "");
        assert!(!read.publishes_a_draft());
    }

    /// **A workflow that types its own notes is caught**, and so is one that
    /// takes them from anywhere but the committed file: the digest in the notes
    /// is the digest in the pin, and a sentence typed here is a second spelling
    /// of it.
    #[test]
    fn a_workflow_that_types_its_notes_is_caught() {
        let from = format!("{NOTES_FROM_A_FILE} image/{THE_NOTES}");
        for instead in [
            "--notes \"alo OS, installed from Windows\"",
            "--notes-file typed.md",
        ] {
            let read = with(&from, instead);
            assert!(!read.takes_its_notes_from_the_committed_file(), "{instead}");
        }
    }

    /// **A workflow that publishes without the checksum is caught**, and so is
    /// one that writes it only afterwards.
    #[test]
    fn a_workflow_that_publishes_without_a_checksum_is_caught() {
        let publish = the_publish();
        let read = with(&publish, &publish.replace(" SHA256SUMS", ""));
        assert!(!read.writes_the_checksum_before_it_publishes());

        let read = TheRelease::read(
            "on:\n  push:\n    tags:\n      - v\nrun: gh release create v0.0.1 SHA256SUMS\nrun: \
             Get-FileHash > SHA256SUMS\n",
        );
        assert!(!read.writes_the_checksum_before_it_publishes());
    }

    /// **A workflow that builds the installer without the environment's list is
    /// caught**: what it published would refuse every download it was given.
    #[test]
    fn a_workflow_building_without_the_environment_is_caught() {
        let read = with(THE_ENVIRONMENTS_LIST, "SOMETHING_ELSE");
        assert!(!read.holds_the_build_to_the_environment());
    }

    /// **A workflow that builds before it holds the tag to the pin is caught**,
    /// because the tag is the one thing in a release a person types.
    #[test]
    fn a_workflow_that_builds_before_it_reads_the_pin_is_caught() {
        let read = with(THE_PIN, "somewhere/else.toml");
        assert!(!read.refuses_a_tag_the_pin_does_not_name());
    }

    /// **A workflow that stopped saying why it does not sign is caught**, so
    /// nobody adds the step without reading why it is not there.
    #[test]
    fn a_workflow_without_its_reason_is_caught() {
        let read = with(THE_REASON, "Why this exists");
        assert!(!read.says_why_it_does_not_sign());
    }
}
