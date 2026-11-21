//! The question before an adapter's card, for every setting.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capability::AloInApplications;

use super::{BeforeTheCard, before_an_adapters_card};

const OPEN_DOCUMENT: &str = "text_editor.open_document";
const EDITOR: &str = "org.gnome.TextEditor";

#[test]
fn a_verb_that_is_no_adapters_is_not_this_questions() {
    let adapters = alo_adapters::shipped_adapters().unwrap();
    for chosen in AloInApplications::ALL {
        assert_eq!(
            before_an_adapters_card("read_file", &adapters, chosen),
            BeforeTheCard::NotAnAdaptersVerb
        );
    }
}

/// Text Editor has no copy of its own: alo may work in the person's window of
/// it at the default and turned up, and not at its own copy only.
#[test]
fn text_editor_may_go_except_at_its_own_copy_only() {
    let adapters = alo_adapters::shipped_adapters().unwrap();
    assert_eq!(
        before_an_adapters_card(OPEN_DOCUMENT, &adapters, AloInApplications::default()),
        BeforeTheCard::MayGo
    );
    assert_eq!(
        before_an_adapters_card(OPEN_DOCUMENT, &adapters, AloInApplications::InTheirWindow),
        BeforeTheCard::MayGo
    );
    assert_eq!(
        before_an_adapters_card(OPEN_DOCUMENT, &adapters, AloInApplications::InItsOwnCopy),
        BeforeTheCard::NotHere(EDITOR)
    );
}
