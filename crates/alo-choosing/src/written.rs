//! The settings as somebody typed them, and the one number that decides whether
//! they are read at all.
//!
//! [`crate::settings`] is the checked value; this is the shape on the disk. They
//! are two files for `alo-agentd`'s reason one crate on: everything here derives
//! `Deserialize`, and nothing that does is also the thing a machine is run from.
//!
//! # The format number
//!
//! [`THE_FORMAT`] is `2` and `1` is still read, and a file saying anything else
//! is **refused rather
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
//!
//! # The weights somebody brought are a shape of their own, and they are
//! kebab-case
//!
//! `alo_models::Weights` has a `Deserialize` already, and it is deliberately
//! not what reads `[[brought]]`. That derive spells its keys the way its fields
//! are named; a file a person types spells them the way
//! `docs/contracts/machine-description.md` does — `turn-seconds`, `for-days`,
//! and here `bytes-on-disk` and `drives-verbs`. This file is the shape on the
//! disk, so the spelling is this file's decision and the checked value is
//! `alo-models`'.
//!
//! What is **not** restated is the rule underneath it: `drives-verbs` has no
//! serde default here for the same reason it has none there, which is that an
//! entry saying nothing about the measurement would read as *probably fine*.

use serde::Deserialize;

use alo_models::{
    Brought, Driving, Provider, ProviderError, Providers, Region, SecretRef, Weights, WeightsError,
};

use crate::chosen::{Chosen, Picked, Which};
use crate::refusing::NotSet;
use crate::settings::{Settings, Unresolved};

/// The shape of settings this alo OS writes, and the newest it reads.
///
/// **`2` since providers.** A file may still say `1` and be read exactly as it
/// always was — that is [`ALSO_READ`], and it is why a machine that has been
/// configured for a year keeps working. What `2` buys is the other direction:
/// an alo OS from before providers, handed a file that chooses one, refuses it
/// as *a shape I do not read* rather than as *a key I have not heard of*, which
/// is the difference between a person being told to upgrade and a person
/// hunting for a typo.
pub const THE_FORMAT: u32 = 2;

/// Every shape this alo OS still reads, oldest first.
///
/// Expand, migrate, contract — `CLAUDE.md`'s rule for a schema, and this is the
/// expand. A file written before providers existed says `1`, has no provider in
/// it, and means exactly what it meant; nothing rewrites it and nothing asks
/// the person to.
pub const ALSO_READ: [u32; 1] = [1];

/// Whether this alo OS reads a file that says it is this shape.
#[must_use]
pub const fn is_a_shape_we_read(format: u32) -> bool {
    format == THE_FORMAT || format == ALSO_READ[0]
}

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
    /// The weights they brought to this machine themselves, where they have
    /// brought any. An array of tables, so a file that has none simply has no
    /// `[[brought]]` in it.
    brought: Option<Vec<WeightsAsWritten>>,
    /// The providers they added themselves, where they have added any.
    provider: Option<Vec<ProviderAsWritten>>,
    /// What they read, where they have said.
    reading: Option<TheReading>,
}

/// One set of weights on the person's own list, exactly as they were written.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct WeightsAsWritten {
    /// What the model runtime on this machine answers to. Matched exactly,
    /// which is `alo_models::Brought`'s rule and item 1's before it.
    id: String,
    /// What the weights take on this machine's disk, as the runtime reported
    /// it.
    bytes_on_disk: u64,
    /// The quantisation the runtime reports, where it says. The one key here
    /// that may be absent, because a runtime does not always say.
    #[serde(default)]
    quantisation: Option<String>,
    /// What a measurement of these weights earned. No serde default, so an
    /// entry that says nothing about it fails to read.
    drives_verbs: Driving,
}

impl WeightsAsWritten {
    /// These as weights, or the reason they are not.
    fn checked(self) -> Result<Weights, WeightsError> {
        let mut weights = Weights::checked(&self.id, self.bytes_on_disk)?;
        weights.quantisation = self.quantisation;
        Ok(weights.measured(self.drives_verbs))
    }
}

