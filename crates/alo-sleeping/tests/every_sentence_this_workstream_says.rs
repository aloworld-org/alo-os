//! Every sentence the session-and-displays workstream can say, against the
//! vocabulary a real process loads — and nothing in any of them that names a
//! part of the machine.
//!
//! Task 7 of `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`: *every
//! sentence these five crates can say — `alo-locking`, `alo-sleeping`,
//! `alo-displays`, `alo-leaving` and `alo-notifying` — is in the vocabulary
//! with a translator's note … no sentence names `logind`, DRM, EDID, a
//! connector name or any other part of the machinery.*
//!
//! Each of the five crates has its own `every_sentence_here_is_collected.rs`,
//! and each asks the question about itself. What none of them can ask is the
//! one this file asks: **does what this workstream says survive being put
//! beside every other crate's sentences**, and is a person who signs in, locks,
//! sleeps, wakes and docks told everything they need without ever learning the
//! name of anything alo OS rented.
//!
//! # Why the five are read by area rather than by name
//!
//! Three of them — `alo-sleeping`, `alo-leaving` and `alo-notifying` — each
//! hold a test that reads **every manifest in the workspace** and refuses any
//! crate but `alo-saying` and `alo-shell` that names them. Those are real
//! guarantees: a crate that answers an agent must not be able to hold the
//! machine awake, read what somebody had open yesterday, or listen to their
//! notifications. So no crate can name all five, and widening any of those
//! three lists to host one test would be paying for an audit with the thing the
//! audit exists to protect.
//!
//! What is read instead is the **assembled vocabulary**, filtered by the area
//! of each key ([`THE_FIVE_AREAS`]). That is not a way around the question; it
//! is a stricter form of it. A crate's `EVERY_WORD` is what the crate claims;
//! the vocabulary is what the machine *says* after `alo-saying` has collected
//! it, which is the thing a person actually meets. And the coverage is
//! complete, because every one of the five holds its own test that each of its
//! keys sorts under its own area — so a sentence of theirs outside these five
//! areas fails in the crate that declared it, before it reaches here.
//!
//! The two this crate may name — `alo-locking`, which it depends on, and
//! `alo-displays`, which holds no such rule — are also read by name, so that
//! the two readings are shown to agree at least once.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::{Filling, Phrase, Said, Strings, Vocabulary};

/// The five crates of this plan, and the area each of their keys sorts under.
///
/// One row per crate, so that a crate dropped from `alo-saying`'s collection
/// fails here naming itself rather than as a number that came out low.
const THE_FIVE_AREAS: [(&str, &str); 5] = [
    ("alo-locking", "locking"),
    ("alo-sleeping", "sleeping"),
    ("alo-displays", "displays"),
    ("alo-leaving", "leaving"),
    ("alo-notifying", "notifying"),
];

/// Nothing a person reads names any of these — not the sentence, not the note
/// a translator works from, not the key.
///
/// The plan names four: `logind`, DRM, EDID, and a connector name. The rest is
/// the same rule applied to everything else this workstream happens to rent or
/// to know about the inside of a machine.
///
/// # What is deliberately **not** on this list
///
/// **`socket`, `HDMI`, `DisplayPort`, `DVI`, `VGA`.** A socket here is the hole
/// in the back of the machine that a cable goes into, and the four names are
/// printed beside it on the person's own hardware — `alo_saying::rented` draws
/// that line already for the logo on a keyboard's modifier key. What is banned
/// is the **connector name**, which is `eDP-1` and `HDMI-A-2`: the string the
/// kernel's side of a graphics card uses, printed nowhere, which no owner of a
/// laptop ever chose to know. [`no_connector_name_is_in_anything_a_person_reads`]
/// is that half.
///
/// **`sleep`, `lid`, `screen`, `display`.** Each is the person's own thing.
const NEVER_NAMED: [&str; 26] = [
    "logind",
    "systemd",
    "drm",
    "edid",
    "kms",
    "modeset",
    "xrandr",
    "wayland",
    "smithay",
    "mesa",
    "compositor",
    "inhibitor",
    "inhibit",
    "d-bus",
    "dbus",
    "zbus",
    "pipewire",
    "flatpak",
    "unix socket",
    "socket path",
    "file descriptor",
    "/sys/",
    "/dev/",
    "/proc/",
    "uid",
    "daemon",
];

