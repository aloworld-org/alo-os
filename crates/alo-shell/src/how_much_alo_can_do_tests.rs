//! **Who is asked, what Continue writes, and what happens when it cannot.**
//!
//! Every write is read back through `alo_remembering::the_level_remembered` and
//! through Settings' own value, so none of these can pass on the value the test
//! handed in.
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capability::HowCodeRuns;

use super::{ARRIVES_CHOSEN, HowMuchAloCanDo, THE_THREE, TheRingIs, ToAsk, Went};
use crate::a_press_on_a_setup_sheet::OnASheet;
use crate::how_code_runs_in_settings::HowCodeRunsHere;
use crate::sign_in_keys::SignInKey;

/// A private directory, and the level's file inside it.
fn somewhere() -> (tempfile::TempDir, std::path::PathBuf) {
    let at = tempfile::tempdir().expect("a temporary directory");
    let file = at.path().join("how-code-runs.toml");
    (at, file)
}

/// The sheet, for a person who has not chosen, or a failure.
fn asked(file: &std::path::Path) -> HowMuchAloCanDo {
    match HowMuchAloCanDo::to_ask(file) {
        ToAsk::Ask(it) => it,
        other => panic!("a person who has not chosen was not asked: {other:?}"),
    }
}

/// What the file says now.
fn kept(file: &std::path::Path) -> Option<HowCodeRuns> {
    alo_remembering::the_level_remembered(file).expect("the level reads")
}

/// **A person with no file is asked, with Full access already chosen**, and
/// the ring on it; and nothing is written by being asked.
#[test]
fn a_person_who_has_not_chosen_is_asked_with_full_access_already_chosen() {
    let (_at, file) = somewhere();
    let asking = asked(&file);
    assert_eq!(ARRIVES_CHOSEN, HowCodeRuns::AtFullTrust);
    assert_eq!(asking.chosen(), HowCodeRuns::AtFullTrust);
    assert_eq!(asking.ring(), TheRingIs::OnAChoice(2));
    assert!(!asking.was_not_saved());
    assert!(!file.exists(), "being asked wrote something");
}

/// **Continue at once writes Full access**, and Settings then reads it as
/// chosen — the same file through the same road, not a second copy.
#[test]
fn continue_at_once_writes_full_access_where_settings_reads_it() {
    let (_at, file) = somewhere();
    let mut asking = asked(&file);
    assert_eq!(asking.pressed_on(OnASheet::WayOn), Went::Kept(HowCodeRuns::AtFullTrust));
    assert_eq!(kept(&file), Some(HowCodeRuns::AtFullTrust));
    assert_eq!(
        HowCodeRunsHere::at_sign_in(&file).chosen(),
        Some(HowCodeRuns::AtFullTrust),
        "Settings does not read what the first sign-in wrote"
    );
}

/// **Every level can be chosen here and is what is written**, by key and by
/// press alike, so this cannot pass by handling one and refusing the rest.
#[test]
fn each_level_can_be_chosen_and_is_what_continue_writes() {
    for (place, level) in THE_THREE.into_iter().enumerate() {
        // By the keyboard: up to the row, Enter to choose, Enter to go on.
        let (_at, file) = somewhere();
        let mut asking = asked(&file);
        while asking.ring() != TheRingIs::OnAChoice(place) {
            assert_eq!(asking.pressed(SignInKey::Earlier), Went::Stayed);
        }
        assert_eq!(asking.pressed(SignInKey::Enter), Went::Stayed);
        assert_eq!(asking.chosen(), level);
        assert_eq!(asking.ring(), TheRingIs::OnContinue);
        assert_eq!(asking.pressed(SignInKey::Enter), Went::Kept(level));
        assert_eq!(kept(&file), Some(level), "by the keyboard");

        // By the pointer: the row, then Continue.
        let (_at, file) = somewhere();
        let mut asking = asked(&file);
        assert_eq!(asking.pressed_on(OnASheet::Choice(place)), Went::Stayed);
        assert_eq!(asking.pressed_on(OnASheet::WayOn), Went::Kept(level));
        assert_eq!(kept(&file), Some(level), "by the pointer");
    }
}

/// **Moving is not choosing**: the arrows and Tab move the ring round every
/// row and Continue, and neither the choice nor the file changes.
#[test]
fn moving_the_ring_chooses_nothing_and_writes_nothing() {
    let (_at, file) = somewhere();
    let mut asking = asked(&file);
    let mut seen = Vec::new();
    for key in [SignInKey::Later, SignInKey::OtherField, SignInKey::Earlier] {
        for _ in 0..5 {
            assert_eq!(asking.pressed(key), Went::Stayed);
            seen.push(asking.ring());
            assert_eq!(asking.chosen(), HowCodeRuns::AtFullTrust, "{key:?} chose");
        }
    }
    for place in 0..THE_THREE.len() {
        assert!(seen.contains(&TheRingIs::OnAChoice(place)), "row {place} unreachable");
    }
    assert!(seen.contains(&TheRingIs::OnContinue), "Continue unreachable");
    assert!(!file.exists(), "moving the ring wrote the level");
}

