//! The whole journey, in the order it really happens: a machine's verbs become
//! a prompt, a model's answers become a grade, the grade becomes a catalogue
//! entry, and the entry decides whether that machine gives the model the agent.
//!
//! It is one test file rather than two crates' worth of unit tests because the
//! thing worth proving is the join. `alo-driving` can grade correctly and
//! `alo-models` can choose correctly while the grade one produces is not the
//! one the other reads, and nothing inside either crate would notice.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capability::Verbs;
use alo_driving::{Exercises, Measured};
use alo_models::{Catalogue, Driving, NoAgentHere, model_words};
use alo_strings::Strings;

/// The verbs alo OS itself offers, as `alo-agentd` would put them on a
/// registry.
fn the_verbs() -> Verbs {
    let mut verbs = Verbs::default();
    alo_files::declare_into(&mut verbs).unwrap();
    alo_applications::declare_into(&mut verbs).unwrap();
    verbs
}

/// What a model that got the shape right would have written, for each exercise.
fn right_answer(named: &str) -> String {
    let (door, verb, given): (&str, &str, &[(&str, &str)]) = match named {
        "list" => (
            "read",
            "list_folder",
            &[("folder", "\"/home/anna/Invoices\"")],
        ),
        "read" => (
            "read",
            "read_file",
            &[("file", "\"/home/anna/Invoices/march.pdf\"")],
        ),
        "find" => (
            "read",
            "find_in_folder",
            &[
                ("folder", "\"/home/anna/Invoices\""),
                ("named", "\"october\""),
                ("most", "20"),
            ],
        ),
        "rename" => (
            "propose",
            "rename_file",
            &[
                ("file", "\"/home/anna/Invoices/scan001.pdf\""),
                ("name", "\"march.pdf\""),
            ],
        ),
        "move" => (
            "propose",
            "move_file",
            &[
                ("file", "\"/home/anna/Invoices/march.pdf\""),
                ("into", "\"/home/anna/Archive\""),
            ],
        ),
        "archive" => (
            "propose",
            "archive_folder",
            &[
                ("folder", "\"/home/anna/Invoices\""),
                ("into", "\"/home/anna/Archive\""),
                ("name", "\"invoices\""),
            ],
        ),
        "open" => (
            "propose",
            "open_application",
            &[("application", "\"org.alo.Writer\"")],
        ),
        "focus" => (
            "propose",
            "focus_application",
            &[("application", "\"org.alo.Writer\"")],
        ),
        "close" => (
            "propose",
            "close_application",
            &[("application", "\"org.alo.Writer\"")],
        ),
        _ => (
            "propose",
            "arrange_application",
            &[
                ("application", "\"org.alo.Writer\""),
                ("where", "\"left_half\""),
            ],
        ),
    };
    let arguments: Vec<String> = given
        .iter()
        .map(|(named, is)| format!("{{\"named\":\"{named}\",\"is\":{is}}}"))
        .collect();
    format!(
        "{{\"format\":1,\"asks\":{{\"{door}\":{{\"verb\":\"{verb}\",\"given\":[{}]}}}}}}",
        arguments.join(",")
    )
}

/// A catalogue of one model, stating the grade it was given.
fn catalogue_of(grade: &str) -> Catalogue {
    Catalogue::parse(&format!(
        "[[model]]\n\
         id = \"measured-one\"\n\
         name = \"The measured one\"\n\
         publisher = \"p\"\n\
         parameters_b = 3.0\n\
         quantisation = \"Q4_K_M\"\n\
         download_bytes = 2_000_000_000\n\
         min_vram_gb = 3.0\n\
         min_ram_gb = 5.0\n\
         on_cpu = \"comfortable\"\n\
         drives_verbs = \"{grade}\"\n\
         upstream = \"https://example.test/measured\"\n\
         licence = {{ name = \"Apache-2.0\", spdx = \"Apache-2.0\", commercial_use = \"permitted\" }}\n"
    ))
    .unwrap()
}

/// How a catalogue entry writes the grade this crate produced.
fn as_the_catalogue_writes_it(grade: Driving) -> &'static str {
    match grade {
        Driving::Reliably => "reliably",
        Driving::Sometimes => "sometimes",
        Driving::Rarely => "rarely",
        Driving::NotMeasured => "not-measured",
    }
}

