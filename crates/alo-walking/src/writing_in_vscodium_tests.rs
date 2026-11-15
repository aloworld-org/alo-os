//! What writing in VSCodium will check, read without a machine — and each
//! phrase it reads for held to the sentence the desktop really writes.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a missing step is the failure being reported"
)]

use alo_acting::said_on_the_log::{a_file_was_opened, alo_typed};
use alo_acting::{Opened, ToType, Typed};
use alo_applications::Application;

use super::*;
use crate::keys;
use crate::step::Step;

/// VSCodium, as the door names it.
fn vscodium() -> Application {
    match Application::identified(VSCODIUM) {
        Ok(it) => it,
        Err(why) => panic!("{VSCODIUM} is not an identifier: {why:?}"),
    }
}

/// **The phrases the walk reads for are the ones the desktop writes**, built
/// by the desktop's own sentence functions.
///
/// What would have to be true for this to pass while broken: the desktop or
/// the shell writing its log line without `alo_acting::said_on_the_log` —
/// which they do not, and which a search for the words would find.
#[test]
fn each_phrase_is_the_start_of_the_desktops_own_sentence() {
    let opened = a_file_was_opened(VSCODIUM, Opened::InAlosCopy);
    assert!(opened.starts_with(&opened_in_vscodium()), "{opened}");
    assert!(opened.contains(Opened::InAlosCopy.written()), "{opened}");

    let typed = alo_typed(
        &ToType::Text {
            application: vscodium(),
            bytes: THE_TEXT.len(),
        },
        Typed::Typed,
    );
    assert!(typed.starts_with(&typed_into_vscodium()), "{typed}");
    assert!(typed.ends_with(&Typed::Typed.written()), "{typed}");

    let saved = alo_typed(
        &ToType::Save {
            application: vscodium(),
        },
        Typed::AskedToSave,
    );
    assert!(saved.starts_with(&saved_in_vscodium()), "{saved}");
    assert!(saved.ends_with(&Typed::AskedToSave.written()), "{saved}");
    // A typing that was refused is not read as a save, nor a save as a
    // typing: the two phrases differ where the kinds do.
    assert!(!saved.starts_with(&typed_into_vscodium()));
}

/// **Every text it types can be typed**, so no run stops halfway through
/// the ask.
#[test]
fn every_text_it_types_can_be_typed() {
    for step in at_full_access(Walk::named("vscodium")).steps() {
        if let Step::Type(text) = step {
            assert!(keys::typing(text).is_ok(), "{text:?} cannot be typed");
        }
    }
}

/// **The file is checked only from what is said after the person asked the
/// journal for it**: the log is marked, then Enter is pressed on the
/// `systemd-cat` line, and the check reads since that mark — so nothing said
/// earlier, the ask itself included, can answer it.
///
/// What would have to be true for this to pass while broken: the ask's own
/// words reaching the journal under [`THE_TAG`], which nothing writes.
#[test]
fn the_file_is_checked_from_the_press_that_asks_for_it() {
    let walk = at_full_access(Walk::named("vscodium"));
    let steps = walk.steps();
    let Some(check) = steps.iter().position(|step| {
        matches!(step, Step::ExpectLine { phrase, and_says: Some(text), since: Since::Mark, .. }
            if phrase == THE_TAG && text == THE_TEXT)
    }) else {
        panic!("the walk does not check the file on disk");
    };
    let Some(mark) = steps
        .get(..check)
        .and_then(|before| before.iter().rposition(|step| matches!(step, Step::Mark)))
    else {
        panic!("no mark before the file's check");
    };
    let typed = mark.checked_sub(1).and_then(|at| steps.get(at));
    assert!(
        matches!(typed, Some(Step::Type(text)) if text.contains("systemd-cat")),
        "the step before the mark is {typed:?}"
    );
    assert!(
        matches!(steps.get(mark + 1), Some(Step::Press(chord)) if chord.the_key() == "ret"),
        "the step after the mark is {:?}",
        steps.get(mark + 1)
    );
}

