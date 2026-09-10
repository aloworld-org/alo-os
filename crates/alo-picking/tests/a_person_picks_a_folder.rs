//! The plan's acceptance for *native folder selection, so a grant can be made
//! at all*, one test per criterion.
//!
//! `docs/autonomy/v0-01-delivery-plan.md`, task 6: *a person picks a folder and
//! a grant exists afterwards that the daemon honours; picking nothing grants
//! nothing; and the grant's scope is the folder picked rather than its parent.*
//!
//! These tests walk a **real filesystem** through [`alo_picking::OnThisDisk`],
//! because the acceptance is about a person picking a folder on a machine and
//! not about a state machine over a table. The folders are made under the
//! machine's own temporary directory and taken away afterwards.
//!
//! What is asked at the end of each is `alo_capability::Grants::permits` — the
//! question `alo-agentd` asks of every verb before it touches anything (see
//! that crate's `authorised.rs`), rather than a second opinion this crate holds
//! about its own work. And the sentences are read out of
//! `alo_saying::everything_this_machine_can_say`, so *the strings are
//! externalised* means collected into the machine's one vocabulary rather than
//! declared in a list only this crate ever loads.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being \
              reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_capability::{Ask, Grantee, Grants};
use alo_picking::{Chosen, Granting, OnThisDisk, Picker};
use alo_strings::Strings;

/// The agent every grant here is made to.
const HERS: &str = "@files";

/// How long a grant made in these tests lasts.
fn an_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// A moment, fixed, because nothing in this repository reads the clock to
/// decide something a record will disagree with later.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A home folder on the real disk: three folders inside it, one file beside
/// them, and one file inside the folder that gets picked.
///
/// Named for the test that asked for it, so two tests running at once never
/// share a tree.
fn a_home_folder_for(what: &str) -> PathBuf {
    let home = std::env::temp_dir().join(format!("alo-picking-{what}-{}", std::process::id()));
    let _gone = fs::remove_dir_all(&home);
    fs::create_dir_all(home.join("Invoices")).unwrap();
    fs::create_dir_all(home.join("Photos")).unwrap();
    fs::create_dir_all(home.join("Work")).unwrap();
    fs::write(home.join("notes.txt"), "not a folder").unwrap();
    fs::write(home.join("Invoices").join("march.pdf"), "an invoice").unwrap();
    home
}

/// Everything this machine can say, which is what a real shell holds.
fn what_this_machine_says() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// **Criterion 1: a person picks a folder, and a grant exists afterwards that
/// the daemon honours.**
///
/// The whole act, end to end: a picker opened where a person is, one of the
/// folders in front of them opened, picked — and then the daemon's own
/// question, about a file inside it, answered yes by a list that was empty a
/// moment before.
#[test]
fn a_person_picks_a_folder_and_the_daemon_honours_the_grant() {
    let home = a_home_folder_for("honours");
    let mut grants = Grants::default();
    let hers = Grantee::named(HERS);
    let march = Ask::path(home.join("Invoices").join("march.pdf"));

    // Before: this is `alo-agentd`'s starting state, and every verb is refused.
    assert!(grants.is_empty());
    assert!(!grants.permits(&hers, &march, noon()));

    // The person opens the picker, sees what is in their home folder, and
    // walks into one of them.
    let mut picker = Picker::standing_in(&home, &OnThisDisk).unwrap();
    assert_eq!(picker.showing(), ["Invoices", "Photos", "Work"]);
    picker.go_into("Invoices", &OnThisDisk).unwrap();
    assert_eq!(picker.at(), home.join("Invoices"));

    // They read what picking it will do, in the machine's own vocabulary, and
    // pick it.
    let chosen = picker.pick().unwrap();
    let Chosen::Folder(picked) = &chosen else {
        panic!("a folder was picked and the picker said otherwise");
    };
    let said = picked.said(&what_this_machine_says());
    assert!(
        !said.is_a_bug(),
        "the machine cannot say what a pick covers"
    );
    assert!(said.text().contains("Invoices"), "{said}");

    let id = Granting::to(HERS, an_hour())
        .of(&chosen, &mut grants, noon())
        .unwrap();
    assert!(id.is_some(), "a pick made no grant");

    // After: the grant is there, the daemon's own question is answered yes,
    // and it is a grant a person can find and revoke.
    assert_eq!(grants.len(), 1);
    assert!(grants.permits(&hers, &march, noon()));
    assert_eq!(grants.held_by(&hers, noon()).count(), 1);
    assert!(grants.revoke(id.unwrap()));
    assert!(
        !grants.permits(&hers, &march, noon()),
        "a revoked grant still permitted a verb"
    );

    let _gone = fs::remove_dir_all(&home);
}

