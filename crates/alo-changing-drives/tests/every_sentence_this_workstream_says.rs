//! Every sentence the broker's workstream can say, against the vocabulary a
//! real process loads — and every crate in it that deliberately says nothing.
//!
//! Task 7 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`: *every
//! sentence these crates can say is in the vocabulary with a translator's note
//! … no sentence names LUKS, TPM, CUPS, NetworkManager, a socket or root.*
//!
//! Each crate's own tests hold its own list — that its keys are its, that its
//! gaps are what it says they are, that its refusals name what to do. What none
//! of them can ask is the question this file asks: **does what this workstream
//! says survive being put beside every other crate's sentences**, and is a
//! person who never learns what any of the machinery is told everything they
//! need. A word declared in a crate and left out of `alo-saying`'s collection
//! compiles, tests, ships, and reaches a real shell as a key — which is the
//! failure `alo-collected` exists for, asked here of these crates by name.
//!
//! # The crates that say nothing are the point of the ones that do
//!
//! The broker runs as root and holds a closed list of verbs. It answers in one
//! word — carried, not kept, refused — and it has no person in front of it. A
//! daemon with a vocabulary would be a daemon deciding what a person reads, and
//! a privileged component whose dependency list can grow a serialiser is a
//! privileged component nobody can audit in an afternoon. So [`SAYS_NOTHING`]
//! is held here as strictly as the sentences are: those crates have no
//! `words.rs` and no `Word::saying` anywhere in them, and the day one grows one
//! this test says so.
//!
//! `alo-encrypting` is on that list for a second reason of its own: it holds a
//! recovery key for the length of one screen, and a vocabulary is a dependency
//! that brings a serialiser with it. Its sentences are `alo-enrolling`'s, which
//! is why that crate exists.
//!
//! # Why this test is in this crate
//!
//! It belongs to the workstream rather than to `alo-changing-drives`, and it
//! was written in `alo-brokerd`, which is the one crate that stands where all
//! four verb families meet. It cannot stay there: the walk beside it needs
//! `alo-capability`, and `alo-brokerd`'s own
//! `the_broker_can_make_no_grant` reads that manifest **as text** and refuses
//! the name wherever it appears, dev-dependencies included. That is the
//! stricter reading and the right one — *the process that mounts a drive can
//! make no grant* is a promise about a file somebody audits. So the two tests
//! live in the crate the walk found missing.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_strings::{Filling, Said, Strings, Vocabulary, Word};

/// Every crate in this workstream that says something to a person, with the
/// whole of what it can say.
fn everything_this_workstream_says() -> Vec<(&'static str, Vec<Word>)> {
    vec![
        (
            "alo-changing-printers",
            alo_changing_printers::EVERY_WORD.to_vec(),
        ),
        (
            "alo-changing-network",
            alo_changing_network::EVERY_WORD.to_vec(),
        ),
        (
            "alo-changing-drives",
            alo_changing_drives::EVERY_WORD.to_vec(),
        ),
        (
            "alo-changing-updates",
            alo_changing_updates::EVERY_WORD.to_vec(),
        ),
        ("alo-enrolling", alo_enrolling::EVERY_WORD.to_vec()),
    ]
}

/// Every crate in this workstream that says nothing to anybody, and why not.
///
/// An exception with no reason beside it is indistinguishable from the failure
/// the check exists for, which is a crate whose words nothing collects: the
/// difference between a documented exception and silence is entirely the
/// sentence. `alo-collected` holds the same rule for the workspace.
const SAYS_NOTHING: [(&str, &str); 4] = [
    (
        "alo-broker",
        "the door and the closed list. It answers in one word from a closed list and has no \
         person in front of it, and its dependency list is held by a test so that a privileged \
         component stays auditable in an afternoon — a vocabulary would add a crate to it for \
         nothing anybody reads.",
    ),
    (
        "alo-brokerd",
        "the broker as a machine runs it: root, holding no capability. What a person meets is \
         this one word worded by the surface that asked, which is why there are four such \
         surfaces and why none of them is this crate.",
    ),
    (
        "alo-drives",
        "what a drive is as the rented disk service reports it. Its refusals keep an English \
         Display for a service log; what a person reads is `alo-changing-drives`.",
    ),
    (
        "alo-encrypting",
        "what full-disk encryption is on this machine. It depends on nothing, and the absence is \
         the argument: it holds a recovery key for the length of one screen, and a vocabulary is \
         a dependency that brings a serialiser with it. Its sentences are `alo-enrolling`'s.",
    ),
];