/// **The three hand-overs are checked in the order the road makes them**:
/// opened, then typed, then saved, then the file.
#[test]
fn the_checks_come_in_the_roads_order() {
    let walk = at_full_access(Walk::named("vscodium"));
    let phrases: Vec<String> = walk
        .steps()
        .iter()
        .filter_map(|step| match step {
            Step::ExpectLine { phrase, .. } => Some(phrase.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        phrases,
        vec![
            THE_FILE_IS_MADE.to_owned(),
            opened_in_vscodium(),
            alo_wears_vscodium(),
            typed_into_vscodium(),
            saved_in_vscodium(),
            alos_work_in_vscodium_is_over(),
            THE_TAG.to_owned(),
        ]
    );
}

/// **The ask names the file, the application and the text**, so a model has
/// everything it needs in the person's own words.
#[test]
fn the_ask_names_the_file_the_application_and_the_text() {
    let ask = the_ask();
    for needed in [THE_FILE, "VSCodium", THE_TEXT] {
        assert!(ask.contains(needed), "{ask:?} does not say {needed}");
    }
}

/// **The command that makes the file is pinned, and makes the file the walk
/// later reads**: exactly this line, typed in the walk before the first check
/// that reads its tag, creating `~/hello.txt` — the path the last check hands
/// to the journal.
///
/// What would have to be true for this to pass while broken: the terminal's
/// shell reading `~` as something other than the person's home, which is the
/// same for both lines, so the two still name one file.
#[test]
fn the_command_that_makes_the_file_is_pinned_and_makes_the_file_checked_later() {
    assert_eq!(
        making_the_file(),
        " touch ~/hello.txt && systemd-cat -t alo-walk-the-file-is-made ls ~/hello.txt"
    );
    let typed: Vec<String> = at_full_access(Walk::named("vscodium"))
        .steps()
        .iter()
        .filter_map(|step| match step {
            Step::Type(text) => Some(text.clone()),
            _ => None,
        })
        .collect();
    let Some(made) = typed.iter().position(|text| *text == making_the_file()) else {
        panic!("the walk never types the command that makes the file: {typed:?}");
    };
    let Some(read) = typed
        .iter()
        .position(|text| text.contains(&format!("cat ~/{THE_FILE}")))
    else {
        panic!("the walk never reads the file back: {typed:?}");
    };
    assert!(made < read, "the file is read before it is made: {typed:?}");
    // The file made is the file read, and the one the ask names.
    assert!(making_the_file().contains(&format!("touch ~/{THE_FILE}")));
    assert!(the_ask().contains(THE_FILE));
    // The check that follows reads the tag this line writes, and the file's name.
    let checks_the_tag = at_full_access(Walk::named("vscodium"))
        .steps()
        .iter()
        .any(|step| {
            matches!(step, Step::ExpectLine { phrase, and_says: Some(also), .. }
                if phrase == THE_FILE_IS_MADE && also == THE_FILE)
        });
    assert!(checks_the_tag, "nothing checks the line the command writes");
}

/// **The phrases for alo's presence are the shell's own sentences**, built by
/// the functions the shell writes them with.
#[test]
fn the_presence_phrases_are_the_shells_own_sentences() {
    use alo_acting::said_on_the_log::{alo_wears_a_window_of, alos_work_is_over_in_a_window_of};
    assert_eq!(alo_wears_vscodium(), alo_wears_a_window_of(VSCODIUM));
    assert_eq!(
        alos_work_in_vscodium_is_over(),
        alos_work_is_over_in_a_window_of(VSCODIUM)
    );
}

/// **The teal edge is checked round VSCodium while alo works in it, and gone
/// once the turn has ended**: drawn after the shell says alo wears the window
/// and before the typing is checked; gone after the shell says the work is
/// over — and the edge drawn is checked first, so *gone* is never the
/// verdict of a check that could not see an edge at all.
#[test]
fn the_edge_is_checked_on_while_alo_works_and_off_after() {
    let walk = at_full_access(Walk::named("vscodium"));
    let steps = walk.steps();
    let at = |wanted: &dyn Fn(&Step) -> bool, what: &str| {
        steps
            .iter()
            .position(|step| wanted(step))
            .unwrap_or_else(|| panic!("the walk has no step: {what}"))
    };
    let on = at(
        &|step| matches!(step, Step::ExpectLine { phrase, .. } if *phrase == alo_wears_vscodium()),
        "alo wears VSCodium",
    );
    let drawn = at(
        &|step| matches!(step, Step::TheEdge { drawn: true, .. }),
        "the edge drawn",
    );
    let typed = at(
        &|step| matches!(step, Step::ExpectLine { phrase, .. } if *phrase == typed_into_vscodium()),
        "the typing",
    );
    let over = at(
        &|step| matches!(step, Step::ExpectLine { phrase, .. } if *phrase == alos_work_in_vscodium_is_over()),
        "alo's work is over",
    );
    let gone = at(
        &|step| matches!(step, Step::TheEdge { drawn: false, .. }),
        "the edge gone",
    );
    assert!(on < drawn && drawn < typed, "{on} {drawn} {typed}");
    assert!(typed < over && over < gone, "{typed} {over} {gone}");
}
