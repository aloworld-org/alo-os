//! A step is taken only by the expected verb, through its own door, with
//! arguments the machine accepts and the values the road decides — and each
//! way of leaving the road is told apart.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_applications::OPEN_FILE_IN_APPLICATION;
use alo_applications::opening_a_file::PATH;
use alo_applications::verbs::APPLICATION;

use super::taking;
use crate::a_road::{Answered, Step};
use crate::testing::{proposing, read, reading, running, the_machine, the_verbs, the_words};
use crate::went_wrong::WentWrong;

const EDITOR: &str = "com.vscodium.codium";
const FILE: &str = "/var/home/alo/hello.txt";

fn opening() -> Step {
    Step::of(OPEN_FILE_IN_APPLICATION, Answered::AFileOpened)
        .expecting(APPLICATION, EDITOR)
        .expecting(PATH, FILE)
}

fn taken(line: &str) -> Result<(), WentWrong> {
    taking(
        &opening(),
        1,
        &read(line),
        &the_verbs(),
        &the_machine(),
        &the_words(),
    )
    .map(|_| ())
}

#[test]
fn the_expected_verb_with_the_expected_values_takes_the_step() {
    assert_eq!(
        taken(&proposing(
            OPEN_FILE_IN_APPLICATION,
            &[(APPLICATION, EDITOR), (PATH, FILE)]
        )),
        Ok(())
    );
}

#[test]
fn the_arguments_in_another_order_still_take_the_step() {
    assert_eq!(
        taken(&proposing(
            OPEN_FILE_IN_APPLICATION,
            &[(PATH, FILE), (APPLICATION, EDITOR)]
        )),
        Ok(())
    );
}

#[test]
fn another_verb_is_another_road() {
    assert_eq!(
        taken(&proposing("open_application", &[(APPLICATION, EDITOR)])),
        Err(WentWrong::AnotherVerb {
            step: 1,
            expected: OPEN_FILE_IN_APPLICATION.to_owned(),
            asked: "open_application".to_owned(),
            given: crate::went_wrong::as_said(&[(
                APPLICATION,
                alo_capability::Given::text(EDITOR)
            )]),
        })
    );
}

#[test]
fn discovery_is_not_a_recording_or_application_action() {
    let result = taking(
        &opening(),
        1,
        &alo_protocol::FromAnAgent::Capabilities,
        &the_verbs(),
        &the_machine(),
        &the_words(),
    );
    assert!(matches!(
        result,
        Err(WentWrong::AnotherVerb { step: 1, .. })
    ));
}

/// The machine reads an argument given twice with the same value as one
/// value (#1268), so the check must too, or it fails a try the machine runs.
#[test]
fn an_argument_given_twice_with_one_value_is_taken_as_the_machine_takes_it() {
    assert_eq!(
        taken(&proposing(
            OPEN_FILE_IN_APPLICATION,
            &[(APPLICATION, EDITOR), (APPLICATION, EDITOR), (PATH, FILE)],
        )),
        Ok(())
    );
}

/// Two different values for one argument are still refused, by the machine
/// and so by the check. This is what the model sent most often on
/// 2026-11-15: `path` twice, the second holding the text to type.
#[test]
fn an_argument_given_twice_with_two_values_is_refused_as_the_machine_refuses_it() {
    let went = taken(&proposing(
        OPEN_FILE_IN_APPLICATION,
        &[(APPLICATION, EDITOR), (PATH, FILE), (PATH, "hello, alo")],
    ));
    assert!(
        matches!(went, Err(WentWrong::ArgumentsRefused { step: 1, .. })),
        "{went:?}"
    );
}

#[test]
fn an_argument_left_out_is_refused() {
    let went = taken(&proposing(OPEN_FILE_IN_APPLICATION, &[(PATH, FILE)]));
    assert!(
        matches!(went, Err(WentWrong::ArgumentsRefused { .. })),
        "{went:?}"
    );
}

#[test]
fn an_argument_the_verb_does_not_take_is_refused() {
    let went = taken(&proposing(
        OPEN_FILE_IN_APPLICATION,
        &[(APPLICATION, EDITOR), (PATH, FILE), ("window", "hello.txt")],
    ));
    assert!(
        matches!(went, Err(WentWrong::ArgumentsRefused { .. })),
        "{went:?}"
    );
}

