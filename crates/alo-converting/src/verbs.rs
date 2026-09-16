//! The converting verb, declared.
//!
//! ADR 0039 §2: **`convert_document(file, into)`**, a change — it writes a new
//! file — with grants required over both arguments. `file` is the document;
//! `into` is the folder the copy is put in. The copy's name is the document's
//! own with `.pdf` in place of its ending, and it is never chosen by the model:
//! a name an agent invented would be a place in a folder nobody picked.
//!
//! **The grant covers where the copy goes, not only the folder**: the executor
//! asks the grants once more about the copy's full path before creating it,
//! the way `alo-files` asks about everything a change would create.

use alo_capability::{Arg, Effect, Requires, Takes, Verb, VerbError, Verbs, VerbsError};

use crate::words;

/// The name the verb is asked for by.
pub const CONVERT_DOCUMENT: &str = "convert_document";

/// The argument naming the document.
pub const FILE: &str = "file";

/// The argument naming the folder the copy is put in.
pub const INTO: &str = "into";

/// Why the converting verb could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Declaring {
    /// A declaration that does not satisfy the contract.
    #[error(transparent)]
    Verb(#[from] VerbError),
    /// A name already on the list.
    #[error(transparent)]
    List(#[from] VerbsError),
}

/// The converting verb, as a list of its own.
///
/// # Errors
/// [`Declaring`], which the verb as written cannot cause.
pub fn converting_verbs() -> Result<Verbs, Declaring> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the converting verb on an existing list.
///
/// # Errors
/// [`Declaring::List`] if the list already has a verb of this name.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    verbs.declare(convert_document()?)?;
    Ok(())
}

/// Convert a document into a PDF copy in a folder.
fn convert_document() -> Result<Verb, VerbError> {
    Verb::checked(
        CONVERT_DOCUMENT,
        words::VERB_PURPOSE,
        Effect::Change,
        vec![
            Arg::taking(FILE, words::VERB_FILE, Takes::Path),
            Arg::taking(INTO, words::VERB_INTO, Takes::Path),
        ],
        Requires::grants_over([FILE, INTO]),
        words::VERB_SENTENCE,
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_capability::Given;

    /// **One verb, a change, over a granted document and a granted folder.**
    #[test]
    fn converting_is_one_change_over_two_granted_paths() {
        let verbs = converting_verbs().unwrap();
        assert_eq!(verbs.len(), 1);
        let verb = verbs.of(CONVERT_DOCUMENT).unwrap();
        assert_eq!(verb.effect(), Effect::Change);
        assert_eq!(verb.requires(), &Requires::grants_over([FILE, INTO]));
    }

    /// **A relative path is not a document**, and a name already taken is not
    /// replaced.
    #[test]
    fn a_relative_path_and_a_second_declaration_are_refused() {
        let mut verbs = converting_verbs().unwrap();
        assert!(
            verbs
                .call(
                    CONVERT_DOCUMENT,
                    &[
                        (FILE, Given::text("report.docx")),
                        (INTO, Given::text("/home/anna/Documents"))
                    ]
                )
                .is_err()
        );
        assert!(matches!(declare_into(&mut verbs), Err(Declaring::List(_))));
    }
}
