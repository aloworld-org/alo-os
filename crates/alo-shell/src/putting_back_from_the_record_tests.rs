//! Which changes offer *Put back*, and what putting one back would do.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err, variant or index is the failure being reported"
)]

use super::*;
use crate::record_testing::{A_PLACE, a_change_with_a_place_kept, an_afternoon_kept, words};
use crate::{RecordShows, RecordWindow};

/// The account `kept` opens on.
fn opened(kept: &crate::record_testing::Kept) -> RecordWindow {
    let mut window = RecordWindow::on_an_output();
    window.opened_by_hand(&kept.recounting());
    window
}

/// The account in `window`.
fn account_of(window: &RecordWindow) -> &Account {
    match window.shows() {
        RecordShows::Account { account, .. } => account,
        other => panic!("no account: {other:?}"),
    }
}

/// **A folder is itself, a file is its folder, one inside another is the
/// outer one**, in the order first met — what a move between two folders, or
/// a rename in one, touched.
#[test]
fn the_folders_a_change_touched() {
    let folders_are = |paths: &[&str], folders: &[&str]| {
        let touched = paths.iter().map(Path::new);
        let is_a_folder = |path: &Path| folders.iter().any(|it| Path::new(it) == path);
        folders_of(touched, is_a_folder)
    };
    assert_eq!(
        folders_are(
            &["/home/alo/Documents/march.pdf", "/home/alo/Archive"],
            &["/home/alo/Archive"]
        ),
        [
            PathBuf::from("/home/alo/Documents"),
            PathBuf::from("/home/alo/Archive")
        ]
    );
    assert_eq!(
        folders_are(
            &[
                "/home/alo/Documents/2026/a.pdf",
                "/home/alo/Documents/b.pdf"
            ],
            &[]
        ),
        [PathBuf::from("/home/alo/Documents")],
        "the inner folder goes back with the outer one"
    );
    assert_eq!(
        folders_are(
            &["/home/alo/Documents/a.pdf", "/home/alo/Documents/b.pdf"],
            &[]
        ),
        [PathBuf::from("/home/alo/Documents")]
    );
}

/// **A change that names a kept place offers Put back; one that does not is
/// older than Put back; one whose place is gone says so; nothing else offers
/// anything.**
#[test]
fn what_each_entry_offers() {
    let change = a_change_with_a_place_kept();
    let window = opened(&change.kept);
    let ran = account_of(&window)
        .told()
        .iter()
        .find(|told| told.changed())
        .unwrap();
    assert_eq!(
        PutBackHere::of(ran, &[A_PLACE.to_owned()]),
        PutBackHere::Offered
    );
    assert_eq!(PutBackHere::of(ran, &[]), PutBackHere::NoLongerKept);

    let afternoon = an_afternoon_kept();
    let window = opened(&afternoon);
    for told in account_of(&window).told() {
        let offered = PutBackHere::of(told, &[A_PLACE.to_owned()]);
        if told.changed() {
            assert_eq!(offered, PutBackHere::OlderThanPutBack, "{told:?}");
        } else {
            assert_eq!(offered, PutBackHere::Nothing, "{told:?}");
        }
    }
}

/// **The plan is every folder the change touched, each with where what is in
/// it now will be** — the invoices it was moved from and the archive it was
/// moved into.
#[test]
fn the_plan_names_every_folder_and_where_each_is_kept() {
    let change = a_change_with_a_place_kept();
    let window = opened(&change.kept);
    let account = account_of(&window);
    let at = account
        .told()
        .iter()
        .position(alo_recounting::Told::changed)
        .unwrap();
    let home = change.home();
    let plan = the_plan(
        account,
        at,
        (change.whose.path(), &home),
        &words(),
        SystemTime::now(),
    )
    .unwrap();
    assert_eq!(plan.place, A_PLACE);
    assert_eq!(plan.folders.len(), 2, "{plan:?}");
    assert!(plan.folders[0].ends_with("invoices"), "{plan:?}");
    assert!(plan.folders[1].ends_with("archive"), "{plan:?}");
    for (folder, aside) in plan.folders.iter().zip(&plan.asides) {
        assert_eq!(folder.parent(), aside.parent(), "beside it");
        assert!(
            named(aside).starts_with(&format!("{}.alo-was-here-", named(folder))),
            "{aside:?}"
        );
    }
    assert_eq!(plan.folders_as_asked().len(), 2);
}

/// **A place that is not among this person's is refused in words**, and
/// nothing is planned.
#[test]
fn a_place_not_kept_is_refused_in_words() {
    let change = a_change_with_a_place_kept();
    let window = opened(&change.kept);
    let account = account_of(&window);
    let at = account
        .told()
        .iter()
        .position(alo_recounting::Told::changed)
        .unwrap();
    let nobodys = tempfile::tempdir().unwrap();
    let refused = the_plan(
        account,
        at,
        (nobodys.path(), &change.home()),
        &words(),
        SystemTime::now(),
    )
    .unwrap_err();
    assert!(refused.contains(A_PLACE), "{refused}");
    assert_eq!(the_places_under(nobodys.path()), Vec::<String>::new());
    assert_eq!(the_places_under(change.whose.path()), [A_PLACE.to_owned()]);
}
