//! That every screen says something, that a password is hidden and a disk's
//! name is not, and that no sentence on any screen was invented here.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::{how_many_cards, what_this_screen_says};
use crate::the_asking_walks::{TheAsking, WhatThePersonDid, WhatTheRingIsOn, WhichScreen};
use crate::which_disks_may_be_offered::Offered;
use crate::{ANetwork, joining_a_network};

const THE_EIGHT: [(&str, &str); 2] = [("be", "Belgisch"), ("fr", "Français")];

fn strings() -> alo_strings::Strings {
    let vocabulary = alo_saying::everything_this_machine_can_say().expect("alo OS's own words");
    alo_strings::Strings::of(vocabulary)
}

/// Two networks, one locked and one not, as a scan reports them.
fn networks() -> Vec<ANetwork> {
    joining_a_network::every_network("Fritz!Box 7590:82:WPA2\nA cafe, open:51:\n")
}

fn disks() -> Vec<Offered> {
    vec![
        Offered {
            disk: "/dev/disk/by-id/one".to_owned(),
            called: "Samsung SSD 870 EVO  ·  1 TB".to_owned(),
            holding: "Windows and about 340 GB of files".to_owned(),
            erases: true,
        },
        Offered {
            disk: "/dev/disk/by-id/two".to_owned(),
            called: "WD Elements  ·  2 TB".to_owned(),
            holding: "Nothing is on it".to_owned(),
            erases: false,
        },
    ]
}

/// Walk to a screen by taking the obvious road to it.
fn at(screen: WhichScreen) -> TheAsking {
    let mut asking = TheAsking::begins();
    let named = |what: &'static str| WhatTheRingIsOn {
        named: Some(what),
        as_the_machine_calls_it: Some(what),
        wants_a_password: false,
    };
    let locked = |what: &'static str| WhatTheRingIsOn {
        named: Some(what),
        as_the_machine_calls_it: Some(what),
        wants_a_password: true,
    };
    if screen == WhichScreen::TheKeyboard {
        return asking;
    }
    if screen == WhichScreen::EveryLayout {
        asking.and_then(WhatThePersonDid::WentDown, WhatTheRingIsOn::default());
        asking.and_then(WhatThePersonDid::Chose, WhatTheRingIsOn::default());
        return asking;
    }
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    if screen == WhichScreen::TheNetwork {
        return asking;
    }
    asking.there_are_this_many_cards(2);
    asking.and_then(WhatThePersonDid::Chose, locked("Fritz!Box 7590"));
    if screen == WhichScreen::ThePassword {
        return asking;
    }
    for character in "a secret".chars() {
        asking.and_then(
            WhatThePersonDid::Typed(character),
            WhatTheRingIsOn::default(),
        );
    }
    asking.and_then(WhatThePersonDid::Chose, WhatTheRingIsOn::default());
    if screen == WhichScreen::TheDisk {
        return asking;
    }
    asking.there_are_this_many_cards(2);
    asking.and_then(
        WhatThePersonDid::Chose,
        named("Samsung SSD 870 EVO  ·  1 TB"),
    );
    asking
}

#[test]
fn every_screen_says_something_rather_than_showing_a_blank() {
    // **The check that no screen is a quiet nothing.** A variant that fell
    // through to empty strings would draw a grey rectangle, and a person
    // would have no way to know whether the machine had stopped.
    let strings = strings();
    for screen in [
        WhichScreen::TheKeyboard,
        WhichScreen::EveryLayout,
        WhichScreen::TheNetwork,
        WhichScreen::ThePassword,
        WhichScreen::TheDisk,
        WhichScreen::TheReview,
    ] {
        let asking = at(screen);
        assert_eq!(asking.on(), screen, "the walk did not reach {screen:?}");
        let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
        assert!(!words.title.is_empty(), "{screen:?} has no question on it");
        assert!(
            !words.eyebrow.is_empty(),
            "{screen:?} has no label above it"
        );
        assert!(
            !words.cards.is_empty(),
            "{screen:?} offers a person nothing at all"
        );
        assert!(
            !words.consequence.is_empty(),
            "{screen:?} has no quiet line under it"
        );
    }
}

