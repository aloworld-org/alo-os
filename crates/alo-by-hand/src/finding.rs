//! What the check found, in sentences whoever is adding a verb acts on.
//!
//! Every finding names the verb or the crate it is about and says what to do
//! about it. That is not politeness: this check fails in the change that adds a
//! verb, which is the one moment somebody knows what a person would do instead —
//! and *the by-hand document is out of date* would send them to read two
//! documents from the top rather than to write one sentence.

/// One thing wrong between the verbs alo OS ships and `docs/by-hand.md`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Finding {
    /// A verb the document says nothing about.
    ///
    /// **The one this check exists for.** ADR 0009's rule has been a sentence in
    /// a document since 2026-09-02, so a verb arriving with no by-hand answer
    /// broke nothing and nobody was told.
    #[error(
        "alo OS ships the verb `{verb}` and docs/by-hand.md says nothing about it. Name how a \
         person does the same thing without the agent, quoting the promise in docs/features.md \
         that gives them the surface — or say what is owed and which release owns the answer. A \
         verb that is the only way to do something is a capability that disappears when somebody's \
         card is declined (ADR 0009)"
    )]
    AVerbNobodyAnswered {
        /// The verb's name, as the machine declares it.
        verb: String,
    },

    /// An entry about a verb this machine does not declare.
    ///
    /// Either the verb was renamed — in which case the by-hand answer needs
    /// looking at again, by whoever renamed it — or it was withdrawn, in which
    /// case the entry goes with it.
    #[error(
        "docs/by-hand.md answers about `{verb}`, and no verb of that name is declared anywhere in \
         this workspace. If it was renamed, rename the entry and read the answer again while you \
         are there; if it was withdrawn, the entry goes with it"
    )]
    AnEntryAboutNoVerb {
        /// The name the entry used.
        verb: String,
    },

    /// A verb two entries answer about.
    #[error(
        "two entries in docs/by-hand.md answer about `{verb}`. One verb, one entry — two answers \
         about one verb is two people's knowledge of it, and the shorter answer is the one that \
         gets read"
    )]
    AVerbAnsweredTwice {
        /// The verb's name.
        verb: String,
    },

    /// A verb with no plain way and nothing owed.
    #[error(
        "docs/by-hand.md's entry for `{verb}` names no plain way and says nothing is owed. One or \
         the other: a verb with neither is a capability nobody can show a person how to reach and \
         nobody has admitted to"
    )]
    AVerbWithNeither {
        /// The verb's name.
        verb: String,
    },

    /// A by-hand answer that quotes nothing.
    ///
    /// A sentence on its own is a sentence. What makes it an answer is that the
    /// surface it describes is somewhere in the definition with a tier on it —
    /// otherwise the plain way is unscheduled, ungated and nobody's.
    #[error(
        "docs/by-hand.md says a person does `{verb}` by hand and quotes nothing from \
         docs/features.md. Quote the promise that gives them the surface, in backticks: a plain way \
         the definition does not promise is not scheduled, and ADR 0009 says that is missing work \
         rather than an acceptable gap"
    )]
    AnAnswerNamingNoSurface {
        /// The verb's name.
        verb: String,
    },

    /// A by-hand answer quoting something the definition does not promise.
    #[error(
        "docs/by-hand.md answers `{verb}` with `{quoted}`, and no promise in docs/features.md says \
         that. If the promise was reworded, quote it as it stands now; if it was withdrawn, this \
         verb has lost its plain way and that is the finding"
    )]
    AWayNothingPromises {
        /// The verb's name.
        verb: String,
        /// What the entry quoted.
        quoted: String,
    },

    /// A quotation that fits more than one promise, so no release owns it.
    ///
    /// Left alone, the release that owes a person their plain way would depend on
    /// which line the check reached first.
    #[error(
        "docs/by-hand.md answers `{verb}` with `{quoted}`, and {promises} promises in \
         docs/features.md carry those words. Quote enough of one of them to name it: the release \
         that owns this answer is read off the line, so a quotation fitting two lines has two \
         answers"
    )]
    AWayPromisedMoreThanOnce {
        /// The verb's name.
        verb: String,
        /// What the entry quoted.
        quoted: String,
        /// How many promises carry it.
        promises: usize,
    },

    /// An owed answer that names no release.
    #[error(
        "docs/by-hand.md says the plain way to do `{verb}` is owed and does not say by which \
         release. Name it in brackets first, like `[v0.5]`: a debt with no release on it is a debt \
         nobody has taken on, and the point of writing it down is that somebody has"
    )]
    AnOwedAnswerWithNoRelease {
        /// The verb's name.
        verb: String,
    },

    /// An owed answer naming a release the definition does not use.
    #[error(
        "docs/by-hand.md owes the plain way to do `{verb}` at `{named}`, and docs/features.md makes \
         no promises for a release of that name. The tiers are the definition's; a verb owed at a \
         release nobody ships is owed at nothing"
    )]
    AReleaseNobodyShips {
        /// The verb's name.
        verb: String,
        /// The release the entry named.
        named: String,
    },

    /// An answer said in a way nobody could act on.
    #[error(
        "docs/by-hand.md answers `{verb}` with `{said}`. That is a shrug rather than an answer: say \
         what a person actually does, or what is missing and why, the way the entries around it do"
    )]
    AShrugRatherThanAnAnswer {
        /// The verb's name.
        verb: String,
        /// What the entry said.
        said: String,
    },

    /// A crate declaring verbs that this check was never handed.
    ///
    /// The failure a list of verb names could never catch, and the one that
    /// really happened one floor down: a new crate whose declarations nothing
    /// collected.
    #[error(
        "`{crate_name}` declares verbs in src/verbs.rs and this check was not handed them, so every \
         verb in it could arrive with no by-hand answer and nothing would notice. Add it where \
         crates/alo-by-hand's own test builds the list of what alo OS ships"
    )]
    AVerbListNobodyHandedIn {
        /// The crate that declares verbs.
        crate_name: String,
    },

    /// A document with no entries, against a machine that ships verbs.
    ///
    /// Everything else here is about one verb. This is the failure that would
    /// otherwise be silent: a heading that moved, a file that was replaced, and a
    /// check passing over a document it never found.
    #[error(
        "alo OS ships {verbs} verb(s) and docs/by-hand.md has no entries at all, so this check is \
         holding nothing. The entries live under `{heading}`; if that heading moved, this crate \
         moves with it"
    )]
    NothingToCheck {
        /// How many verbs went unanswered.
        verbs: usize,
        /// The heading the entries are read from.
        heading: &'static str,
    },

    /// A workspace this check could not walk.
    ///
    /// Same shape as [`Finding::NothingToCheck`] and the same reason: an empty
    /// list of crates is indistinguishable from a workspace where nothing
    /// declares a verb, and one of those is a check and the other is a green
    /// light.
    #[error(
        "`{manifest}` named no crate that declares verbs, and this workspace has two. Either it \
         could not be read or the convention moved — `src/verbs.rs` with a `pub fn declare_into` — \
         and either way nothing was checked for declaring verbs behind this check's back"
    )]
    NoWorkspaceToWalk {
        /// Where the workspace was looked for.
        manifest: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::THE_ANSWERS;

    /// Every finding names the verb it is about and what to do next, because it
    /// is read by whoever has just added a verb and has to write one sentence.
    #[test]
    fn a_finding_names_the_verb_and_what_to_do() {
        let said = Finding::AVerbNobodyAnswered {
            verb: "delete_folder".to_owned(),
        }
        .to_string();
        assert!(said.contains("delete_folder"), "{said}");
        assert!(said.contains("docs/by-hand.md"), "{said}");
        assert!(said.contains("ADR 0009"), "{said}");
    }

    /// And the two silent failures name where to look, because there is no verb
    /// to name in either.
    #[test]
    fn the_silent_failures_name_the_file_they_could_not_read() {
        let empty = Finding::NothingToCheck {
            verbs: 10,
            heading: THE_ANSWERS,
        }
        .to_string();
        assert!(empty.contains(THE_ANSWERS), "{empty}");

        let workspace = Finding::NoWorkspaceToWalk {
            manifest: "Cargo.toml",
        }
        .to_string();
        assert!(workspace.contains("Cargo.toml"), "{workspace}");
    }
}
