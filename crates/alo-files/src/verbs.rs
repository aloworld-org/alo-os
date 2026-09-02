//! The six file verbs, declared.
//!
//! `docs/features.md` promises list, read, find, rename, move and archive at
//! v0.01, over granted paths only, and `docs/contracts/agent-verbs.md` says
//! what a verb has to be. This file is those six and nothing else — every rule
//! about *how* a verb may be declared is [`alo_capability::verb`]'s, and this
//! file would not compile past its own tests if one of these broke one.
//!
//! **Nothing is declared by default.** [`file_verbs`] hands back a list;
//! [`declare_into`] puts them on somebody else's. A registry nobody gave these
//! to has no file capabilities, which is the honest starting state for a
//! machine where every capability is somebody's decision.
//!
//! # Two rules these six keep, that the contract does not require of everyone
//!
//! **Every path a file verb names is one its grant has to cover.** The
//! contract lets a verb require a grant over some of its paths; these require
//! one over all of them, and a test asserts it. [`crate::Touching`] enforces
//! the same rule at the moment of execution for every verb there is, so the
//! two cannot drift apart — but a verb that needed the enforcement to save it
//! would be a verb declared wrongly.
//!
//! **Every path a file verb names is something that is already there.** A
//! folder to list, a file to read, a folder to move into: all of them exist
//! before the call. Nothing here names the thing it is about to create — a new
//! name is a [`Takes::Name`], never a path — so resolving every path argument
//! is always a question with an answer, and "there is nothing there" is always
//! a refusal rather than an ordinary case to be worked around.
//!
//! # What "archive" means here
//!
//! *Make an archive*, not *move it to the archive folder*. The second is
//! `move_file` wearing a different name, and a closed list with two names for
//! one action is a list a model picks from at random. So `archive_folder`
//! makes one file out of a folder and puts it somewhere, and both places are
//! granted.

use alo_capability::{Arg, Effect, Requires, Takes, Verb, VerbError, Verbs, VerbsError};

/// The most characters a file's or folder's name may be.
///
/// The limit every mainstream filesystem shares. A verb that allowed more would
/// be offering something no disk here can store.
const LONGEST_NAME: usize = 255;

/// The most files one search may answer with.
///
/// A bound rather than no bound, because the answer is read by a person and
/// then by a model, and an unbounded answer is a way to fill both with a
/// folder somebody granted by accident.
const MOST_FOUND: i64 = 1000;

/// Why the file verbs could not be declared.
///
/// Neither of these can happen to the six as they are written — the tests in
/// this file are what say so. They are here because a `Result` that cannot fail
/// is still better than an unwrap in a library, and because [`declare_into`]
/// can genuinely fail against a list that already has one of these names.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Declaring {
    /// A declaration that does not satisfy the contract.
    #[error(transparent)]
    Verb(#[from] VerbError),
    /// A name already on the list.
    #[error(transparent)]
    List(#[from] VerbsError),
}

/// The file verbs, as a list of their own.
///
/// ```
/// use alo_capability::Given;
/// use alo_files::file_verbs;
///
/// let verbs = file_verbs()?;
/// let call = verbs.call("move_file", &[
///     ("file", Given::text("/home/anna/Invoices/march.pdf")),
///     ("into", Given::text("/home/anna/Archive")),
/// ])?;
///
/// assert_eq!(
///     call.sentence(),
///     "move /home/anna/Invoices/march.pdf into /home/anna/Archive",
/// );
/// assert!(call.waits_for_approval());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// [`Declaring`], which the six as written cannot cause.
pub fn file_verbs() -> Result<Verbs, Declaring> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the file verbs on an existing list.
///
/// All six or none of them. A name already taken is found before anything is
/// added, because a list holding half the file verbs would be a machine where
/// an agent can move a file and not read one, which is not a state anybody
/// chose.
///
/// # Errors
/// [`Declaring::List`] if the list already holds one of these names — a name
/// means one thing, so whoever took `move_file` first keeps it and nothing is
/// silently replaced.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    let declaring = [
        list_folder()?,
        read_file()?,
        find_in_folder()?,
        rename_file()?,
        move_file()?,
        archive_folder()?,
    ];
    for verb in &declaring {
        if verbs.of(verb.name()).is_some() {
            return Err(Declaring::List(VerbsError::AlreadyDeclared {
                name: verb.name().to_owned(),
            }));
        }
    }
    for verb in declaring {
        verbs.declare(verb)?;
    }
    Ok(())
}

/// List what is in a folder.
fn list_folder() -> Result<Verb, VerbError> {
    Verb::checked(
        "list_folder",
        "list what is in a folder",
        Effect::Read,
        vec![Arg::taking("folder", "the folder to list", Takes::Path)],
        Requires::grants_over(["folder"]),
        "list what is in {folder}",
    )
}