#[test]
fn a_change_sent_as_a_read_is_the_wrong_door() {
    assert_eq!(
        taken(&reading(
            OPEN_FILE_IN_APPLICATION,
            &[(APPLICATION, EDITOR), (PATH, FILE)]
        )),
        Err(WentWrong::TheWrongDoor {
            step: 1,
            verb: OPEN_FILE_IN_APPLICATION.to_owned(),
            door: "read",
        })
    );
}

#[test]
fn another_path_is_another_value() {
    assert_eq!(
        taken(&proposing(
            OPEN_FILE_IN_APPLICATION,
            &[(APPLICATION, EDITOR), (PATH, "/home/alo/hello.txt")]
        )),
        Err(WentWrong::AnotherValue {
            step: 1,
            argument: PATH.to_owned(),
            expected: FILE.to_owned(),
            given: "/home/alo/hello.txt".to_owned(),
        })
    );
}

#[test]
fn another_application_is_another_value() {
    let went = taken(&proposing(
        OPEN_FILE_IN_APPLICATION,
        &[(APPLICATION, "org.gnome.TextEditor"), (PATH, FILE)],
    ));
    assert!(
        matches!(&went, Err(WentWrong::AnotherValue { argument, .. }) if argument == APPLICATION),
        "{went:?}"
    );
}

#[test]
fn a_command_where_a_verb_was_expected_is_its_own_way_off_the_road() {
    assert_eq!(
        taken(&running("codium /var/home/alo/hello.txt")),
        Err(WentWrong::RanACommand {
            step: 1,
            expected: OPEN_FILE_IN_APPLICATION.to_owned(),
            command: "codium /var/home/alo/hello.txt".to_owned(),
        })
    );
}

#[test]
fn a_read_sent_as_a_change_is_the_wrong_door() {
    let looking = Step::of(
        alo_adapters::fallback_verbs::READ_WINDOW,
        Answered::AFileOpened,
    )
    .expecting(APPLICATION, EDITOR);
    let went = taking(
        &looking,
        2,
        &read(&proposing(
            alo_adapters::fallback_verbs::READ_WINDOW,
            &[(APPLICATION, EDITOR)],
        )),
        &the_verbs(),
        &the_machine(),
        &the_words(),
    )
    .map(|_| ());
    assert_eq!(
        went,
        Err(WentWrong::TheWrongDoor {
            step: 2,
            verb: alo_adapters::fallback_verbs::READ_WINDOW.to_owned(),
            door: "change",
        })
    );
}

/// An installation of the catalogue's GIMP, from the place it comes from.
fn installing_gimp() -> (Step, String) {
    let catalogue = alo_software::Catalogue::decided().expect("the catalogue reads");
    let gimp = catalogue
        .every()
        .iter()
        .find(|listed| listed.application().identifier().ends_with(".GIMP"))
        .expect("the catalogue offers GIMP");
    let step = Step::of(
        alo_software::INSTALL_APPLICATION,
        Answered::With(alo_protocol::Done::Installed {
            application: gimp.application().identifier().to_owned(),
            sandboxed: true,
        }),
    )
    .expecting(
        alo_software::words::APPLICATION,
        gimp.application().identifier(),
    )
    .expecting(alo_software::words::SOURCE, gimp.source().as_str());
    (step, gimp.source().as_str().to_owned())
}

/// **An application named by its name takes the step**, as the machine reads
/// it: the catalogue's identifier, by the turn's own function.
#[test]
fn an_installation_named_by_its_name_is_read_as_the_machine_reads_it() {
    let (step, source) = installing_gimp();
    let line = proposing(
        alo_software::INSTALL_APPLICATION,
        &[
            (alo_software::words::APPLICATION, "GIMP"),
            (alo_software::words::SOURCE, &source),
        ],
    );
    assert_eq!(
        taking(
            &step,
            1,
            &read(&line),
            &the_verbs(),
            &the_machine(),
            &the_words()
        )
        .map(|_| ()),
        Ok(())
    );
}

