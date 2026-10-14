//! What the language list offers, read from the rows it would draw rather
//! than from the call that built it.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::{ROWS_AT_A_TIME, TheLanguagesOffered};

#[test]
fn every_offered_language_brings_a_keyboard_and_a_name() {
    let offered = TheLanguagesOffered::every_official();
    assert_eq!(
        offered.how_many(),
        22,
        "the 22 Latin-script languages are offered; the other two are owed a second group"
    );
    // A row a person can pick that leaves them unable to type is the bug this
    // screen exists to prevent, so every row carries a layout and both names.
    let mut row = TheLanguagesOffered::every_official();
    for _ in 0..22 {
        for it in row.window() {
            assert!(
                !it.keyboard.name().is_empty(),
                "{} was offered with no keyboard",
                it.tag
            );
            assert!(
                !it.calls_itself.is_empty(),
                "{} was offered with no name of its own",
                it.tag
            );
            assert!(
                !it.in_english.is_empty(),
                "{} was offered with no English name on its second line",
                it.tag
            );
        }
        row.down();
    }
}

#[test]
fn a_row_carries_its_own_name_first_and_the_english_one_beside_it() {
    // **This test used to assert that no row reads an English name**, on the
    // reasoning that a person who does not read English should not be shown
    // it. Frame 09b reads `Nederlands` over `Dutch`, so the design shows both:
    // the language's own name identifies the row and the English one helps
    // whoever knows only that.
    let offered = TheLanguagesOffered::every_official();
    let mut walk = TheLanguagesOffered::every_official();
    for _ in 0..walk.how_many() {
        walk.up();
    }
    let mut seen = Vec::new();
    for _ in 0..offered.how_many() {
        if let Some(within) = walk.cursor_in_the_window() {
            let row = &walk.window()[within];
            seen.push((row.calls_itself, row.in_english));
        }
        walk.down();
    }

    // Czech and Finnish prove the first line is the language's own: neither
    // name resembles its English one. Greek and Bulgarian made the point more
    // sharply and are no longer offered -- see `OWED_A_SECOND_GROUP`.
    assert!(
        seen.contains(&("Čeština", "Czech")),
        "Czech is offered as Čeština over Czech"
    );
    assert!(
        seen.contains(&("Suomi", "Finnish")),
        "Finnish is offered as Suomi over Finnish"
    );
    assert!(seen.contains(&("Nederlands", "Dutch")));
    assert!(seen.contains(&("Français", "French")));

    for (its_own, in_english) in &seen {
        assert!(!its_own.is_empty() && !in_english.is_empty());
    }
}

#[test]
fn nothing_is_chosen_until_somebody_chooses() {
    let mut offered = TheLanguagesOffered::every_official();
    assert!(
        offered.chosen().is_none(),
        "ADR 0025: nothing is pre-selected at first start"
    );
    assert!(offered.chosen_in_the_window().is_none());

    // Moving the cursor the whole way down and back must not choose anything.
    // `docs/features.md`: moving focus must never silently commit a choice.
    for _ in 0..40 {
        offered.down();
    }
    for _ in 0..40 {
        offered.up();
    }
    assert!(
        offered.chosen().is_none(),
        "moving focus committed a choice, which the ruling forbids"
    );

    offered.choose();
    assert!(offered.chosen().is_some(), "and choosing chooses");
}

#[test]
fn the_cursor_starts_on_the_language_this_machine_already_speaks() {
    let offered = TheLanguagesOffered::every_official();
    let within = offered
        .cursor_in_the_window()
        .expect("a list with rows has its cursor in its window");
    assert_eq!(
        offered.window()[within].tag,
        alo_strings::union::THE_SOURCE,
        "the cursor opens on the language the machine is already in"
    );
}

#[test]
fn the_window_shows_three_rows_and_the_cursor_is_always_in_it() {
    let mut offered = TheLanguagesOffered::every_official();
    // Walk the whole list from the top, and at every single step both facts
    // must hold. A window that lost the cursor would be a list a person is
    // moving through and cannot see themselves in.
    for _ in 0..30 {
        offered.up();
    }
    for step in 0..offered.how_many() {
        assert_eq!(
            offered.window().len(),
            ROWS_AT_A_TIME,
            "at step {step} the window was not {ROWS_AT_A_TIME} rows"
        );
        let within = offered
            .cursor_in_the_window()
            .unwrap_or_else(|| panic!("at step {step} the cursor was outside the window"));
        assert!(
            within < ROWS_AT_A_TIME,
            "at step {step} the cursor was at {within}, past the window"
        );
        offered.down();
    }
}

#[test]
fn the_window_reaches_the_last_language_and_the_cursor_stops_there() {
    let mut offered = TheLanguagesOffered::every_official();
    for _ in 0..100 {
        offered.down();
    }
    assert_eq!(
        offered.cursor(),
        offered.how_many() - 1,
        "the cursor stops at the last row rather than wrapping"
    );
    let within = offered
        .cursor_in_the_window()
        .expect("the cursor is visible");
    assert_eq!(
        offered.window()[within].tag,
        "sv",
        "the last language is reachable, which is the whole reason the window \
         moves, and it is Svenska at the end of the Latin run"
    );
}

#[test]
fn the_window_stops_at_the_top_rather_than_showing_blank_rows() {
    let mut offered = TheLanguagesOffered::every_official();
    for _ in 0..100 {
        offered.up();
    }
    assert_eq!(offered.cursor(), 0);
    assert_eq!(
        offered.cursor_in_the_window(),
        Some(0),
        "at the top of the list the cursor is the first visible row"
    );
    assert_eq!(offered.window().len(), ROWS_AT_A_TIME);
    assert_eq!(
        offered.window()[0].tag,
        "cs",
        "the first row is Čeština, where the declared order begins"
    );
}

