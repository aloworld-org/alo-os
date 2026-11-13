//! **A GitHub sign-in in the record window**: the clause in the person's
//! language, the login beneath it, and why it did not finish where `gh` said —
//! kept the way the daemon keeps it, and read back the way the window reads.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use alo_record::{AtGitHub, Entry, WhyNotSignedIn};

use super::*;
use crate::record_kinds::EntryKind;
use crate::record_testing::{a_clock, an_empty_record, noon, words};
use crate::{RecordOpened, RecordShows, RecordWindow};

/// The words the window draws for `kept`, newest first.
fn drawn_for(kept: &[AtGitHub]) -> Vec<EntryWords> {
    let record = an_empty_record();
    for what in kept {
        record.keeping(&Entry::at_github(what.clone(), noon()));
    }
    let mut window = RecordWindow::on_an_output();
    assert_eq!(
        window.opened_by_hand(&record.recounting()),
        RecordOpened::Shown
    );
    let RecordShows::Account { account, .. } = window.shows() else {
        panic!("the record did not open");
    };
    let strings = words();
    let nothing_to_put_back = crate::ThePutBack::default();
    newest_first(account, (0, &nothing_to_put_back), &strings, a_clock()).collect()
}

/// **Signing in reads as signing in, with the login beneath and nobody's
/// authority beside it.**
#[test]
fn a_sign_in_is_drawn_with_the_login_beneath_it() {
    let drawn = drawn_for(&[AtGitHub::signed_in("anna").unwrap()]);
    let [entry] = drawn.as_slice() else {
        panic!("{drawn:?}");
    };
    assert_eq!(entry.clause, "you signed in to GitHub");
    assert_eq!(entry.agent, None);
    assert_eq!(entry.after, vec!["anna".to_owned()]);
    assert_eq!(entry.approved, None);
    assert_eq!(EntryKind::of(entry.outcome), EntryKind::Changed);
}

/// **A sign-in that failed is drawn as a refusal, with what `gh` said**, and
/// one the person cancelled is drawn as neither a change nor a refusal.
#[test]
fn a_failed_sign_in_is_a_refusal_and_a_cancelled_one_is_not() {
    let drawn = drawn_for(&[
        AtGitHub::did_not_sign_in(WhyNotSignedIn::gh_failed(Some("HTTP 502"))),
        AtGitHub::did_not_sign_in(WhyNotSignedIn::TheyCancelled),
    ]);
    let [cancelled, failed] = drawn.as_slice() else {
        panic!("{drawn:?}");
    };
    assert!(
        failed.clause.contains("gh stopped with an error"),
        "{failed:?}"
    );
    assert_eq!(failed.after, vec!["HTTP 502".to_owned()]);
    assert_eq!(EntryKind::of(failed.outcome), EntryKind::Refused);
    assert!(cancelled.clause.contains("cancelled"), "{cancelled:?}");
    assert!(cancelled.after.is_empty(), "{cancelled:?}");
    assert_eq!(EntryKind::of(cancelled.outcome), EntryKind::Else);
}
