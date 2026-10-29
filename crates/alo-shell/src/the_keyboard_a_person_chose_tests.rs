//! What becomes of the keyboard a person picked, read from the file it wrote
//! and the answer it gave.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err or variant is the failure being reported"
)]

use super::{AboutTheKeyboard, TheKeyboardAPersonChose};

/// The keyboard a language is offered first, which is what the sheets hand
/// out for a language chosen with no keyboard chosen.
fn the_keyboard_for(tag: &str) -> alo_keyboards::Layout {
    alo_keyboards::Keyboards::offered_with(
        &alo_strings::Language::written(tag).expect("a language tag"),
    )
    .in_use()
    .clone()
}

/// A folder this test alone writes in, emptied of whatever a previous run left.
fn a_folder_of_our_own(what: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-shell-chose-{what}"));
    if folder.exists() {
        std::fs::remove_dir_all(&folder).expect("a folder of this test's own");
    }
    std::fs::create_dir_all(&folder).expect("a folder of this test's own");
    folder
}

#[test]
fn nothing_chosen_does_nothing_and_writes_nothing() {
    let at = a_folder_of_our_own("nothing").join(alo_keyboards::keeping::THE_FILE);
    let mut chose = TheKeyboardAPersonChose::nothing_yet();
    assert_eq!(chose.chose(None, &at), AboutTheKeyboard::Unchanged);
    assert!(
        !at.exists(),
        "a screen nobody had chosen on wrote a keyboards file"
    );
}

#[test]
fn choosing_french_keeps_it_and_hands_back_the_layout_to_apply() {
    let at = a_folder_of_our_own("french").join(alo_keyboards::keeping::THE_FILE);
    let mut chose = TheKeyboardAPersonChose::nothing_yet();
    let AboutTheKeyboard::Apply(layout) = chose.chose(Some(&the_keyboard_for("fr")), &at) else {
        panic!("choosing French did not ask for a keyboard to be applied");
    };
    assert_eq!(layout.name(), "fr", "French is typed on an AZERTY keyboard");
    assert!(at.exists(), "the choice was not written down");

    // **Read back through the crate that owns the file**, not by parsing it
    // here: a second reading of this shape is a second thing to get wrong, and
    // the compositor reads it with exactly this call at the next boot.
    let read = alo_keyboards::keeping::read(&at).expect("what was written reads back");
    assert_eq!(read.in_use().name(), "fr");
}

#[test]
fn one_layout_is_kept_and_it_is_the_language_s_first_offer() {
    // **This test first asserted that the alternatives were kept too**, and
    // reading the file back found one layout. `Keyboards::offered_with` sets
    // `These::just(offer.first())`, and that is right: `be`, `ca` and
    // `ch(fr)` are what a setting offers a French speaker next, and
    // installing all four here would hand somebody a switch key cycling
    // through three keyboards they never asked for.
    let at = a_folder_of_our_own("one-layout").join(alo_keyboards::keeping::THE_FILE);
    let mut chose = TheKeyboardAPersonChose::nothing_yet();
    assert!(matches!(
        chose.chose(Some(&the_keyboard_for("fr")), &at),
        AboutTheKeyboard::Apply(_)
    ));
    let read = alo_keyboards::keeping::read(&at).expect("what was written reads back");
    assert_eq!(
        read.all().how_many(),
        1,
        "more than the first offer was kept"
    );

    // And it is that crate's answer rather than a second table here.
    let offer = alo_keyboards::offered_with(
        &alo_strings::Language::written("fr").expect("fr is a language tag"),
    )
    .expect("French has an offer");
    assert_eq!(read.in_use().name(), offer.first().name());
    // The alternatives are still reachable, which is what makes keeping one
    // defensible rather than lossy.
    assert!(
        offer.beside_it().iter().any(|it| it.name() == "be"),
        "Belgian is not offered beside French, so keeping one layout does lose it"
    );
}

