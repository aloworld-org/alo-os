//! A person's settings as the text of their file, and the one check that makes
//! writing them safe.
//!
//! `crate::written` is the shape on the disk and this is the way **out** of it.
//! They are two files because they are two responsibilities, and they are one
//! shape because anything else is a release renaming a key in the reader and
//! not in the writer.
//!
//! # Nothing is written that this alo OS cannot read back
//!
//! [`written`] serialises the settings, **reads the text it produced back
//! through `crate::written::read`, and refuses unless what comes back is the
//! same settings.** That is not belt and braces: it is the only way this file
//! can promise what `crate::Choosing`'s caller needs, which is that a change
//! made through it is a change the machine will act on.
//!
//! The failure it exists for is quiet rather than loud. `alo_models::Provider`
//! has public fields and this file has nowhere to put two of them — the list of
//! model names a provider offers, and a credential kept under a name other than
//! the one `crate::written::a_key_for` derives. A writer that simply dropped
//! them would hand a settings panel back an `Ok(())` for a provider the machine
//! would afterwards describe differently, and nobody would ever be told. So the
//! round trip is asked at the one door, before a byte reaches the disk, and the
//! person is told the change was not made.
//!
//! It costs one parse of a file measured in hundreds of bytes, at the moment
//! somebody clicks something. The thing it buys is that *reads back through
//! `Settings::at`* is a property of the code rather than of the tests that
//! happened to be written.
//!
//! # And every provider is held to its own crate's rule first
//!
//! `alo_models::Provider` has public fields, so a value can reach [`written`]
//! that `alo_models::Provider::checked` never made — an address over `http://`
//! to somewhere on the person's own network, most often. The round trip would
//! refuse it, because the reader will not take it; but the round trip's
//! sentence is *a defect in alo OS*, and this is a person who typed `http`.
//! So `crate::holding` asks `alo-models`' rule again, here, before the text is
//! made, and the refusal is [`NotWritten::NotAProvider`] in that crate's own
//! words: *use https, or a service on this machine.* It is asked here rather
//! than at `crate::Choosing`'s doors because this is the one function every
//! door goes through, so there is no second path to the file.
//!
//! # The format written is this alo OS's own
//!
//! [`crate::THE_FORMAT`], always. `crate::ALSO_READ` is there so a machine
//! configured before providers existed keeps working; it is not a menu of
//! shapes a writer picks from. A writer that chose the oldest shape a value
//! happened to fit would grow one branch per format for ever, each exercised
//! only by somebody who has gone backwards — and the whole of what going
//! backwards costs is already written down in
//! `docs/contracts/person-settings.md`.

use std::path::Path;

use alo_models::{Provider, Region, Weights};
use alo_strings::Language;

use crate::chosen::{Picked, Which};
use crate::holding::every_provider_holds;
use crate::settings::Settings;
use crate::setup::Setup;
use crate::unwritten::NotWritten;
use crate::written::{
    AsWritten, ProviderAsWritten, ProviderChosen, THE_FORMAT, TheAnswers, TheReading, TheSetup,
    WeightsAsWritten, read,
};