#[test]
fn the_networks_a_scan_found_are_the_cards() {
    let strings = strings();
    let asking = at(WhichScreen::TheNetwork);
    let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
    let named: Vec<&str> = words.cards.iter().map(|(name, _)| name.as_str()).collect();
    assert!(
        named.contains(&"Fritz!Box 7590"),
        "the network a scan found is not on the screen: {named:?}"
    );
    assert_eq!(words.cards.len(), networks().len());
}

#[test]
fn the_disks_offered_are_the_cards_and_each_says_what_is_on_it() {
    let strings = strings();
    let asking = at(WhichScreen::TheDisk);
    let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
    assert_eq!(words.cards.len(), 2);
    assert_eq!(words.cards[0].0, "Samsung SSD 870 EVO  ·  1 TB");
    assert_eq!(words.cards[0].1, "Windows and about 340 GB of files");
    assert_eq!(words.cards[1].1, "Nothing is on it");
}

#[test]
fn a_password_is_hidden_as_it_is_typed_and_the_dots_are_honest() {
    // One dot per character, because counting that they typed twelve
    // characters is the only honest feedback a hidden field can give.
    let strings = strings();
    let mut asking = at(WhichScreen::ThePassword);
    for character in "secret".chars() {
        asking.and_then(
            WhatThePersonDid::Typed(character),
            WhatTheRingIsOn::default(),
        );
    }
    let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
    let field = &words.cards[0].0;
    assert!(
        !field.contains("secret"),
        "the password is drawn in the clear: {field:?}"
    );
    assert_eq!(
        field.chars().filter(|one| *one == '•').count(),
        6,
        "six characters did not make six dots: {field:?}"
    );
}

#[test]
fn the_password_screen_names_the_network_being_joined() {
    // So that a person who chose the wrong one finds out here rather than
    // after the router refuses them.
    let strings = strings();
    let asking = at(WhichScreen::ThePassword);
    let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
    assert_eq!(words.title, "Fritz!Box 7590");
}

#[test]
fn the_disks_name_is_not_hidden_because_it_has_to_be_read_and_typed_back() {
    let strings = strings();
    let mut asking = at(WhichScreen::TheReview);
    for character in "Samsung".chars() {
        asking.and_then(
            WhatThePersonDid::Typed(character),
            WhatTheRingIsOn::default(),
        );
    }
    let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
    let field = &words.cards[0].1;
    assert!(
        field.contains("Samsung"),
        "the disk's name is hidden as it is typed: {field:?}"
    );
    assert!(
        !field.contains(super::A_HIDDEN_CHARACTER),
        "the disk's name is drawn as dots: {field:?}"
    );
    // And the screen names the disk at the top, which is what a person reads
    // in order to type it.
    assert_eq!(words.title, "Samsung SSD 870 EVO  ·  1 TB");
}

#[test]
fn a_field_shows_a_caret_so_the_screen_does_not_look_stopped() {
    let strings = strings();
    for screen in [WhichScreen::ThePassword, WhichScreen::TheReview] {
        let asking = at(screen);
        let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
        let field = if screen == WhichScreen::ThePassword {
            &words.cards[0].0
        } else {
            &words.cards[0].1
        };
        assert!(
            // **Asked of the constant, not of a character written here.** This
            // read `'▌'` until 2026-10-21 — the same literal the constant held
            // — so when the constant turned out to have no glyph in the
            // bundled face, this test agreed with it and passed. A test that
            // repeats the value it is checking cannot disagree with it.
            field.contains(super::THE_CARET),
            "{screen:?} shows no caret, so an empty field looks like a stopped machine"
        );
    }
}