/// This crate's own reason for a `[[brought]]` entry that is not weights.
///
/// `alo_models::WeightsError` is about a **list** and these are about a
/// **file**, which is why they are not carried across and reworded: what a
/// person needs in order to act is the path, and the list has no path in it.
fn not_weights(at: &std::path::Path, why: WeightsError) -> NotSet {
    match why {
        WeightsError::Unnamed => NotSet::WeightsUnnamed { at: at.to_owned() },
        WeightsError::AlreadyBrought(id) => NotSet::WeightsTwice {
            at: at.to_owned(),
            id,
        },
    }
}

/// Which place, and which entry in it.
///
/// One key, and the key **is** the place: `catalogue = "mistral-small"`,
/// `brought = "my-finetune"`, or
/// `provider = { name = "Mistral", model = "mistral-small-latest" }`. Two keys
/// at once is not a choice and does not read; a key that is none of the three
/// is refused naming the three that are.
///
/// The provider form carries **two** names where the other two carry one, and
/// they are two different pairs: on this machine it is *which list* and *which
/// entry*, and for a provider it is *which provider* and *which of its models*.
/// A shape with one name for both would have to guess which question it was
/// answering.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum TheAnswers {
    /// A model in the catalogue alo OS ships.
    Catalogue(String),
    /// Weights somebody brought themselves.
    Brought(String),
    /// A provider the person added, and the model they want from it.
    Provider(ProviderChosen),
}

/// Which provider, and which of its models.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderChosen {
    /// The person's own name for it, matched against their `[[provider]]` list.
    name: String,
    /// What that provider is asked for, exactly as they wrote it.
    model: String,
}

/// One provider on the person's own list, exactly as it was written.
///
/// # There is no key here, and that is the protection
///
/// A settings file is a text file in a person's home directory, and the single
/// most reliable way for a credential to end up in one is for there to be a
/// field called `key`. So there is not one. The keyring name a provider's
/// credential lives under is **derived** from the provider's own name, and
/// `deny_unknown_fields` refuses a file that invents a place to paste a
/// credential — naming the key, so the person is told rather than left with a
/// file that silently did nothing.
///
/// `alo_models::SecretRef` is the handle, `alo_models::Secret` is the
/// credential, and nothing in this crate ever holds the second.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct ProviderAsWritten {
    /// What the person calls it, and what an answer says it came from.
    name: String,
    /// Where it is. `https://` unless it is on this machine, which
    /// `alo_models::Provider::checked` is what decides.
    endpoint: String,
    /// Where it runs, as **stated** by whoever added it. Absent is
    /// `Region::Unknown`, which is honest: a region inferred from a domain name
    /// would be a reassuring label over a breach.
    #[serde(default)]
    region: Option<String>,
    /// Whether this provider is asked for a credential at all.
    ///
    /// `true` and absent both mean *it needs one*, because almost every hosted
    /// API does and the safe default is the common one. A compatible service
    /// that takes no credential says `needs-a-key = false`, and then nothing is
    /// looked up and nothing is sent.
    #[serde(default = "needs_a_key")]
    needs_a_key: bool,
}

/// What a provider that says nothing about a credential is taken to need.
const fn needs_a_key() -> bool {
    true
}

impl ProviderAsWritten {
    /// This as a provider, or the reason it is not.
    fn checked(self) -> Result<Provider, ProviderError> {
        let region = self.region.map_or(Region::Unknown, Region::Declared);
        // Derived, never read from the file: `provider/<their own name for it>`
        // is where this provider's credential lives, and the file has nowhere
        // to say otherwise.
        let key = self
            .needs_a_key
            .then(|| SecretRef::named(&format!("provider/{}", self.name.trim())));
        Provider::checked(&self.name, &self.endpoint, region, key)
    }
}

