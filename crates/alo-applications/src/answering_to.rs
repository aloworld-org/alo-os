//! **Which installed application a name means**, when an agent's call gives a
//! name rather than the identifier.
//!
//! # Why this exists, measured
//!
//! **The walk of 2026-11-14** (an image from `main` at `3959dd65`). The first
//! boot installed VSCodium as `com.vscodium.codium`, and `alo-app list` showed
//! it. Asked *open hello.txt from my home folder in vscodium*, alo answered that
//! vscodium was not installed; asked *open VSCodium*, it installed an
//! application from Flathub. The list a call is checked against
//! ([`crate::Installed::knows`]) matches an identifier **exactly**, and a model
//! told only *an installed application's identifier* sends the name the person
//! said. `vscodium` is not `com.vscodium.codium`, so *nothing installed on this
//! machine is vscodium — install it* was the true answer to the question asked,
//! and the wrong answer to the person.
//!
//! # The order, and why a display name is in it
//!
//! A name is read in this order, and the first step that finds anything
//! answers:
//!
//! 1. the identifier, exactly;
//! 2. the identifier, whatever its case (`org.gnome.texteditor`);
//! 3. for a name with no dot in it, the short name ([`crate::short_name`]:
//!    `codium`, `texteditor`, `writer`);
//! 4. for a name with no dot in it, the name the application's entry gives,
//!    whatever its case and with its spaces left out (`VSCodium`, `vscodium`,
//!    `TextEditor`).
//!
//! **Two applications answering at one step make it name neither**, and that
//! is what lets step 4 be here at all. `alo-app` reads no display name, because
//! two applications may both call themselves *Text Editor* and the one acted on
//! must be the one meant; this answers [`AnsweringTo::Several`] for exactly that
//! case, and nothing is guessed. Step 4 is needed because the shipped code
//! editor's identifier holds no word a person says: `codium` is its short name,
//! and *VSCodium* is only its entry's `Name`.
//!
//! # What this does not decide
//!
//! Whether the agent may reach what the name means. That is the grants', and
//! `alo-turn` asks them before it puts the identifier in the call — so a name
//! for an application nobody granted is refused exactly as a name for one that
//! is not installed, and a refusal says nothing about what is on this machine
//! ([`crate::Reaching`]'s header is why).

use crate::application::{Application, Called};
use crate::installed::Installed;
use crate::short_name::{short_name, the_last_part};

/// What a name means among what is installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnsweringTo<'a> {
    /// One installed application, and only one, answers to it.
    One(&'a Application),
    /// More than one answers to it at the same step, sorted by identifier, so
    /// it names none of them.
    Several(Vec<&'a Application>),
    /// Nothing installed answers to it.
    Nothing,
}

impl AnsweringTo<'_> {
    /// Whether anything installed answers to the name — one application or
    /// several. Used where *is it here at all* is the whole question, such as
    /// refusing to install what is already installed.
    #[must_use]
    pub const fn is_here(&self) -> bool {
        !matches!(self, Self::Nothing)
    }
}

impl Installed {
    /// **Which installed application this name means**, in this file's order.
    #[must_use]
    pub fn answering_to(&self, said: &str) -> AnsweringTo<'_> {
        let said = said.trim();
        if said.is_empty() {
            return AnsweringTo::Nothing;
        }
        if let Some(exactly) = self.knows(said) {
            return AnsweringTo::One(exactly);
        }
        let lowered = said.to_lowercase();
        let found =
            self.at_one_step(|application| application.identifier().to_lowercase() == lowered);
        if found.is_here() || said.contains('.') {
            return found;
        }
        let found = self.at_one_step(|application| {
            let identifier = application.identifier();
            short_name(identifier) == lowered || the_last_part(identifier) == lowered
        });
        if found.is_here() {
            return found;
        }
        let folded = without_spaces(&lowered);
        self.at_one_step(|application| {
            application
                .name()
                .is_some_and(|called| without_spaces(&as_written(called).to_lowercase()) == folded)
        })
    }

    /// Every installed application this test holds for, as one answer.
    fn at_one_step(&self, answers: impl Fn(&Application) -> bool) -> AnsweringTo<'_> {
        // `Installed::all` is in identifier order already, so `Several` is
        // sorted without sorting it.
        let found: Vec<&Application> = self
            .all()
            .filter(|application| answers(application))
            .collect();
        if found.len() > 1 {
            return AnsweringTo::Several(found);
        }
        found
            .first()
            .copied()
            .map_or(AnsweringTo::Nothing, AnsweringTo::One)
    }
}

