//! What a person is asked the first time their machine starts, end to end,
//! against the machine's own vocabulary and a real settings file on a real disk.
//!
//! The crate's own tests are written against its own list of strings and its
//! own fixtures. These ask the questions that list cannot: whether what setup
//! says survives being put beside everything else alo OS says, whether the
//! answer a person gives is one the daemon would read back, and whether the two
//! states that look the same really are two states on a disk.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_choosing::{Chosen, Settings, Which};
use alo_saying::everything_this_machine_can_say;
use alo_setting_up::{
    Answer, EVERY_WORD, NotSetUp, Offered, SettingUp, THE_FOUR, TheAgent,
    what_would_lean_on_a_person,
};
use alo_strings::Strings;

/// A folder on this machine's own disk that only this test uses, made fresh.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-setting-up-asked-{what}"));
    if folder.exists() {
        std::fs::remove_dir_all(&folder).unwrap();
    }
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// Where a person's settings would be, on a machine nobody has configured.
fn a_machine_on_its_first_morning(what: &str) -> PathBuf {
    a_folder_of_our_own(what)
        .join(alo_choosing::THE_FOLDER)
        .join(alo_choosing::THE_SETTINGS)
}

/// Everything this machine can say, as a person reads it.
fn the_machines_words() -> Strings {
    Strings::of(everything_this_machine_can_say().expect("everything alo OS can say"))
}

/// **The four are four, the local one is first, and nothing is selected.**
///
/// ADR 0025's whole reading of *by default*, in the state a machine arrives in:
/// the four configurations are listed, the local one leads because it is the
/// one that needs nothing added, and ordering is not weight — no selection
/// exists until a person makes one.
#[test]
fn setup_lists_four_with_the_local_one_first_and_nothing_chosen() {
    let at = a_machine_on_its_first_morning("four");

    let setting_up = SettingUp::at(&at).expect("a machine nobody has configured");

    assert_eq!(setting_up.offers().len(), 4);
    assert_eq!(setting_up.offers(), THE_FOUR);
    assert_eq!(setting_up.offers()[0], Offered::OnThisMachine);
    assert_eq!(setting_up.selected(), None);
    assert!(!setting_up.is_answered());
    assert!(!at.exists(), "opening setup wrote in somebody's settings");
}

/// **A setup nobody has answered and a setup answered *not at all* are two
/// different machines, on the disk.**
///
/// Both have nothing answering questions, which is why they would otherwise be
/// one file — and the second is a **finished** setup rather than a skipped one.
/// Read back through `alo_choosing::Settings::at`, which is the door
/// `alo-agentd` reads a person's settings through, so what is measured here is
/// what the machine itself would see.
#[test]
fn a_setup_nobody_answered_and_one_answered_not_at_all_are_two_different_files() {
    let unasked = a_machine_on_its_first_morning("unasked");
    let declined = a_machine_on_its_first_morning("declined");

    // Nobody has been asked: no file at all, and settings that say so.
    assert!(!unasked.exists());
    let never = Settings::at(&unasked).expect("a person who has not chosen");
    assert_eq!(never, Settings::untouched());
    assert!(!never.setup().is_answered());

    // And a person who declined.
    let mut setting_up = SettingUp::at(&declined).expect("a machine nobody has configured");
    setting_up.select(Offered::NotAtAll);
    setting_up
        .answer(&Answer::NotAtAll)
        .expect("an answer their settings take");

    let after = Settings::at(&declined).expect("the settings setup wrote");
    assert!(after.chosen().is_none(), "declining chose something");
    assert!(
        after.setup().is_answered(),
        "declining did not finish setup"
    );
    assert_ne!(
        after, never,
        "a declined setup reads as one nobody answered"
    );

    // And the agent's surfaces are absent rather than greyed out (ADR 0009).
    assert_eq!(setting_up.the_agent(), TheAgent::Absent);
}