/// These settings as the text of a settings file.
///
/// `at` is where the text is going, and it is carried only so that a refusal
/// names the file a person has to look at. Nothing here touches a disk, which
/// is what makes every refusal below a test rather than a fixture.
///
/// # Errors
///
/// [`NotWritten::NotAProvider`] when a provider on the list is one
/// `alo_models::Provider::checked` would have refused — an address that is not
/// `https://` and is not a service on this machine, before anything else —
/// and [`NotWritten::NotExpressible`] when the text this produced is not text
/// this alo OS reads back as the same settings. The file has not been touched
/// in either.
pub(crate) fn written(settings: &Settings, at: &Path) -> Result<String, NotWritten> {
    every_provider_holds(settings).map_err(|why| NotWritten::NotAProvider {
        at: at.to_owned(),
        why,
    })?;
    let unreadable = |why: String| NotWritten::NotExpressible {
        at: at.to_owned(),
        why,
    };
    let text = toml::to_string(&as_written(settings)).map_err(|why| unreadable(why.to_string()))?;
    match read(&text, at) {
        Ok(back) if back == *settings => Ok(text),
        // The two are kept apart because they are two different defects to
        // whoever is fixing the machine: text this reader refuses outright, and
        // text it accepts as something else. Neither reaches a person — they
        // read `choosing.change.not-expressible` — and both name the shape of
        // the thing rather than quoting the file, because a settings file is
        // where somebody pastes what they should not have.
        Ok(_) => Err(unreadable(
            "the settings read back from the text written for them were different settings"
                .to_owned(),
        )),
        Err(why) => Err(unreadable(format!(
            "the text written for these settings was not settings this alo OS reads: {why:?}"
        ))),
    }
}

/// These settings in the shape the file has.
fn as_written(settings: &Settings) -> AsWritten {
    AsWritten {
        format: THE_FORMAT,
        answers: settings.chosen().map(the_answers),
        brought: unless_empty(
            settings
                .brought()
                .weights
                .iter()
                .map(WeightsAsWritten::of)
                .collect(),
        ),
        provider: unless_empty(
            settings
                .providers()
                .configured
                .iter()
                .map(ProviderAsWritten::of)
                .collect(),
        ),
        reading: unless_empty(
            settings
                .languages()
                .iter()
                .map(Language::tag)
                .map(str::to_owned)
                .collect(),
        )
        .map(|languages| TheReading { languages }),
        // Absent on a machine nobody has taken through setup, so settings
        // nobody has touched stay a `format` line and nothing else — a section
        // written to say *and nobody has been asked* would be alo OS putting a
        // value in the one file ADR 0016 keeps for the person.
        setup: match settings.setup() {
            Setup::Answered => Some(TheSetup { answered: true }),
            Setup::NotAnswered => None,
        },
    }
}

/// A list, or nothing at all where there is nothing on it.
///
/// The difference is a key in the file: a person who has brought no weights has
/// no `[[brought]]` in their settings rather than an empty one, which is what
/// `docs/contracts/person-settings.md` promises and what somebody reading their
/// own file expects to see.
fn unless_empty<T>(listed: Vec<T>) -> Option<Vec<T>> {
    (!listed.is_empty()).then_some(listed)
}

/// What this person chose, in the shape `[answers]` has.
///
/// The match is exhaustive on both halves — which list, and which of the two
/// kinds of choice — so a third place a question can be answered is a compiler
/// error here rather than a choice this writer silently declines to write.
fn the_answers(picked: &Picked) -> TheAnswers {
    match picked {
        Picked::OnThisMachine(chosen) => match chosen.which() {
            Which::Catalogue => TheAnswers::Catalogue(chosen.model().to_owned()),
            Which::Brought => TheAnswers::Brought(chosen.model().to_owned()),
        },
        Picked::FromAProvider { provider, model } => TheAnswers::Provider(ProviderChosen {
            name: provider.clone(),
            model: model.clone(),
        }),
    }
}

impl WeightsAsWritten {
    /// One set of weights, in the shape `[[brought]]` has.
    fn of(weights: &Weights) -> Self {
        Self {
            id: weights.id.clone(),
            bytes_on_disk: weights.bytes_on_disk,
            quantisation: weights.quantisation.clone(),
            drives_verbs: weights.drives_verbs,
        }
    }
}