#[test]
fn how_many_cards_agrees_with_how_many_the_screen_shows() {
    // **The pair that must agree or the ring points at nothing.** The walk
    // asks `how_many_cards` to bound the ring and the screen draws
    // `words.cards`; if they ever disagree, Enter takes a card a person is not
    // looking at.
    let strings = strings();
    for screen in [
        WhichScreen::TheKeyboard,
        WhichScreen::EveryLayout,
        WhichScreen::TheNetwork,
        WhichScreen::ThePassword,
        WhichScreen::TheDisk,
        WhichScreen::TheReview,
    ] {
        let asking = at(screen);
        let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
        assert_eq!(
            how_many_cards(&asking, &THE_EIGHT, &networks(), &disks()),
            words.cards.len(),
            "{screen:?} bounds the ring at a different number than it draws"
        );
    }
}

#[test]
fn a_scan_that_found_nothing_leaves_the_screen_with_no_cards_rather_than_a_wrong_one() {
    // Real: a card with no firmware finds no networks at all. The screen must
    // still say what it is for.
    let strings = strings();
    let asking = at(WhichScreen::TheNetwork);
    let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &[], &disks());
    assert!(words.cards.is_empty());
    assert!(
        !words.title.is_empty(),
        "an empty network screen says nothing"
    );
    assert_eq!(how_many_cards(&asking, &THE_EIGHT, &[], &disks()), 0);
}

/// Many networks, as a person in a block of flats actually sees them.
fn a_roomful_of_networks() -> Vec<ANetwork> {
    let mut lines = String::new();
    for which in 0..19 {
        lines.push_str(&format!("network-{which}:{}:WPA2\n", 40 + which));
    }
    joining_a_network::every_network(&lines)
}

#[test]
fn every_screen_is_drawn_with_as_much_on_it_as_really_arrives() {
    // **The regression, at the seam where it happened.** The walk asked
    // `what_a_screen_shows` for every screen, and that function asks the
    // layout for one card slot per card. With eight layouts or nineteen
    // networks the layout refused — correctly, rather than draw over the
    // accessibility button — so the walk had nothing and drew nothing. A grey
    // screen, on every display, for the layout screen always and the network
    // screen in any room with five networks in it.
    //
    // This walks all six screens with as much on them as really arrives and
    // asserts each one produces lines, choosing the road by `is_a_list()`
    // exactly as `the_environment_asks` does. If the two ever disagree about
    // which screens are lists, this test does not notice — so it is the
    // production predicate that is called here, not a copy of it.
    let strings = strings();
    // The eight `main.rs` offers, copied because `THE_LAYOUTS_OFFERED` is in
    // the binary and this test is in the library. **Eight is the figure that
    // matters**: it is more than the ordinary layout holds, which is why the
    // layout screen drew nothing on every display rather than only on small
    // ones.
    let layouts: Vec<(&str, &str)> = vec![
        ("be", "Belgisch"),
        ("fr", "Français"),
        ("de", "Deutsch"),
        ("es", "Español"),
        ("it", "Italiano"),
        ("nl", "Nederlands"),
        ("gb", "English (UK)"),
        ("us", "English (US)"),
    ];
    let networks = a_roomful_of_networks();
    let disks = disks();
    let face = crate::alos_own_face::AlosOwnFace::carried().expect("the compiled-in typeface");
    let (across, down) = crate::where_a_screen_goes::THE_DESIGN_WAS_DRAWN_AT;

    for screen in [
        WhichScreen::TheKeyboard,
        WhichScreen::EveryLayout,
        WhichScreen::TheNetwork,
        WhichScreen::ThePassword,
        WhichScreen::TheDisk,
        WhichScreen::TheReview,
    ] {
        let mut asking = at(screen);
        // **The walk has to have arrived.** Mutating `at` to return the
        // first screen every time left three of this file's checks passing —
        // including the one that renders all six — because they would then
        // test one screen six times and agree with themselves about it. A
        // fixture that does not reach a surface removes it from every guard
        // over that surface, silently.
        assert_eq!(asking.on(), screen, "the walk did not reach {screen:?}");
        asking.there_are_this_many_cards(how_many_cards(&asking, &layouts, &networks, &disks));
        let words = what_this_screen_says(&strings, &asking, &layouts, &networks, &disks);

        let lines = if asking.is_a_list() {
            crate::a_screen_is_drawn::what_a_list_screen_shows(
                across,
                down,
                None,
                face,
                &words,
                asking.card(),
            )
            .map(|list| list.shows.lines.len())
        } else {
            crate::a_screen_is_drawn::what_a_screen_shows(
                across,
                down,
                None,
                face,
                &words,
                Some(asking.card()),
            )
            .map(|shows| shows.lines.len())
        };

        let drew = lines.unwrap_or(0);
        assert!(
            drew > 0,
            "{screen:?} drew nothing with {} layouts, {} networks and {} disks \
             on it. A person looking at this screen sees no words at all and \
             has no way to find out why.",
            layouts.len(),
            networks.len(),
            disks.len()
        );
    }
}

