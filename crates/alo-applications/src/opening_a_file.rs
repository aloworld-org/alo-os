//! **`open_file_in_application`**: open a file the person names in an
//! application they name — *open hello.txt in VSCodium*.
//!
//! *The owner, 2026-11-14: "Yes, add it."* Declared here, beside the three
//! application acts, because it is one of them with a file: what reaches the
//! machine is still only an installed application, started from its own
//! entry, and the one thing the agent adds is the path — a granted file, or at
//! full access any file the person's account can open (ADR 0149).
//!
//! # A change, so it waits for one approval
//!
//! Opening a file puts a window in front of the person and hands an application
//! their file, so it is proposed with *open {path} in {application}* and waits
//! for one OK at *ask me each time*, as every change does. The grant is over
//! **both** arguments: the application, as for every application act, and the
//! path, as for every file verb.
//!
//! # Where it opens is not an argument
//!
//! At the default it opens in **alo's own copy** of the application (ADR 0143
//! Q6, ADR 0153), so what alo types into it next is typed there and what that
//! copy sends is counted on its own; where the person chose *alo may work in
//! your own window*, it opens as their own press would. That is the person's
//! setting, read by the desktop at the moment it opens — never a word the
//! model sends, so there is no argument for it.

use alo_capability::{Arg, Effect, Requires, Takes, Verb, VerbError, Verbs, VerbsError};

use crate::verbs::Declaring;
use crate::words;

/// The verb's name, which a model sends and the record keeps.
pub const OPEN_FILE_IN_APPLICATION: &str = "open_file_in_application";

/// Put `open_file_in_application` on a list.
///
/// # Errors
/// [`Declaring::List`] if the list already holds the name — nothing is
/// silently replaced.
pub fn declare_opening_a_file_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    let verb = open_file_in_application()?;
    if verbs.of(verb.name()).is_some() {
        return Err(Declaring::List(VerbsError::AlreadyDeclared {
            name: verb.name().to_owned(),
        }));
    }
    verbs.declare(verb)?;
    Ok(())
}

/// Open a file in an application.
fn open_file_in_application() -> Result<Verb, VerbError> {
    Verb::checked(
        OPEN_FILE_IN_APPLICATION,
        words::OPEN_FILE_IN_APPLICATION,
        Effect::Change,
        vec![
            Arg::taking(
                "application",
                words::OPEN_FILE_IN_APPLICATION_APPLICATION,
                Takes::Application,
            ),
            Arg::taking("path", words::OPEN_FILE_IN_APPLICATION_PATH, Takes::Path),
        ],
        Requires::grants_over(["application", "path"]),
        words::OPEN_FILE_IN_APPLICATION_SENTENCE,
    )?
    .changing(
        words::OPEN_FILE_IN_APPLICATION_CHANGES,
        Some(words::OPEN_FILE_IN_APPLICATION_LEAVES),
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use alo_capability::{CallError, Given};

    /// The one verb, on a list of its own.
    fn the_verb() -> Verbs {
        let mut verbs = Verbs::default();
        declare_opening_a_file_into(&mut verbs).unwrap();
        verbs
    }

    /// **A change, over both arguments, and the sentence names both** — what a
    /// person approves is *open hello.txt in VSCodium*, not a verb name.
    #[test]
    fn it_is_a_change_over_the_application_and_the_file_and_reads_as_one_sentence() {
        let verbs = the_verb();
        let verb = verbs.of(OPEN_FILE_IN_APPLICATION).unwrap();
        assert_eq!(verb.effect(), Effect::Change);
        assert!(verb.effect().waits_for_approval());
        assert_eq!(
            verb.requires(),
            &Requires::Grants(vec!["application".to_owned(), "path".to_owned()])
        );
        let call = verbs
            .call(
                OPEN_FILE_IN_APPLICATION,
                &[
                    ("application", Given::text("com.vscodium.codium")),
                    ("path", Given::text("/var/home/anna/hello.txt")),
                ],
            )
            .unwrap();
        assert_eq!(
            call.sentence(&in_english()).text(),
            "open /var/home/anna/hello.txt in com.vscodium.codium"
        );
        assert!(call.waits_for_approval());
        assert_eq!(call.asks().len(), 2);
    }

    /// **Neither argument is a way in**: a path that is not a full path, and an
    /// application that is not an identifier, never become a call.
    #[test]
    fn what_is_not_a_path_or_an_application_is_refused_at_the_door() {
        let verbs = the_verb();
        for (application, path) in [
            ("com.vscodium.codium", "hello.txt"),
            ("/usr/bin/codium", "/var/home/anna/hello.txt"),
            ("com vscodium", "/var/home/anna/hello.txt"),
        ] {
            let refused = verbs
                .call(
                    OPEN_FILE_IN_APPLICATION,
                    &[
                        ("application", Given::text(application)),
                        ("path", Given::text(path)),
                    ],
                )
                .unwrap_err();
            assert!(
                matches!(refused, CallError::Argument(_)),
                "{application} {path}: {refused:?}"
            );
        }
        assert!(verbs.call(OPEN_FILE_IN_APPLICATION, &[]).is_err());
    }

    /// **A name already taken is not quietly replaced.**
    #[test]
    fn a_name_already_taken_is_not_replaced() {
        let mut verbs = the_verb();
        assert!(matches!(
            declare_opening_a_file_into(&mut verbs),
            Err(Declaring::List(_))
        ));
        assert_eq!(verbs.len(), 1);
    }

    /// **Everything it says is something this crate declares**, so a person
    /// never reads a key where the sentence they approve belongs.
    #[test]
    fn everything_it_says_is_declared() {
        let strings = in_english();
        let verbs = the_verb();
        let verb = verbs.of(OPEN_FILE_IN_APPLICATION).unwrap();
        assert!(!verb.purpose(&strings).is_a_bug());
        for arg in verb.args() {
            assert!(!arg.purpose(&strings).is_a_bug(), "{}", arg.name());
        }
    }
}