/// **Nothing else does anything**: letters, Erase, Escape, the way back and
/// *Accessibility* — none is drawn as a way out, so none is one.
#[test]
fn keys_and_presses_that_mean_nothing_here_change_nothing() {
    let (_at, file) = somewhere();
    let mut asking = asked(&file);
    for key in [
        SignInKey::Letter('a'),
        SignInKey::Erase,
        SignInKey::Clear,
        SignInKey::Nothing,
    ] {
        assert_eq!(asking.pressed(key), Went::Stayed, "{key:?}");
    }
    for on in [
        OnASheet::WayBack,
        OnASheet::Accessibility,
        OnASheet::Choice(THE_THREE.len()),
    ] {
        assert_eq!(asking.pressed_on(on), Went::Stayed, "{on:?}");
    }
    assert_eq!(asking.chosen(), HowCodeRuns::AtFullTrust);
    assert_eq!(asking.ring(), TheRingIs::OnAChoice(2));
    assert!(!file.exists(), "a key that means nothing wrote the level");
}

/// **A person who chose is never asked again**, whichever level, whether they
/// chose it here or in Settings.
#[test]
fn a_person_who_chose_is_not_asked() {
    for level in THE_THREE {
        let (_at, file) = somewhere();
        alo_remembering::the_level_kept(&file, level).expect("kept");
        assert!(
            matches!(HowMuchAloCanDo::to_ask(&file), ToAsk::AlreadyChosen(it) if it == level),
            "{level:?} was asked again"
        );
    }
    // And the sheet's own answer counts the same.
    let (_at, file) = somewhere();
    let mut asking = asked(&file);
    assert_eq!(asking.pressed_on(OnASheet::Choice(0)), Went::Stayed);
    assert_eq!(asking.pressed_on(OnASheet::WayOn), Went::Kept(HowCodeRuns::InASealedBox));
    assert!(matches!(
        HowMuchAloCanDo::to_ask(&file),
        ToAsk::AlreadyChosen(HowCodeRuns::InASealedBox)
    ));
}

/// **A file that would not read is not asked about and is not written over.**
/// Asking would replace a file nobody has read with whatever is pre-selected.
#[test]
fn a_file_that_would_not_read_is_not_asked_about_and_is_left_as_it_is() {
    let (_at, file) = somewhere();
    let written = "format = 1\nchose = \"full-trust\"\n";
    std::fs::write(&file, written).expect("written");
    assert!(matches!(HowMuchAloCanDo::to_ask(&file), ToAsk::WouldNotRead));
    assert_eq!(std::fs::read_to_string(&file).expect("read"), written);
}

/// **A choice that cannot be written is said, and changes nothing**; a second
/// that cannot goes on with nothing chosen rather than keeping the person from
/// their desktop — and the next sign-in asks again.
#[test]
fn a_choice_that_cannot_be_written_is_said_and_never_traps_anybody() {
    let at = tempfile::tempdir().expect("a temporary directory");
    // A folder that is not there: no reader, root or not, can write into it.
    let file = at.path().join("not-there").join("how-code-runs.toml");
    let mut asking = asked(&file);
    assert_eq!(asking.pressed_on(OnASheet::WayOn), Went::Stayed);
    assert!(asking.was_not_saved(), "the refusal is not said");
    assert!(!file.exists());
    assert_eq!(asking.chosen(), HowCodeRuns::AtFullTrust, "the refusal changed the choice");

    assert_eq!(asking.pressed(SignInKey::Enter), Went::OnWithNothingChosen);
    assert!(!file.exists(), "a refusal wrote something");
    assert!(
        matches!(HowMuchAloCanDo::to_ask(&file), ToAsk::Ask(_)),
        "a person whose choice was not saved is not asked again"
    );
}

/// **Once the disk agrees, the next Continue writes**, and choosing again
/// takes the refusal's sentence away.
#[test]
fn a_refusal_then_a_disk_that_agrees_writes_the_choice() {
    let at = tempfile::tempdir().expect("a temporary directory");
    let folder = at.path().join("later");
    let file = folder.join("how-code-runs.toml");
    let mut asking = asked(&file);
    assert_eq!(asking.pressed_on(OnASheet::WayOn), Went::Stayed);
    assert!(asking.was_not_saved());
    assert_eq!(asking.pressed_on(OnASheet::Choice(1)), Went::Stayed);
    assert!(!asking.was_not_saved(), "a new choice still shows the old refusal");

    std::fs::create_dir(&folder).expect("the folder");
    assert_eq!(asking.pressed_on(OnASheet::WayOn), Went::Kept(HowCodeRuns::AskingEachTime));
    assert_eq!(kept(&file), Some(HowCodeRuns::AskingEachTime));
}