#[test]
fn the_three_screens_whose_cards_arrive_are_the_three_that_scroll() {
    // **The control for the test above.** It would pass on a build where every
    // screen used the scrolling layout, and that would be wrong in the other
    // direction: the keyboard question is two cards the design drew, and
    // moving its actions down to make room for a list it does not have would
    // put them somewhere nobody decided.
    //
    // So this asserts the division itself, which is what `is_a_list` is for.
    for screen in [
        WhichScreen::EveryLayout,
        WhichScreen::TheNetwork,
        WhichScreen::TheDisk,
    ] {
        assert!(
            at(screen).is_a_list(),
            "{screen:?}'s cards are data that arrives, so it has no length \
             anybody drew and it must scroll"
        );
    }
    for screen in [
        WhichScreen::TheKeyboard,
        WhichScreen::ThePassword,
        WhichScreen::TheReview,
    ] {
        assert!(
            !at(screen).is_a_list(),
            "{screen:?} chooses between things the six frames drew, so it keeps \
             the layout they were drawn in"
        );
    }
}

/// Where the rendered screens are left, for a person to look at.
///
/// **This is the answer to a question the owner asked after a day of reports
/// saying *built*: how is something done before it is tested on hardware?** By
/// drawing it and looking at it. The grey-screen fault that shipped this
/// morning was invisible to every test in this crate and would have been
/// obvious in one glance at a picture.
fn where_the_pictures_go() -> std::path::PathBuf {
    let at = std::env::temp_dir().join("alo-installer-screens");
    let _ = std::fs::create_dir_all(&at);
    at
}

