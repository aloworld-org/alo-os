//! The search verb, declared.
//!
//! `docs/features.md` promises *search your own files, without asking
//! anything*, and says of the agent's *"where is that file?"* that it is a
//! nicer road to the index and never the only one. This file is that road,
//! declared the way `docs/contracts/agent-verbs.md` says a verb has to be
//! and the way `alo-files` declares its six: a name, a purpose, an effect,
//! typed arguments, the grant it requires and the sentence a person reads —
//! every rule about *how* is [`alo_capability::verb`]'s, and this file would
//! not compile past its own tests if the declaration broke one.
//!
//! **Nothing is declared by default.** [`finding_verbs`] hands back a list;
//! [`declare_into`] puts it on somebody else's. A registry nobody gave this
//! to cannot search anybody's index, which is the honest starting state.
//!
//! # One verb, and why it is a read over a granted folder
//!
//! `search_files` answers a question and changes nothing, so it is
//! [`Effect::Read`]: it runs inside the turn, nobody is asked to approve it,
//! and the record's entry has no approval to name. It still requires a grant,
//! over the folder whose index it searches — *read inside the turn* is about
//! approval and never about reach, and an index of a folder nobody granted
//! would be a way to read the names in it. The grant is over the folder as
//! the person picked it, so revoking that folder stops the search the same
//! instant it stops `list_folder`.
//!
//! # Why it is not `find_in_folder` again
//!
//! `alo-files`' `find_in_folder` walks the disk, bounded, and answers with
//! what it met. This verb asks the **index** — the thing the file manager
//! asks — and answers with what matched beside what the index does not hold,
//! without touching the folder at all; an index of a folder that has since
//! been unplugged still answers. Two verbs, two different facts about a
//! folder, and the purpose of each says which.
//!
//! # One axis, deliberately
//!
//! The index answers by name, kind, date and words; this verb asks by name.
//! Every argument of a verb is required, so a verb that also took words
//! would make every search by name carry a sentence of words as well, and a
//! verb per axis would be four names for one action on a list a model picks
//! from. *"Where is that file?"* is a question about a name. The other three
//! axes are the file manager's, and a second verb for words is a decision
//! for `docs/features.md` rather than for this file.
//!
//! # Where the words come from
//!
//! Every sentence in this file is [`crate::words`]'. The purpose, each
//! argument's purpose and the sentence a person reads are the constants a
//! translator is handed, passed to `alo_capability::Verb::checked` so that
//! the string being translated is the string that was checked.

use alo_capability::{Arg, Effect, Requires, Takes, Verb, VerbError, Verbs, VerbsError};

use crate::asking::A_NAME;
use crate::words;

/// Why the search verb could not be declared.
///
/// Neither can happen to the verb as it is written — the tests in this file
/// say so. It is a `Result` because a library that panics on its own
/// declaration takes the daemon with it, and because [`declare_into`] can
/// genuinely fail against a list that already holds the name.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Declaring {
    /// A declaration that does not satisfy the contract.
    #[error(transparent)]
    Verb(#[from] VerbError),
    /// A name already on the list.
    #[error(transparent)]
    List(#[from] VerbsError),
}

/// The search verb, as a list of its own.
///
/// ```
/// use alo_capability::Given;
/// use alo_finding::{finding_verbs, finding_words};
/// use alo_strings::Strings;
///
/// let verbs = finding_verbs()?;
/// let call = verbs.call("search_files", &[
///     ("folder", Given::text("/home/anna/Documents")),
///     ("named", Given::text("march")),
/// ]).expect("the verb takes a folder and part of a name");
///
/// let strings = Strings::of(finding_words()?);
/// assert_eq!(
///     call.sentence(&strings).text(),
///     "search the index of /home/anna/Documents for files whose name contains march",
/// );
/// // A read: it answers inside the turn, and nobody is asked to approve it.
/// assert!(!call.waits_for_approval());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// [`Declaring`], which the verb as written cannot cause.
pub fn finding_verbs() -> Result<Verbs, Declaring> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the search verb on an existing list.
///
/// # Errors
/// [`Declaring::List`] if the list already holds the name — a name means one
/// thing, so whoever took `search_files` first keeps it and nothing is
/// silently replaced.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    let searching = search_files()?;
    if verbs.of(searching.name()).is_some() {
        return Err(Declaring::List(VerbsError::AlreadyDeclared {
            name: searching.name().to_owned(),
        }));
    }
    verbs.declare(searching)?;
    Ok(())
}

