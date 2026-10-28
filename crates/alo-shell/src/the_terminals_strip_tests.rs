//! The strip across the top of the terminal, held to where the design puts its
//! words, to the display's scale, and to the vocabulary.
//!
//! **Measured as ink, not as items placed.** A face that is missing lays out a
//! box of exactly the right size with nothing in it, so every placement here is
//! checked to have glyphs in it as well.

#![expect(
    clippy::panic,
    reason = "in a test, a panic naming how many boxes were drawn is the failure being reported"
)]

use alo_terminal::{WhereTheShellRuns, measures};
use smithay::utils::{Physical, Rectangle};

use super::{WhatTheStripSays, the_strip, the_words_in, where_and_who};
use crate::painted::Inked;

/// The panel's white and the strip's `text/muted`.
const COLOURS: ([u8; 3], [u8; 3]) = ([255, 255, 255], [0x59, 0x6B, 0x78]);

/// A window the size the design drew one at.
fn a_window() -> Rectangle<i32, Physical> {
    Rectangle::new((200, 120).into(), (1040, 800).into())
}

/// The shell's own fonts, as a machine builds them.
fn fonts() -> cosmic_text::FontSystem {
    crate::WindowControlLabels::new()
        .expect("the shell's bundled faces load")
        .fonts
}

/// The terminal's words, in English.
fn says() -> alo_strings::Strings {
    let mut vocabulary = alo_strings::Vocabulary::empty();
    alo_terminal::words::declare_into(&mut vocabulary).expect("the terminal's own words declare");
    alo_strings::Strings::of(vocabulary)
}

/// A strip that knows everything the design shows.
fn everything() -> WhatTheStripSays<'static> {
    WhatTheStripSays {
        the_folder: Some("~/Projects/launch"),
        runs: WhereTheShellRuns::OnThisMachine,
        who: Some("disan"),
    }
}

/// How many pixels of a placed box are not the ground.
fn ink(placed: &Inked) -> usize {
    placed
        .pixels
        .iter()
        .filter(|pixel| **pixel != COLOURS.0)
        .count()
}

/// The right edge of the ink in a placed box, in the display's pixels.
fn where_the_ink_ends(placed: &Inked) -> i32 {
    let wide = placed.area.size.w.max(1);
    let last = placed
        .pixels
        .iter()
        .enumerate()
        .filter(|(_, pixel)| **pixel != COLOURS.0)
        .map(|(at, _)| i32::try_from(at).unwrap_or(0) % wide)
        .max()
        .unwrap_or(0);
    placed.area.loc.x + last + 1
}

#[test]
fn the_words_at_the_right_are_where_and_who_joined_as_the_design_joins_them() {
    assert_eq!(where_and_who(&says(), everything()), "Local \u{b7} disan");
}

/// **No name is said as no name**, not as an empty one after a dot.
#[test]
fn with_no_name_the_words_are_where_it_runs_alone() {
    assert_eq!(
        where_and_who(&says(), WhatTheStripSays::ONLY_WHERE_IT_RUNS),
        "Local"
    );
    let empty = WhatTheStripSays {
        who: Some(""),
        ..everything()
    };
    assert_eq!(where_and_who(&says(), empty), "Local");
}

/// **The word comes from the vocabulary**, so a translation reaches the strip:
/// a vocabulary that says something else for the key draws something else.
#[test]
fn where_it_runs_is_said_in_the_vocabularys_word() {
    let mut vocabulary = alo_strings::Vocabulary::empty();
    vocabulary
        .says(
            alo_strings::Phrase::says(WhereTheShellRuns::OnThisMachine.the_word().key(), "Lokal")
                .expect("a phrase"),
        )
        .expect("one phrase declares");
    let translated = alo_strings::Strings::of(vocabulary);
    assert_eq!(
        where_and_who(&translated, everything()),
        "Lokal \u{b7} disan"
    );
}

