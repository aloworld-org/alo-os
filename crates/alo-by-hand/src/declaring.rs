//! Which crates of this workspace declare verbs, read out of the workspace.
//!
//! This is the half of the check that no list can do. Reading the verbs out of
//! `alo_capability::Verbs` means a verb *added to a crate this check already
//! knows* is a verb it sees — that is the easy half, and it is bought by not
//! keeping verb names here. The other half is the failure that actually happens:
//! **somebody writes a new crate, declares verbs in it, and nothing connects it
//! to this check at all.** It happened to `alo-saying` in this repository —
//! `crates/alo-overlay` declared nine strings that nothing collected, and the
//! only reason a shell did not show a key instead of a sentence is that a person
//! noticed.
//!
//! So the workspace's own member list is walked, and a crate that declares verbs
//! and was not handed to this check is a finding naming it.
//!
//! # The convention this reads, which is therefore a rule
//!
//! **A crate declares verbs in `src/verbs.rs`, through a `pub fn declare_into`
//! that puts them on somebody else's list.** `alo-files` and `alo-applications`
//! both do exactly that, `docs/contracts/agent-verbs.md` now says so where
//! whoever adds a verb reads it, and this file is what makes it load-bearing
//! rather than tidy.
//!
//! What it does not catch is a crate that declares verbs somewhere else on
//! purpose. Nothing mechanical reaches that — a file can always be called
//! something else — which is why the convention is written into the contract as
//! well as read here: the check catches the author who did not know, and the
//! contract answers the author who is deciding.

/// Where the workspace says what is in it.
pub const THE_WORKSPACE: &str = "Cargo.toml";

/// Where a crate declares its verbs.
const WHERE_THE_VERBS_ARE: &str = "src/verbs.rs";

/// How a crate hands its verbs to somebody else's list.
const HANDS_THEM_OVER: &str = "pub fn declare_into";

/// Every member of the workspace, as the manifest names them.
///
/// Nothing is inferred from the disk: a member the manifest does not name is a
/// crate `cargo` does not build either. Comments are cut off each line first, so
/// a member commented out is not a member.
#[must_use]
pub fn members_of(manifest: &str) -> Vec<String> {
    let without_comments: String = manifest
        .lines()
        .map(|line| line.split('#').next().unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");

    let Some((_, after)) = without_comments.split_once("members") else {
        return Vec::new();
    };
    let Some((between, listed)) = after.split_once('[') else {
        return Vec::new();
    };
    if !between.trim().chars().all(|c| c == '=') {
        return Vec::new();
    }
    let Some((listed, _)) = listed.split_once(']') else {
        return Vec::new();
    };
    listed
        .split(',')
        .filter_map(|piece| {
            let quoted = piece.trim().strip_prefix('"')?;
            quoted.strip_suffix('"').map(str::to_owned)
        })
        .collect()
}

/// Every crate of this workspace that declares verbs, by name.
///
/// `reading` answers with the text of a file named by its repository-relative
/// path, or [`None`] where there is no such file. Nothing here opens anything —
/// which is what lets the finding this file exists for be shown happening
/// against a fixture, rather than only after somebody has written the crate that
/// would cause it.
#[must_use]
pub fn whoever_declares_verbs(
    manifest: &str,
    reading: &dyn Fn(&str) -> Option<String>,
) -> Vec<String> {
    members_of(manifest)
        .iter()
        .filter(|member| {
            reading(&format!("{member}/{WHERE_THE_VERBS_ARE}"))
                .is_some_and(|source| source.contains(HANDS_THEM_OVER))
        })
        .filter_map(|member| member.rsplit('/').next().map(str::to_owned))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A workspace written the way this one is written, with a commented-out
    /// member and an `exclude` list after the members.
    const A_WORKSPACE: &str = "\
[workspace]
resolver = \"3\"
members = [
  \"crates/alo-files\",
  \"crates/alo-capability\",
  # \"crates/alo-nothing\", not a member yet
  \"crates/alo-by-hand\",
]
exclude = [
  \"crates/alo-bounding-kernel\",
]
";

    /// The crates this workspace holds, in the order it names them, and nothing
    /// from the list of what it excludes.
    #[test]
    fn the_members_are_the_ones_the_manifest_names() {
        assert_eq!(
            members_of(A_WORKSPACE),
            [
                "crates/alo-files".to_owned(),
                "crates/alo-capability".to_owned(),
                "crates/alo-by-hand".to_owned(),
            ],
            "a commented-out member, or the excluded crate, was read as a member"
        );
    }

    /// A manifest this parser does not understand yields nothing, which is what
    /// [`crate::finding::Finding::NoWorkspaceToWalk`] is for: an empty list is
    /// never quietly taken for a workspace with no crates that declare verbs.
    #[test]
    fn a_manifest_that_says_nothing_yields_nothing() {
        for unreadable in [
            "",
            "[workspace]\nresolver = \"3\"\n",
            "[package]\nname = \"members\"\n",
            "members = [\n  \"crates/alo-files\",\n",
        ] {
            assert!(members_of(unreadable).is_empty(), "{unreadable:?}");
        }
    }

    /// **A crate that declares verbs is one whose `verbs.rs` hands them over.**
    /// `alo-capability` has a `verbs.rs` too — it is the registry the others are
    /// declared into — and it is not a crate that declares any.
    #[test]
    fn the_crates_that_declare_verbs_are_the_ones_that_hand_them_over() {
        let workspace = |path: &str| match path {
            "crates/alo-files/src/verbs.rs" => {
                Some("pub fn declare_into(verbs: &mut Verbs) {}".to_owned())
            }
            "crates/alo-capability/src/verbs.rs" => {
                Some("pub fn declare(&mut self, verb: Verb) {}".to_owned())
            }
            _ => None,
        };
        assert_eq!(
            whoever_declares_verbs(A_WORKSPACE, &workspace),
            ["alo-files".to_owned()],
            "the registry the verbs are declared into was read as a crate that \
             declares verbs, or the crate that declares them was missed"
        );
    }
}
