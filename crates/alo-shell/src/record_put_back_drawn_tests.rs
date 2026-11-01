//! *Put back…* and its confirmation, as drawn in the record window.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use crate::putting_back_from_the_record::{PutBackHere, ThePlan};
use crate::record_raster::{EntryDrawn, RecordPicture, picture};
use crate::record_testing::{
    A_PLACE, a_change_with_a_place_kept, a_clock, an_afternoon_kept, light, words,
};
use crate::{RecordKey, RecordShows, RecordWindow, WindowControlLabels};

/// `window` drawn on the approved frame's display.
fn drawn(window: &RecordWindow) -> RecordPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        window.shows(),
        &words(),
        &mut labels,
        (1440, 960),
        light(),
        a_clock(),
        crate::display_scale::DisplayScale::ONE,
    )
    .unwrap()
}

/// The entry that ran a change.
fn the_change(picture: &RecordPicture) -> &EntryDrawn {
    picture
        .entries
        .iter()
        .find(|entry| entry.words.put_back != PutBackHere::Nothing)
        .expect("a change is drawn")
}

/// **A change that can be put back has *Put back…* at its end** — 40 tall,
/// 20 in from the card's edge, clear of its words.
#[test]
fn a_change_that_can_be_put_back_has_the_button_at_its_end() {
    let change = a_change_with_a_place_kept();
    let mut window = RecordWindow::on_an_output();
    window.opened_by_hand(&change.kept.recounting());
    window.the_places_are(vec![A_PLACE.to_owned()]);
    let picture = drawn(&window);
    let entry = the_change(&picture);
    assert_eq!(entry.words.put_back, PutBackHere::Offered);
    let button = entry.put_back_button.expect("drawn");
    assert_eq!(button.size.h, 40);
    assert_eq!(
        button.loc.x + button.size.w,
        entry.area.loc.x + entry.area.size.w - 20
    );
    assert!(entry.clause.area.loc.x + entry.clause.area.size.w <= button.loc.x);
    assert_eq!(entry.why_not, None);
}

/// **A change written down before Put back says so, in a plain line**, and
/// draws no button.
#[test]
fn a_change_older_than_put_back_says_so() {
    let afternoon = an_afternoon_kept();
    let mut window = RecordWindow::on_an_output();
    window.opened_by_hand(&afternoon.recounting());
    window.the_places_are(vec![A_PLACE.to_owned()]);
    let _ = window.pressed(RecordKey::Oldest, &afternoon.recounting());
    let picture = drawn(&window);
    let entry = the_change(&picture);
    assert_eq!(entry.words.put_back, PutBackHere::OlderThanPutBack);
    assert_eq!(entry.put_back_button, None);
    assert_eq!(
        entry.why_not.as_ref().map(|line| line.text.as_str()),
        Some("This entry is older than Put back, so it cannot be put back from here.")
    );
}

/// **The confirmation asks, names every folder and where it is kept, says it
/// is written down, and offers *Put back* and *Cancel*** — inside the entry
/// it is under.
#[test]
fn the_confirmation_asks_names_every_folder_and_offers_two_buttons() {
    let change = a_change_with_a_place_kept();
    let mut window = RecordWindow::on_an_output();
    window.opened_by_hand(&change.kept.recounting());
    window.the_places_are(vec![A_PLACE.to_owned()]);
    let RecordShows::Account { account, .. } = window.shows() else {
        panic!("no account");
    };
    let at = account
        .told()
        .iter()
        .position(alo_recounting::Told::changed)
        .unwrap();
    let plan = ThePlan {
        place: A_PLACE.to_owned(),
        taken: std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000),
        folders: vec!["/home/alo/invoices".into(), "/home/alo/archive".into()],
        asides: vec![
            "/home/alo/invoices.alo-was-here-1".into(),
            "/home/alo/archive.alo-was-here-1".into(),
        ],
    };
    window.the_confirmation_opens(at, Ok(plan));
    let picture = drawn(&window);
    let entry = the_change(&picture);
    let shown = entry
        .confirmation
        .as_ref()
        .expect("the confirmation is drawn");
    let question = shown.question.as_ref().expect("a question");
    assert!(
        question
            .text
            .starts_with("Put these folders back as they were on"),
        "{}",
        question.text
    );
    let lines: Vec<&str> = shown.lines.iter().map(|line| line.text.as_str()).collect();
    assert_eq!(
        lines,
        [
            "What is in invoices now is kept beside it, as invoices.alo-was-here-1.",
            "What is in archive now is kept beside it, as archive.alo-was-here-1.",
            "Putting back is written in this list too.",
        ]
    );
    let (put_back, cancel) = (shown.put_back.unwrap(), shown.cancel.unwrap());
    assert!(put_back.loc.x + put_back.size.w < cancel.loc.x);
    assert_eq!(put_back.size.h, 48);
    assert!(
        entry.area.loc.y + entry.area.size.h >= shown.area.loc.y + shown.area.size.h,
        "the confirmation is inside its entry"
    );
}

/// **Where nothing can be put back, the confirmation says why and offers
/// *Cancel* alone.**
#[test]
fn a_confirmation_that_cannot_put_back_says_why_and_offers_cancel() {
    let change = a_change_with_a_place_kept();
    let mut window = RecordWindow::on_an_output();
    window.opened_by_hand(&change.kept.recounting());
    window.the_places_are(vec![A_PLACE.to_owned()]);
    let RecordShows::Account { account, .. } = window.shows() else {
        panic!("no account");
    };
    let at = account
        .told()
        .iter()
        .position(alo_recounting::Told::changed)
        .unwrap();
    window.the_confirmation_opens(at, Err("invoices was not there".to_owned()));
    let picture = drawn(&window);
    let shown = the_change(&picture).confirmation.clone().unwrap();
    assert_eq!(shown.question, None);
    assert_eq!(
        shown.refusal.map(|line| line.text),
        Some("invoices was not there".to_owned())
    );
    assert_eq!(shown.put_back, None);
    assert!(shown.cancel.is_some());
}
