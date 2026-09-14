//! The record window opened, moved, read again and closed on a real record on a
//! real disk — and every way it is refused.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use super::*;
use crate::WindowControlLabels;
use crate::record_raster::picture;
use crate::record_testing::{
    a_long_record, an_afternoon_kept, an_empty_record, light, noon, ours_alone, words,
};
use alo_capability::Grantee;
use alo_keeping::NotKept;
use alo_record::Entry;
use alo_recounting::{Outcome, SurfaceRefused};
use std::os::unix::fs::PermissionsExt;

/// The account the window is showing, or a failure naming what it shows.
fn showing(window: &RecordWindow) -> &Account {
    match window.shows() {
        RecordShows::Account { account, .. } => account,
        other => panic!("the window is not showing an account: {other:?}"),
    }
}

/// How far the view has moved past the most recent entry.
fn moved(window: &RecordWindow) -> usize {
    match window.shows() {
        RecordShows::Account { moved_past, .. } => moved_past,
        other => panic!("the window is not showing an account: {other:?}"),
    }
}

/// **ADR 0009: the window is reachable without asking an agent anything.** On
/// a machine whose person's side knows only where its description is — no
/// turn, no model, no overlay, no agent — a person opens the window by hand
/// and the whole record is in it, refusals and all.
#[test]
fn the_window_opens_by_hand_with_no_agent_anywhere() {
    let kept = an_afternoon_kept();
    let recounting = Recounting::described_at(&kept.described()).unwrap();
    assert_eq!(recounting.where_it_is(), kept.at.as_path());

    let mut window = RecordWindow::on_an_output();
    assert!(!window.is_open());
    assert_eq!(window.opened_by_hand(&recounting), RecordOpened::Shown);
    assert!(window.is_open());

    let account = showing(&window);
    assert_eq!(account.how_many(), account.how_many_in_the_record());
    assert!(account.is_all_that_answered());
    let outcomes: Vec<Outcome> = account.told().iter().map(|told| told.outcome()).collect();
    for expected in [
        Outcome::Ran,
        Outcome::ThePersonSaidNo,
        Outcome::NeverBecameACall,
        Outcome::AnsweredHere,
        Outcome::Left,
        Outcome::HeldBack,
        Outcome::LeftOnItsOwn,
    ] {
        assert!(
            outcomes.contains(&expected),
            "{expected:?} is not in {outcomes:?}"
        );
    }
    assert_eq!(moved(&window), 0);
}

/// **Asking the agent *what did you do?* reaches the same account**, not a
/// second telling of it: the two roads, on one record, show equal accounts and
/// draw the same pixels.
#[test]
fn asking_the_agent_what_it_did_reaches_the_same_account() {
    let kept = an_afternoon_kept();
    let recounting = kept.recounting();
    let mut by_hand = RecordWindow::on_an_output();
    let mut asked = RecordWindow::on_an_output();
    assert_eq!(by_hand.opened_by_hand(&recounting), RecordOpened::Shown);
    assert_eq!(asked.asked_what_it_did(&recounting), RecordOpened::Shown);
    assert_eq!(showing(&by_hand), showing(&asked));

    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let drawn_by_hand = picture(
        by_hand.shows(),
        &strings,
        &mut labels,
        (1920, 1080),
        light(),
    )
    .unwrap();
    let drawn_asked = picture(asked.shows(), &strings, &mut labels, (1920, 1080), light()).unwrap();
    assert_eq!(drawn_by_hand, drawn_asked);
}

/// **The window reads the record and writes nothing to it.** Opening it, moving
/// through it, reading it again and closing it leave the file byte for byte as
/// the machine wrote it.
#[test]
fn the_window_reads_the_record_and_writes_nothing_to_it() {
    let kept = a_long_record(30);
    let before = std::fs::read(&kept.at).unwrap();
    let modified = std::fs::metadata(&kept.at).unwrap().modified().unwrap();
    let recounting = kept.recounting();

    let mut window = RecordWindow::on_an_output();
    window.opened_by_hand(&recounting);
    for key in [
        RecordKey::Older,
        RecordKey::Oldest,
        RecordKey::Newer,
        RecordKey::Newest,
        RecordKey::Nothing,
    ] {
        assert_eq!(window.pressed(key, &recounting), None);
    }
    assert_eq!(
        window.pressed(RecordKey::ReadAgain, &recounting),
        Some(RecordOpened::Shown)
    );
    window.asked_what_it_did(&recounting);
    window.pressed(RecordKey::Close, &recounting);

    assert_eq!(std::fs::read(&kept.at).unwrap(), before);
    assert_eq!(
        std::fs::metadata(&kept.at).unwrap().modified().unwrap(),
        modified
    );
}