/// **The refusal case**: a wrong identifier is still the wrong application,
/// and a name nothing answers to is refused, as on a machine.
#[test]
fn a_wrong_identifier_or_an_unknown_name_leaves_the_road() {
    let (step, source) = installing_gimp();
    for said in ["org.gnome.GIMP", "Paintbox"] {
        let line = proposing(
            alo_software::INSTALL_APPLICATION,
            &[
                (alo_software::words::APPLICATION, said),
                (alo_software::words::SOURCE, &source),
            ],
        );
        assert!(
            taking(
                &step,
                1,
                &read(&line),
                &the_verbs(),
                &the_machine(),
                &the_words()
            )
            .is_err(),
            "{said} took the step"
        );
    }
}

/// **The editor named as the model is told it** — *VSCodium*, on a machine at
/// full access that tells it so — is the editor, as a booted machine takes it.
#[test]
fn the_editor_named_by_its_name_takes_the_step() {
    assert_eq!(
        taken(&proposing(
            OPEN_FILE_IN_APPLICATION,
            &[(APPLICATION, "VSCodium"), (PATH, FILE)]
        )),
        Ok(())
    );
}

/// **And only there**: on a machine that told the model no name, *VSCodium*
/// is another value, as that machine would refuse it.
#[test]
fn a_name_the_machine_does_not_give_is_another_value() {
    let unnamed = crate::as_the_machine_names::AsTheMachineNames::of(
        &alo_instructing::WhatItMayReach::nothing()
            .at_full_access(true)
            .with_these_applications(Some(alo_instructing::TheApplicationsInstalled::of([(
                EDITOR, None,
            )]))),
    );
    let went = taking(
        &opening(),
        1,
        &read(&proposing(
            OPEN_FILE_IN_APPLICATION,
            &[(APPLICATION, "VSCodium"), (PATH, FILE)],
        )),
        &the_verbs(),
        &unnamed,
        &the_words(),
    )
    .map(|_| ());
    assert_eq!(
        went,
        Err(WentWrong::AnotherValue {
            step: 1,
            argument: APPLICATION.to_owned(),
            expected: EDITOR.to_owned(),
            given: "VSCodium".to_owned(),
        })
    );
}

/// The typing step as the set writes it: the editor's entry named after the
/// file, in a window titled after it, with the keyboard.
fn typing_in_the_editor() -> Step {
    Step::of(
        alo_adapters::fallback_verbs::TYPE_TEXT,
        Answered::TypedAndSaved,
    )
    .expecting(alo_adapters::fallback_verbs::APPLICATION, EDITOR)
    .expecting_the_field(
        alo_adapters::fallback_verbs::NAME,
        alo_adapters::places_for_text::APlaceForText::as_shown(
            "hello.txt",
            "hello.txt - VSCodium",
            true,
        ),
    )
}

fn typed_into(name: &str) -> Result<(), WentWrong> {
    let line = proposing(
        alo_adapters::fallback_verbs::TYPE_TEXT,
        &[
            (alo_adapters::fallback_verbs::APPLICATION, EDITOR),
            (alo_adapters::fallback_verbs::NAME, name),
            (alo_adapters::fallback_verbs::TEXT, "hello"),
        ],
    );
    taking(
        &typing_in_the_editor(),
        3,
        &read(&line),
        &the_verbs(),
        &the_machine(),
        &the_words(),
    )
    .map(|_| ())
}

/// **The field named as the machine's typing takes it**: by its own name, or
/// by the title of the window it has the keyboard in — the name the model
/// read, which a booted machine types into (walk 43748155951, relayed).
#[test]
fn the_field_named_by_its_name_or_its_windows_title_takes_the_step() {
    assert_eq!(typed_into("hello.txt"), Ok(()));
    assert_eq!(typed_into("hello.txt - VSCodium"), Ok(()));
}

/// **A name the machine would not find is another value**, as the machine
/// refuses it.
#[test]
fn a_field_the_machine_would_not_find_is_another_value() {
    assert_eq!(
        typed_into("Document"),
        Err(WentWrong::AnotherValue {
            step: 3,
            argument: alo_adapters::fallback_verbs::NAME.to_owned(),
            expected: "hello.txt".to_owned(),
            given: "Document".to_owned(),
        })
    );
}