#[test]
fn choosing_the_same_language_again_changes_nothing() {
    // **A person's keys arrive one at a time and the choice is asked after
    // every one.** Without this, every arrow press would rewrite the file and
    // rebuild the keymap -- and `Server::the_keyboard_is_now` releases every
    // held key first, so a rebuild nobody asked for drops a key somebody is
    // holding.
    let at = a_folder_of_our_own("again").join(alo_keyboards::keeping::THE_FILE);
    let mut chose = TheKeyboardAPersonChose::nothing_yet();
    assert!(matches!(
        chose.chose(Some(&the_keyboard_for("de")), &at),
        AboutTheKeyboard::Apply(_)
    ));
    for _ in 0..5 {
        assert_eq!(
            chose.chose(Some(&the_keyboard_for("de")), &at),
            AboutTheKeyboard::Unchanged,
            "choosing the same language again asked for a keymap rebuild"
        );
    }
}

#[test]
fn changing_the_choice_applies_the_new_one() {
    let at = a_folder_of_our_own("changed").join(alo_keyboards::keeping::THE_FILE);
    let mut chose = TheKeyboardAPersonChose::nothing_yet();
    assert!(matches!(
        chose.chose(Some(&the_keyboard_for("de")), &at),
        AboutTheKeyboard::Apply(_)
    ));
    let AboutTheKeyboard::Apply(layout) = chose.chose(Some(&the_keyboard_for("fr")), &at) else {
        panic!("changing the choice did not apply the new one");
    };
    assert_eq!(layout.name(), "fr");
    let read = alo_keyboards::keeping::read(&at).expect("what was written reads back");
    assert_eq!(
        read.in_use().name(),
        "fr",
        "the file still names the language that was changed away from"
    );
}

#[test]
fn a_choice_that_cannot_be_written_is_not_applied() {
    // **The order is the point.** A machine whose keyboard changed and whose
    // file did not would type through one layout now and another after a
    // restart, and a person who set their keyboard and found it back tomorrow
    // has been told a lie by a screen. So a failed write means the keyboard
    // stays as it was -- which is what the file still says.
    //
    // **The parent is a FILE**, not a missing folder: `alo_kept` creates a
    // folder that is not there, so a missing parent is not a failure at all.
    // This test first used one and the write succeeded -- the refusal path was
    // never reached and the assertion below passed for the wrong reason.
    let folder = a_folder_of_our_own("unwritable");
    let in_the_way = folder.join("in-the-way");
    std::fs::write(&in_the_way, b"not a folder").expect("a file in the way");
    let at = in_the_way.join(alo_keyboards::keeping::THE_FILE);
    let mut chose = TheKeyboardAPersonChose::nothing_yet();
    let answer = chose.chose(Some(&the_keyboard_for("fr")), &at);
    assert!(
        matches!(answer, AboutTheKeyboard::NotKept(_)),
        "a choice that could not be written was applied anyway: {answer:?}"
    );

    // And it did not remember it, so the next frame tries again rather than
    // believing a keyboard is in use that never was.
    let folder = a_folder_of_our_own("unwritable-then-writable");
    let at = folder.join(alo_keyboards::keeping::THE_FILE);
    assert!(
        matches!(
            chose.chose(Some(&the_keyboard_for("fr")), &at),
            AboutTheKeyboard::Apply(_)
        ),
        "after a failed write the choice was treated as already applied"
    );
}

/// **A keyboard that is not the language's is kept as chosen** — English
/// read on a German keyboard, the owner's example of 2026-10-28.
#[test]
fn a_keyboard_other_than_the_languages_is_kept_as_chosen() {
    let at = a_folder_of_our_own("english-on-german").join(alo_keyboards::keeping::THE_FILE);
    let mut chose = TheKeyboardAPersonChose::nothing_yet();
    let german = alo_keyboards::Layout::named("de").expect("de");
    assert_eq!(
        chose.chose(Some(&german), &at),
        AboutTheKeyboard::Apply(german.clone())
    );
    let read = alo_keyboards::keeping::read(&at).expect("what was written reads back");
    assert_eq!(read.in_use(), &german);
}
