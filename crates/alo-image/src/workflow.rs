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

/// Every spelling of signing a workflow could use: the command, and the
/// variables `cosign` reads a key and its password from.
const SIGNING: [&str; 2] = ["cosign sign", "COSIGN_"];

/// What pushes.
const PUSH: &str = "podman push";

/// What asks the registry whether a tag already exists.
const LOOK: &str = "skopeo inspect";

/// The branch a candidate is built from.
const MAIN: &str = "refs/heads/main";

/// The words the reason it is not yet the road is recorded under.
const THE_REASON: &str = "not yet the road";

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

    /// Whether it records why it is not yet the road the image is published by.
    #[must_use]
    pub fn says_why_it_is_not_yet_the_road(&self) -> bool {
        self.comments.contains(THE_REASON)
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
        assert!(read.says_why_it_is_not_yet_the_road());
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

    /// **A workflow that stopped saying why it is not the road is caught**, so
    /// nobody switches it on without reading why it was off.
    #[test]
    fn a_workflow_without_its_reason_is_caught() {
        let read = with("Why this is not yet the road", "Why this exists");
        assert!(!read.says_why_it_is_not_yet_the_road());
    }
}
