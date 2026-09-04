//! The settings as somebody typed them, and the one number that decides whether
//! they are read at all.
//!
//! [`crate::settings`] is the checked value; this is the shape on the disk. They
//! are two files for `alo-agentd`'s reason one crate on: everything here derives
//! `Deserialize`, and nothing that does is also the thing a machine is run from.
//!
//! # The format number
//!
//! [`THE_FORMAT`] is `1`, and a file saying anything else is **refused rather
//! than guessed at**. It is the same rule `docs/contracts/record-file.md` makes
//! about a record from a newer alo OS, and it is what makes this file's future
//! safe: a provider, a paired machine and whatever else somebody may one day
//! choose are keys this alo OS has never heard of, and a machine that ignored
//! them would answer somewhere the person did not pick while showing them a
//! settings panel that says otherwise.
//!
//! # A key nobody declared is refused, and so is the whole file
//!
//! `deny_unknown_fields`, and then the file is refused whole rather than in
//! part. The alternative — honour what parsed, drop what did not — is the
//! machine choosing the rest of somebody's settings for them, quietly, in the
//! release that renamed a key. `alo-saying` makes the opposite decision about a
//! *translation* and the difference is who is harmed: a translation file covers
//! everybody's machine and refusing it whole would turn one person's language
//! off over somebody else's line, while a settings file is one person's and
//! refusing it costs only them — and tells them.
//!
//! # Nothing here has a default except being absent
//!
//! Both sections are optional and neither has a serde default inside it. A file
//! with no `[answers]` is a person who has not chosen, which is the ordinary
//! state of a machine nobody has configured and is answered by *nothing here
//! has been chosen to answer questions*. A file with `[answers]` and nothing in
//! it is a mistake, and it is refused.

use serde::Deserialize;

use crate::chosen::{Chosen, Which};
use crate::refusing::NotSet;
use crate::settings::Settings;

/// The shape of settings this alo OS reads.
pub const THE_FORMAT: u32 = 1;

/// Which shape of settings this is, and nothing else.
///
/// Read first, from the same text, and it is the one type here that does **not**
/// deny a key it does not know — because a file written for a newer alo OS is
/// exactly a file with keys this one has never heard of, and refusing it as a
/// typo would send a person looking for one. `alo-agentd` reads a machine's
/// description the same way and for the same reason.
#[derive(Debug, Deserialize)]
struct WhichFormat {
    /// Which shape of settings this is.
    format: u32,
}

/// A person's settings exactly as they were typed.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AsWritten {
    /// Which shape of settings this is.
    ///
    /// Declared and never read: [`read`] has already answered it through
    /// [`WhichFormat`], before this shape was asked for at all. It is here
    /// because `deny_unknown_fields` would otherwise refuse the one key every
    /// settings file has, and it is named with an underscore rather than
    /// checked a second time — a rule stated twice is a rule two readers can
    /// disagree about, and the second statement is the one no test can reach.
    #[serde(rename = "format")]
    _format: u32,
    /// What answers this person's questions, where they have chosen.
    answers: Option<TheAnswers>,
    /// What they read, where they have said.
    reading: Option<TheReading>,
}

/// Which list, and which entry in it.
///
/// One key, and the key **is** the list: `catalogue = "mistral-small"` or
/// `brought = "my-finetune"`. Two keys at once is not a choice and does not
/// read; a key that is neither is refused naming the two that are, which is
/// where a provider lands until this machine keeps a list of those.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum TheAnswers {
    /// A model in the catalogue alo OS ships.
    Catalogue(String),
    /// Weights somebody brought themselves.
    Brought(String),
}

/// What this person reads.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TheReading {
    /// The languages they read, best first, as tags: `["de", "en"]`.
    ///
    /// A list rather than one, because `alo-strings` says a person names their
    /// own second language and nothing infers one from a first. The broader
    /// form of each — `pt` behind `pt-BR` — is `alo_strings::Strings`' own
    /// arithmetic and is deliberately not written out here.
    languages: Vec<String>,
}

