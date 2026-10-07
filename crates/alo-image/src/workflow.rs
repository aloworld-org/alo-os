//! The workflow that builds and pushes a candidate image, and the things it may
//! never do.
//!
//! The installer plan's first task asks for the image to be pushed by a
//! workflow in `.github/workflows/` — or, until a GitHub runner can hold an
//! image carrying 4.5 GB of weights, by the machine that built it, *with the
//! workflow file written and the reason it is not yet the road recorded in it*.
//! [ADR 0036](../../../docs/decisions/0036-the-image-is-signed-by-a-key-a-person-holds.md)
//! then says what a workflow is allowed to be when it becomes the road: it
//! builds and pushes a candidate, and **it never signs**.
//!
//! A workflow is the file in this repository nobody reads twice, and the change
//! that would undo ADR 0036 is one step added to it. So this reads it for the
//! handful of things it may never do, and `tests/how_the_image_is_published.rs`
//! holds the shipped one to them.
//!
//! # Not a YAML reader
//!
//! It reads lines, the way `crate::unit` reads a systemd unit: the triggers
//! under `on:`, and every line that is not a comment. A workflow that would need
//! a full YAML parser to be understood is one this test should refuse to believe
//! anyway, and a comment is excluded because the comment explaining that the
//! owner signs is exactly where the word belongs.

/// Where the workflow is, in this repository.
pub const THE_WORKFLOW: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../.github/workflows/image.yml"
);

/// The only trigger it may have: a person asking.
const ASKED: &str = "workflow_dispatch";

/// Every spelling of signing a workflow in this repository could reach for.
///
/// **Both kinds, because this repository signs two different things.** ADR 0036
/// reserves the *image's* signature, which is `cosign`; ADR 0046 reserves the
/// *installer's*, which is Authenticode. A reader that saw only one of them
/// would answer *it never signs* about a workflow signing the other.
///
/// # This was two spellings until 2026-10-07, and the gap was real
///
/// It held `cosign sign` and `COSIGN_` alone, which was right for `image.yml`
/// — where this constant was born and which signs nothing anyway. But
/// `installer-candidate.yml` builds a **Windows executable**, and
/// `the_installer_candidate_is_not_a_release.rs::it_never_signs` cites ADR 0046
/// in its own message. So the name and the comment promised Authenticode while
/// the inputs could only see `cosign`: **a candidate workflow that signed the
/// executable would have passed.**
///
/// That is not a hypothetical in the ordinary way. The whole reason the
/// candidate road exists is to iterate on a Windows installer, and *just sign
/// it on the runner so SmartScreen stops warning* is exactly the
/// reasonable-sounding edit somebody makes in three months.
///
/// Found by the third PC reviewing ADR 0096's own change — a check less
/// specific than the question it is named for, which is this repository's
/// standing fault.
///
/// Kept beside `crate::releasing`'s identical list rather than shared between
/// them is what this is **not**: `releasing.rs` imports this one, so there is
/// one list and a spelling added here reaches both roads.
pub(crate) const SIGNING: [&str; 6] = [
    "cosign sign",
    "COSIGN_",
    "signtool",
    "osslsigncode",
    "Set-AuthenticodeSignature",
    ".pfx",
];

/// What pushes.
const PUSH: &str = "podman push";

/// What asks the registry whether a tag already exists.
const LOOK: &str = "skopeo inspect";

/// The branch a candidate is built from.
const MAIN: &str = "refs/heads/main";

/// What makes a Release.
const PUBLISH: &str = "gh release create";

/// What deletes one.
const DELETE: &str = "gh release delete";

/// The flag that keeps a Release off `/releases/latest`.
///
/// GitHub defines that endpoint as the most recent **non-prerelease,
/// non-draft** release and refuses to make a pre-release latest, so this one
/// flag is enforced by the API rather than by us — ADR 0096's first control,
/// and the only one of its six that holds whether or not this reader runs.
const PRERELEASE: &str = "--prerelease";

/// The flag that would undo it.
const LATEST: &str = "--latest";

/// What the workflow says, as far as the rules above need it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TheWorkflow {
    /// The events it runs on.
    triggers: Vec<String>,
    /// Every line that is not a comment, trimmed, in order.
    live: Vec<String>,
    /// Every comment, joined.
    comments: String,
}