/// Read what is in a file.
fn read_file() -> Result<Verb, VerbError> {
    Verb::checked(
        "read_file",
        "read what is in a file",
        Effect::Read,
        vec![Arg::taking("file", "the file to read", Takes::Path)],
        Requires::grants_over(["file"]),
        "read what is in {file}",
    )
}

/// Find files in a folder by what they are called.
///
/// `named` is one name and not a path, and it is not an expression: the search
/// it describes is built inside the verb from a name, which is ADR 0001 §1 at
/// the place somebody would most reasonably ask for a pattern language.
fn find_in_folder() -> Result<Verb, VerbError> {
    Verb::checked(
        "find_in_folder",
        "find files in a folder by what they are called",
        Effect::Read,
        vec![
            Arg::taking("folder", "the folder to look in", Takes::Path),
            Arg::taking(
                "named",
                "the words a file's name has to contain",
                Takes::name(LONGEST_NAME),
            ),
            Arg::taking(
                "most",
                "how many files to answer with at most",
                Takes::count(1, MOST_FOUND),
            ),
        ],
        Requires::grants_over(["folder"]),
        "find up to {most} files in {folder} whose name contains {named}",
    )
}

/// Give a file a different name, where it already is.
fn rename_file() -> Result<Verb, VerbError> {
    Verb::checked(
        "rename_file",
        "give a file a different name, where it already is",
        Effect::Change,
        vec![
            Arg::taking("file", "the file to rename", Takes::Path),
            Arg::taking("name", "what to call it instead", Takes::name(LONGEST_NAME)),
        ],
        Requires::grants_over(["file"]),
        "rename {file} to {name}",
    )
}

/// Move a file into a folder.
fn move_file() -> Result<Verb, VerbError> {
    Verb::checked(
        "move_file",
        "move a file into a folder",
        Effect::Change,
        vec![
            Arg::taking("file", "the file to move", Takes::Path),
            Arg::taking("into", "the folder it goes into", Takes::Path),
        ],
        Requires::grants_over(["file", "into"]),
        "move {file} into {into}",
    )
}