/// **The folder at the left and the words at the right, where the design puts
/// them**: the folder 24 in from the strip, the words ending 114 before its
/// right edge, both centred in its 56 rows — and both with glyphs in them.
#[test]
fn the_folder_and_the_words_sit_where_the_design_draws_them() {
    let window = a_window();
    let drawn = the_strip(&mut fonts(), &says(), everything(), window, 1.0, COLOURS);
    let [words, folder] = drawn.as_slice() else {
        panic!(
            "the strip drew {} boxes, not the words and the folder",
            drawn.len()
        );
    };
    assert!(
        ink(words) > 20,
        "the words were placed with no glyphs in them"
    );
    assert!(
        ink(folder) > 20,
        "the folder was placed with no glyphs in it"
    );

    // The strip begins one hairline in.
    let (strip_left, strip_top) = (window.loc.x + 1, window.loc.y + 1);
    let strip_right = window.loc.x + window.size.w - 1;
    assert_eq!(
        folder.area.loc.x,
        strip_left + 24,
        "the folder is not 24 in"
    );
    let words_end = words.area.loc.x + words.area.size.w;
    assert_eq!(
        words_end,
        strip_right - 114,
        "the words do not end 114 before the strip's right edge"
    );
    assert!(
        where_the_ink_ends(words) <= words_end && where_the_ink_ends(words) > words_end - 4,
        "the words' ink does not reach the end of their box"
    );
    for (what, placed) in [("the folder", folder), ("the words", words)] {
        assert_eq!(
            placed.area.loc.y,
            strip_top + 19,
            "{what} is not centred at row 19"
        );
    }
}

/// **At 200% every position doubles**, measured from the window, and the
/// glyphs are still there — the same strip drawn sharper.
#[test]
fn at_two_hundred_percent_the_strip_is_twice_the_size() {
    let window = Rectangle::new((400, 240).into(), (2080, 1600).into());
    let drawn = the_strip(&mut fonts(), &says(), everything(), window, 2.0, COLOURS);
    let [words, folder] = drawn.as_slice() else {
        panic!("the strip drew {} boxes at 200%", drawn.len());
    };
    assert!(
        ink(folder) > 40 && ink(words) > 40,
        "the strip lost its glyphs at 200%"
    );
    let (strip_left, strip_top) = (window.loc.x + 2, window.loc.y + 2);
    let strip_right = window.loc.x + window.size.w - 2;
    assert_eq!(folder.area.loc.x, strip_left + 48);
    assert_eq!(folder.area.loc.y, strip_top + 38);
    assert_eq!(folder.area.size.h, 36, "the folder's line is not twice 18");
    assert_eq!(words.area.loc.x + words.area.size.w, strip_right - 228);
}

/// **A folder too long for its room is cut before the words** with the row's
/// gap between them, rather than drawn under them.
#[test]
fn a_long_folder_is_cut_before_the_words() {
    let long = "~/".to_owned() + &"a-very-long-folder-name/".repeat(40);
    let what = WhatTheStripSays {
        the_folder: Some(&long),
        ..everything()
    };
    let drawn = the_strip(&mut fonts(), &says(), what, a_window(), 1.0, COLOURS);
    let [words, folder] = drawn.as_slice() else {
        panic!("the strip drew {} boxes", drawn.len());
    };
    let gap = i32::try_from(measures::BESIDE_THE_FOLDER).expect("a gap fits");
    assert_eq!(
        folder.area.loc.x + folder.area.size.w,
        words.area.loc.x - gap,
        "the folder does not stop the row's gap before the words"
    );
}

/// **A folder the kernel would not name leaves the left empty** and the words
/// where they were — the control for the two tests above.
#[test]
fn with_no_folder_only_the_words_are_drawn() {
    let what = WhatTheStripSays {
        the_folder: None,
        ..everything()
    };
    let drawn = the_strip(&mut fonts(), &says(), what, a_window(), 1.0, COLOURS);
    assert_eq!(
        drawn.len(),
        1,
        "a strip with no folder drew {} boxes",
        drawn.len()
    );
}

/// **The refusal.** A window too narrow to hold the words draws nothing rather
/// than words over its edge.
#[test]
fn a_window_too_narrow_for_the_words_draws_no_strip() {
    let narrow = Rectangle::new((0, 0).into(), (120, 800).into());
    let drawn = the_strip(&mut fonts(), &says(), everything(), narrow, 1.0, COLOURS);
    assert!(
        drawn.is_empty(),
        "a 120-wide window drew {} boxes",
        drawn.len()
    );
}