/// **A model that drives the verbs is measured, graded, catalogued and given
/// the agent** — the whole of what ADR 0007's *since it was accepted* section
/// asked for, in one line of travel.
#[test]
fn a_model_that_drives_the_verbs_is_the_one_a_laptop_is_given() {
    let exercises = Exercises::over(&the_verbs()).unwrap();
    let attempts = exercises
        .all()
        .map(|exercise| exercises.attempt(exercise, &right_answer(exercise.named())))
        .collect();
    let grade = Measured::of(&exercises, attempts).unwrap().grade();
    assert_eq!(grade, Driving::Reliably);

    let catalogue = catalogue_of(as_the_catalogue_writes_it(grade));
    let chosen = catalogue.agent_for_cpu(16.0).unwrap();
    assert_eq!(chosen.id, "measured-one");
    assert!(chosen.can_be_the_agent());
}

/// **And a model that answers in prose is measured, graded, catalogued and
/// refused** — with sentences naming the weights the person may already have
/// and the two places ADR 0008 leaves open, rather than a machine quietly
/// asking one of them.
#[test]
fn a_model_that_only_writes_sentences_is_refused_and_the_person_is_told_where_else() {
    let exercises = Exercises::over(&the_verbs()).unwrap();
    let attempts = exercises
        .all()
        .map(|exercise| {
            exercises.attempt(
                exercise,
                "Of course — I've gone ahead and taken care of that for you.",
            )
        })
        .collect();
    let grade = Measured::of(&exercises, attempts).unwrap().grade();
    assert_eq!(grade, Driving::Rarely);

    let catalogue = catalogue_of(as_the_catalogue_writes_it(grade));
    let refused = catalogue.agent_for_cpu(16.0).unwrap_err();
    assert_eq!(
        refused,
        NoAgentHere::NoneClearsTheBar {
            to_choose_from: 1,
            measured: 1,
        }
    );

    let strings = Strings::of(model_words().unwrap());
    let [why, brought, elsewhere] = refused.lines(&strings);
    assert!(why.text().contains("often enough"), "{why}");
    assert!(
        brought.text().contains("weights you already have"),
        "{brought}"
    );
    assert!(elsewhere.text().contains("paired with"), "{elsewhere}");
    assert!(elsewhere.text().contains("provider"), "{elsewhere}");
    // It still runs here. The machine lost the agent, not the model.
    assert_eq!(catalogue.runnable_on_cpu(16.0).len(), 1);
}

/// **A model nobody has measured is refused with the reason that is true of
/// it**, which is not the same reason as having failed. This is the state the
/// catalogue we ship is in today.
#[test]
fn an_unmeasured_model_is_refused_without_being_accused_of_anything() {
    let catalogue = catalogue_of("not-measured");
    let refused = catalogue.agent_for_cpu(16.0).unwrap_err();
    assert_eq!(refused, NoAgentHere::NoneMeasured { to_choose_from: 1 });

    let strings = Strings::of(model_words().unwrap());
    let [why, _, _] = refused.lines(&strings);
    assert!(why.text().contains("has been measured"), "{why}");

    // And the catalogue this repository ships is exactly that everywhere.
    let shipped = Catalogue::built_in().unwrap();
    assert!(matches!(
        shipped.agent_for_cpu(16.0),
        Err(NoAgentHere::NoneMeasured { .. })
    ));
}

/// The prompt a model is given describes the verbs the machine really has, in
/// the verbs' own declared words — so a model measured on this machine was
/// measured on this machine's capabilities.
#[test]
fn the_prompt_is_built_from_the_registry_it_is_scored_against() {
    let verbs = the_verbs();
    let exercises = Exercises::over(&verbs).unwrap();
    let text = exercises.prompt(exercises.of("move").unwrap());
    for verb in verbs.all() {
        assert!(text.contains(verb.name()), "{}", verb.name());
        assert!(text.contains(verb.purpose_as_written()), "{}", verb.name());
    }
    assert!(text.contains("/home/anna/Archive"), "{text}");
    // And nothing in it is a way to run something: the prompt describes the
    // closed list, which has no shape for a command in it.
    assert!(!text.contains("command"), "{text}");
}