/// This crate's own reason for a `[[provider]]` entry that is not a provider.
///
/// `alo_models::ProviderError` is about a **list** and these are about a
/// **file**: what a person needs in order to act is the path, and the list has
/// no path in it. The same argument [`not_weights`] makes one list to the left.
fn not_a_provider(at: &std::path::Path, why: ProviderError) -> NotSet {
    NotSet::NotAProvider {
        at: at.to_owned(),
        why,
    }
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
            Some(TheAnswers::Catalogue(model)) => Some(
                Chosen::of(Which::Catalogue, &model)
                    .map(Picked::OnThisMachine)
                    .map_err(|_| NotSet::Nameless { at: at.to_owned() })?,
            ),
            Some(TheAnswers::Brought(model)) => Some(
                Chosen::of(Which::Brought, &model)
                    .map(Picked::OnThisMachine)
                    .map_err(|_| NotSet::Nameless { at: at.to_owned() })?,
            ),
            Some(TheAnswers::Provider(picked)) => Some(
                Picked::from_a_provider(&picked.name, &picked.model)
                    .map_err(|_| NotSet::Nameless { at: at.to_owned() })?,
            ),
            None => None,
        };

        // `Brought::add` is what refuses two entries answering to one name, so
        // the list is built through its door rather than collected into one.
        let mut brought = Brought::default();
        for entry in self.brought.unwrap_or_default() {
            let weights = entry.checked().map_err(|why| not_weights(at, why))?;
            brought.add(weights).map_err(|why| not_weights(at, why))?;
        }

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
        // `Providers::add` is what refuses two providers answering to one
        // name, for the reason `Brought::add` refuses two sets of weights: an
        // answer that said *by Mistral* would not say which one.
        let mut providers = Providers::default();
        for entry in self.provider.unwrap_or_default() {
            let provider = entry.checked().map_err(|why| not_a_provider(at, why))?;
            providers
                .add(provider)
                .map_err(|why| not_a_provider(at, why))?;
        }

        // Last, because it is the only question that needs two halves of the
        // file: a choice into a list has to name something on that list.
        Settings::of(chosen, brought, providers, languages).map_err(|why| match why {
            Unresolved::Weights(model) => NotSet::NotBrought {
                at: at.to_owned(),
                model,
            },
            Unresolved::Provider(provider) => NotSet::NoSuchProvider {
                at: at.to_owned(),
                provider,
            },
        })
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
    if !is_a_shape_we_read(which.format) {
        return Err(NotSet::AnotherFormat {
            at: at.to_owned(),
            format: which.format,
            reads: THE_FORMAT,
        });
    }

    let written: AsWritten = toml::from_str(said).map_err(not_understood)?;
    // A file from before providers existed cannot choose or list one. The keys
    // parse — they are the same shape either way — and honouring them would be
    // this machine believing the half of a disagreement it preferred.
    if which.format < THE_FORMAT
        && (written.provider.is_some() || matches!(written.answers, Some(TheAnswers::Provider(_))))
    {
        return Err(NotSet::ProviderNeedsANewerShape {
            at: at.to_owned(),
            format: which.format,
            reads: THE_FORMAT,
        });
    }
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