impl TheWorkflow {
    /// What this workflow file says, read off its text.
    #[must_use]
    pub fn read(workflow: &str) -> Self {
        let mut read = Self::default();
        let mut under_on = false;

        for line in workflow.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(comment) = trimmed.strip_prefix('#') {
                read.comments.push_str(comment);
                read.comments.push('\n');
                continue;
            }
            read.live.push(trimmed.to_owned());

            let indented = line.starts_with(' ');
            if !indented {
                under_on = false;
                if let Some(inline) = line.strip_prefix("on:") {
                    under_on = true;
                    read.triggers.extend(
                        inline
                            .trim()
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .split(',')
                            .map(str::trim)
                            .filter(|it| !it.is_empty())
                            .map(str::to_owned),
                    );
                }
                continue;
            }
            let at_first_level = line.starts_with("  ") && !line.starts_with("   ");
            if under_on
                && at_first_level
                && let Some((event, _)) = trimmed.split_once(':')
            {
                read.triggers.push(event.trim().to_owned());
            }
        }
        read
    }

    /// The events it runs on.
    #[must_use]
    pub fn triggers(&self) -> &[String] {
        &self.triggers
    }

    /// Every line that is not a comment, trimmed, in order.
    ///
    /// For `crate::releasing`, which asks a different workflow a different set
    /// of questions off the same reading: two readers of GitHub's file format
    /// in one crate would be the drift this crate exists to catch.
    #[must_use]
    pub fn live(&self) -> &[String] {
        &self.live
    }

    /// Every comment in it, joined, in order.
    ///
    /// Kept apart from [`Self::live`] because the sentence explaining that a
    /// person signs is exactly the place the word `sign` belongs, and a rule
    /// that read it would refuse the file for saying why it obeys the rule.
    #[must_use]
    pub fn comments(&self) -> &str {
        &self.comments
    }

    /// Whether it runs only when a person asks it to.
    ///
    /// Not on a push, a tag, a schedule or a pull request: the build does not
    /// fit on a runner yet, and a workflow that failed on every change to main
    /// would be a red run nobody asked for.
    #[must_use]
    pub fn runs_only_when_asked(&self) -> bool {
        self.triggers == [ASKED]
    }

    /// Whether it builds only from main.
    #[must_use]
    pub fn builds_only_from_main(&self) -> bool {
        self.live.iter().any(|line| line.contains(MAIN))
    }

    /// Whether anything it runs signs, or reaches for a signing key.
    #[must_use]
    pub fn signs(&self) -> bool {
        self.live
            .iter()
            .any(|line| SIGNING.iter().any(|signing| line.contains(signing)))
    }

    /// Whether it pushes, and names this registry to push to.
    #[must_use]
    pub fn pushes_to(&self, registry: &str) -> bool {
        self.live.iter().any(|line| line.contains(PUSH))
            && self.live.iter().any(|line| line.contains(registry))
    }

    /// Whether it asks the registry whether the release exists before it
    /// pushes, so a published tag is never moved.
    #[must_use]
    pub fn looks_before_it_pushes(&self) -> bool {
        let first = |what: &str| self.live.iter().position(|line| line.contains(what));
        matches!((first(LOOK), first(PUSH)), (Some(look), Some(push)) if look < push)
    }

    /// Whether it makes a Release at all.
    ///
    /// **Not a refusal any more, and that is ADR 0096.** This read `false` for
    /// `installer-candidate.yml` until 2026-10-07, because ADR 0046 had
    /// rejected publishing anything that is not a draft. The owner published
    /// an unsigned release deliberately on that date, which left *unsigned*
    /// carrying no information about what is a release, so the question moved
    /// from *does it publish* to *can what it publishes be mistaken for a
    /// release*.
    #[must_use]
    pub fn publishes(&self) -> bool {
        self.live.iter().any(|line| line.contains(PUBLISH))
    }

    /// Whether everything it publishes is marked a pre-release.
    ///
    /// Asked of **every** publishing line rather than of any one of them: a
    /// workflow that marked one and not another would pass a rule written the
    /// other way round, and the one that was not marked is the one that would
    /// become `latest`.
    ///
    /// **The flag has to be on the command's own line.** This reads lines, as
    /// this file's header says, so a `--prerelease` on a continuation is a
    /// `--prerelease` this cannot see. That is a real limit and the right
    /// answer to it is the workflow putting the flag where the command is —
    /// the one flag that decides whether a stranger downloads this belongs
    /// beside `gh release create` and not three lines below it. Measured
    /// 2026-10-07: the first draft of the shipped workflow put it below, and
    /// this reader refused the file.
    #[must_use]
    pub fn publishes_only_prereleases(&self) -> bool {
        let publishing: Vec<&String> = self
            .live
            .iter()
            .filter(|line| line.contains(PUBLISH))
            .collect();
        !publishing.is_empty() && publishing.iter().all(|line| line.contains(PRERELEASE))
    }

    /// Whether anything it runs would make a Release the latest one.
    #[must_use]
    pub fn makes_something_latest(&self) -> bool {
        self.live.iter().any(|line| line.contains(LATEST))
    }

    /// Whether it deletes the previous candidate before publishing the next.
    ///
    /// ADR 0096's sixth control: **exactly one candidate exists at a time.**
    /// Before rather than merely somewhere, for `looks_before_it_pushes`'s
    /// reason — a delete that ran afterwards would take the one just made.
    #[must_use]
    pub fn deletes_the_previous_before_publishing(&self) -> bool {
        let first = |what: &str| self.live.iter().position(|line| line.contains(what));
        matches!((first(DELETE), first(PUBLISH)), (Some(gone), Some(made)) if gone < made)
    }

    /// Whether this word appears in what it runs.
    ///
    /// For the two things ADR 0096 asks to be present by name rather than by
    /// prose: the marker a candidate's notes must carry, and the variable the
    /// build date is compiled in through. A reader that asked *does it say
    /// plainly what it is* could not fail, which is the fault four of this
    /// repository's `docs/misreadings/` entries are about.
    #[must_use]
    pub fn runs_something_naming(&self, word: &str) -> bool {
        self.live.iter().any(|line| line.contains(word))
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
        std::fs::read_to_string(THE_WORKFLOW).unwrap()
    }

    /// The shipped workflow with one thing changed.
    fn with(from: &str, to: &str) -> TheWorkflow {
        let text = shipped();
        assert!(
            text.contains(from),
            "the workflow does not contain `{from}`"
        );
        TheWorkflow::read(&text.replace(from, to))
    }

    /// **The shipped workflow keeps every rule**, so the refusals below are
    /// about the rules rather than a reader that never worked.
    #[test]
    fn the_shipped_workflow_pushes_a_candidate_and_nothing_more() {
        let read = TheWorkflow::read(&shipped());

        assert_eq!(read.triggers(), [ASKED]);
        assert!(read.runs_only_when_asked());
        assert!(read.builds_only_from_main());
        assert!(!read.signs());
        assert!(read.pushes_to(crate::THE_REGISTRY));
        assert!(read.looks_before_it_pushes());
    }

    /// **A workflow that signs is caught**, however it spells it: the command,
    /// or a key or password handed to it as a secret.
    #[test]
    fn a_workflow_that_signs_is_caught() {
        let push = "podman push --digestfile digest \"$REGISTRY:$VERSION\"";
        for signing in [
            format!("{push}\n          cosign sign --key cosign.key \"$REGISTRY@$(cat digest)\""),
            format!("{push}\n        env:\n          COSIGN_PASSWORD: ${{{{ secrets.KEY }}}}"),
        ] {
            let read = with(push, &signing);
            assert!(read.signs(), "{signing}");
        }
    }

    /// **Every spelling is caught, Authenticode included.**
    ///
    /// The fixture the third PC asked for when they found that `SIGNING` held
    /// only `cosign`: without this test the widened list is a change nobody
    /// has checked, and the gap it closes was invisible precisely because the
    /// reader answered confidently about the wrong kind of signing.
    ///
    /// Each spelling on its own line, so a failure names which one is not
    /// caught rather than that something is not.
    #[test]
    fn signing_a_windows_executable_is_caught_as_well_as_signing_an_image() {
        for spelling in [
            "cosign sign --key env://COSIGN_KEY $digest",
            "COSIGN_PASSWORD: ${{ secrets.COSIGN_PASSWORD }}",
            "signtool sign /fd sha256 alo-installer.exe",
            "osslsigncode sign -certs cert.pem alo-installer.exe",
            "Set-AuthenticodeSignature alo-installer.exe $cert",
            "$cert = Get-PfxCertificate alo.pfx",
        ] {
            let read = TheWorkflow::read(&format!("    - run: {spelling}\n"));
            assert!(
                read.signs(),
                "a workflow signing with `{spelling}` was not caught, so a road that signed \
                 this way would pass a test naming ADR 0036 or ADR 0046"
            );
        }
    }

    /// **But the comment saying the owner signs is not signing.**
    #[test]
    fn a_comment_about_signing_is_not_signing() {
        let read =
            TheWorkflow::read("# only the owner runs cosign sign\non:\n  workflow_dispatch:\n");
        assert!(!read.signs());
    }

    /// **A workflow that runs on its own is caught**: a push, a tag, a
    /// schedule, beside the person asking or instead of it, inline or as a
    /// block.
    #[test]
    fn a_workflow_that_runs_without_being_asked_is_caught() {
        for (from, to) in [
            (
                "on:\n  workflow_dispatch:\n",
                "on:\n  workflow_dispatch:\n  push:\n    branches: [main]\n",
            ),
            (
                "on:\n  workflow_dispatch:\n",
                "on:\n  schedule:\n    - cron: '0 3 * * *'\n",
            ),
            (
                "on:\n  workflow_dispatch:\n",
                "on: [push, workflow_dispatch]\n",
            ),
            ("on:\n  workflow_dispatch:\n", "on: push\n"),
        ] {
            let read = with(from, to);
            assert!(!read.runs_only_when_asked(), "{to}: {:?}", read.triggers());
        }
    }

    /// **A workflow that would build from any branch is caught.**
    #[test]
    fn a_workflow_that_builds_from_any_branch_is_caught() {
        let read = with("    if: github.ref == 'refs/heads/main'\n", "");
        assert!(!read.builds_only_from_main());
    }

    /// **A workflow that pushes without looking is caught**, and so is one that
    /// looks only afterwards: a published tag would already have moved.
    #[test]
    fn a_workflow_that_pushes_over_a_published_release_is_caught() {
        let look = "if skopeo inspect --raw";
        assert!(!with(look, "if false").looks_before_it_pushes());

        let read = TheWorkflow::read(
            "on:\n  workflow_dispatch:\nrun: podman push x\nrun: skopeo inspect x\n",
        );
        assert!(!read.looks_before_it_pushes());
    }

    /// **A workflow pushing somewhere else is caught.**
    #[test]
    fn a_workflow_pushing_to_another_registry_is_caught() {
        let read = with(
            "REGISTRY: ghcr.io/aloworld-org/alo-os",
            "REGISTRY: docker.io/somebody/alo-os",
        );
        assert!(!read.pushes_to(crate::THE_REGISTRY));
    }

    /// **A workflow that publishes a Release that is not a pre-release is
    /// caught**, and so is one that marks only some of them.
    ///
    /// Asked of every publishing line because the unmarked one is the one that
    /// becomes `latest`, and a rule written as *any of them* would pass while
    /// exactly that line existed. ADR 0096's first control.
    #[test]
    fn a_release_that_is_not_a_prerelease_is_caught() {
        let both = TheWorkflow::read(
            "    - run: gh release create one --prerelease\n    - run: gh release create two\n",
        );
        assert!(both.publishes());
        assert!(
            !both.publishes_only_prereleases(),
            "a workflow marking one Release and not the other read as marking them all"
        );

        let marked = TheWorkflow::read("    - run: gh release create one --prerelease\n");
        assert!(marked.publishes_only_prereleases());
        assert!(!marked.makes_something_latest());

        let latest = TheWorkflow::read("    - run: gh release create one --prerelease --latest\n");
        assert!(
            latest.makes_something_latest(),
            "a workflow asking for latest beside prerelease was not caught"
        );
    }

    /// **A workflow that deletes the previous candidate afterwards is
    /// caught**, which would take the one it just made.
    ///
    /// The same ordering fault `looks_before_it_pushes` exists for, and the
    /// same shape: a step in the right file at the wrong moment.
    #[test]
    fn deleting_the_previous_candidate_too_late_is_caught() {
        let right = TheWorkflow::read(
            "    - run: gh release delete old\n    - run: gh release create new --prerelease\n",
        );
        assert!(right.deletes_the_previous_before_publishing());

        let wrong = TheWorkflow::read(
            "    - run: gh release create new --prerelease\n    - run: gh release delete old\n",
        );
        assert!(
            !wrong.deletes_the_previous_before_publishing(),
            "a workflow deleting after it published read as deleting before"
        );

        let neither = TheWorkflow::read("    - run: gh release create new --prerelease\n");
        assert!(!neither.deletes_the_previous_before_publishing());
    }

    /// **A word nothing runs is not found**, which is what makes
    /// [`TheWorkflow::runs_something_naming`] a guard rather than a comment.
    ///
    /// It reads what the workflow *runs*, never its comments — the same split
    /// `a_comment_about_signing_is_not_signing` relies on, and the reason a
    /// file may explain the rule it obeys without breaking it.
    #[test]
    fn a_marker_only_a_comment_carries_is_not_found() {
        let run = TheWorkflow::read("    - run: gh release create one --prerelease\n");
        assert!(run.runs_something_naming("--prerelease"));
        assert!(
            !run.runs_something_naming("candidate-"),
            "a word the workflow never runs was found anyway"
        );

        let commented = TheWorkflow::read("    # candidate-2026-10-07\n    - run: echo hello\n");
        assert!(
            !commented.runs_something_naming("candidate-"),
            "a marker only a comment carries read as something the workflow runs"
        );
    }
}