#[test]
fn every_screen_has_ink_on_it_and_is_left_as_a_picture() {
    use crate::a_surface_to_draw_on::{Grey, TheSurface};

    // 1920 x 1080, because that is what the owner's machine will be on and it
    // is the size the four-card limit was measured at.
    let across = 1920_u32;
    let down = 1080_u32;
    let stride = across * 4;
    let surface = TheSurface::from_what_the_kernel_says(
        &format!("{across},{down}\n"),
        &format!("{stride}\n"),
        "32\n",
    )
    .expect("a surface the shape a framebuffer is");

    let beneath = Grey::the_canvas();
    let ink = Grey::the_ink();
    let strings = strings();
    let layouts: Vec<(&str, &str)> = vec![
        ("be", "Belgisch"),
        ("fr", "Français"),
        ("de", "Deutsch"),
        ("es", "Español"),
        ("it", "Italiano"),
        ("nl", "Nederlands"),
        ("gb", "English (UK)"),
        ("us", "English (US)"),
    ];
    let networks = a_roomful_of_networks();
    let disks = disks();
    let face = crate::alos_own_face::AlosOwnFace::carried().expect("the compiled-in typeface");
    let into = where_the_pictures_go();

    let mut thinnest: Option<(WhichScreen, usize)> = None;

    for screen in [
        WhichScreen::TheKeyboard,
        WhichScreen::EveryLayout,
        WhichScreen::TheNetwork,
        WhichScreen::ThePassword,
        WhichScreen::TheDisk,
        WhichScreen::TheReview,
    ] {
        let mut asking = at(screen);
        // **The walk has to have arrived.** Mutating `at` to return the
        // first screen every time left three of this file's checks passing —
        // including the one that renders all six — because they would then
        // test one screen six times and agree with themselves about it. A
        // fixture that does not reach a surface removes it from every guard
        // over that surface, silently.
        assert_eq!(asking.on(), screen, "the walk did not reach {screen:?}");
        asking.there_are_this_many_cards(how_many_cards(&asking, &layouts, &networks, &disks));
        let words = what_this_screen_says(&strings, &asking, &layouts, &networks, &disks);

        // The production road, chosen by the production predicate.
        let shows = if asking.is_a_list() {
            crate::a_screen_is_drawn::what_a_list_screen_shows(
                across,
                down,
                None,
                face,
                &words,
                asking.card(),
            )
            .map(|list| list.shows)
        } else {
            crate::a_screen_is_drawn::what_a_screen_shows(
                across,
                down,
                None,
                face,
                &words,
                Some(asking.card()),
            )
        }
        .unwrap_or_else(|| panic!("{screen:?} would not lay out at {across}x{down}"));

        let picture = into.join(format!("{screen:?}.fb"));
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&picture)
            .expect("a file to stand in for the framebuffer");
        file.set_len(surface.altogether())
            .expect("a framebuffer-sized file");

        crate::a_screen_is_drawn::draw_a_screen(&file, surface, beneath, ink, face, &shows)
            .expect("the screen is drawn");

        // **Ink, counted against the background rather than against a
        // brightness.** Every test in this crate asked whether the screen laid
        // out. None asked whether anything arrived on it, and a screen that
        // lays out and draws nothing is exactly what shipped this morning.
        //
        // Counted as *differs from the surface* and not as *is bright*,
        // because the first version of this counter asked whether bytes were
        // above `0x30` — which was only a measure of ink while the surface was
        // near black, and silently became a measure of the **background** the
        // moment the design's white surface was restored. A counter that
        // changes meaning when a colour changes is the same family of fault as
        // the one this test exists to catch.
        let drawn = std::fs::read(&picture).expect("the bytes back");
        let surface_value = beneath.as_a_value();
        let lit = drawn
            .iter()
            .filter(|byte| byte.abs_diff(surface_value) > 0x20)
            .count();

        // **And the screen is the design's way up.** `the-canvas-in-numbers.md`
        // records `bg/surface` as #FFFFFF and `text/primary` as navy. The
        // environment drew the two the other way round until 2026-10-21 and no
        // document ever decided it; the owner saw the first picture ever taken
        // of a screen and asked why it was black.
        let most = {
            let mut tally = [0_usize; 2];
            for byte in &drawn {
                tally[usize::from(*byte < 0x80)] += 1;
            }
            tally
        };
        assert!(
            most[0] > most[1],
            "{screen:?} is drawn dark-on-light: {} bytes are dark against {} \
             light. The design's surface is #FFFFFF and its ink is navy, so \
             most of a screen is the surface and the surface is light.",
            most[1],
            most[0]
        );
        if thinnest.is_none_or(|(_, least)| lit < least) {
            thinnest = Some((screen, lit));
        }

        assert!(
            lit > 20_000,
            "{screen:?} drew only {lit} lit bytes into a {across}x{down} \
             framebuffer, which is a screen a person would call blank. The \
             picture is at {}",
            picture.display()
        );
    }

    let (screen, least) = thinnest.expect("six screens were drawn");
    println!(
        "six screens drawn into {} — thinnest was {screen:?} at {least} lit bytes",
        into.display()
    );
}

