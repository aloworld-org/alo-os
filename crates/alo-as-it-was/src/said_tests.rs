//! Every refusal says something, and says all of it.

use alo_strings::Strings;

use super::NotGoingBack;
use crate::words::as_it_was_words;

/// Every refusal there is, each built the way a caller would see it.
///
/// Listed rather than generated, because the point is that **this list and
/// `NotGoingBack` agree**: a variant added to the enum has to be added here for
/// `said` to compile, and `said`'s `match` has no wildcard arm.
fn every_refusal() -> Vec<NotGoingBack> {
    let somewhere = std::path::Path::new("/home/alo/src").to_owned();
    vec![
        NotGoingBack::NotInTheirHome {
            folder: somewhere.clone(),
            home: std::path::Path::new("/home/alo").to_owned(),
        },
        NotGoingBack::TheWholeHome {
            home: std::path::Path::new("/home/alo").to_owned(),
        },
        NotGoingBack::NotAFolder {
            folder: somewhere.clone(),
        },
        NotGoingBack::NotInThatPlace {
            folder: somewhere.clone(),
        },
        NotGoingBack::ALinkRatherThanAFolder {
            at: somewhere.clone(),
        },
        NotGoingBack::OneIsHalfDone {
            at: somewhere.clone(),
        },
        NotGoingBack::NowhereToSetAsideWhatIsThere {
            at: somewhere.clone(),
        },
        NotGoingBack::NotCopied {
            why: "no room on the disk".to_owned(),
        },
        NotGoingBack::NotSwapped {
            why: "no room on the disk".to_owned(),
        },
        NotGoingBack::SetAsideUnderTheWorkingName {
            folder: std::path::Path::new("/home/alo/src").to_owned(),
            at: somewhere,
            why: "no room on the disk".to_owned(),
        },
    ]
}

/// **Every refusal has a sentence, and it is this crate's and not a key.**
///
/// `alo-strings` answers a key it does not hold with the key in guillemets
/// rather than failing, which is the right behaviour and is also how a missing
/// declaration would otherwise reach a person's screen looking like a sentence.
#[test]
fn every_refusal_says_something_that_is_not_its_key() {
    let strings = Strings::of(as_it_was_words().unwrap());
    for refusal in every_refusal() {
        let said = refusal.said(&strings);
        assert!(!said.text().is_empty(), "{refusal:?} said nothing at all");
        assert!(
            !said.text().starts_with('«'),
            "{refusal:?} has no sentence declared for it: {}",
            said.text()
        );
    }
}

/// **No sentence reaches a person with a gap still in it.**
///
/// The fault this catches is a filling that names one gap where the sentence
/// has two: the text comes out with `{aside}` in it, which is a sentence a
/// person reads and cannot act on. `Said::unfilled` is what knows, and nothing
/// else does — the text itself looks like prose.
#[test]
fn no_refusal_leaves_a_gap_unfilled() {
    let strings = Strings::of(as_it_was_words().unwrap());
    for refusal in every_refusal() {
        let said = refusal.said(&strings);
        assert!(
            said.unfilled().is_empty(),
            "{refusal:?} left {:?} unfilled: {}",
            said.unfilled(),
            said.text()
        );
    }
}

/// **A path that was named is in the sentence**, so a person who typed two
/// things knows which one was refused.
#[test]
fn the_path_a_person_named_is_in_the_sentence() {
    let strings = Strings::of(as_it_was_words().unwrap());
    let said = NotGoingBack::NotAFolder {
        folder: std::path::Path::new("/home/alo/typo").to_owned(),
    }
    .said(&strings);
    assert!(
        said.text().contains("/home/alo/typo"),
        "the path they named is missing: {}",
        said.text()
    );
}