/// Search the index of a folder for files by what they are called.
///
/// `named` is part of a name and not a path, and it is not an expression: the
/// query it becomes is built inside [`crate::Searched`] from a name, which is
/// ADR 0001 §1 at the place somebody would most reasonably ask for a pattern
/// language. Its length is [`A_NAME`], the bound the index's own refusal
/// uses, so a name the index would refuse never becomes a call.
fn search_files() -> Result<Verb, VerbError> {
    Verb::checked(
        "search_files",
        words::SEARCH_FILES,
        Effect::Read,
        vec![
            Arg::taking("folder", words::SEARCH_FILES_FOLDER, Takes::Path),
            Arg::taking("named", words::SEARCH_FILES_NAMED, Takes::name(A_NAME)),
        ],
        Requires::grants_over(["folder"]),
        words::SEARCH_FILES_SENTENCE,
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_capability::{CallError, Given};
    use alo_strings::Strings;

    /// This crate's words beside the capability model's, with nothing
    /// translated — a refusal at the door is worded by the deciding crate.
    fn in_english() -> Strings {
        let mut vocabulary = crate::finding_words().unwrap();
        alo_capability::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// A call of the verb, over this folder, for this part of a name.
    fn searching(folder: &str, named: &str) -> Result<alo_capability::Call, CallError> {
        finding_verbs().unwrap().call(
            "search_files",
            &[
                ("folder", Given::text(folder)),
                ("named", Given::text(named)),
            ],
        )
    }

    /// The list is the one verb, and nothing arrived with it.
    #[test]
    fn the_list_is_the_one_search_verb() {
        let verbs = finding_verbs().unwrap();
        let names: Vec<_> = verbs.all().map(Verb::name).collect();
        assert_eq!(names, ["search_files"]);
    }

    /// **A search is a read.** It runs inside the turn, nobody approves it,
    /// and that is a property of the verb rather than of whoever calls it.
    #[test]
    fn a_search_is_a_read_that_nobody_is_asked_to_approve() {
        let verbs = finding_verbs().unwrap();
        let verb = verbs.of("search_files").unwrap();
        assert_eq!(verb.effect(), Effect::Read);
        assert!(!verb.effect().waits_for_approval());
        assert!(
            !searching("/home/anna/Documents", "march")
                .unwrap()
                .waits_for_approval()
        );
    }

    /// **The grant is over the folder**, which is the one path the verb
    /// names, and a call asks the grants about exactly that.
    #[test]
    fn the_grant_is_over_the_folder_whose_index_is_searched() {
        let verbs = finding_verbs().unwrap();
        let verb = verbs.of("search_files").unwrap();
        assert_eq!(
            verb.requires(),
            &Requires::Grants(vec!["folder".to_owned()])
        );
        let call = searching("/home/anna/Documents", "march").unwrap();
        assert_eq!(
            call.asks(),
            &[alo_capability::Ask::path("/home/anna/Documents")]
        );
    }

    /// The sentence a person reads while it runs names both arguments.
    #[test]
    fn what_a_person_reads_is_a_sentence_naming_the_folder_and_the_name() {
        let said = searching("/home/anna/Documents", "march")
            .unwrap()
            .sentence(&in_english());
        assert_eq!(
            said.text(),
            "search the index of /home/anna/Documents for files whose name contains march"
        );
        assert!(!said.is_a_bug());
        assert!(said.unfilled().is_empty());
    }

    /// A call that does not survive the door is not a call: a folder that is
    /// not a full path, a name that is a path, an empty name, a name longer
    /// than any name — each refused before any grant or index is consulted.
    #[test]
    fn a_call_that_does_not_survive_the_door_is_not_a_call() {
        let strings = in_english();
        let relative = searching("Documents", "march").unwrap_err();
        assert!(matches!(relative, CallError::Argument(_)), "{relative:?}");
        assert!(relative.said(&strings).text().contains("full path"));

        for attempt in ["../../etc", "/etc/shadow", "sub/march.pdf", ".."] {
            let err = searching("/home/anna/Documents", attempt).unwrap_err();
            assert!(matches!(err, CallError::Argument(_)), "{attempt}: {err:?}");
            assert!(
                err.said(&strings).text().contains("one name"),
                "{attempt}: {}",
                err.said(&strings)
            );
        }

        let empty = searching("/home/anna/Documents", "   ").unwrap_err();
        assert!(matches!(empty, CallError::Argument(_)), "{empty:?}");

        let too_long = searching("/home/anna/Documents", &"m".repeat(A_NAME + 1)).unwrap_err();
        assert!(matches!(too_long, CallError::Argument(_)), "{too_long:?}");
        assert!(
            searching("/home/anna/Documents", &"m".repeat(A_NAME)).is_ok(),
            "the bound is the index's own, and a name at it is a name"
        );

        let missing = finding_verbs()
            .unwrap()
            .call("search_files", &[("folder", Given::text("/home/anna"))])
            .unwrap_err();
        assert!(missing.said(&strings).text().contains("named"));
    }

    /// The list is closed, and this verb does not open it.
    #[test]
    fn nothing_that_runs_something_is_on_the_list() {
        let verbs = finding_verbs().unwrap();
        for asked in [
            "search_by_expression",
            "exec",
            "find_in_folder",
            "read_file",
        ] {
            assert!(verbs.of(asked).is_none(), "{asked}");
            assert!(matches!(
                verbs.call(asked, &[]),
                Err(CallError::NoSuchVerb { .. })
            ));
        }
    }

    /// A list that already holds the name keeps its own.
    #[test]
    fn a_name_already_taken_is_not_quietly_replaced() {
        let mut verbs = finding_verbs().unwrap();
        let again = declare_into(&mut verbs).unwrap_err();
        assert!(matches!(again, Declaring::List(_)), "{again}");
        assert!(again.to_string().contains("search_files"), "{again}");
        assert_eq!(verbs.len(), 1);
    }

    /// The verb sits beside the file verbs on one list, which is the
    /// arrangement a daemon has: one registry, every crate declaring into it.
    #[test]
    fn it_joins_a_list_that_already_holds_the_file_verbs() {
        let mut verbs = alo_files::file_verbs().unwrap();
        declare_into(&mut verbs).unwrap();
        assert_eq!(verbs.len(), 7);
        assert!(verbs.of("search_files").is_some());
        assert!(verbs.of("find_in_folder").is_some());
    }

    /// **Every word the verb is declared with is one this crate declares.**
    /// A constant left out of [`crate::words`]' list would compile, declare
    /// and reach a person as a key in the place where the sentence belongs.
    #[test]
    fn everything_the_verb_says_is_something_this_crate_declares() {
        let strings = in_english();
        for verb in finding_verbs().unwrap().all() {
            let purpose = verb.purpose(&strings);
            assert!(!purpose.is_a_bug(), "{}: {purpose}", verb.name());
            for arg in verb.args() {
                let said = arg.purpose(&strings);
                assert!(!said.is_a_bug(), "{} {}: {said}", verb.name(), arg.name());
            }
            assert_eq!(verb.sentence().key(), &words::SEARCH_FILES_SENTENCE.key());
        }
    }

    /// Nothing arrives declared.
    #[test]
    fn nothing_is_declared_until_somebody_declares_it() {
        let empty = Verbs::default();
        assert!(empty.of("search_files").is_none());
        let mut mine = Verbs::default();
        declare_into(&mut mine).unwrap();
        assert_eq!(mine.len(), 1);
    }
}