/// The kinds of connector the kernel's side of a graphics card names, as they
/// appear at the front of one — `eDP-1`, `HDMI-A-2`, `DP-1`.
///
/// Each is checked **with its trailing hyphen**, which is what makes this a
/// check for a connector name rather than for the word beside a socket: *HDMI
/// socket 1* is a sentence alo OS says on purpose, and `HDMI-A-1` is one it
/// must never say.
const CONNECTOR_KINDS: [&str; 12] = [
    "edp-",
    "lvds-",
    "dsi-",
    "dpi-",
    "dp-",
    "hdmi-a",
    "hdmi-b",
    "dvi-d",
    "dvi-i",
    "dvi-a",
    "writeback-",
    "svideo-",
];

/// Everything the machine can say, which is what a real process holds.
fn everything_this_machine_can_say() -> Vocabulary {
    match alo_saying::everything_this_machine_can_say() {
        Ok(vocabulary) => vocabulary,
        Err(why) => panic!("alo OS's own words are wrong: {why}"),
    }
}

/// Every sentence of this workstream in the machine's one vocabulary, as the
/// crate it belongs to and the phrase itself.
fn everything_this_workstream_says(vocabulary: &Vocabulary) -> Vec<(&'static str, &Phrase)> {
    let mut found = Vec::new();
    for (crate_named, area) in THE_FIVE_AREAS {
        let theirs: Vec<&Phrase> = vocabulary
            .phrases()
            .filter(|phrase| phrase.key().area() == area)
            .collect();
        assert!(
            !theirs.is_empty(),
            "{crate_named} says nothing the machine collected: no key under {area}"
        );
        found.extend(theirs.into_iter().map(|phrase| (crate_named, phrase)));
    }
    found
}

/// **The five crates of this plan are collected into the machine's one
/// vocabulary.** A crate declared and left out of `alo-saying`'s collection
/// compiles, tests, ships, and reaches a real shell as a key.
#[test]
fn the_five_crates_of_this_plan_are_collected() {
    let vocabulary = everything_this_machine_can_say();
    for (crate_named, area) in THE_FIVE_AREAS {
        assert!(
            alo_saying::EVERY_LIST.contains(&crate_named),
            "{crate_named} is not on alo-saying's list"
        );
        assert!(
            vocabulary
                .phrases()
                .any(|phrase| phrase.key().area() == area),
            "{crate_named} is on the list and says nothing"
        );
    }
    let all = everything_this_workstream_says(&vocabulary).len();
    let three_of_them = alo_locking::EVERY_WORD.len()
        + alo_sleeping::EVERY_WORD.len()
        + alo_displays::EVERY_WORD.len();
    assert!(
        all > three_of_them,
        "the two crates this one may not name say nothing: {all} against {three_of_them}"
    );
}

/// **Every sentence the two crates this one may name can say is one the
/// machine says**, with that crate's own English — so the reading by area and
/// the reading by name agree.
#[test]
fn what_the_crates_declare_and_what_the_machine_says_are_the_same_list() {
    let vocabulary = everything_this_machine_can_say();
    for (area, declared) in [
        ("locking", alo_locking::EVERY_WORD.to_vec()),
        ("sleeping", alo_sleeping::EVERY_WORD.to_vec()),
        ("displays", alo_displays::EVERY_WORD.to_vec()),
    ] {
        for word in &declared {
            let Some(phrase) = vocabulary.phrase(&word.key()) else {
                panic!("{} is declared here and collected nowhere", word.named());
            };
            assert_eq!(
                phrase.source().as_written(),
                word.says(),
                "{}",
                word.named()
            );
        }
        let collected = vocabulary
            .phrases()
            .filter(|phrase| phrase.key().area() == area)
            .count();
        assert_eq!(
            collected,
            declared.len(),
            "{area}: the machine says {collected} and the crate declares {}",
            declared.len()
        );
    }
}

/// **Every sentence comes out whole, in the language a person reads.** Not a
/// key, and not a sentence with a gap nobody filled — the two ways a correct
/// vocabulary still reaches somebody as machinery.
///
/// Each gap is filled with something a person would recognise, because what is
/// being asked is whether the sentence arrives as a sentence, not what goes in
/// the gap. Which gaps a sentence has and what may go in them is each crate's
/// own test.
#[test]
fn every_sentence_comes_out_whole() {
    let vocabulary = everything_this_machine_can_say();
    let strings = Strings::of(vocabulary.clone());
    for (crate_named, phrase) in everything_this_workstream_says(&vocabulary) {
        let mut filling = Filling::nothing();
        for gap in phrase.source().gaps() {
            filling = filling.and(gap.clone(), "something the person has");
        }
        let said: Said = strings.say(phrase.key(), &filling);
        assert!(!said.is_a_bug(), "{crate_named}: {said}");
        assert!(said.unfilled().is_empty(), "{crate_named}: {said}");
        assert!(
            !said.text().is_empty(),
            "{crate_named}: {} says nothing",
            phrase.key()
        );
    }
}

