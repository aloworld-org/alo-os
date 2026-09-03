//! What a machine ends up saying, against the vocabulary it really has.
//!
//! The unit tests inside this crate are written against three strings, because
//! what this crate does is take a vocabulary it is handed and put translations
//! onto it, and a test written against several hundred would be a test about
//! whichever crate last declared something.
//!
//! These are the other half: a translation naming strings that really exist,
//! checked against the vocabulary alo OS really has. They are what would catch
//! a crate dropping out of the collection, a key changing shape, or the loading
//! meaning something different once there is more than a handful of strings in
//! front of it.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use alo_saying::{Loaded, NotSpoken, everything_this_machine_can_say};
use alo_strings::{Filling, Key, Language};

/// A folder of this test's own, on the disk the tests are running on.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-saying-really-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(fs::remove_dir_all(&folder));
    fs::create_dir_all(&folder).unwrap();
    folder
}

/// Three strings from three different crates, translated into German.
const THREE_CRATES: &str = "\
format = 1
language = \"de\"

[says]
\"appearance.token.navy\" = \"Marineblau\"
\"dock.edge.bottom\" = \"Unten\"
\"models.source.this-machine\" = \"auf diesem Gerät\"
";

/// **A translation naming strings that really exist loads with nothing wrong**,
/// which is the whole claim this crate makes and the one a release note counts.
#[test]
fn a_translation_of_what_this_machine_really_says_loads_whole() {
    let folder = a_folder_of_our_own("three-crates");
    fs::write(folder.join("de.toml"), THREE_CRATES).unwrap();

    let loaded = Loaded::at(everything_this_machine_can_say().unwrap(), &folder);
    assert!(loaded.damage().is_none(), "{:?}", loaded.damage().lines());
    assert_eq!(loaded.spoken().count(), 1);

    let mut strings = loaded.into_strings();
    strings.prefers(&[Language::written("de").unwrap()]);
    for (named, says) in [
        ("appearance.token.navy", "Marineblau"),
        ("dock.edge.bottom", "Unten"),
        ("models.source.this-machine", "auf diesem Gerät"),
    ] {
        let said = strings.say(&Key::named(named).unwrap(), &Filling::nothing());
        assert_eq!(said.text(), says);
        assert!(said.is_translated(), "{named}");
    }
}

/// **The strings a process says on top of the machine's are left out rather
/// than costing the language**, which is what makes one vocabulary for the
/// machine survivable in a process that says less than the machine does.
///
/// `alo-agentd` is the case: it is Linux, so it is not collected here, and its
/// three strings are declared by the daemon itself. A shell loading the same
/// file leaves those three lines out and still speaks German.
#[test]
fn a_line_only_the_daemon_says_is_left_out_and_german_survives() {
    let folder = a_folder_of_our_own("the-daemons-own");
    fs::write(
        folder.join("de.toml"),
        format!("{THREE_CRATES}\"agentd.a-turn-is-under-way\" = \"Gerade läuft schon etwas\"\n"),
    )
    .unwrap();

    let loaded = Loaded::at(everything_this_machine_can_say().unwrap(), &folder);
    assert_eq!(loaded.spoken().count(), 1);
    assert!(loaded.damage().not_spoken_of().is_empty());
    assert_eq!(loaded.damage().left_out_of().len(), 1);
    assert!(
        loaded
            .damage()
            .lines()
            .iter()
            .any(|line| line.contains("agentd.a-turn-is-under-way")),
        "{:?}",
        loaded.damage().lines()
    );

    let mut strings = loaded.into_strings();
    strings.prefers(&[Language::written("de").unwrap()]);
    assert_eq!(
        strings
            .say(
                &Key::named("dock.edge.bottom").unwrap(),
                &Filling::nothing()
            )
            .text(),
        "Unten"
    );
}

/// **A machine with nothing translated speaks English and can say everything**,
/// which is the machine alo OS ships today: `docs/features.md` puts the 24
/// languages at v0.5 and there are none yet.
#[test]
fn a_machine_with_no_translations_can_still_say_everything_it_says() {
    let loaded = Loaded::in_english(everything_this_machine_can_say().unwrap());
    let strings = loaded.strings();

    // Nothing is translated, so every string the machine has is unanswered —
    // and `unanswered` counting them is what a release note counts.
    assert_eq!(
        strings.unanswered().len(),
        strings.vocabulary().phrases().count()
            + strings
                .vocabulary()
                .counted()
                .map(|_| 1_usize)
                .sum::<usize>()
    );
    assert!(strings.unanswered().len() > 200);

    let said = strings.say(
        &Key::named("appearance.token.navy").unwrap(),
        &Filling::nothing(),
    );
    assert_eq!(said.text(), "Navy");
    assert!(!said.is_translated());
}

/// A machine whose translations did not arrive says where it looked, and
/// carries on. This is what a service reads out of `damage` on the machine alo
/// OS ships today, where the directory does not exist yet.
#[test]
fn the_directory_alo_os_ships_with_is_reported_rather_than_fatal() {
    let folder = a_folder_of_our_own("nowhere").join("translations");
    let loaded = Loaded::at(everything_this_machine_can_say().unwrap(), &folder);

    assert_eq!(loaded.spoken().count(), 0);
    assert!(matches!(
        loaded.damage().not_spoken_of().first(),
        Some(NotSpoken::NoneHere { .. })
    ));
    assert_eq!(
        loaded
            .strings()
            .say(
                &Key::named("dock.edge.bottom").unwrap(),
                &Filling::nothing()
            )
            .text(),
        "Bottom"
    );
}