impl ProviderAsWritten {
    /// One provider, in the shape `[[provider]]` has.
    ///
    /// **The credential is a `bool` here and nothing else**, because there is
    /// nowhere in this file to say where a key is: `crate::written::a_key_for`
    /// derives that from the provider's own name. A provider whose key is kept
    /// somewhere else is therefore not expressible, and [`written`]'s round
    /// trip is what turns that into a refusal instead of a silent move.
    fn of(provider: &Provider) -> Self {
        Self {
            name: provider.name.clone(),
            endpoint: provider.endpoint.clone(),
            region: match &provider.region {
                Region::Declared(said) => Some(said.clone()),
                Region::Unknown => None,
            },
            needs_a_key: provider.key.is_some(),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_models::{Brought, Driving, Providers, SecretRef};
    use std::path::PathBuf;

    use crate::chosen::Chosen;

    /// Where the text in these tests is going.
    fn somewhere() -> PathBuf {
        PathBuf::from("/home/ada/.config/alo/settings.toml")
    }

    /// Settings holding exactly these parts, with nothing else in them.
    fn settings(
        chosen: Option<Picked>,
        brought: Brought,
        providers: Providers,
        languages: Vec<Language>,
    ) -> Settings {
        Settings::of(chosen, brought, providers, languages, Setup::NotAnswered).unwrap()
    }

    /// One set of weights on somebody's own list.
    fn theirs(id: &str) -> Brought {
        let mut brought = Brought::default();
        brought
            .add(
                Weights::checked(id, 4_700_000_000)
                    .unwrap()
                    .measured(Driving::Reliably),
            )
            .unwrap();
        brought
    }

    /// One provider on somebody's own list, as the file's own way in builds it.
    fn mistral() -> Providers {
        let mut providers = Providers::default();
        providers
            .add(
                Provider::checked(
                    "Mistral",
                    "https://api.mistral.ai",
                    Region::Declared("the EU".to_owned()),
                    Some(crate::written::a_key_for("Mistral")),
                )
                .unwrap(),
            )
            .unwrap();
        providers
    }

    /// **Everything a person can have chosen survives being written and read
    /// again**, which is the whole promise `crate::Choosing` makes and the one
    /// thing no amount of care in the writer would establish on its own.
    #[test]
    fn every_part_of_a_persons_settings_comes_back_the_same() {
        for chosen in [
            None,
            Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
            )),
            Some(Picked::OnThisMachine(
                Chosen::of(Which::Brought, "my-finetune").unwrap(),
            )),
            Some(Picked::from_a_provider("Mistral", "mistral-small-latest").unwrap()),
        ] {
            let theirs = settings(
                chosen,
                theirs("my-finetune"),
                mistral(),
                vec![
                    Language::written("de").unwrap(),
                    Language::written("pt-BR").unwrap(),
                ],
            );
            let text = written(&theirs, &somewhere()).unwrap();
            assert_eq!(read(&text, &somewhere()).unwrap(), theirs, "{text}");
        }
    }

    /// **A machine nobody has configured writes a file that says only which
    /// shape it is**, with no section invented for a person who has chosen
    /// nothing. ADR 0025's reading is that no default is written that a person
    /// did not choose, and an empty `[answers]` would be one.
    #[test]
    fn settings_nobody_has_touched_are_a_format_and_nothing_else() {
        let text = written(&Settings::untouched(), &somewhere()).unwrap();
        assert_eq!(text.trim(), format!("format = {THE_FORMAT}"));
        assert_eq!(read(&text, &somewhere()).unwrap(), Settings::untouched());
    }

    /// **What is written is this alo OS's own shape**, rather than the oldest
    /// one the value would have fitted in.
    #[test]
    fn what_is_written_says_which_shape_this_alo_os_writes() {
        let text = written(
            &settings(None, Brought::default(), Providers::default(), Vec::new()),
            &somewhere(),
        )
        .unwrap();
        assert!(text.contains(&format!("format = {THE_FORMAT}")), "{text}");
    }

