//! **How a model is told to install an application**: look in the catalogue
//! first, unless the person gave the identifier, and install exactly what it
//! shows.
//!
//! Every-try, set 2, on 2026-11-15 (`gpt-4o-mini` at OpenAI, full access):
//! asked to *install GIMP*, the model sent `install_application` straight away
//! in 5 of 5 tries, naming `org.gnome.TextEditor` twice (the example in the
//! application argument's own description), `gimp` once and `org.gnome.GIMP`
//! twice. The catalogue it was offered holds `org.gimp.GIMP`, from Flathub; it
//! never looked. An identifier this machine has never heard of is refused or
//! installs something else, so a guess costs the person either a wasted step
//! or the wrong application.
//!
//! This paragraph says both halves, after the verbs and only when the registry
//! has the two verbs it names. **No grade is about this text**, as none is about
//! [`crate::working_in_an_application`]: a grade names the instructions above
//! the verbs by digest (ADR 0034), and this is after them.

use alo_capability::Verbs;

/// The verb that reads the catalogue — `alo_software::WHAT_CAN_BE_INSTALLED`,
/// which `alo-turn`'s tests hold this to. Named here rather than depended on:
/// this crate reads a registry and names verbs, and declares none.
pub const WHAT_CAN_BE_INSTALLED: &str = "what_can_be_installed";

/// The verb that installs one application — `alo_software::INSTALL_APPLICATION`.
pub const INSTALL_APPLICATION: &str = "install_application";

/// The option that looks under every kind of work —
/// `alo_software::verbs::EVERYTHING`, held to it by the same test.
pub const EVERYTHING: &str = "everything";

/// **The paragraph a model is told, for this registry** — empty when it does
/// not declare both [`WHAT_CAN_BE_INSTALLED`] and [`INSTALL_APPLICATION`].
#[must_use]
pub fn installing_an_application(verbs: &Verbs) -> String {
    if verbs.of(WHAT_CAN_BE_INSTALLED).is_none() || verbs.of(INSTALL_APPLICATION).is_none() {
        return String::new();
    }
    format!(
        "\n\nTo install an application, send {WHAT_CAN_BE_INSTALLED} first unless the person \
         gave you its identifier: an identifier is never guessed, and the example in a verb's \
         description is an example, not an answer. Never look for it with a run line either: \
         {WHAT_CAN_BE_INSTALLED} is what this machine installs from. Then send \
         {INSTALL_APPLICATION} with the identifier and the source exactly as that answer showed \
         them. When the answer does not list the application, look under {EVERYTHING} before \
         you install anything."
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::the_verbs;
    use alo_capability::{Arg, Effect, Requires, Takes, Verb};
    use alo_strings::Word;

    /// A verb of the given name, taking one application.
    fn named(name: &str, effect: Effect) -> Verb {
        Verb::checked(
            name,
            Word::saying("testing.installing.purpose", "install"),
            effect,
            vec![Arg::taking(
                "application",
                Word::saying("testing.installing.application", "the application"),
                Takes::Application,
            )],
            Requires::grants_over(["application"]),
            Word::saying("testing.installing.sentence", "install {application}"),
        )
        .unwrap()
    }

    /// **A machine without the two verbs is told nothing about them.**
    #[test]
    fn a_registry_without_both_verbs_is_told_nothing() {
        assert_eq!(installing_an_application(&the_verbs()), "");
        let mut only_looking = the_verbs();
        only_looking
            .declare(named(WHAT_CAN_BE_INSTALLED, Effect::Read))
            .unwrap();
        assert_eq!(installing_an_application(&only_looking), "");
    }

    /// **With both, the catalogue comes first**, and the example is said to be
    /// an example.
    #[test]
    fn with_both_verbs_the_catalogue_is_named_first() {
        let mut verbs = the_verbs();
        verbs
            .declare(named(WHAT_CAN_BE_INSTALLED, Effect::Read))
            .unwrap();
        verbs
            .declare(named(INSTALL_APPLICATION, Effect::Change))
            .unwrap();
        let told = installing_an_application(&verbs);
        let looking = told.find(WHAT_CAN_BE_INSTALLED).unwrap();
        let installing = told.find(INSTALL_APPLICATION).unwrap();
        assert!(looking < installing, "{told}");
        assert!(told.contains("never guessed"), "{told}");
    }
}