/// The words' metrics at 100%, as the strip shapes them.
fn the_words_metrics() -> cosmic_text::Metrics {
    cosmic_text::Metrics::new(
        f32::from(u16::try_from(measures::WHERE_AND_WHO).expect("a type size fits")),
        f32::from(u16::try_from(measures::WHERE_AND_WHOS_LINE).expect("a line fits")),
    )
}

/// The words' box when the strip says this name, with no folder beside it.
fn the_words_drawn_for(name: &str) -> Inked {
    let what = WhatTheStripSays {
        the_folder: None,
        who: Some(name),
        ..everything()
    };
    let mut drawn = the_strip(&mut fonts(), &says(), what, a_window(), 1.0, COLOURS);
    let words = drawn.pop();
    assert!(drawn.is_empty(), "the strip drew more than the words");
    words.unwrap_or_else(|| panic!("the strip drew no words for {name:?}"))
}

/// **A name in another script draws with its own letters**, not as an empty
/// box: Greek and Cyrillic, the two non-Latin scripts among the languages this
/// machine ships in, each leave more ink than `Local` and the dot alone.
#[test]
fn a_name_in_another_script_draws_with_its_letters() {
    let without = ink(&the_words_drawn_for("a"));
    for name in ["Αλέξανδρος", "Анастасия", "Zoë Ångström"] {
        let with = ink(&the_words_drawn_for(name));
        assert!(
            with > without + 40,
            "{name:?} drew {with} pixels of ink against {without} for a one-letter name, so its \
             letters are not on the screen"
        );
    }
}

/// **A right-to-left name is kept whole and drawn inside the words' box.**
///
/// The strip is one line of `Shaping::Advanced`, which lays a Hebrew or Arabic
/// run out right to left inside a left-to-right line, so the name sits after
/// `Local ·` as a reader of either script expects. What this holds is that the
/// text reaching the shaper is the person's name untouched, and that the box
/// ends where the design puts the words' end.
#[test]
fn a_right_to_left_name_is_kept_whole_and_inside_its_box() {
    let window = a_window();
    let words_end = window.loc.x + window.size.w - 1 - 114;
    for name in ["שרה", "سارة"] {
        let what = WhatTheStripSays {
            who: Some(name),
            ..everything()
        };
        let said = the_words_in(&mut fonts(), &says(), what, 800, the_words_metrics());
        assert_eq!(said, format!("Local \u{b7} {name}"), "the name was changed");

        let words = the_words_drawn_for(name);
        assert_eq!(
            words.area.loc.x + words.area.size.w,
            words_end,
            "{name:?} moved the words off the design's end"
        );
        assert!(ink(&words) > 20, "{name:?} drew no ink at all");
    }
}

/// **A name too long for the strip ends in an ellipsis inside the room**: the
/// words begin where the folder would, end where the design ends them, keep
/// `Local · ` whole.
#[test]
fn an_over_long_name_is_cut_with_an_ellipsis_inside_its_room() {
    let long = "anna".repeat(60);
    let what = WhatTheStripSays {
        who: Some(&long),
        the_folder: None,
        ..everything()
    };
    let window = a_window();
    let (strip_left, strip_right) = (window.loc.x + 1, window.loc.x + window.size.w - 1);
    let room = (strip_right - 114) - (strip_left + 24);

    let said = the_words_in(&mut fonts(), &says(), what, room, the_words_metrics());
    assert!(said.starts_with("Local \u{b7} anna"), "{said}");
    assert!(said.ends_with(crate::AN_ELLIPSIS), "{said} was not cut");
    assert!(said.len() < long.len(), "{said} is the whole name");

    let drawn = the_strip(&mut fonts(), &says(), what, window, 1.0, COLOURS);
    let [words] = drawn.as_slice() else {
        panic!(
            "a name that fills the strip drew {} boxes, not the words alone",
            drawn.len()
        );
    };
    assert!(
        words.area.loc.x >= strip_left + 24,
        "the words began before the strip's inset"
    );
    assert!(
        where_the_ink_ends(words) <= strip_right - 114,
        "the words' ink ran past where the design ends them"
    );
}