/// **Criterion 2: picking nothing grants nothing.**
///
/// The person opens the picker, walks into a folder — so that there is a
/// folder in hand for something to go wrong with — and closes it without
/// picking. Nothing is granted, the list is byte for byte what it was, and the
/// folder they were standing in is not reachable.
#[test]
fn picking_nothing_grants_nothing() {
    let home = a_home_folder_for("nothing");
    let mut grants = Grants::default();
    let hers = Grantee::named(HERS);
    let before = serde_json::to_string(&grants).unwrap();

    let mut picker = Picker::standing_in(&home, &OnThisDisk).unwrap();
    picker.go_into("Invoices", &OnThisDisk).unwrap();

    let chosen = picker.closed_without_picking();
    assert!(chosen.is_nothing());
    assert_eq!(chosen.folder(), None);

    let nothing = Granting::to(HERS, an_hour())
        .of(&chosen, &mut grants, noon())
        .unwrap();
    assert_eq!(nothing, None, "closing the picker made a grant");
    assert!(grants.is_empty());
    assert_eq!(serde_json::to_string(&grants).unwrap(), before);
    assert!(!grants.permits(&hers, &Ask::path(home.join("Invoices")), noon()));
    assert!(!grants.permits(&hers, &Ask::path(&home), noon()));

    let _gone = fs::remove_dir_all(&home);
}

/// **Criterion 3: the grant's scope is the folder picked rather than its
/// parent.**
///
/// The person walked through their home folder to get to the one they picked,
/// which is the shape of the mistake this criterion exists to catch: the
/// picker was opened at the parent, and a grant over *where the picker
/// started* would have been the easy thing to build.
#[test]
fn the_grant_is_over_the_folder_picked_and_not_the_one_it_was_picked_from() {
    let home = a_home_folder_for("scope");
    let mut grants = Grants::default();
    let hers = Grantee::named(HERS);

    let mut picker = Picker::standing_in(&home, &OnThisDisk).unwrap();
    picker.go_into("Invoices", &OnThisDisk).unwrap();
    let chosen = picker.pick().unwrap();
    assert_eq!(chosen.folder(), Some(home.join("Invoices").as_path()));

    Granting::to(HERS, an_hour())
        .of(&chosen, &mut grants, noon())
        .unwrap();

    // Inside what was picked: permitted, the folder itself included.
    assert!(grants.permits(&hers, &Ask::path(home.join("Invoices")), noon()));
    assert!(grants.permits(
        &hers,
        &Ask::path(home.join("Invoices").join("march.pdf")),
        noon()
    ));

    // The parent it was picked from, and everything else under that parent:
    // refused. A grant that had widened by one folder would permit all three.
    for elsewhere in [
        home.clone(),
        home.join("notes.txt"),
        home.join("Photos"),
        home.join("Photos").join("holiday.jpg"),
        home.join("Work"),
    ] {
        assert!(
            !grants.permits(&hers, &Ask::path(&elsewhere), noon()),
            "{} was granted, and only Invoices was picked",
            elsewhere.display()
        );
    }

    // And a folder whose name merely begins the same way is a different
    // folder, which is `alo-capability`'s component-by-component comparison
    // seen from the surface that made the grant.
    let sibling = home.join("Invoices2");
    assert!(!grants.permits(&hers, &Ask::path(&sibling), noon()));

    let _gone = fs::remove_dir_all(&home);
}