/// The name as its entry wrote it, or as this project's own word is written in
/// the language the code is in — the only text there is without a vocabulary.
fn as_written(called: &Called) -> &str {
    match called {
        Called::ByWhoeverPackagedIt(called) => called,
        Called::InOurOwnWords(word) => word.says(),
    }
}

/// The text with every space left out, so `Text Editor` is `texteditor`: a
/// call's application may hold no space (`alo_capability`'s rule for
/// `Takes::Application`), so a name that has one could never be matched.
fn without_spaces(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// VSCodium, as Flathub's entry names it.
    fn vscodium() -> Application {
        Application::called("com.vscodium.codium", "VSCodium").unwrap()
    }

    /// GNOME Text Editor, whose name has a space in it.
    fn the_text_editor() -> Application {
        Application::called("org.gnome.TextEditor", "Text Editor").unwrap()
    }

    /// The one identifier an answer names, or nothing.
    fn identifier<'a>(answer: &AnsweringTo<'a>) -> Option<&'a str> {
        match answer {
            AnsweringTo::One(application) => Some(application.identifier()),
            AnsweringTo::Several(_) | AnsweringTo::Nothing => None,
        }
    }

    /// **The walk's two requests**, by every name a person or a model says.
    #[test]
    fn vscodium_answers_to_its_identifier_its_short_name_and_its_name_in_any_case() {
        let installed = Installed::holding([vscodium(), the_text_editor()]);
        for said in [
            "com.vscodium.codium",
            "com.vscodium.Codium",
            "COM.VSCODIUM.CODIUM",
            "codium",
            "Codium",
            "vscodium",
            "VSCodium",
            "VSCODIUM",
            " VSCodium ",
        ] {
            assert_eq!(
                identifier(&installed.answering_to(said)),
                Some("com.vscodium.codium"),
                "{said}"
            );
        }
    }

    /// A name with spaces in its entry is matched without them.
    #[test]
    fn a_name_with_a_space_is_matched_without_it() {
        let installed = Installed::holding([vscodium(), the_text_editor()]);
        for said in ["TextEditor", "texteditor", "org.gnome.texteditor"] {
            assert_eq!(
                identifier(&installed.answering_to(said)),
                Some("org.gnome.TextEditor"),
                "{said}"
            );
        }
    }

    /// **Nothing is guessed**: what is not installed answers nothing, and
    /// neither does a near miss or an identifier that is only a prefix.
    #[test]
    fn what_is_not_installed_answers_nothing() {
        let installed = Installed::holding([vscodium()]);
        for said in [
            "",
            "  ",
            "firefox",
            "code",
            "vscode",
            "com.vscodium",
            "codium2",
            "org.vscodium.codium",
        ] {
            assert_eq!(installed.answering_to(said), AnsweringTo::Nothing, "{said}");
        }
        assert!(!Installed::nothing().answering_to("vscodium").is_here());
    }

    /// **Two applications calling themselves one name make it name neither**,
    /// the reason `alo-app` reads no display name, answered here instead.
    #[test]
    fn two_applications_with_one_name_make_it_name_neither() {
        let installed = Installed::holding([
            the_text_editor(),
            Application::called("org.kde.kwrite", "Text Editor").unwrap(),
        ]);
        let AnsweringTo::Several(both) = installed.answering_to("TextEditor") else {
            panic!("two applications called Text Editor named one of them");
        };
        let both: Vec<&str> = both
            .iter()
            .map(|application| application.identifier())
            .collect();
        assert_eq!(both, ["org.gnome.TextEditor", "org.kde.kwrite"]);
        assert!(installed.answering_to("TextEditor").is_here());
        // And the identifier still names exactly one.
        assert_eq!(
            identifier(&installed.answering_to("org.kde.kwrite")),
            Some("org.kde.kwrite")
        );
    }

    /// An earlier step wins: an exact identifier is never overruled by another
    /// application's name.
    #[test]
    fn an_identifier_is_never_overruled_by_another_applications_name() {
        let installed =
            Installed::holding([Application::identified("codium").unwrap(), vscodium()]);
        assert_eq!(
            identifier(&installed.answering_to("codium")),
            Some("codium")
        );
        assert_eq!(
            identifier(&installed.answering_to("vscodium")),
            Some("com.vscodium.codium")
        );
    }

    /// A name with a dot in it is an identifier or nothing, as in `alo-app`.
    #[test]
    fn a_name_with_a_dot_is_an_identifier_or_nothing() {
        let installed =
            Installed::holding([Application::called("org.example.Notes", "x.y").unwrap()]);
        assert_eq!(installed.answering_to("x.y"), AnsweringTo::Nothing);
    }
}