/// Nothing a person reads names any of these — not the sentence, not the note a
/// translator works from.
///
/// The plan names six; the rest are the same rule applied to the machinery this
/// workstream happens to rent. A person is told about *this machine's security
/// chip*, *the key you wrote down*, *a printer*, *a network* and *a drive*,
/// because those are things they have.
const NEVER_NAMED: [&str; 24] = [
    "luks",
    "tpm",
    "cups",
    "networkmanager",
    "socket",
    "root",
    "cryptsetup",
    "cryptenroll",
    "keyslot",
    "key slot",
    "pcr",
    "udisks",
    "bootc",
    "ostree",
    "systemd",
    "d-bus",
    "dbus",
    "/dev/",
    "ipp:",
    "capability",
    "daemon",
    "sudo",
    "unmount",
    "partition",
];

/// Everything the machine can say, which is what a real process holds.
fn everything_this_machine_can_say() -> Vocabulary {
    match alo_saying::everything_this_machine_can_say() {
        Ok(vocabulary) => vocabulary,
        Err(why) => panic!("alo OS's own words are wrong: {why}"),
    }
}

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// Every `.rs` file under a directory.
fn sources_under(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(sources_under(&path));
        } else if path.extension().is_some_and(|end| end == "rs") {
            found.push(path);
        }
    }
    found
}

/// **Every sentence this workstream can say is one the machine can say.** A
/// word declared in a crate and left out of `alo-saying`'s collection reaches a
/// person as a key, and a key is what `alo-strings` calls a bug.
#[test]
fn every_sentence_this_workstream_says_is_one_the_machine_says() {
    let vocabulary = everything_this_machine_can_say();
    for (crate_named, words) in everything_this_workstream_says() {
        assert!(!words.is_empty(), "{crate_named} declares nothing");
        for word in words {
            assert!(
                vocabulary.phrase(&word.key()).is_some(),
                "{crate_named}: the machine cannot say {}",
                word.named()
            );
        }
    }
}

/// **Every sentence comes out whole, in the language a person reads.** Not a
/// key, and not a sentence with a gap nobody filled — the two ways a correct
/// vocabulary still reaches somebody as machinery.
///
/// Each gap is filled with something a person would recognise, because what is
/// being asked is whether the sentence arrives as a sentence, not what goes in
/// the gap. Which gaps a word has and what may go in them is each crate's own
/// test.
#[test]
fn every_sentence_comes_out_whole() {
    let strings = Strings::of(everything_this_machine_can_say());
    for (crate_named, words) in everything_this_workstream_says() {
        for word in words {
            let phrase = word.phrase().unwrap_or_else(|why| {
                panic!("{crate_named}: {} is not a phrase: {why}", word.named())
            });
            let mut filling = Filling::nothing();
            for gap in phrase.source().gaps() {
                filling = filling.and(gap.clone(), "something the person has");
            }
            let said: Said = strings.say(&word.key(), &filling);
            assert!(!said.is_a_bug(), "{crate_named}: {said}");
            assert!(said.unfilled().is_empty(), "{crate_named}: {said}");
            assert!(
                !said.text().is_empty(),
                "{crate_named}: {} says nothing",
                word.named()
            );
        }
    }
}