/// **A source chosen at setup is written through `alo-choosing`'s own shapes and
/// nowhere else**, and the machine reads it back as the choice it was.
#[test]
fn a_choice_made_at_setup_is_the_choice_the_machine_reads_back() {
    let at = a_machine_on_its_first_morning("chosen");
    let mut setting_up = SettingUp::at(&at).expect("a machine nobody has configured");
    setting_up.select(Offered::OnThisMachine);

    setting_up
        .answer(&Answer::OnThisMachine(
            Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
        ))
        .expect("an answer their settings take");

    let read = Settings::at(&at).expect("the settings setup wrote");
    assert_eq!(read.chosen().unwrap().model(), "mistral-small");
    assert_eq!(
        read.chosen().unwrap().on_this_machine().unwrap().which(),
        Which::Catalogue
    );
    assert!(read.setup().is_answered());
    assert_eq!(setting_up.the_agent(), TheAgent::Present);

    // The file is the person's own, written where their settings live and
    // nowhere else.
    assert!(at.exists());
    assert!(std::fs::read_to_string(&at).unwrap().contains("format = 3"));
}

/// **Every string setup can say is in the machine's one vocabulary.** A word
/// declared here and left out of `alo-saying`'s list is a sentence that reaches
/// a person as a key — which is what `alo-collected` exists for, met here for
/// the words this change adds.
#[test]
fn everything_setup_says_is_something_the_machine_can_say() {
    let vocabulary = everything_this_machine_can_say().expect("everything alo OS can say");
    for word in EVERY_WORD {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "the machine cannot say {}",
            word.named()
        );
    }
    assert_eq!(EVERY_WORD.len(), 14);
}

/// **Nothing setup says asks anybody to buy anything or leans on one of the
/// four**, asked of the strings as the machine really holds them rather than of
/// this crate's own copy of them.
///
/// This is ADR 0009's *no persuasion attached* and ADR 0025's *ordering is not
/// weight* in their copy half: a flow that selects nothing and then calls one of
/// the four the sensible one has moved the persuasion where no structural rule
/// can see it.
#[test]
fn nothing_setup_says_leans_on_a_person_who_is_choosing() {
    let vocabulary = everything_this_machine_can_say().expect("everything alo OS can say");
    let only_setups = alo_strings::Vocabulary::empty();
    let only_setups = EVERY_WORD.iter().fold(only_setups, |gathered, word| {
        gathered
            .and(
                vocabulary
                    .phrase(&word.key())
                    .expect("a string setup declared")
                    .clone(),
            )
            .expect("one key, one string")
    });

    let leant = what_would_lean_on_a_person(&only_setups);
    assert!(
        leant.is_empty(),
        "{}",
        leant
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// **Every one of the four reads as a name and a line in the machine's own
/// words**, with nothing coming back as a key. A choice a person cannot read is
/// a choice they cannot weigh against the other three.
#[test]
fn each_of_the_four_reads_in_the_machines_own_words() {
    let strings = the_machines_words();
    for offered in THE_FOUR {
        for said in [offered.called(&strings), offered.described(&strings)] {
            assert!(!said.is_a_bug(), "{offered:?}: {said}");
            assert!(!said.text().trim().is_empty(), "{offered:?}");
        }
    }
}

/// **Every way an answer goes nowhere reaches the person as a sentence**, in
/// the machine's own words — including the one that is `alo-choosing`'s, which
/// is carried rather than reworded.
#[test]
fn every_refusal_reaches_the_person_as_a_sentence_the_machine_can_say() {
    let strings = the_machines_words();
    let at = a_machine_on_its_first_morning("refusals");
    let mut setting_up = SettingUp::at(&at).expect("a machine nobody has configured");

    // Nothing selected.
    let nothing_selected = setting_up.answer(&Answer::NotAtAll).unwrap_err();
    assert!(matches!(nothing_selected, NotSetUp::NothingSelected));

    // A choice that is not the selected one.
    setting_up.select(Offered::NotAtAll);
    let another = setting_up
        .answer(&Answer::OnThisMachine(
            Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
        ))
        .unwrap_err();
    assert!(matches!(another, NotSetUp::AnotherChoice { .. }));

    // A machine on this network, which this machine keeps no list of.
    setting_up.select(Offered::OnAMachineOnThisNetwork);
    let unpaired = setting_up
        .answer(&Answer::OnAMachineOnThisNetwork)
        .unwrap_err();
    assert!(matches!(unpaired, NotSetUp::NoPairedMachine));

    for refused in [nothing_selected, another, unpaired] {
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{refused:?}: {said}");
        assert!(said.text().contains("setup is still waiting"), "{said}");
    }

    // And not one of the three wrote a byte of anybody's settings.
    assert!(!at.exists(), "a refused answer wrote a file");
}
