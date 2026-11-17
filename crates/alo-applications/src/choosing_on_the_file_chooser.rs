//! **`choose_on_the_file_chooser`**: alo answers the file chooser an
//! application has up, with a file in a folder the person granted — the
//! coordinator's approval of 2026-11-17, on the owner's decision that alo OS
//! has its own file chooser and *alo can answer it only inside folders the
//! person granted: one card at "ask each time", none at full access*.
//!
//! # A folder and a name, not a path
//!
//! A file chosen to save under does not exist yet, so it cannot be a path the
//! grants resolve. The verb takes the place as the file verbs that make
//! something take it (`archive_folder`'s `into` and `name`): **the folder**, a
//! path the grants are judged over exactly as the file verbs' are, and **one
//! name** in it. Where the file is already there, the turn asks the grants
//! about where it really leads too, so a link out of a granted folder is
//! refused as it is for every file verb (`alo_turn`'s
//! `choosing_on_the_file_chooser`).
//!
//! # A change, so the person approves it
//!
//! Answering the sheet hands a file to an application, so it is a change:
//! one card at *ask me each time*, none at full access — the same rule every
//! change follows, decided where every change's is.

use alo_capability::{Arg, Effect, Requires, Takes, Verb, VerbError, Verbs, VerbsError};

use crate::verbs::Declaring;
use crate::words;

/// The verb's name, which a model sends and the record keeps.
pub const CHOOSE_ON_THE_FILE_CHOOSER: &str = "choose_on_the_file_chooser";

/// The argument naming the folder the file is in.
pub const FOLDER: &str = "folder";

/// The argument naming the file in it.
pub const NAME: &str = "name";

/// The longest name one file may have, in bytes.
pub const LONGEST_NAME: usize = 255;

/// Put `choose_on_the_file_chooser` on a list.
///
/// # Errors
/// [`Declaring::List`] if the list already holds the name — nothing is
/// silently replaced.
pub fn declare_choosing_on_the_file_chooser_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    let verb = choose_on_the_file_chooser()?;
    if verbs.of(verb.name()).is_some() {
        return Err(Declaring::List(VerbsError::AlreadyDeclared {
            name: verb.name().to_owned(),
        }));
    }
    verbs.declare(verb)?;
    Ok(())
}

/// Answer the file chooser that is up.
fn choose_on_the_file_chooser() -> Result<Verb, VerbError> {
    Verb::checked(
        CHOOSE_ON_THE_FILE_CHOOSER,
        words::CHOOSE_ON_THE_FILE_CHOOSER,
        Effect::Change,
        vec![
            Arg::taking(FOLDER, words::CHOOSE_ON_THE_FILE_CHOOSER_FOLDER, Takes::Path),
            Arg::taking(
                NAME,
                words::CHOOSE_ON_THE_FILE_CHOOSER_NAME,
                Takes::name(LONGEST_NAME),
            ),
        ],
        Requires::grants_over([FOLDER]),
        words::CHOOSE_ON_THE_FILE_CHOOSER_SENTENCE,
    )?
    .changing(
        words::CHOOSE_ON_THE_FILE_CHOOSER_CHANGES,
        Some(words::CHOOSE_ON_THE_FILE_CHOOSER_LEAVES),
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A change over a granted folder**, taking a folder and one name, and
    /// declared once.
    #[test]
    fn it_is_a_change_over_a_granted_folder_and_declares_once() {
        let mut verbs = Verbs::default();
        declare_choosing_on_the_file_chooser_into(&mut verbs).unwrap();
        let verb = verbs.of(CHOOSE_ON_THE_FILE_CHOOSER).unwrap();
        assert_eq!(verb.effect(), Effect::Change);
        assert!(declare_choosing_on_the_file_chooser_into(&mut verbs).is_err());
    }
}