#[test]
fn a_choice_that_scrolls_out_of_sight_draws_no_border() {
    let mut offered = TheLanguagesOffered::every_official();
    for _ in 0..100 {
        offered.up();
    }
    offered.choose();
    assert_eq!(
        offered.chosen_in_the_window(),
        Some(0),
        "the chosen row is visible where it was picked"
    );
    let picked = offered.chosen().expect("something was picked").tag;

    // Walk away from it. The choice is still the choice -- it is what
    // `Use this language` would act on -- and no visible row may wear its
    // border.
    for _ in 0..10 {
        offered.down();
    }
    assert_eq!(
        offered.chosen().expect("the choice survives the walk").tag,
        picked,
        "walking away from a choice must not unchoose it"
    );
    assert!(
        offered.chosen_in_the_window().is_none(),
        "the chosen row is off screen, so no visible row wears its mark"
    );
}

#[test]
fn the_keyboard_that_comes_with_a_language_is_that_crates_answer() {
    let mut offered = TheLanguagesOffered::every_official();
    for _ in 0..100 {
        offered.up();
    }
    // French, whose layout is AZERTY -- a different arrangement of the same
    // ASCII letters, which is exactly what choosing a layout fixes and why
    // every Latin-script language can be offered today.
    while offered
        .cursor_in_the_window()
        .and_then(|within| offered.window().get(within))
        .map(|it| it.tag)
        != Some("fr")
    {
        let before = offered.cursor();
        offered.down();
        assert_ne!(before, offered.cursor(), "French was never reached");
    }
    offered.choose();
    let french = offered.chosen().expect("French was picked");
    let from_that_crate = alo_keyboards::offered_with(
        &alo_strings::Language::written("fr").expect("fr is a language tag"),
    )
    .expect("French has an offer")
    .first();
    assert_eq!(
        french.keyboard.name(),
        from_that_crate.name(),
        "the keyboard shown is that crate's answer and not a second table"
    );
}

#[test]
fn the_declared_order_and_the_owed_are_exactly_the_official_languages() {
    // **The two lists together, held to all 24.** A language can move from
    // offered to owed or back; it cannot leave this file. Splitting a list is
    // exactly where one would otherwise go missing, which is the whole reason
    // the owed ones are a constant rather than a comment.
    let mut both = super::IN_THE_ORDER_A_PERSON_READS.to_vec();
    both.extend_from_slice(&super::OWED_A_SECOND_GROUP);
    assert_eq!(both.len(), 24, "there are 24 official languages");

    let mut sorted = both.clone();
    sorted.sort_unstable();
    let mut without_repeats = sorted.clone();
    without_repeats.dedup();
    assert_eq!(
        sorted, without_repeats,
        "a language is in both lists, or declared twice in one"
    );

    let mut official = alo_strings::union::OFFICIAL
        .iter()
        .map(|it| it.tag)
        .collect::<Vec<_>>();
    official.sort_unstable();
    assert_eq!(
        sorted, official,
        "the two lists and the official languages are not the same set"
    );
}

#[test]
fn the_two_owed_are_the_ones_a_person_could_not_type_their_name_on() {
    // **The reason, as a test rather than a comment.** `alo_accounts` wants a
    // name a Unix login can have: ASCII lowercase, digits, `-`, `_`. These two
    // layouts put their own script on the letter keys, so somebody who chose
    // one could not type their own name on the next screen. What they are owed
    // is a Latin layout beside their own, which neither has yet.
    for owed in super::OWED_A_SECOND_GROUP {
        let language = alo_strings::Language::written(owed).expect("an official tag");
        let offer = alo_keyboards::offered_with(&language).expect("every language has an offer");
        let beside = offer.beside_it();
        assert!(
            beside
                .iter()
                .all(|it| it.name() != "us" && it.name() != "gb"),
            "{owed} now has a Latin layout beside it, so it is no longer owed \
             one and belongs in the offered list"
        );
    }
    assert_eq!(super::OWED_A_SECOND_GROUP.len(), 2);
    assert_eq!(super::IN_THE_ORDER_A_PERSON_READS.len(), 22);
}

#[test]
fn each_name_decides_its_place_and_no_english_name_does() {
    // The point of declaring the order: a row's position follows the name the
    // row READS. Čeština before Dansk before Deutsch is that; the English
    // order would be Croatian, Czech, Danish, Dutch -- so `hr` would come
    // first and `nl` fourth.
    let declared = super::IN_THE_ORDER_A_PERSON_READS;
    let at = |tag: &str| {
        declared
            .iter()
            .position(|it| *it == tag)
            .expect("every offered tag is declared")
    };
    assert!(at("cs") < at("da"), "Čeština before Dansk");
    assert!(at("da") < at("de"), "Dansk before Deutsch");
    assert!(at("de") < at("et"), "Deutsch before Eesti");
    assert!(at("fi") < at("sv"), "Suomi before Svenska");

    // And the English order is NOT what this is: a control, so a list that
    // satisfied the four above by coincidence still fails.
    assert!(
        at("hr") > 0,
        "Croatian is first in the English order and must not be first here"
    );
    assert!(
        at("nl") > 3,
        "Dutch is fourth in the English order and must not be fourth here"
    );

    // The two non-Latin ones are not in this list at all.
    assert!(!declared.contains(&"el") && !declared.contains(&"bg"));
}