/// **Every sentence carries a note a translator can work from.** A translator
/// with no note writes a guess, and a guess in somebody's own language is worse
/// than English they can at least recognise as not theirs.
///
/// Forty characters is not a measure of quality; it is a floor under *said
/// once the screen locks*, which is when a sentence is shown rather than what
/// it is for.
#[test]
fn every_sentence_carries_a_note_a_translator_can_work_from() {
    let vocabulary = everything_this_machine_can_say();
    for (crate_named, phrase) in everything_this_workstream_says(&vocabulary) {
        let note = phrase.note().unwrap_or_default();
        assert!(
            note.len() > 40,
            "{crate_named}: {} has no note worth the name: {note}",
            phrase.key()
        );
    }
}

/// **No sentence names anything alo OS has rather than the person** — and
/// neither does a note, because a note is written for somebody translating into
/// a language where the machinery has no name either, nor a key, which is the
/// one thing a person is shown when a sentence is missing.
#[test]
fn no_sentence_or_note_names_the_machinery() {
    let vocabulary = everything_this_machine_can_say();
    for (crate_named, phrase) in everything_this_workstream_says(&vocabulary) {
        let read = format!(
            "{} {} {}",
            phrase.key(),
            phrase.source().as_written(),
            phrase.note().unwrap_or_default()
        )
        .to_lowercase();
        for never in NEVER_NAMED {
            assert!(
                !read.contains(never),
                "{crate_named}: {} says \"{never}\"",
                phrase.key()
            );
        }
    }
}

/// **No connector name is in anything a person reads.** This is the half of
/// the rule that a list of words cannot catch, because a connector name is not
/// a word: it is a kind and a number, `eDP-1`, and it arrives in a sentence
/// through a gap rather than in the sentence itself.
///
/// The sentences are checked here; the gap is checked in
/// `tests/the_walk_from_lock_to_resume_to_a_new_desk.rs`, which fills it from
/// a screen the machine really reported, and in `alo-displays`'
/// `plugged_into`, which is the crate that stopped it arriving.
#[test]
fn no_connector_name_is_in_anything_a_person_reads() {
    let vocabulary = everything_this_machine_can_say();
    for (crate_named, phrase) in everything_this_workstream_says(&vocabulary) {
        let read = format!(
            "{} {} {}",
            phrase.key(),
            phrase.source().as_written(),
            phrase.note().unwrap_or_default()
        )
        .to_lowercase();
        for kind in CONNECTOR_KINDS {
            assert!(
                !read.contains(kind),
                "{crate_named}: {} names the connector kind \"{kind}\"",
                phrase.key()
            );
        }
    }
}

/// **The check above is looking for something it could find.** A reader that
/// never matched would pass on anything, so it is shown the sentences this
/// workstream used to say and refuses them.
#[test]
fn the_reader_refuses_a_sentence_that_names_the_machinery() {
    let once_said = "eDP-1 does not say which screen it is".to_lowercase();
    assert!(
        CONNECTOR_KINDS.iter().any(|kind| once_said.contains(kind)),
        "the connector reader would not have caught what this task removed"
    );
    let would_be = "logind would not take the inhibitor".to_lowercase();
    assert!(NEVER_NAMED.iter().any(|never| would_be.contains(never)));
}

/// **No sentence here names anything on `alo-saying`'s own list of what this
/// product rents**, asked of all five areas at once.
///
/// Each crate asks this of itself already. What is different here is that it
/// is asked of the **assembled** vocabulary, so a crate that stopped being
/// collected — and whose own test would then be checking a list nobody
/// loads — is caught by [`the_five_crates_of_this_plan_are_collected`] above
/// and this one is asked of what a machine really holds.
#[test]
fn nothing_here_names_anything_the_machine_rents() {
    let vocabulary = everything_this_machine_can_say();
    let overheard = alo_saying::what_a_person_would_have_to_learn(&vocabulary);
    let ours: Vec<String> = overheard
        .iter()
        .filter(|one| {
            THE_FIVE_AREAS
                .iter()
                .any(|(_, area)| one.key().area() == *area)
        })
        .map(|one| format!("{one}"))
        .collect();
    assert!(ours.is_empty(), "{ours:?}");
}