[[brought]]
id = "my-finetune"
bytes-on-disk = 4700000000
quantisation = "Q4_K_M"
drives-verbs = "reliably"
"#
        .to_owned()
    }

    /// The example in the contract is somebody's settings, and every value in
    /// it arrives where the machine reads it.
    #[test]
    fn the_settings_in_the_contract_are_settings() {
        let settings = read(&as_the_contract_writes_them(), somewhere()).unwrap();
        let chosen = settings.chosen().unwrap();
        assert_eq!(chosen.on_this_machine().unwrap().which(), Which::Catalogue);
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
        let said = as_the_contract_writes_them().replace(
            r#"catalogue = "mistral-small""#,
            r#"brought = "my-finetune""#,
        );
        let settings = read(&said, somewhere()).unwrap();
        assert_eq!(
            settings
                .chosen()
                .unwrap()
                .on_this_machine()
                .unwrap()
                .which(),
            Which::Brought
        );
        assert_eq!(settings.chosen().unwrap().model(), "my-finetune");
        // And what it resolves to is the entry, not the name again.
        assert_eq!(settings.weights().unwrap().bytes_on_disk, 4_700_000_000);
    }

    /// **The list a person brought is theirs, and every part of an entry
    /// arrives** — including the grade a measurement of their own weights
    /// earned, which is the thing nobody would want re-run at every boot.
    #[test]
    fn the_weights_in_the_contract_are_weights() {
        let settings = read(&as_the_contract_writes_them(), somewhere()).unwrap();
        let weights = settings.brought().get("my-finetune").unwrap();
        assert_eq!(weights.bytes_on_disk, 4_700_000_000);
        assert_eq!(weights.quantisation.as_deref(), Some("Q4_K_M"));
        assert_eq!(weights.drives_verbs, Driving::Reliably);
        assert!(weights.can_be_the_agent());
    }

    /// **What a runtime reports it does not always say**, so the quantisation is
    /// the one key on an entry that may be absent — and its absence is not the
    /// entry failing to read.
    #[test]
    fn weights_whose_quantisation_the_runtime_never_said_are_still_weights() {
        let said = as_the_contract_writes_them().replace("quantisation = \"Q4_K_M\"\n", "");
        let settings = read(&said, somewhere()).unwrap();
        assert_eq!(
            settings.brought().get("my-finetune").unwrap().quantisation,
            None
        );
    }

    /// **An entry that says nothing about the measurement does not read**,
    /// which is `alo_models::Weights`' own rule reaching the file it is stored
    /// in: *not measured* is a thing to state rather than a blank to leave.
    #[test]
    fn weights_that_say_nothing_about_the_measurement_refuse_the_file() {
        let said = as_the_contract_writes_them().replace("drives-verbs = \"reliably\"\n", "");
        assert!(matches!(
            read(&said, somewhere()).unwrap_err(),
            NotSet::NotUnderstood { .. }
        ));
    }

    /// **Weights with no name are refused**, because there would be nothing to
    /// ask the runtime for — and the whole file goes with them.
    #[test]
    fn weights_with_no_name_refuse_the_file() {
        let said = as_the_contract_writes_them().replace(r#"id = "my-finetune""#, r#"id = "  ""#);
        assert!(matches!(
            read(&said, somewhere()).unwrap_err(),
            NotSet::WeightsUnnamed { .. }
        ));
    }

    /// **The same weights twice is refused**, because *these answered it* could
    /// not then say which — `alo_models::Brought::add`'s rule, met at the file
    /// that holds the list.
    #[test]
    fn the_same_weights_listed_twice_refuse_the_file() {
        let said = as_the_contract_writes_them()
            + "\n[[brought]]\nid = \"my-finetune\"\nbytes-on-disk = 1\ndrives-verbs = \"rarely\"\n";
        let refused = read(&said, somewhere()).unwrap_err();
        assert!(
            matches!(&refused, NotSet::WeightsTwice { id, .. } if id == "my-finetune"),
            "{refused:?}"
        );
    }

    /// **Two ids differing in case are two models**, which is
    /// `alo_models::Brought`'s rule and item 1's before it: a runtime matches
    /// exactly, so a file listing both is listing two things.
    #[test]
    fn weights_differing_only_in_case_are_two_entries_rather_than_one() {
        let said = as_the_contract_writes_them()
            + "\n[[brought]]\nid = \"My-Finetune\"\nbytes-on-disk = 1\ndrives-verbs = \"rarely\"\n";
        let settings = read(&said, somewhere()).unwrap();
        assert_eq!(settings.brought().weights.len(), 2);
    }

    /// **A choice naming weights the list does not have refuses the file**, and
    /// the refusal quotes the name back: the two halves of a settings file
    /// disagree, and taking either one would be the machine deciding which of
    /// them the person meant.
    #[test]
    fn a_choice_naming_weights_that_are_not_listed_refuses_the_file() {
        let said = as_the_contract_writes_them().replace(
            r#"catalogue = "mistral-small""#,
            r#"brought = "my-finetunes""#,
        );
        let refused = read(&said, somewhere()).unwrap_err();
        assert!(
            matches!(&refused, NotSet::NotBrought { model, .. } if model == "my-finetunes"),
            "{refused:?}"
        );
    }

    /// **A file with no `[[brought]]` in it is a person who brought nothing**,
    /// which is most machines — and it is not a mistake any more than choosing
    /// nothing is.
    #[test]
    fn a_person_who_brought_nothing_has_an_empty_list_rather_than_a_refusal() {
        let settings = read("format = 1\n", somewhere()).unwrap();
        assert!(settings.brought().weights.is_empty());
        assert!(settings.weights().is_none());
    }

    /// **A catalogued choice is not checked against anything here.** The
    /// catalogue ships with the release rather than living in this file, and a
    /// model already on somebody's disk is theirs to ask — so the one list this
    /// file can contradict itself about is the one it holds.
    #[test]
    fn a_catalogued_choice_is_not_looked_for_in_the_list_the_person_brought() {
        let settings = read(&as_the_contract_writes_them(), somewhere()).unwrap();
        assert_eq!(
            settings
                .chosen()
                .unwrap()
                .on_this_machine()
                .unwrap()
                .which(),
            Which::Catalogue
        );
        assert_eq!(settings.chosen().unwrap().model(), "mistral-small");
        assert!(settings.brought().get("mistral-small").is_none());
        assert!(settings.weights().is_none());
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
    fn a_provider_chosen_in_the_older_shape_is_refused_as_the_older_shape() {
        // This test used to assert that a provider was refused outright,
        // naming the two lists this machine had. It has two lists and a
        // provider now, and what survives of the old rule is the half that
        // matters: a file that says it is the shape from before providers
        // existed does not get one, and is told which number it needs rather
        // than which key it should not have used.
        let said = "format = 1\n\n[answers]\nprovider = { name = \"Mistral\", model = \"m\" }\n";
        assert!(
            matches!(
                read(said, somewhere()).unwrap_err(),
                NotSet::ProviderNeedsANewerShape {
                    format: 1,
                    reads: 2,
                    ..
                }
            ),
            "a provider in a format 1 file was not refused as an older shape"
        );

        // And so is a list of them, because either half alone is the same
        // disagreement between a file's number and its keys.
        let listed = "format = 1\n\n[[provider]]\nname = \"Mistral\"\nendpoint = \"https://api.mistral.ai\"\n";
        assert!(matches!(
            read(listed, somewhere()).unwrap_err(),
            NotSet::ProviderNeedsANewerShape { .. }
        ));
    }

    /// **A key nobody declared is still refused, and `key` is one of them.**
    ///
    /// The protection is that there is nowhere in this file to put a
    /// credential: the keyring name is derived from the provider's own name, so
    /// a person who pastes their API key into their settings is told the key is
    /// not a key this file has, rather than left with a credential on their
    /// disk in a file alo OS quietly read.
    #[test]
    fn a_settings_file_has_nowhere_to_put_a_credential() {
        let said = "format = 2\n\n[[provider]]\nname = \"Mistral\"\n\
                    endpoint = \"https://api.mistral.ai\"\nkey = \"sk-live-0123456789\"\n";
        let refused = read(said, somewhere()).unwrap_err();
        let NotSet::NotUnderstood { ref why, .. } = refused else {
            unreachable!("a key nobody declared is refused as text that is not settings")
        };
        assert!(why.to_string().contains("key"), "{why}");

        // **And the sentence the person reads carries no credential.** That is
        // the guarantee: `choosing.settings.not-understood` is filled with the
        // path and nothing else, so a pasted key does not reach a screen, a
        // notification or anywhere it could be read over a shoulder.
        //
        // What it is *not* is a claim that the value has been forgotten. The
        // parse error quotes the line it failed on, as every TOML parser does,
        // and it is carried inside this refusal — so a `Debug` of one would
        // show it. Nothing renders that today and nothing logs it; the residual
        // is named in
        // `docs/autonomy/updates/three-model-choices-in-the-backend.md` rather
        // than asserted away here.
        let strings = alo_strings::Strings::of({
            let mut vocabulary = alo_strings::Vocabulary::default();
            crate::declare_into(&mut vocabulary).unwrap();
            vocabulary
        });
        let sentence = refused.said(&strings);
        assert!(
            !sentence.text().contains("sk-live-0123456789"),
            "the credential reached the sentence a person reads: {sentence}"
        );
        assert!(
            sentence
                .text()
                .contains("nothing in the file has been used")
        );
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
        let said = as_the_contract_writes_them().replace("format = 1", "format = 3");
        assert!(matches!(
            read(&said, somewhere()).unwrap_err(),
            NotSet::AnotherFormat {
                format: 3,
                reads: 2,
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
        assert_eq!(THE_FORMAT, 2);
        // And the one before it is still read, which is what makes a machine
        // configured a year ago keep working. Expand, then migrate, then
        // contract — and nothing here is the contract.
        assert_eq!(ALSO_READ, [1]);
        assert!(is_a_shape_we_read(1) && is_a_shape_we_read(2));
        assert!(!is_a_shape_we_read(3));
    }
}
