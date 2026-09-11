//! What the check found, in sentences whoever is adding a crate acts on.
//!
//! Every finding names the crate it is about and says what to do about it. That
//! is not politeness: this check fails in the change that adds the crate, which
//! is the one moment somebody knows whether its words belong in the machine's
//! vocabulary — and *the vocabulary is out of date* would send them to read a
//! file from the top rather than to write one line.

/// One thing wrong between the crates of this workspace that declare words and
/// the vocabulary they are collected into.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Finding {
    /// A crate that declares words and nothing collects.
    ///
    /// **The one this check exists for**, and it is not hypothetical:
    /// `crates/alo-overlay` declared nine strings that `alo-saying` collected
    /// nothing of, and the only reason a shell would not have shown a key where
    /// a sentence belongs is that a person happened to notice.
    #[error(
        "`{crate_name}` declares words in src/words.rs and nothing collects them, so every sentence \
         in it reaches a person as a missing key — in English and in every language somebody has \
         translated. Declare it in crates/alo-saying's `everything_this_machine_can_say`, or name it \
         apart with the reason it cannot be collected"
    )]
    ACrateNothingCollects {
        /// The crate that declares words.
        crate_name: String,
    },

    /// A crate the vocabulary collects that no longer declares any words.
    ///
    /// A dead entry is how a list stops meaning anything. Left alone, the list
    /// beside `alo-saying` becomes a record of a workspace that has moved on,
    /// and the day it is wrong about a crate that *does* declare words nobody
    /// believes it either way.
    #[error(
        "crates/alo-saying collects `{crate_name}`, and no crate of that name in this workspace \
         declares words in src/words.rs. If it was renamed, rename the entry; if its words went \
         somewhere else, the entry goes with them; if it has none left, take it off the list — a \
         list with a dead name in it is a list nobody can read as a check"
    )]
    ACollectedCrateThatSaysNothing {
        /// The name the vocabulary used.
        crate_name: String,
    },

    /// A crate named apart that declares no words.
    ///
    /// An exception is a standing permission to say nothing. One that is not
    /// needed is one nobody will read again, and it will still be there the day
    /// a crate of that name declares words.
    #[error(
        "`{crate_name}` is named as deliberately outside the one vocabulary, and no crate of that \
         name in this workspace declares words at all. Take it off the list: an exception nobody \
         needs is one that is still standing on the day somebody does declare words there, and then \
         it is silence with a reason attached"
    )]
    AnExceptionNobodyNeeds {
        /// The name the exception used.
        crate_name: String,
    },

    /// A crate named apart with no reason worth reading.
    ///
    /// The difference between a documented exception and the failure this crate
    /// exists to catch is entirely the sentence.
    #[error(
        "`{crate_name}` is named as deliberately outside the one vocabulary and the reason given is \
         `{said}`. Say what would break if it were collected: a name on this list with a shrug \
         beside it is a crate that says nothing to anybody in any language, which is the failure \
         this check is for, with permission"
    )]
    AnExceptionWithNoReason {
        /// The crate standing apart.
        crate_name: String,
        /// What was said instead of a reason.
        said: String,
    },

    /// A crate both collected and named apart.
    ///
    /// The two lists contradict each other, and which one is true would depend
    /// on which one a reader opened.
    #[error(
        "`{crate_name}` is both collected into the one vocabulary and named as deliberately outside \
         it. One or the other: while both stand, whether this crate's words reach a person depends \
         on which list somebody reads"
    )]
    ACrateBothCollectedAndApart {
        /// The crate on both lists.
        crate_name: String,
    },

    /// A crate named twice on one list.
    #[error(
        "`{crate_name}` is named {times} times on {list}. Once each: a list that counts a crate twice \
         cannot be counted against the workspace, and the count is the only thing that says nothing \
         was quietly dropped"
    )]
    ACrateNamedTwice {
        /// The crate named more than once.
        crate_name: String,
        /// Which list names it twice.
        list: &'static str,
        /// How many times it appears.
        times: usize,
    },

    /// A workspace this check could not walk.
    ///
    /// Everything else here is about one crate. This is the failure that would
    /// otherwise be silent: a manifest that could not be read, or a convention
    /// that moved, leaving an empty list of crates that declare words — which is
    /// indistinguishable from a workspace where nothing says anything, and one
    /// of those is a check and the other is a green light.
    #[error(
        "`{manifest}` named no crate that declares words, and this workspace has more than twenty. \
         Either it could not be read or the convention moved — `src/words.rs` with a `pub fn \
         declare_into` — and either way nothing was checked for saying something nobody collects"
    )]
    NoWorkspaceToWalk {
        /// Where the workspace was looked for.
        manifest: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holding::THE_VOCABULARY;

    /// The finding this crate exists for names the crate, the file it read and
    /// both ways out, because it is read by whoever has just written that crate
    /// and has one line to add.
    #[test]
    fn a_finding_names_the_crate_and_what_to_do() {
        let said = Finding::ACrateNothingCollects {
            crate_name: "alo-overlay".to_owned(),
        }
        .to_string();
        assert!(said.contains("alo-overlay"), "{said}");
        assert!(said.contains("src/words.rs"), "{said}");
        assert!(said.contains("crates/alo-saying"), "{said}");
        assert!(said.contains("apart"), "{said}");
    }

    /// An exception refused for its reason quotes the reason, so whoever wrote
    /// it can see what was read.
    #[test]
    fn an_exception_refused_for_its_reason_quotes_it() {
        let said = Finding::AnExceptionWithNoReason {
            crate_name: "alo-agentd".to_owned(),
            said: "it is Linux".to_owned(),
        }
        .to_string();
        assert!(said.contains("alo-agentd"), "{said}");
        assert!(said.contains("it is Linux"), "{said}");
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

        let twice = Finding::ACrateNamedTwice {
            crate_name: "alo-dock".to_owned(),
            list: THE_VOCABULARY,
            times: 2,
        }
        .to_string();
        assert!(twice.contains(THE_VOCABULARY), "{twice}");
    }
}