impl AsWritten {
    /// These settings as a person's, or the first reason they are not.
    ///
    /// The format is not asked about here: [`read`] answered it before this
    /// shape was parsed, which is what makes a file from a newer alo OS refused
    /// as one rather than as whichever of its keys this alo OS has not heard
    /// of.
    fn checked(self, at: &std::path::Path) -> Result<Settings, NotSet> {
        let chosen = match self.answers {
            Some(TheAnswers::Catalogue(model)) => Some(Chosen::of(Which::Catalogue, &model)),
            Some(TheAnswers::Brought(model)) => Some(Chosen::of(Which::Brought, &model)),
            None => None,
        }
        .transpose()
        .map_err(|_| NotSet::Nameless { at: at.to_owned() })?;

        let mut languages = Vec::new();
        for tag in self
            .reading
            .map(|reading| reading.languages)
            .unwrap_or_default()
        {
            let language =
                alo_strings::Language::written(&tag).map_err(|why| NotSet::NotALanguage {
                    at: at.to_owned(),
                    tag: tag.clone(),
                    why,
                })?;
            languages.push(language);
        }
        Ok(Settings::of(chosen, languages))
    }
}

/// The settings this text is.
///
/// `at` is where the text came from, and it is carried only so that every
/// refusal names the file somebody has to open — nothing here reads a disk,
/// which is what makes each of these refusals a test rather than a fixture.
///
/// # Errors
///
/// [`NotSet::AnotherFormat`] for settings this alo OS does not read,
/// [`NotSet::NotUnderstood`] for text that is not settings, and whatever the
/// values themselves refuse.
pub(crate) fn read(said: &str, at: &std::path::Path) -> Result<Settings, NotSet> {
    let not_understood = |why: toml::de::Error| NotSet::NotUnderstood {
        at: at.to_owned(),
        why: Box::new(why),
    };

    let which: WhichFormat = toml::from_str(said).map_err(not_understood)?;
    if which.format != THE_FORMAT {
        return Err(NotSet::AnotherFormat {
            at: at.to_owned(),
            format: which.format,
            reads: THE_FORMAT,
        });
    }

    let written: AsWritten = toml::from_str(said).map_err(not_understood)?;
    written.checked(at)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::path::Path;

    /// The path every refusal in these tests names.
    fn somewhere() -> &'static Path {
        Path::new("/home/ada/.config/alo/settings.toml")
    }

    /// Settings exactly as `docs/contracts/person-settings.md` writes them.
    fn as_the_contract_writes_them() -> String {
        r#"
format = 1

[answers]
catalogue = "mistral-small"

[reading]
languages = ["de", "en"]
"#
        .to_owned()
    }

    /// The example in the contract is somebody's settings, and every value in
    /// it arrives where the machine reads it.
    #[test]
    fn the_settings_in_the_contract_are_settings() {
        let settings = read(&as_the_contract_writes_them(), somewhere()).unwrap();
        let chosen = settings.chosen().unwrap();
        assert_eq!(chosen.which(), Which::Catalogue);
        assert_eq!(chosen.model(), "mistral-small");
        assert_eq!(
            settings
                .languages()
                .iter()
                .map(|language| language.tag().to_owned())
                .collect::<Vec<_>>(),
            ["de", "en"]
        );
    }

    /// **Weights somebody brought are the other list**, named by the key rather
    /// than by a value beside it — so a file cannot say *the catalogue* and
    /// mean the other one.
    #[test]
    fn weights_somebody_brought_are_a_different_choice_from_a_catalogued_model() {
        let said = as_the_contract_writes_them().replace("catalogue =", "brought =");
        let settings = read(&said, somewhere()).unwrap();
        assert_eq!(settings.chosen().unwrap().which(), Which::Brought);
        assert_eq!(settings.chosen().unwrap().model(), "mistral-small");
    }

    /// **A file with nothing chosen in it is not an error**, which is the
    /// ordinary state of a machine nobody has configured: the person is asked
    /// nothing and told, when they ask a question, that nothing has been chosen.
    #[test]
    fn settings_that_choose_nothing_are_settings() {
        let settings = read("format = 1\n", somewhere()).unwrap();
        assert!(settings.chosen().is_none());
        assert!(settings.languages().is_empty());
    }

    /// **Two lists at once is not a choice.** Whichever one a reader took would
    /// be the machine picking between them.
    #[test]
    fn naming_both_lists_at_once_is_refused() {
        let said = "format = 1\n\n[answers]\ncatalogue = \"a\"\nbrought = \"b\"\n";
        assert!(matches!(
            read(said, somewhere()).unwrap_err(),
            NotSet::NotUnderstood { .. }
        ));
    }

    /// **A place this machine keeps no list of is refused, and the refusal
    /// names the lists there are.** A provider is the one somebody will write
    /// first; it fails to read rather than reading as a setting that quietly
    /// does nothing.
    #[test]
    fn a_provider_is_refused_naming_the_two_lists_this_machine_has() {
        let said = "format = 1\n\n[answers]\nprovider = \"mistral\"\n";
        let refused = read(said, somewhere()).unwrap_err();
        assert!(matches!(refused, NotSet::NotUnderstood { .. }));
        let NotSet::NotUnderstood { why, .. } = refused else {
            unreachable!("the shape was matched above")
        };
        assert!(why.to_string().contains("catalogue"), "{why}");
        assert!(why.to_string().contains("brought"), "{why}");
    }

    /// **A list named with no model is refused**, rather than read as a person
    /// who chose nothing: they chose, and what they chose is not a model.
    #[test]
    fn a_list_named_with_no_model_is_refused() {
        let said = "format = 1\n\n[answers]\ncatalogue = \"\"\n";
        assert!(matches!(
            read(said, somewhere()).unwrap_err(),
            NotSet::Nameless { .. }
        ));
    }

    /// **Settings from a newer alo OS are refused rather than guessed at**, and
    /// the refusal says both numbers.
    #[test]
    fn settings_from_a_newer_alo_os_are_refused() {
        let said = as_the_contract_writes_them().replace("format = 1", "format = 2");
        assert!(matches!(
            read(&said, somewhere()).unwrap_err(),
            NotSet::AnotherFormat {
                format: 2,
                reads: 1,
                ..
            }
        ));
    }

    /// **The format is answered before anything in the file is used**, so a
    /// file written for a newer alo OS is refused as one rather than as
    /// whichever of its keys this one happened not to know.
    #[test]
    fn the_format_is_answered_before_the_values() {
        let said = as_the_contract_writes_them()
            .replace("format = 1", "format = 7")
            .replace("catalogue =", "provider =");
        assert!(matches!(
            read(&said, somewhere()).unwrap_err(),
            NotSet::AnotherFormat { format: 7, .. }
        ));
    }

    /// **A key nobody declared is refused**, because the only other thing to do
    /// with a typo is run under whatever the key it was meant to be says.
    #[test]
    fn a_key_nobody_declared_is_refused() {
        let said = as_the_contract_writes_them().replace("[reading]", "[readng]");
        let refused = read(&said, somewhere()).unwrap_err();
        assert!(matches!(refused, NotSet::NotUnderstood { .. }));
        let NotSet::NotUnderstood { why, .. } = refused else {
            unreachable!("the shape was matched above")
        };
        assert!(why.to_string().contains("readng"), "{why}");
    }

    /// **A language that is not one is refused, and the whole file with it.**
    /// Honouring the model and dropping the language would be the machine
    /// deciding which half of somebody's settings to believe.
    #[test]
    fn a_language_that_is_not_one_refuses_the_whole_file() {
        let said = as_the_contract_writes_them().replace(r#""de""#, r#""Deutsch""#);
        let refused = read(&said, somewhere()).unwrap_err();
        assert!(matches!(
            refused,
            NotSet::NotALanguage { ref tag, .. } if tag == "Deutsch"
        ));
    }

    /// **Text that is not settings at all is refused as settings**, and the
    /// refusal names the file — whoever reads it has several.
    #[test]
    fn something_that_is_not_settings_is_refused_as_settings() {
        assert!(matches!(
            read("this is not a settings file", somewhere()).unwrap_err(),
            NotSet::NotUnderstood { .. }
        ));
    }

    /// **An empty file is not empty settings.** It says nothing about which
    /// shape it is, so nothing in it can be believed.
    #[test]
    fn an_empty_file_is_not_empty_settings() {
        assert!(matches!(
            read("", somewhere()).unwrap_err(),
            NotSet::NotUnderstood { .. }
        ));
    }

    /// The shape number is what `docs/contracts/person-settings.md` fixes, and
    /// it is one string in one place.
    #[test]
    fn the_shape_is_numbered_once() {
        assert_eq!(THE_FORMAT, 1);
    }
}