/// **Every sentence carries a note a translator can work from.** A translator
/// with no note writes a guess, and a guess in somebody's own language is worse
/// than English they can at least recognise as not theirs.
#[test]
fn every_sentence_carries_a_note_a_translator_can_work_from() {
    for (crate_named, words) in everything_this_workstream_says() {
        for word in words {
            let note = word.note().unwrap_or_default();
            assert!(
                note.len() > 40,
                "{crate_named}: {} has no note worth the name: {note}",
                word.named()
            );
        }
    }
}

/// **No sentence names anything alo OS has rather than the person** — and
/// neither does a note, because a note is written for somebody translating into
/// a language where the machinery has no name either.
#[test]
fn no_sentence_or_note_names_the_machinery() {
    for (crate_named, words) in everything_this_workstream_says() {
        for word in words {
            let said =
                format!("{} {}", word.says(), word.note().unwrap_or_default()).to_lowercase();
            for never in NEVER_NAMED {
                assert!(
                    !said.contains(never),
                    "{crate_named}: {} says \"{never}\"",
                    word.named()
                );
            }
        }
    }
}

/// **No two crates in this workstream claim one key.** Two crates claiming a
/// key is one of them unreachable in every language at once, and the vocabulary
/// refuses it — asked here of these crates alone, so that the answer names
/// which two.
#[test]
fn no_two_crates_here_claim_one_key() {
    let mut vocabulary = Vocabulary::empty();
    let mut seen: Vec<(String, &'static str)> = Vec::new();
    for (crate_named, words) in everything_this_workstream_says() {
        for word in words {
            let key = word.key().to_string();
            if let Some((_, first)) = seen.iter().find(|(held, _)| *held == key) {
                panic!("{first} and {crate_named} both claim {key}");
            }
            seen.push((key, crate_named));
            let phrase = word.phrase().unwrap_or_else(|why| {
                panic!("{crate_named}: {} is not a phrase: {why}", word.named())
            });
            if let Err(why) = vocabulary.says(phrase) {
                panic!("{crate_named}: {} would not declare: {why}", word.named());
            }
        }
    }
    assert_eq!(vocabulary.how_many(), seen.len());
}

/// **The crates that hold the machine's authority say nothing, and each says
/// why.** No `words.rs`, and no sentence written anywhere else in them either.
#[test]
fn the_crates_that_hold_authority_say_nothing_and_say_why() {
    let repository = the_repository();
    for (crate_named, reason) in SAYS_NOTHING {
        assert!(
            reason.len() > 80,
            "{crate_named} stands apart with a shrug rather than a reason"
        );
        let source = repository.join("crates").join(crate_named).join("src");
        assert!(source.is_dir(), "{crate_named} has no src/");
        assert!(
            !source.join("words.rs").exists(),
            "{crate_named} has grown a words.rs"
        );
        for file in sources_under(&source) {
            let written = std::fs::read_to_string(&file)
                .unwrap_or_else(|why| panic!("{} could not be read: {why}", file.display()));
            assert!(
                !written.contains("Word::saying"),
                "{} says something to a person",
                file.display()
            );
        }
    }
}

/// **The two crates this task added are collected**, by the same road every
/// other crate's words take. Their own tests read their own lists; this reads
/// the machine's.
#[test]
fn the_two_surfaces_this_task_added_are_collected() {
    let vocabulary = everything_this_machine_can_say();
    for (crate_named, listed) in [
        ("alo-changing-drives", alo_changing_drives::EVERY_WORD.len()),
        (
            "alo-changing-updates",
            alo_changing_updates::EVERY_WORD.len(),
        ),
    ] {
        assert!(
            alo_saying::EVERY_LIST.contains(&crate_named),
            "{crate_named} is not on alo-saying's list"
        );
        assert!(listed > 0, "{crate_named} declares nothing");
    }
    for word in alo_changing_drives::EVERY_WORD {
        assert!(vocabulary.phrase(&word.key()).is_some(), "{}", word.named());
    }
    for word in alo_changing_updates::EVERY_WORD {
        assert!(vocabulary.phrase(&word.key()).is_some(), "{}", word.named());
    }
}