/// **The whole record, unfiltered.** Whatever the roads, the window asks for
/// everything, so what it holds is every entry the record holds — the refusals
/// included — and nothing narrower.
#[test]
fn the_window_holds_every_entry_the_record_holds() {
    let kept = an_afternoon_kept();
    let recounting = kept.recounting();
    let everything = recounting
        .about(&Asking::anything(), AtMost::ONE_SITTING)
        .unwrap();
    let mut window = RecordWindow::on_an_output();
    window.opened_by_hand(&recounting);
    assert_eq!(showing(&window), &everything);
    assert!(
        showing(&window)
            .told()
            .iter()
            .any(|told| told.outcome() != Outcome::Ran)
    );
}

/// **The view moves through time and never past the record's ends**, and a
/// newer entry is reached by moving back towards the most recent.
#[test]
fn the_view_moves_through_the_account_and_stops_at_its_ends() {
    let kept = a_long_record(5);
    let recounting = kept.recounting();
    let mut window = RecordWindow::on_an_output();
    window.opened_by_hand(&recounting);

    window.pressed(RecordKey::Newer, &recounting);
    assert_eq!(moved(&window), 0);
    window.pressed(RecordKey::Older, &recounting);
    window.pressed(RecordKey::Older, &recounting);
    assert_eq!(moved(&window), 2);
    window.pressed(RecordKey::Oldest, &recounting);
    assert_eq!(moved(&window), 4);
    window.pressed(RecordKey::Older, &recounting);
    assert_eq!(moved(&window), 4, "the view moved past the oldest entry");
    window.pressed(RecordKey::Newer, &recounting);
    assert_eq!(moved(&window), 3);
    window.pressed(RecordKey::Newest, &recounting);
    assert_eq!(moved(&window), 0);
}

/// **The account is one reading of the disk.** An entry the machine keeps
/// while the window is open is not in the window until the person reads the
/// record again — and then it is, at the top.
#[test]
fn reading_again_reads_the_disk_again() {
    let kept = a_long_record(2);
    let recounting = kept.recounting();
    let mut window = RecordWindow::on_an_output();
    window.opened_by_hand(&recounting);
    window.pressed(RecordKey::Older, &recounting);
    assert_eq!(showing(&window).how_many(), 2);

    kept.keeping(&Entry::answered_here(&Grantee::named("@mail"), noon()));
    assert_eq!(
        showing(&window).how_many(),
        2,
        "the window read the disk by itself"
    );

    assert_eq!(
        window.pressed(RecordKey::ReadAgain, &recounting),
        Some(RecordOpened::Shown)
    );
    assert_eq!(showing(&window).how_many(), 3);
    assert_eq!(moved(&window), 0);
    assert_eq!(
        showing(&window).told().last().map(|told| told.outcome()),
        Some(Outcome::AnsweredHere)
    );
}

/// **A record that is not there is a refusal in the window, never an empty
/// list** — in `alo-keeping`'s words, which say why a missing record matters.
#[test]
fn a_record_that_is_not_there_is_a_refusal_rather_than_an_empty_window() {
    let kept = an_afternoon_kept();
    std::fs::remove_file(&kept.at).unwrap();
    let recounting = kept.recounting();
    let mut window = RecordWindow::on_an_output();

    assert_eq!(
        window.opened_by_hand(&recounting),
        RecordOpened::RefusedInTheWindow
    );
    let RecordShows::Refusal(why) = window.shows() else {
        panic!("a missing record was shown as {:?}", window.shows());
    };
    assert!(why.there_is_no_record());
    let said = why.said(&words());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("has done nothing"), "{said}");
    assert!(window.is_open());

    // The keys that move through an account do nothing to a refusal, and
    // Escape closes it.
    assert_eq!(window.pressed(RecordKey::Older, &recounting), None);
    assert!(matches!(window.shows(), RecordShows::Refusal(_)));
    window.pressed(RecordKey::Close, &recounting);
    assert!(!window.is_open());
}