/// **And the refusal path of the same criterion: the whole machine is never
/// what gets picked.**
///
/// Going up is how a person reaches a folder that is not under where the
/// picker was opened, and it ends at the top of the disk. Standing there is
/// allowed; picking it is refused, in a sentence out of the machine's own
/// vocabulary — because ADR 0001 §3 says there is no grant to `/`, and a
/// picker that quietly did nothing at the top would leave a person pressing a
/// button that never works.
#[test]
fn the_top_of_the_disk_cannot_be_picked_and_says_so() {
    let home = a_home_folder_for("the-top");
    let mut picker = Picker::standing_in(&home, &OnThisDisk).unwrap();

    // Up, as far as it goes — a real path, so however many folders that is.
    while !picker.is_at_the_top() {
        picker.go_up(&OnThisDisk).unwrap();
    }
    assert!(picker.at().has_root());

    let refused = picker.pick().unwrap_err();
    assert_eq!(refused, alo_picking::NotPicked::TheWholeMachine);
    let said = refused.said(&what_this_machine_says());
    assert!(!said.is_a_bug(), "the machine cannot say why it refused");
    assert!(said.text().contains("whole machine"), "{said}");

    // Nothing above the top, and no grant came out of any of it.
    assert_eq!(
        picker.go_up(&OnThisDisk),
        Err(alo_picking::NotPicked::NothingAbove)
    );
    let mut grants = Grants::default();
    assert!(
        Granting::to(HERS, an_hour())
            .of(&Chosen::Nothing, &mut grants, noon())
            .unwrap()
            .is_none()
    );
    assert!(grants.is_empty());

    let _gone = fs::remove_dir_all(&home);
}

/// **And the refusal path that keeps a person inside the tree they walked**,
/// on a real disk: a name that is not one of the rows in front of them opens
/// nothing, whether it is a step upwards, a path from the top of the disk, or
/// a folder that is really there but somewhere else.
#[test]
fn a_folder_that_is_not_shown_cannot_be_opened_or_granted() {
    let home = a_home_folder_for("not-shown");
    let mut picker = Picker::standing_in(&home, &OnThisDisk).unwrap();

    for named in ["..", ".", "notes.txt", "invoices", "Invoices/march.pdf"] {
        assert_eq!(
            picker.go_into(named, &OnThisDisk),
            Err(alo_picking::NotPicked::NotShownHere),
            "`{named}` was opened"
        );
        assert_eq!(picker.at(), home, "the picker moved on a refusal");
    }

    // A file is not a folder, so it is not a row and it is not somewhere to
    // stand — the same fact said the other way round.
    assert_eq!(
        Picker::standing_in(&home.join("notes.txt"), &OnThisDisk),
        Err(alo_picking::NotPicked::NotAFolder)
    );
    assert_eq!(
        Picker::standing_in(&home.join("Ledgers"), &OnThisDisk),
        Err(alo_picking::NotPicked::NothingThere)
    );

    let _gone = fs::remove_dir_all(&home);
}

/// Everything this crate can say is **in the machine's one vocabulary**, and a
/// shell that loads the machine's words can therefore say all of it.
///
/// The crate's own list cannot show this: it would pass while nothing
/// collected it, which is the bug task 3 found in `alo-saying` and fixed for
/// `alo-overlay`. Task 2's sentences would have reached a real shell as keys.
#[test]
fn everything_this_crate_says_is_collected_into_the_machines_vocabulary() {
    let strings = what_this_machine_says();
    for word in alo_picking::EVERY_WORD {
        // Every gap this crate's list has is filled, so that a sentence with a
        // hole in it fails as a hole rather than as a string nothing declared.
        let mut filling = alo_strings::Filling::nothing();
        for gap in word.phrase().unwrap().source().gaps() {
            filling = filling.and(gap.clone(), "/home/anna/Invoices");
        }
        let said = strings.say(&word.key(), &filling);
        assert!(!said.is_a_bug(), "{} is not collected", word.named());
        assert!(!said.text().is_empty(), "{} says nothing", word.named());
    }
}

/// The picker's heading is one of them, read the way a shell reads it.
#[test]
fn the_picker_asks_for_a_folder_in_the_machines_own_words() {
    let said = Picker::ask(&what_this_machine_says());
    assert!(!said.is_a_bug());
    assert!(said.text().contains("Pick a folder"), "{said}");
}

/// A last honesty check on the fixture itself: the folders these tests walk
/// are really there, so a passing suite is not a suite that navigated nothing.
#[test]
fn the_folders_these_tests_walk_are_really_on_the_disk() {
    let home = a_home_folder_for("really-there");
    assert!(
        fs::symlink_metadata(home.join("Invoices"))
            .unwrap()
            .is_dir()
    );
    assert!(
        fs::symlink_metadata(home.join("Invoices").join("march.pdf"))
            .unwrap()
            .is_file()
    );
    assert_eq!(
        Picker::standing_in(&home, &OnThisDisk).unwrap().showing(),
        ["Invoices", "Photos", "Work"]
    );
    assert!(Path::new(&home).is_absolute());

    let _gone = fs::remove_dir_all(&home);
}
