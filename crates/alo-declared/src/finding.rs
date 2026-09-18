//! What the check found, in sentences whoever is adding a crate acts on.
//!
//! Every finding names the crate it is about and the one file to add it in.
//! That is not politeness: this check fails in the change that adds the crate,
//! and the person reading it has just written a `src/verbs.rs` and has one entry
//! to write — a name paired with its declaration function. *The verb list is out of date*
//! would send them to read three files from the top instead, which is what the
//! three lanes refused on 2026-09-17 each had to do.

/// One thing wrong between the crates of this workspace that declare verbs and
/// the one list that hands them to the checks.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Finding {
    /// A crate that declares verbs and is on nobody's list.
    ///
    /// **The one this check exists for.** Every verb in that crate would be
    /// invisible to `alo-by-hand`, so none of them would be asked how a person
    /// does the same thing without the agent (ADR 0009), and invisible to
    /// `alo-software`, so none of them would be asked whether it reaches the
    /// terminal (ADR 0043) — and both checks would go on passing in exactly the
    /// same colour.
    #[error(
        "`{crate_name}` declares verbs in src/verbs.rs and nothing hands them to the checks every \
         verb alo OS ships is held to, so not one of them is asked how a person does the same \
         thing without the agent (ADR 0009) or whether it reaches the terminal (ADR 0043). Add it \
         to crates/alo-declared/src/shipped.rs — one entry deriving `WHO_DECLARES_THEM` and the \
         `declare_into` call beside it — and to that crate's Cargo.toml"
    )]
    ACrateNothingHandsIn {
        /// The crate that declares verbs.
        crate_name: String,
    },

    /// A crate on the one list that no longer declares any verbs.
    ///
    /// A dead entry is how a list stops meaning anything. Left alone, the list
    /// becomes a record of a workspace that has moved on, and the day it is
    /// wrong about a crate that *does* declare verbs nobody believes it either
    /// way.
    #[error(
        "crates/alo-declared names `{crate_name}` among the crates whose verbs alo OS ships, and no \
         crate of that name in this workspace declares verbs in src/verbs.rs. If it was renamed, \
         rename the entry and its call; if its verbs went somewhere else, the entry goes with them; \
         if it has none left, take it off — a list with a dead name in it is a list nobody can read \
         as a check"
    )]
    AListedCrateWithNoVerbs {
        /// The name the list used.
        crate_name: String,
    },

    /// A crate named twice on the one list.
    #[error(
        "`{crate_name}` is named {times} times among the crates whose verbs alo OS ships. Once each: \
         a list that counts a crate twice cannot be counted against the workspace, and the count is \
         the only thing that says nothing was quietly dropped"
    )]
    ACrateNamedTwice {
        /// The crate named more than once.
        crate_name: String,
        /// How many times it appears.
        times: usize,
    },

    /// A workspace this check could not walk.
    ///
    /// Everything else here is about one crate. This is the failure that would
    /// otherwise be silent: a manifest that could not be read, or a convention
    /// that moved, leaving an empty list of crates that declare verbs — which is
    /// indistinguishable from a workspace where nothing declares any, and one of
    /// those is a check and the other is a green light.
    #[error(
        "`{manifest}` named no crate that declares verbs, although this check requires a nonempty workspace. Either it \
         could not be read or the convention moved — `src/verbs.rs` with a `pub fn declare_into` — \
         and either way nothing was checked for declaring verbs nobody hands in"
    )]
    NoWorkspaceToWalk {
        /// Where the workspace was looked for.
        manifest: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shipped::WHERE_THE_LIST_IS;

    /// The finding this crate exists for names the crate, the convention it read
    /// and **the one file to add it in**, because it is read by whoever has just
    /// written that crate and has one entry to write.
    #[test]
    fn a_finding_names_the_crate_and_the_file_to_add_it_in() {
        let said = Finding::ACrateNothingHandsIn {
            crate_name: "alo-capturing".to_owned(),
        }
        .to_string();
        assert!(said.contains("alo-capturing"), "{said}");
        assert!(said.contains("src/verbs.rs"), "{said}");
        assert!(said.contains(WHERE_THE_LIST_IS), "{said}");
        assert!(said.contains("WHO_DECLARES_THEM"), "{said}");
    }

    /// A dead name names itself and all three ways out, because which one is
    /// right is knowledge the reader has and this check does not.
    #[test]
    fn a_dead_name_names_itself_and_what_to_do() {
        let said = Finding::AListedCrateWithNoVerbs {
            crate_name: "alo-nothing".to_owned(),
        }
        .to_string();
        assert!(said.contains("alo-nothing"), "{said}");
        assert!(said.contains("renamed"), "{said}");
    }

    /// And the silent failure names where it looked, because there is no crate
    /// to name in it.
    #[test]
    fn the_silent_failure_names_the_file_it_could_not_read() {
        let workspace = Finding::NoWorkspaceToWalk {
            manifest: "Cargo.toml",
        }
        .to_string();
        assert!(workspace.contains("Cargo.toml"), "{workspace}");
        assert!(workspace.contains("src/verbs.rs"), "{workspace}");

        let twice = Finding::ACrateNamedTwice {
            crate_name: "alo-files".to_owned(),
            times: 2,
        }
        .to_string();
        assert!(twice.contains("alo-files"), "{twice}");
        assert!(twice.contains('2'), "{twice}");
    }
}