/// **A record somebody else could have written is refused**, and nothing in
/// it reaches the window — not a line of it, and not an empty account either.
#[test]
fn a_record_somebody_else_could_write_is_refused() {
    let kept = an_afternoon_kept();
    std::fs::set_permissions(&kept.at, std::fs::Permissions::from_mode(0o666)).unwrap();
    let recounting = kept.recounting();
    let mut window = RecordWindow::on_an_output();

    assert_eq!(
        window.asked_what_it_did(&recounting),
        RecordOpened::RefusedInTheWindow
    );
    assert!(matches!(
        window.shows(),
        RecordShows::Refusal(NotRecounted::Record(NotKept::WritableByOthers { .. }))
    ));

    // Made the machine's own again, reading again shows it.
    ours_alone(&kept.at);
    assert_eq!(
        window.pressed(RecordKey::ReadAgain, &recounting),
        Some(RecordOpened::Shown)
    );
    assert!(matches!(window.shows(), RecordShows::Account { .. }));
}

/// **What is not a record is refused**, in the words of the crate that reads
/// records, and the account the window was showing comes down with it rather
/// than standing in for a file that is no longer one.
#[test]
fn a_file_that_is_not_a_record_takes_the_account_down() {
    let kept = a_long_record(3);
    let recounting = kept.recounting();
    let mut window = RecordWindow::on_an_output();
    window.opened_by_hand(&recounting);

    std::fs::write(&kept.at, "notes about the invoice\n").unwrap();
    ours_alone(&kept.at);
    assert_eq!(
        window.pressed(RecordKey::ReadAgain, &recounting),
        Some(RecordOpened::RefusedInTheWindow)
    );
    assert!(matches!(
        window.shows(),
        RecordShows::Refusal(NotRecounted::Record(NotKept::NotARecord { .. }))
    ));
}

/// **With nowhere to show it, the window stays closed** and the refusal goes to
/// the host's log in `alo-recounting`'s words; nothing is kept to be drawn
/// later.
#[test]
fn with_nowhere_to_show_it_the_window_stays_closed() {
    let kept = an_afternoon_kept();
    let recounting = kept.recounting();
    let mut window = RecordWindow::with_no_output();
    let opened = window.opened_by_hand(&recounting);
    assert_eq!(
        opened,
        RecordOpened::NowhereToShow(NotRecounted::Surface(SurfaceRefused::NothingToShowOn))
    );
    let RecordOpened::NowhereToShow(why) = opened else {
        unreachable!("matched above");
    };
    assert!(!why.said(&words()).is_a_bug());
    assert!(!window.is_open());
    assert_eq!(window.shows(), RecordShows::Nothing);
    assert_eq!(window.pressed(RecordKey::ReadAgain, &recounting), None);
}

/// **An empty record is an account that says so**, beside what the record says
/// about itself — never a blank window.
#[test]
fn an_empty_record_is_an_account_that_says_so() {
    let kept = an_empty_record();
    let recounting = kept.recounting();
    let mut window = RecordWindow::on_an_output();
    assert_eq!(window.opened_by_hand(&recounting), RecordOpened::Shown);
    let account = showing(&window);
    assert!(account.is_empty());
    let said = account.said(&words());
    assert!(said.len() >= 2, "{said:?}");
    assert!(said.iter().all(|said| !said.is_a_bug()), "{said:?}");
    window.pressed(RecordKey::Oldest, &recounting);
    assert_eq!(moved(&window), 0);
}

/// **A closed window does nothing with a key**, including reading the record:
/// only opening it does.
#[test]
fn a_closed_window_does_nothing_with_a_key() {
    let kept = a_long_record(3);
    let recounting = kept.recounting();
    let mut window = RecordWindow::on_an_output();
    for key in [
        RecordKey::ReadAgain,
        RecordKey::Older,
        RecordKey::Close,
        RecordKey::Nothing,
    ] {
        assert_eq!(window.pressed(key, &recounting), None);
        assert!(!window.is_open());
    }
}
