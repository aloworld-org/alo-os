//! What writing in VSCodium at *ask each time* will do, read without a
//! machine.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a missing step is the failure being reported"
)]

use alo_acting::said_on_the_log::{A_CARD_IS_UP, a_card_is_up};

use super::*;
use crate::keys::Chord;
use crate::step::Step;
use crate::the_first_walk::the_first_walk;

/// Where a step is in the first walk, by a predicate, or a panic naming it.
fn at(steps: &[Step], what: &str, is: impl Fn(&Step) -> bool) -> usize {
    steps
        .iter()
        .position(is)
        .unwrap_or_else(|| panic!("the walk never {what}"))
}

/// Where the look by this name is.
fn the_look(steps: &[Step], name: &str) -> usize {
    at(
        steps,
        &format!("looks at {name}"),
        |it| matches!(it, Step::Look { name: n, .. } if n == name),
    )
}

/// Where the line saying a card is up, with this many characters, is checked.
fn the_card(steps: &[Step], characters: usize) -> usize {
    let said = a_card_is_up(characters);
    let rest = said.trim_start_matches(A_CARD_IS_UP).to_owned();
    at(
        steps,
        &format!("checks a card showing {characters}"),
        |it| {
            matches!(it, Step::ExpectLine { phrase, and_says: Some(also), .. }
            if phrase == A_CARD_IS_UP && rest.starts_with(also.as_str()))
        },
    )
}

/// **It runs after the walk at full access**, behind the provider's gate,
/// because alo cannot be asked without a provider.
#[test]
fn it_follows_the_walk_at_full_access() {
    let steps = the_first_walk().steps().to_vec();
    let gate = at(&steps, "waits for the provider", |it| {
        matches!(it, Step::OnlyWith(_))
    });
    let full = the_look(&steps, "the-file-checked");
    let asked = the_look(&steps, "the-ask-at-each-time-typed");
    assert!(gate < full && full < asked, "{gate} {full} {asked}");
}

/// **The person sets the level and gives alo VSCodium, in Settings, before
/// asking** — a day long — so what is asked is answered at *ask each time*
/// and the open is not refused for want of a grant.
#[test]
fn the_level_and_the_grant_come_before_the_ask() {
    let steps = the_first_walk().steps().to_vec();
    let level = the_look(&steps, "ask-each-time-chosen");
    let given = the_look(&steps, "vscodium-given");
    let asked = the_look(&steps, "the-ask-at-each-time-typed");
    assert!(level < asked && given < asked, "{level} {given} {asked}");
    let how_long = the_look(&steps, "on-a-day");
    assert!(how_long < given);
}

/// **Each change is approved on its own card, by a person's keys**: after the
/// line saying the card is up, Tab moves to *Allow once* and Enter chooses
/// it — once for the open, which types nothing, and once for the typing.
#[test]
fn each_card_is_answered_with_allow_once() {
    let steps = the_first_walk().steps().to_vec();
    for characters in [0, THE_TEXT_AT_EACH_ASK.chars().count()] {
        let card = the_card(&steps, characters);
        let keys: Vec<&Step> = steps
            .iter()
            .skip(card + 1)
            .filter(|it| matches!(it, Step::Press(_)))
            .take(2)
            .collect();
        assert_eq!(
            keys,
            [
                &Step::Press(Chord::key("tab")),
                &Step::Press(Chord::key("ret"))
            ],
            "after the card showing {characters} characters"
        );
    }
}

/// **The typing card shows the whole text**, counted in characters, before
/// it is approved; and the file is read back for that text afterwards.
#[test]
fn the_text_is_on_the_card_and_then_in_the_file() {
    let steps = the_first_walk().steps().to_vec();
    let card = the_card(&steps, THE_TEXT_AT_EACH_ASK.chars().count());
    let in_the_file = at(&steps, "reads the second file back", |it| {
        matches!(it, Step::ExpectLine { phrase, and_says: Some(also), .. }
            if phrase == THE_TAG_AT_EACH_ASK && also == THE_TEXT_AT_EACH_ASK)
    });
    assert!(card < in_the_file);
    assert_ne!(THE_FILE_AT_EACH_ASK, crate::writing_in_vscodium::THE_FILE);
    assert_ne!(THE_TEXT_AT_EACH_ASK, crate::writing_in_vscodium::THE_TEXT);
}

/// **The row of *ask each time* and the row of *Give alo an application* are
/// counted from the lists Settings draws from**, so a list that grows moves
/// the walk with it.
#[test]
fn the_rows_are_counted_from_the_lists_settings_draws() {
    let ask_each_time = alo_capability::HowCodeRuns::ALL
        .iter()
        .position(|it| *it == alo_capability::HowCodeRuns::AskingEachTime);
    assert_eq!(Some(ASK_EACH_TIME), ask_each_time);
    // *See what alo did*, then one row for each place alo can work.
    assert_eq!(
        UP_TO_GIVE_AN_APPLICATION,
        alo_capability::AloInApplications::ALL.len() + 1
    );
}

/// **On a fresh machine VSCodium is the last application Settings lists**,
/// which is why End reaches it: the shipped names, ordered as `alo-picking`
/// orders what Settings shows.
#[test]
fn vscodium_is_the_last_application_listed() {
    let names = alo_software::ShippedNames::pinned()
        .unwrap_or_else(|why| panic!("the shipped names: {why:?}"));
    let listed = alo_picking::Applications::of(
        names
            .every()
            .iter()
            .map(|(identifier, name)| (identifier.clone(), Some(name.clone()))),
    );
    let last = listed.listed().last().map(alo_picking::Listed::identifier);
    assert_eq!(last, Some(crate::writing_in_vscodium::VSCODIUM));
}

/// **What the person types names the file and the text**: the line that makes
/// the file, beginning with a space, and the ask, both typed by the walk.
#[test]
fn what_is_typed_names_the_file_and_the_text() {
    let making = making_the_file_at_each_ask();
    assert!(making.starts_with(' '), "{making:?}");
    assert!(
        making.contains(&format!("~/{THE_FILE_AT_EACH_ASK}")),
        "{making}"
    );
    assert!(making.contains(THE_FILE_IS_MADE_AT_EACH_ASK), "{making}");
    let ask = the_ask_at_each_time();
    assert!(ask.contains(THE_FILE_AT_EACH_ASK) && ask.contains(THE_TEXT_AT_EACH_ASK));
    assert!(ask.contains("VSCodium"), "{ask}");
    let steps = the_first_walk().steps().to_vec();
    for typed in [making, ask] {
        assert!(
            steps.contains(&Step::Type(typed.clone())),
            "never typed {typed:?}"
        );
    }
}

/// **Each card's check names its count exactly, and waits five minutes** for
/// a model to propose it.
#[test]
fn each_card_check_names_its_count_and_waits_long_enough() {
    let steps = the_first_walk().steps().to_vec();
    for characters in [0, THE_TEXT_AT_EACH_ASK.chars().count()] {
        let at = the_card(&steps, characters);
        let Some(Step::ExpectLine {
            and_says: Some(also),
            within,
            ..
        }) = steps.get(at)
        else {
            panic!("no card check at {at}");
        };
        assert_eq!(also, &format!(" {characters} characters of text to type"));
        assert!(a_card_is_up(characters).contains(also.as_str()));
        assert_eq!(*within, std::time::Duration::from_secs(300));
    }
}
