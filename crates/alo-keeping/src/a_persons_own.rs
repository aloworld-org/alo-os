//! **Where a person's own commands are written down** — the one file `alo-try`
//! and `alo-app` both write, named once.
//!
//! Here rather than in `alo-try`, because `alo-app`'s words are read by the
//! shell through `alo-saying`, and a road from the shell to `alo-try` reaches
//! `alo-bounding`, whose build compiles the kernel half the image's compositor
//! step cannot link (`alo-desktop`'s `the_desktop_is_built_without_the_agent_service.rs`).

use std::path::{Path, PathBuf};

/// Where a person's own runs are written down.
///
/// **Their own file, under their own state folder**, which ADR 0125's amendment
/// settles and `docs/contracts/record-file.md` already prescribes the shape of:
/// *two files, because a record file has one writer.* That contract's own answer
/// to a new writer is a new file, and this is it — written by the person's own
/// commands rather than by a service.
///
/// **Not `/var/lib/alo/record.jsonl`.** That file has one writer and it is
/// `alo-agentd`. Appending to it from a second process is the thing the rule
/// forbids, and it is forbidden for a reason that has nothing to do with
/// permission: on a shipped machine the person and the daemon are the same
/// account, so the permission is there and the rule is still right.
#[must_use]
pub fn the_record_a_person_keeps(home: &Path) -> PathBuf {
    home.join(".local")
        .join("state")
        .join("alo")
        .join("commands.jsonl")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_persons_own_record_is_under_their_own_state_folder() {
        assert_eq!(
            the_record_a_person_keeps(Path::new("/home/anna")),
            Path::new("/home/anna/.local/state/alo/commands.jsonl")
        );
    }
}
