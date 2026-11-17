use std::path::Path;

use alo_acting::said_of_publishing::{Publishing, a_publication};
use alo_publishing::{Account, NotPublished, Publication, Published};

use super::{said, said_before_asking};
use crate::refusing::NotDone;
use crate::what_a_verb_did::WhatAVerbDid;

fn published(nothing_new: bool) -> Result<WhatAVerbDid, NotDone> {
    let publication = Publication::of(
        Path::new("/home/anna/Documents/site"),
        Account::named("anna").unwrap(),
    );
    Ok(WhatAVerbDid::from(Published::of(
        &publication,
        nothing_new,
        false,
    )))
}

/// **A publication that pushed says *published*, and one that found the site
/// already holding the folder says *nothing new* — never *published*,** so a
/// walk that waits for a push is not answered by a run that sent nothing.
#[test]
fn a_push_and_nothing_new_are_said_apart() {
    assert_eq!(
        said(&published(false)),
        a_publication(Publishing::Published)
    );
    assert_eq!(
        said(&published(true)),
        a_publication(Publishing::NothingNew)
    );
}

/// **Nobody signed in is said as itself, carried out or refused before
/// asking**, and every other refusal is *not done* — never *published*.
#[test]
fn nobody_signed_in_is_said_as_itself_and_every_other_refusal_as_not_done() {
    let signed_in = a_publication(Publishing::NotSignedIn);
    assert_eq!(
        said(&Err(NotDone::CouldNotPublish(NotPublished::NotSignedIn))),
        signed_in
    );
    assert_eq!(said_before_asking(&NotPublished::NotSignedIn), signed_in);

    let not_done = a_publication(Publishing::NotDone);
    for why in [
        NotPublished::Empty,
        NotPublished::NoGh,
        NotPublished::TookTooLong,
    ] {
        assert_eq!(said_before_asking(&why), not_done, "{why:?}");
        assert_eq!(said(&Err(NotDone::CouldNotPublish(why))), not_done);
    }
    assert_eq!(said(&Err(NotDone::TurnClosed)), not_done);
}

/// **The line never names the folder or the account**, which are the
/// person's.
#[test]
fn the_line_names_neither_the_folder_nor_the_account() {
    let line = said(&published(false));
    assert!(!line.contains("anna"), "{line}");
    assert!(!line.contains("site"), "{line}");
}