    /// **A person who brought nothing has no `[[brought]]`**, and one who added
    /// no provider has no `[[provider]]` — the file says what they said and not
    /// that they said nothing.
    #[test]
    fn nothing_brought_and_nothing_added_are_keys_that_are_not_there() {
        let text = written(&Settings::untouched(), &somewhere()).unwrap();
        assert!(!text.contains("brought"), "{text}");
        assert!(!text.contains("provider"), "{text}");
        assert!(!text.contains("reading"), "{text}");
        assert!(!text.contains("answers"), "{text}");
        assert!(!text.contains("setup"), "{text}");
    }

    /// **An answered setup survives being written and read again**, in both of
    /// the two shapes it arrives in: a person who chose a source, and a person
    /// who declined. The second is the one that would otherwise be lost — its
    /// settings are a `format` line and a `[setup]` section, and without the
    /// section it is indistinguishable from a machine nobody has asked.
    #[test]
    fn an_answered_setup_comes_back_the_same_whether_or_not_anything_was_chosen() {
        for chosen in [
            None,
            Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
            )),
        ] {
            let theirs = Settings::of(
                chosen,
                Brought::default(),
                Providers::default(),
                Vec::new(),
                Setup::Answered,
            )
            .unwrap();

            let text = written(&theirs, &somewhere()).unwrap();
            assert!(text.contains("answered = true"), "{text}");
            assert_eq!(read(&text, &somewhere()).unwrap(), theirs, "{text}");
        }
    }

    /// **Whether a provider is asked for a key is written out either way.** The
    /// value that matters is the one the absent form does not cover, and a
    /// person reading their own file should not have to know which way an
    /// absent key falls.
    #[test]
    fn a_provider_that_takes_no_credential_says_so_in_the_file() {
        let mut providers = Providers::default();
        providers
            .add(
                Provider::checked("Local", "http://127.0.0.1:11434", Region::Unknown, None)
                    .unwrap(),
            )
            .unwrap();
        let text = written(
            &settings(None, Brought::default(), providers, Vec::new()),
            &somewhere(),
        )
        .unwrap();
        assert!(text.contains("needs-a-key = false"), "{text}");
        // And a region nobody stated is absent rather than written as a word
        // that would read like one.
        assert!(!text.contains("region"), "{text}");
    }

    /// **A provider carrying something this file cannot hold is refused, and
    /// nothing is written.** The list of model names a provider offers is a
    /// public field on `alo_models::Provider` and has no key here; a writer that
    /// dropped it would answer `Ok(())` for a provider the machine afterwards
    /// describes differently.
    #[test]
    fn a_provider_this_file_cannot_hold_is_refused_rather_than_trimmed() {
        let mut providers = mistral();
        providers
            .configured
            .first_mut()
            .unwrap()
            .models
            .push("mistral-small-latest".to_owned());

        let refused = written(
            &settings(None, Brought::default(), providers, Vec::new()),
            &somewhere(),
        )
        .unwrap_err();

        assert!(
            matches!(refused, NotWritten::NotExpressible { .. }),
            "{refused:?}"
        );
    }

    /// **And so is a credential kept anywhere but where this file derives it.**
    /// `crate::written::a_key_for` is the one answer to *where is this
    /// provider's key*, asked in both directions, and a value disagreeing with
    /// it is a change the machine would not have carried out.
    #[test]
    fn a_credential_kept_somewhere_this_file_cannot_name_is_refused() {
        let mut providers = Providers::default();
        providers
            .add(
                Provider::checked(
                    "Mistral",
                    "https://api.mistral.ai",
                    Region::Unknown,
                    Some(SecretRef::named("somewhere/else")),
                )
                .unwrap(),
            )
            .unwrap();

        assert!(matches!(
            written(
                &settings(None, Brought::default(), providers, Vec::new()),
                &somewhere()
            )
            .unwrap_err(),
            NotWritten::NotExpressible { .. }
        ));
    }

    /// **A provider whose address is not https is refused as what it is**,
    /// in `alo-models`' words, rather than as a defect in alo OS — the round
    /// trip would have caught it too, and said the wrong thing.
    #[test]
    fn a_provider_whose_address_is_not_https_is_refused_as_a_provider_rather_than_as_a_defect() {
        let mut providers = Providers::default();
        providers
            .add(Provider {
                name: "Somewhere".to_owned(),
                endpoint: "http://192.168.1.10:11434".to_owned(),
                region: Region::Unknown,
                key: Some(crate::written::a_key_for("Somewhere")),
                models: Vec::new(),
            })
            .unwrap();

        let refused = written(
            &settings(None, Brought::default(), providers, Vec::new()),
            &somewhere(),
        )
        .unwrap_err();

        assert!(
            matches!(
                refused,
                NotWritten::NotAProvider {
                    why: alo_models::ProviderError::InsecureEndpoint,
                    ..
                }
            ),
            "{refused:?}"
        );
        let said = refused.said(&crate::testing::in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("https"), "{said}");
    }

    /// **A name with space around it is refused rather than quietly trimmed.**
    /// `Picked::from_a_provider` trims, so this choice cannot be made through
    /// it — but the variant has public fields, and a settings panel that built
    /// one by hand would otherwise be told a name was written that was not.
    #[test]
    fn a_choice_the_file_would_change_on_the_way_back_is_refused() {
        let refused = written(
            &settings(
                Some(Picked::FromAProvider {
                    provider: " Mistral ".to_owned(),
                    model: "mistral-small-latest".to_owned(),
                }),
                Brought::default(),
                mistral(),
                Vec::new(),
            ),
            &somewhere(),
        )
        .unwrap_err();

        assert!(
            matches!(refused, NotWritten::NotExpressible { .. }),
            "{refused:?}"
        );
    }

    /// **The two lists stay apart on the way out**, which is the ambiguity
    /// `crate::chosen` exists to resolve: the same name in the catalogue and on
    /// the person's own list are two different answers to *what runs my turn*.
    #[test]
    fn the_same_name_from_the_two_lists_is_written_as_two_different_choices() {
        let one = written(
            &settings(
                Some(Picked::OnThisMachine(
                    Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
                )),
                Brought::default(),
                Providers::default(),
                Vec::new(),
            ),
            &somewhere(),
        )
        .unwrap();
        let other = written(
            &settings(
                Some(Picked::OnThisMachine(
                    Chosen::of(Which::Brought, "mistral-small").unwrap(),
                )),
                theirs("mistral-small"),
                Providers::default(),
                Vec::new(),
            ),
            &somewhere(),
        )
        .unwrap();

        assert!(one.contains(r#"catalogue = "mistral-small""#), "{one}");
        assert!(other.contains(r#"brought = "mistral-small""#), "{other}");
    }

    /// **The languages keep their order**, which is the whole of what the list
    /// means: best first, and a second language the person named themselves.
    #[test]
    fn the_languages_are_written_best_first() {
        let text = written(
            &settings(
                None,
                Brought::default(),
                Providers::default(),
                vec![
                    Language::written("de").unwrap(),
                    Language::written("en").unwrap(),
                ],
            ),
            &somewhere(),
        )
        .unwrap();
        assert!(text.contains(r#"languages = ["de", "en"]"#), "{text}");
    }

    /// **Nothing a person reads comes out of this file.** The reasons carried
    /// here are for whoever is fixing the machine, and the sentence the person
    /// gets is `crate::unwritten`'s — which says only that nothing changed.
    #[test]
    fn a_reason_kept_for_a_maintainer_is_not_a_sentence_for_a_person() {
        let mut providers = mistral();
        providers
            .configured
            .first_mut()
            .unwrap()
            .models
            .push("mistral-small-latest".to_owned());
        let refused = written(
            &settings(None, Brought::default(), providers, Vec::new()),
            &somewhere(),
        )
        .unwrap_err();

        let said = refused.said(&crate::testing::in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(
            said.text()
                .contains("nothing in your settings has been changed"),
            "{said}"
        );
    }
}
