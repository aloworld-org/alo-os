//! **Every sentence a person meets at their desk, and what none of them may
//! say.**
//!
//! Task 7 of `docs/autonomy/v0-5-hands-on-the-desktop-plan.md`, its first half.
//! Five crates put words in front of a person while they work — splitting a
//! screen, moving between desktops, dropping a file, choosing from a menu,
//! typing in their own language — and this holds all five to one standard at
//! once, because a person does not meet them one crate at a time.
//!
//! | | |
//! |---|---|
//! | `alo-dividing` | halves, quarters, and why a boundary will not move |
//! | `alo-desktops` | which desktop, and why a name or a chord is refused |
//! | `alo-keyboards` | layouts, the compose key, and what could not be kept |
//! | `alo-handing` | what letting go of a drag would do |
//! | `alo-menus` | what a thing offers |
//!
//! # Why the machinery may never be named
//!
//! A person moving a window has not agreed to learn what a keysym is. The
//! libraries underneath — the one that reads a touchpad, the one that maps a
//! key to a letter, the one that turns a sequence into a character — are rented
//! and replaceable (ADR 0011), and a sentence that named one would be a sentence
//! that has to change when we change what we rent. The person's sentence must
//! outlive the machinery, so it may not mention it.
//!
//! This is the same clause the access-and-language plan holds over its own
//! words, and the list here is this plan's own: the touchpad library, the
//! keyboard map, a key's code, and the frameworks that compose characters.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_strings::Word;

/// Every crate whose words a person meets at their desk, and its vocabulary.
fn what_the_desktop_says() -> Vec<(&'static str, Vec<Word>)> {
    vec![
        ("alo-dividing", alo_dividing::words::EVERY_WORD.to_vec()),
        ("alo-desktops", alo_desktops::words::EVERY_WORD.to_vec()),
        ("alo-keyboards", alo_keyboards::words::EVERY_WORD.to_vec()),
        ("alo-handing", alo_handing::words::EVERY_WORD.to_vec()),
        ("alo-menus", alo_menus::words::EVERY_WORD.to_vec()),
    ]
}

/// **Every sentence these crates can say is in the machine's vocabulary, and
/// carries a note for whoever translates it.**
///
/// Read out of `alo-saying`'s assembled vocabulary rather than out of each
/// crate's own constants, because what a person hears is what the assembled
/// machine says — a word a crate declares and the machine never collected is a
/// sentence nobody will ever read.
#[test]
fn every_sentence_is_collected_and_carries_a_note() {
    let machine =
        alo_saying::everything_this_machine_can_say().expect("this machine's own vocabulary");
    let mut how_many = 0;
    for (crate_named, words) in what_the_desktop_says() {
        for word in words {
            assert!(
                machine.phrase(&word.key()).is_some(),
                "{crate_named} says {} and this machine does not know it",
                word.named()
            );
            assert!(
                word.note().is_some(),
                "{crate_named}'s {} has no note for whoever translates it",
                word.named()
            );
            assert!(
                !word.says().trim().is_empty(),
                "{crate_named}'s {} says nothing",
                word.named()
            );
            how_many += 1;
        }
    }
    assert!(
        how_many >= 80,
        "only {how_many} sentences were checked, which is fewer than these crates have"
    );
}

/// **No sentence names the machinery underneath.**
///
/// The plan's own list, and the names each of those things actually goes by —
/// because a rule that forbade only the polite name would pass a sentence
/// saying `evdev`.
#[test]
fn no_sentence_names_a_library_a_keymap_or_a_key_code() {
    const NEVER: [&str; 14] = [
        // The library that reads a touchpad, and the interface under it.
        "libinput",
        "evdev",
        // The keyboard map, by each of its names.
        "xkb",
        "xkbcommon",
        "keymap",
        "keysym",
        "scancode",
        "keycode",
        // The frameworks that compose characters.
        "ibus",
        "fcitx",
        "input method framework",
        "input-method framework",
        // The window system itself, which a person is not asked to know either.
        "wayland",
        "compositor",
    ];
    for (crate_named, words) in what_the_desktop_says() {
        for word in words {
            let said = word.says().to_lowercase();
            for never in NEVER {
                assert!(
                    !said.contains(never),
                    "{crate_named}'s {} says `{never}` to a person",
                    word.named()
                );
            }
        }
    }
}

/// **A translator's note is a note, not a name.**
///
/// A note is the one place a machine's own word may appear — it is written for
/// somebody translating, who needs to know what a string is for. What it may
/// not do is stand in for the explanation: a note that says only *the name of a
/// thing* tells a translator nothing, and this is the smallest check that
/// catches one.
#[test]
fn every_note_says_enough_to_translate_by() {
    for (crate_named, words) in what_the_desktop_says() {
        for word in words {
            let note = word.note().unwrap_or_default();
            assert!(
                note.len() >= 40,
                "{crate_named}'s {} has a note too short to translate by: {note:?}",
                word.named()
            );
        }
    }
}