/// Make one archive file out of a folder.
fn archive_folder() -> Result<Verb, VerbError> {
    Verb::checked(
        "archive_folder",
        "make one archive file out of a folder",
        Effect::Change,
        vec![
            Arg::taking("folder", "the folder to make an archive of", Takes::Path),
            Arg::taking("into", "the folder the archive goes into", Takes::Path),
            Arg::taking(
                "name",
                "what to call the archive",
                Takes::name(LONGEST_NAME),
            ),
        ],
        Requires::grants_over(["folder", "into"]),
        "make an archive of {folder} called {name}, in {into}",
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

    /// The six `docs/features.md` promised, and no seventh that arrived with
    /// them.
    #[test]
    fn the_list_is_the_six_the_feature_list_promised() {
        let verbs = file_verbs().unwrap();
        let names: Vec<_> = verbs.all().map(Verb::name).collect();
        assert_eq!(
            names,
            [
                "list_folder",
                "read_file",
                "find_in_folder",
                "rename_file",
                "move_file",
                "archive_folder",
            ]
        );
    }

    /// Reads answer and changes wait, and which is which is a property of the
    /// verb rather than of whoever calls it (ADR 0001 §5).
    #[test]
    fn looking_answers_and_changing_waits() {
        let verbs = file_verbs().unwrap();
        for looking in ["list_folder", "read_file", "find_in_folder"] {
            assert!(
                !verbs.of(looking).unwrap().effect().waits_for_approval(),
                "{looking}"
            );
        }
        for changing in ["rename_file", "move_file", "archive_folder"] {
            assert!(
                verbs.of(changing).unwrap().effect().waits_for_approval(),
                "{changing}"
            );
        }
    }

    /// **Every path a file verb names is one its grant has to cover.** The
    /// contract permits less than this; these six do not take it.
    #[test]
    fn every_path_a_file_verb_names_is_one_its_grant_covers() {
        for verb in file_verbs().unwrap().all() {
            let over: &[String] = match verb.requires() {
                Requires::Grants(over) => over,
                Requires::Nothing { .. } => &[],
            };
            assert!(!over.is_empty(), "{} requires no grant", verb.name());
            for arg in verb.args() {
                if arg.takes == Takes::Path {
                    assert!(
                        over.contains(&arg.name),
                        "{} does not require a grant over {}",
                        verb.name(),
                        arg.name
                    );
                }
            }
        }
    }

    /// The sentence a person approves, for each change there is. It is
    /// generated from the arguments and it names all of them — a verb that
    /// broke either rule could not have been declared at all, so this asserts
    /// what a person actually reads.
    #[test]
    fn what_a_person_would_be_approving_reads_as_a_sentence() {
        let verbs = file_verbs().unwrap();
        let rename = verbs
            .call(
                "rename_file",
                &[
                    ("file", Given::text("/home/anna/Invoices/march.pdf")),
                    ("name", Given::text("march-2026.pdf")),
                ],
            )
            .unwrap();
        assert_eq!(
            rename.sentence(),
            "rename /home/anna/Invoices/march.pdf to march-2026.pdf"
        );

        let archive = verbs
            .call(
                "archive_folder",
                &[
                    ("folder", Given::text("/home/anna/Invoices")),
                    ("into", Given::text("/home/anna/Archive")),
                    ("name", Given::text("invoices-2026.zip")),
                ],
            )
            .unwrap();
        assert_eq!(
            archive.sentence(),
            "make an archive of /home/anna/Invoices called invoices-2026.zip, in /home/anna/Archive"
        );
        assert_eq!(archive.asks().len(), 2, "the name is not a place");

        let find = verbs
            .call(
                "find_in_folder",
                &[
                    ("folder", Given::text("/home/anna/Invoices")),
                    ("named", Given::text("march")),
                    ("most", Given::number(20)),
                ],
            )
            .unwrap();
        assert_eq!(
            find.sentence(),
            "find up to 20 files in /home/anna/Invoices whose name contains march"
        );
    }

    /// **A new name cannot be a path.** Renaming is the verb where a path
    /// hidden in a name would move a file somewhere nobody granted, and it is
    /// refused at the door rather than by the grants.
    #[test]
    fn a_new_name_cannot_be_a_path_that_leads_somewhere_else() {
        let verbs = file_verbs().unwrap();
        for attempt in ["../../etc/shadow", "/etc/shadow", "sub/folder.pdf", ".."] {
            let err = verbs
                .call(
                    "rename_file",
                    &[
                        ("file", Given::text("/home/anna/Invoices/march.pdf")),
                        ("name", Given::text(attempt)),
                    ],
                )
                .unwrap_err();
            assert!(matches!(err, CallError::Argument(_)), "{attempt}: {err}");
            assert!(err.to_string().contains("one name"), "{attempt}: {err}");
        }
    }

    /// A search that is a command, or a count nobody offered, never becomes a
    /// call — the refusals happen before any grant is consulted.
    #[test]
    fn a_call_that_does_not_survive_the_door_is_not_a_call() {
        let verbs = file_verbs().unwrap();
        let too_many = verbs
            .call(
                "find_in_folder",
                &[
                    ("folder", Given::text("/home/anna/Invoices")),
                    ("named", Given::text("march")),
                    ("most", Given::number(MOST_FOUND + 1)),
                ],
            )
            .unwrap_err();
        assert!(too_many.to_string().contains("between 1 and"), "{too_many}");

        let not_a_path = verbs
            .call("list_folder", &[("folder", Given::text("Invoices"))])
            .unwrap_err();
        assert!(not_a_path.to_string().contains("full path"), "{not_a_path}");

        let missing = verbs
            .call(
                "move_file",
                &[("file", Given::text("/home/anna/Invoices/march.pdf"))],
            )
            .unwrap_err();
        assert!(missing.to_string().contains("into"), "{missing}");
    }

    /// The list is closed, and these six do not open it.
    #[test]
    fn nothing_that_runs_something_is_on_the_list() {
        let verbs = file_verbs().unwrap();
        for asked in ["run_command", "exec", "read_file_as_shell", "delete_folder"] {
            assert!(verbs.of(asked).is_none(), "{asked}");
            assert!(matches!(
                verbs.call(asked, &[]),
                Err(CallError::NoSuchVerb { .. })
            ));
        }
    }

    /// A list that already holds one of these names keeps its own, rather than
    /// having it replaced by something with the same name and a different
    /// meaning — and it gets none of the others either, so nothing ends up with
    /// half a set of file capabilities.
    #[test]
    fn a_name_already_taken_is_not_quietly_replaced_and_the_rest_do_not_arrive() {
        let mut verbs = file_verbs().unwrap();
        let again = declare_into(&mut verbs).unwrap_err();
        assert!(matches!(again, Declaring::List(_)), "{again}");
        assert!(again.to_string().contains("list_folder"), "{again}");
        assert_eq!(verbs.len(), 6);

        // The clash is on the fifth of the six, and the four before it are not
        // added on the way to finding it.
        let mut theirs = Verbs::default();
        declare_into(&mut theirs).unwrap();
        let mut mine = Verbs::default();
        mine.declare(theirs.of("move_file").unwrap().clone())
            .unwrap();
        let clash = declare_into(&mut mine).unwrap_err();
        assert!(clash.to_string().contains("move_file"), "{clash}");
        assert_eq!(mine.len(), 1);
        assert!(mine.of("list_folder").is_none());
    }

    /// Nothing arrives declared. A registry nobody gave these to has no file
    /// capabilities at all.
    #[test]
    fn nothing_is_declared_until_somebody_declares_it() {
        let empty = Verbs::default();
        assert!(empty.is_empty());
        assert!(empty.of("list_folder").is_none());

        let mut mine = Verbs::default();
        declare_into(&mut mine).unwrap();
        assert_eq!(mine.len(), 6);
    }
}
