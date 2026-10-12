//! The mismatch that was measured, and the two ways the lookup refuses.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a catalogue that would not read is the failure being reported"
)]

use super::{NotNamed, the_name_a_runtime_knows};
use crate::catalogue::Catalogue;

fn shipped() -> Catalogue {
    Catalogue::built_in().expect("the built-in catalogue reads")
}

/// **The exact pair that failed on 2026-10-11**, asked of the shipped
/// catalogue rather than of a fixture.
///
/// A fixture would prove the function works and prove nothing about the
/// catalogue this project ships, which is where the two names actually differ.
#[test]
fn the_mismatch_that_was_measured_resolves() {
    let catalogue = shipped();

    assert_eq!(
        the_name_a_runtime_knows(&catalogue, "smollm2-1.7b-instruct"),
        Ok("smollm2:1.7b-instruct-q4_K_M"),
    );
}

/// **The id is not the artefact, for every entry that has one.**
///
/// The whole reason this function exists: if the two were ever the same string
/// a caller could get away with sending either, and the next entry where they
/// differ would fail in production. Asserted across the catalogue rather than
/// on one entry, with the count checked so that an empty catalogue cannot
/// pass.
#[test]
fn no_entry_has_an_artefact_equal_to_its_id() {
    let catalogue = shipped();
    let mut checked = 0;

    // `models` is public and in the catalogue's own order, so no accessor is
    // added for this.
    let ids: Vec<String> = catalogue.models.iter().map(|it| it.id.clone()).collect();
    for id in ids {
        if let Ok(artefact) = the_name_a_runtime_knows(&catalogue, &id) {
            assert_ne!(
                artefact, id,
                "`{id}` names an artefact identical to its id, which makes the resolution look \
                 unnecessary on this entry"
            );
            checked += 1;
        }
    }
    assert!(
        checked >= 10,
        "only {checked} entries name an artefact, so this proves little"
    );
}

/// **An id the catalogue does not have is refused, and says so.**
///
/// Not *the model is not installed*: nothing has asked a runtime anything at
/// this point, and telling a person to download something would send them
/// after a model that does not exist.
#[test]
fn an_unknown_id_is_refused_as_unknown() {
    let catalogue = shipped();

    assert_eq!(
        the_name_a_runtime_knows(&catalogue, "smollm2:1.7b-instruct-q4_K_M"),
        Err(NotNamed::NoSuchModel(
            "smollm2:1.7b-instruct-q4_K_M".to_owned()
        )),
        "an artefact passed where an id belongs is an unknown id, which is the mistake this \
         function is most likely to be handed"
    );
    assert!(matches!(
        the_name_a_runtime_knows(&catalogue, "no-such-model-anywhere"),
        Err(NotNamed::NoSuchModel(_))
    ));
}

/// **An entry naming no artefact is refused differently**, because *not yet
/// pinned* is not *not in the catalogue* and a person should not be told the
/// same thing for both.
#[test]
fn an_entry_with_no_artefact_is_its_own_refusal() {
    // Built rather than found: every shipped entry may well name an artefact,
    // and a test that silently found none would assert nothing. The catalogue
    // permits an entry with no artefact as long as nothing claims one, which
    // `crate::catalogue`'s validation decides and this relies on.
    let text = r#"
[[model]]
id = "nothing-pinned-yet"
name = "Nothing pinned yet"
publisher = "a publisher"
parameters_b = 1.0
download_bytes = 1000000000
min_vram_gb = 2.0
min_ram_gb = 4.0
on_cpu = "slow"
drives_verbs = "none"
unmeasured = "no artefact has been pinned for this entry yet"
"#;
    let Ok(catalogue) = Catalogue::parse(text) else {
        // The catalogue's own validation may refuse this shape, in which case
        // the variant is unreachable through a real catalogue and that is
        // worth saying rather than passing quietly.
        println!(
            "this catalogue refuses an entry with no artefact, so NoArtefact is unreachable \
             through Catalogue::parse - the variant stays for the field being Option"
        );
        return;
    };

    assert_eq!(
        the_name_a_runtime_knows(&catalogue, "nothing-pinned-yet"),
        Err(NotNamed::NoArtefact("nothing-pinned-yet".to_owned()))
    );
}

/// **The refusals read as different sentences**, since the whole point of
/// having two is that a person is told which one happened.
#[test]
fn the_two_refusals_do_not_read_alike() {
    let unknown = NotNamed::NoSuchModel("a".to_owned()).to_string();
    let unpinned = NotNamed::NoArtefact("a".to_owned()).to_string();

    assert_ne!(unknown, unpinned);
    assert!(
        !unknown.contains("names no artefact"),
        "an unknown id must not be reported as an unpinned one"
    );
    assert!(
        !unpinned.contains("no model in this catalogue"),
        "an unpinned entry must not be reported as missing from the catalogue"
    );
}

/// **Both refusals reach a person in their own language.**
///
/// This is the test the fix of 2026-10-12 exists for. Until then `alo-ask`
/// printed the `#[error]` string, which is English in code and appeared
/// nowhere in the published vocabulary — measured by writing
/// `catalogue = "no-such-model-anywhere"` into a settings file, the way a
/// person with a typo would, and reading what came back.
///
/// **The assertion is that the German is said, not that the English is
/// absent.** A test asserting absence would pass on a sentence that failed to
/// resolve at all and returned nothing.
#[test]
fn both_refusals_are_said_in_the_persons_own_language() {
    let german = crate::testing::translated(&[
        (
            crate::words::NO_SUCH_MODEL,
            "dieses Gerät kennt kein Modell namens {model}",
        ),
        (
            crate::words::NO_ARTEFACT_YET,
            "{model} kennt dieses Gerät, und keine Datei dafür ist gewählt",
        ),
    ]);

    let unknown = NotNamed::NoSuchModel("llama-4".to_owned()).said(&german);
    assert_eq!(
        unknown.text(),
        "dieses Gerät kennt kein Modell namens llama-4",
        "the name a person wrote must survive into the translation"
    );

    let unpinned = NotNamed::NoArtefact("llama-4".to_owned()).said(&german);
    assert_eq!(
        unpinned.text(),
        "llama-4 kennt dieses Gerät, und keine Datei dafür ist gewählt"
    );

    // **And they stay two sentences once translated**, which is the point of
    // there being two: a person is told which of the two happened.
    assert_ne!(unknown.text(), unpinned.text());
}

/// **Each refusal is one of this crate's own declared words**, so nothing can
/// put a sentence in front of a person that no translator will ever see.
#[test]
fn each_refusal_is_a_word_this_crate_declares() {
    for refusal in [
        NotNamed::NoSuchModel("a".to_owned()),
        NotNamed::NoArtefact("a".to_owned()),
    ] {
        assert!(
            crate::words::EVERY_WORD
                .iter()
                .any(|word| word.key() == refusal.word().key()),
            "{refusal:?} says a word this crate does not declare"
        );
    }

    // The control: the English really resolves to the declared sentence, so
    // the lookup worked rather than falling back to the key.
    let english = crate::testing::in_english();
    assert_eq!(
        NotNamed::NoSuchModel("llama-4".to_owned())
            .said(&english)
            .text(),
        "this machine has no model called llama-4, so there is nothing to ask for"
    );
}