#[test]
fn no_screen_shows_a_placeholder_to_a_person() {
    // **The fault this catches reached a person on the first screen of the
    // install.** `words::PRESS_THESE_TO_CHECK` is
    // `"Press the keys shown to check: {keys}"`, every sentence here was said
    // with `Filling::nothing()`, and so the card read *Press the keys shown to
    // check: [keys]* — the placeholder, drawn as a placeholder, on the card the
    // whole keyboard question rests on.
    //
    // Nothing saw it. The sentence was fetched, said, wrapped, laid out and
    // drawn, all correctly. It was found by rendering the screen to an image
    // and looking at it.
    //
    // So this walks every screen and refuses any word of any card, title,
    // eyebrow, introduction or quiet line that still carries a placeholder's
    // brackets. It does not need to know which sentences have placeholders:
    // an unfilled one is rendered as `[name]`, and no sentence this crate
    // publishes contains a square bracket for any other reason — which the
    // control below is what proves.
    let strings = strings();
    let layouts: Vec<(&str, &str)> = vec![("be", "Belgisch"), ("us", "English (US)")];
    let networks = a_roomful_of_networks();
    let disks = disks();

    let mut showing = Vec::new();
    for screen in [
        WhichScreen::TheKeyboard,
        WhichScreen::EveryLayout,
        WhichScreen::TheNetwork,
        WhichScreen::ThePassword,
        WhichScreen::TheDisk,
        WhichScreen::TheReview,
    ] {
        let mut asking = at(screen);
        // **The walk has to have arrived.** Mutating `at` to return the
        // first screen every time left three of this file's checks passing —
        // including the one that renders all six — because they would then
        // test one screen six times and agree with themselves about it. A
        // fixture that does not reach a surface removes it from every guard
        // over that surface, silently.
        assert_eq!(asking.on(), screen, "the walk did not reach {screen:?}");
        asking.there_are_this_many_cards(how_many_cards(&asking, &layouts, &networks, &disks));
        let words = what_this_screen_says(&strings, &asking, &layouts, &networks, &disks);

        let mut every = vec![
            ("the eyebrow", words.eyebrow.clone()),
            ("the title", words.title.clone()),
            ("the introduction", words.introduction.clone()),
            ("the quiet line", words.consequence.clone()),
        ];
        for (which, (called, under)) in words.cards.iter().enumerate() {
            every.push((
                Box::leak(format!("card {which}").into_boxed_str()),
                called.clone(),
            ));
            every.push((
                Box::leak(format!("card {which}'s quiet line").into_boxed_str()),
                under.clone(),
            ));
        }

        for (what, text) in every {
            if text.contains('[') || text.contains('{') {
                showing.push(format!("{screen:?}, {what}: {text:?}"));
            }
        }
    }

    assert!(
        showing.is_empty(),
        "{} place(s) on the environment's screens show a person a placeholder \
         rather than a word:\n  {}",
        showing.len(),
        showing.join("\n  ")
    );
}

#[test]
fn a_square_bracket_is_not_something_this_crate_says() {
    // **The control for the check above**, and it is the one that makes it a
    // check rather than a coincidence. That test reads a square bracket as
    // *this is an unfilled placeholder*. If any sentence this crate publishes
    // legitimately contained one, the check would be silently weaker than it
    // reads — passing on screens that do show a placeholder, because brackets
    // would have become ordinary.
    let vocabulary = alo_saying::everything_this_machine_can_say().expect("alo OS's own words");
    let mut bracketed = Vec::new();
    let mut looked_at = 0_usize;
    for phrase in vocabulary.phrases() {
        let key = phrase.key();
        let key = key.as_str();
        if !key.starts_with("installing.") {
            continue;
        }
        looked_at += 1;
        let english = phrase.source().as_written();
        if english.contains('[') || english.contains(']') {
            bracketed.push(format!("{key}: {english:?}"));
        }
    }
    assert!(
        looked_at > 50,
        "only {looked_at} sentences were read, so this control is not reading \
         the environment's words"
    );
    assert!(
        bracketed.is_empty(),
        "{} of this crate's sentences contain a square bracket, so a bracket on \
         a screen no longer means an unfilled placeholder and \
         `no_screen_shows_a_placeholder_to_a_person` is weaker than it \
         reads:\n  {}",
        bracketed.len(),
        bracketed.join("\n  ")
    );
}
