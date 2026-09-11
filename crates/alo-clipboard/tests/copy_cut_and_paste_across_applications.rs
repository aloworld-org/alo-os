//! The acceptance for *copy, cut and paste*, walked between two applications.
//!
//! `docs/features.md` promises at v0.01: **copy, cut and paste — text, images
//! and files, across applications.** This file is that promise as a sequence:
//! one application takes the selection, another asks for a form, and every way
//! it can go wrong is asked for as carefully as the way it goes right.
//!
//! # Two things this file does that the unit tests cannot
//!
//! **It is written against the machine's own vocabulary**, not against this
//! crate's list. `alo_saying::everything_this_machine_can_say` is what a real
//! shell holds, and a refusal looked up in it is the sentence a person would
//! actually read — which is the one question this crate's own tests cannot ask,
//! because a crate holding only its own strings says them whether or not
//! anybody collects them. `crates/alo-overlay` declared nine strings that
//! nothing collected and every one of its own tests passed.
//!
//! **It uses two owners at once**, which is what *across applications* means
//! and is where the promise is either kept or quietly broken: the moment one
//! application takes the selection from another is the moment a clipboard can
//! serve the wrong data to somebody who asked for the right thing.
//!
//! No pixels are claimed and none are tested. The compositor half belongs to
//! `crates/alo-shell`, which is the desktop lane's.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::rc::Rc;

use alo_clipboard::{
    Clipboard, CouldNotGive, EVERY_WORD, Form, Gives, Kind, NotPasted, Offer, Taking,
};
use alo_strings::{CameFrom, Filling, Strings};

/// What an application was asked for, readable after that application is gone —
/// which is what lets a test say *nobody was asked* rather than *nothing came
/// back*.
#[derive(Debug, Clone, Default)]
struct Asked {
    /// Every form it was asked for, in order.
    forms: Rc<RefCell<Vec<Kind>>>,
}

impl Asked {
    /// How many times the owner was asked for anything.
    fn how_many(&self) -> usize {
        self.forms.borrow().len()
    }
}

/// An application that owns a selection: it produces the text it was given, for
/// every form it was asked for.
struct AnApplication {
    /// What this one copied.
    copied: &'static str,
    /// What it has been asked for.
    asked: Asked,
}

/// An application that copied this, watched by this counter.
fn copying(copied: &'static str, asked: &Asked) -> Box<dyn Gives> {
    Box::new(AnApplication {
        copied,
        asked: asked.clone(),
    })
}

impl Gives for AnApplication {
    fn give(&mut self, form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
        self.asked.forms.borrow_mut().push(form.clone());
        Ok(format!("{} as {}", self.copied, form.media()).into_bytes())
    }
}

/// Everything this machine can say, which is what a real shell holds.
fn what_this_machine_says() -> Strings {
    Strings::of(
        alo_saying::everything_this_machine_can_say()
            .expect("alo OS's own words are collected into one vocabulary"),
    )
}

/// Plain text, which is the form nearly every copy offers.
fn text() -> Kind {
    Kind::text()
}

/// **What is pasted is what was copied, in a form the copier offered** — from
/// one application into another, which is the promise's *across applications*.
#[test]
fn what_is_pasted_is_what_was_copied_in_a_form_the_copier_offered() {
    let mut clipboard = Clipboard::nothing_copied_yet();
    let asked = Asked::default();

    // The editor copies, offering HTML first because that is what it would
    // rather be pasted as, and plain text after it.
    let html = Kind::named("text/html").expect("a media type");
    clipboard.taken(
        Offer::copied(vec![html.clone(), text()]).expect("two forms are an offer"),
        copying("the second paragraph", &asked),
    );

    // The other application accepts only plain text, and says so.
    let offered = clipboard.to_paste().expect("something was copied");
    let form = offered
        .best_of(&[text()])
        .expect("the copier offered plain text")
        .clone();
    assert_eq!(form, text());

    let pasted = clipboard.paste(&offered, &form).expect("a form on offer");
    assert_eq!(
        pasted.bytes(),
        b"the second paragraph as text/plain;charset=utf-8"
    );
    assert_eq!(pasted.form(), &text());
    assert!(!pasted.the_owner_gave_it_up());

    // The owner was asked once, for exactly the form that was pasted. Nothing
    // was fetched ahead of time and nothing was converted.
    assert_eq!(asked.forms.borrow().as_slice(), [text()]);

    // And a third application that wants HTML gets the copier's HTML, from the
    // same selection, because a copy is not spent by being pasted.
    let again = clipboard.paste(&offered, &html).expect("a form on offer");
    assert_eq!(again.bytes(), b"the second paragraph as text/html");
    assert_eq!(asked.how_many(), 2);
}

/// **A form that was never offered is refused in words**, in the sentence a
/// person on this machine would read — and the application that copied it is
/// never asked, so nothing could have been converted into something it did not
/// say it could give.
#[test]
fn a_form_that_was_never_offered_is_refused_in_words() {
    let strings = what_this_machine_says();
    let mut clipboard = Clipboard::nothing_copied_yet();
    let asked = Asked::default();

    clipboard.taken(
        Offer::copied(vec![Kind::image_png()]).expect("one form is an offer"),
        copying("a drawing", &asked),
    );
    let offered = clipboard.to_paste().expect("something was copied");

    // A text field asks for text. There is an image on the clipboard and no
    // text, and the answer is a sentence rather than an image's bytes.
    let refused = clipboard
        .paste(&offered, &text())
        .expect_err("nothing offered plain text");
    assert_eq!(refused, NotPasted::NotThatForm { form: text() });

    let said = refused.said(&strings);
    assert!(!said.is_a_bug(), "the machine cannot say this: {said}");
    assert!(said.unfilled().is_empty(), "a gap was left in {said}");
    assert!(said.text().contains("as text:"), "{said}");
    assert_eq!(asked.how_many(), 0, "the copier was asked for something");

    // What it *can* be pasted as is read off the offer, so a surface can say so
    // without this crate guessing what the other application wants.
    assert_eq!(offered.forms(), [Kind::image_png()]);
}

/// **Taking the selection retires the previous owner's offer at once**, so a
/// transfer against a stale one moves nothing.
///
/// The strong half is the last assertion: the previous application is never
/// asked for anything. A clipboard that quietly served the new owner's data to
/// somebody who asked the old one is a leak between two applications, and reads
/// to everybody involved exactly like a paste.
#[test]
fn taking_the_selection_retires_the_previous_owners_offer_at_once() {
    let strings = what_this_machine_says();
    let mut clipboard = Clipboard::nothing_copied_yet();

    let first = Asked::default();
    clipboard.taken(
        Offer::copied(vec![text()]).expect("one form is an offer"),
        copying("a password", &first),
    );
    let stale = clipboard.to_paste().expect("something was copied");

    // Another application copies. The offers are identical in every visible
    // way, which is exactly the case a clipboard must not get wrong.
    let second = Asked::default();
    clipboard.taken(
        Offer::copied(vec![text()]).expect("one form is an offer"),
        copying("a shopping list", &second),
    );

    assert!(!clipboard.still_offering(&stale));
    let refused = clipboard
        .paste(&stale, &text())
        .expect_err("that selection is gone");
    assert_eq!(refused, NotPasted::CopiedOver);
    let said = refused.said(&strings);
    assert!(!said.is_a_bug(), "the machine cannot say this: {said}");
    assert!(said.text().contains("copied since"), "{said}");

    assert_eq!(
        first.how_many(),
        0,
        "the previous owner was asked for something"
    );
    assert_eq!(second.how_many(), 0, "the new owner served a stale request");

    // What the current handle pastes is the current owner's, and nothing of the
    // previous one ever appears.
    let now = clipboard.to_paste().expect("something was copied");
    let pasted = clipboard.paste(&now, &text()).expect("a form on offer");
    let read = String::from_utf8(pasted.into_bytes()).expect("this fixture writes text");
    assert!(read.starts_with("a shopping list"), "{read}");
    assert!(!read.contains("password"), "{read}");
}

/// **Pasting when nothing has been copied is *nothing to copy from*** — not the
/// thing before it, and not silence.
///
/// Both halves: a machine where nothing was ever copied, and a machine where
/// the selection was taken and then given up. They are two sentences on
/// purpose, because they send a person to two different places.
#[test]
fn pasting_when_nothing_has_been_copied_says_nothing_has_been_copied() {
    let strings = what_this_machine_says();
    let mut clipboard = Clipboard::nothing_copied_yet();

    let refused = clipboard.to_paste().expect_err("nothing has been copied");
    assert_eq!(refused, NotPasted::NothingCopied);
    let said = refused.said(&strings);
    assert!(!said.is_a_bug(), "the machine cannot say this: {said}");
    assert!(said.text().contains("nothing has been copied"), "{said}");

    // Something is copied, and then the application closes.
    let asked = Asked::default();
    clipboard.taken(
        Offer::copied(vec![text()]).expect("one form is an offer"),
        copying("a line from a document", &asked),
    );
    let offered = clipboard.to_paste().expect("something was copied");
    assert!(clipboard.given_up());

    // Somebody who was already holding the offer is told what happened to it,
    // and somebody starting a paste now is told there is nothing there. Two
    // facts, two sentences, and the owner is asked for nothing either way.
    let gone = clipboard
        .paste(&offered, &text())
        .expect_err("the owner gave it up");
    assert_eq!(gone, NotPasted::NoLongerOffered);
    assert_ne!(
        gone.said(&strings).text(),
        NotPasted::NothingCopied.said(&strings).text()
    );
    assert_eq!(clipboard.to_paste(), Err(NotPasted::NothingCopied));
    assert_eq!(asked.how_many(), 0);
}

/// **Text, images and files are three offered forms rather than three
/// mechanisms**, so the promise's three are one shape.
///
/// One application offers all three at once, and each is pasted through the
/// same call with the same refusals around it. Cut is here too, because the
/// promise names it: a cut moves once, and the second attempt at the same move
/// is refused rather than duplicating what the owner has already given up.
#[test]
fn text_images_and_files_are_three_forms_of_one_clipboard() {
    let mut clipboard = Clipboard::nothing_copied_yet();
    let asked = Asked::default();

    let every_form = vec![text(), Kind::image_png(), Kind::files()];
    assert_eq!(
        every_form.iter().map(Kind::form).collect::<Vec<Form>>(),
        [Form::Text, Form::AnImage, Form::Files]
    );

    clipboard.taken(
        Offer::copied(every_form.clone()).expect("three forms are an offer"),
        copying("what was selected", &asked),
    );
    let offered = clipboard.to_paste().expect("something was copied");

    for form in &every_form {
        let pasted = clipboard.paste(&offered, form).expect("a form on offer");
        assert_eq!(pasted.form(), form);
        assert_eq!(
            pasted.bytes(),
            format!("what was selected as {}", form.media()).as_bytes()
        );
    }
    assert_eq!(asked.forms.borrow().as_slice(), every_form.as_slice());

    // The same three, cut instead of copied: one shape, one flag, and a move
    // that cannot happen twice.
    let mut clipboard = Clipboard::nothing_copied_yet();
    let asked = Asked::default();
    clipboard.taken(
        Offer::cut(every_form).expect("three forms are an offer"),
        copying("three files", &asked),
    );
    let offered = clipboard.to_paste().expect("something was cut");
    assert_eq!(offered.taking(), Taking::Cut);

    let moved = clipboard
        .paste(&offered, &Kind::files())
        .expect("a form on offer");
    assert!(moved.the_owner_gave_it_up());
    assert_eq!(
        clipboard.paste(&offered, &Kind::files()),
        Err(NotPasted::NoLongerOffered)
    );
    assert_eq!(asked.how_many(), 1, "a move happened more than once");
}

/// **Every string this crate says is in the vocabulary `alo-saying` collects**,
/// which is the difference between a sentence a person reads and a key nobody
/// ever sees translated.
///
/// It is asked of the machine's whole vocabulary rather than of this crate's
/// own list, because that is the question the crate's own tests cannot ask: a
/// crate holding only its own strings says them whether or not anybody
/// collected it.
#[test]
fn every_string_this_crate_says_is_collected_into_the_machines_vocabulary() {
    let strings = what_this_machine_says();
    for word in EVERY_WORD {
        // Every gap the sentence declares is filled, so that what is being
        // asked here is whether the machine knows the string — an unfilled gap
        // is also a bug, and it would be this test's rather than the shell's.
        let mut filling = Filling::nothing();
        for gap in word
            .phrase()
            .expect("this crate's own words are phrases")
            .source()
            .gaps()
        {
            filling = filling.and(gap.clone(), "text");
        }
        let said = strings.say(&word.key(), &filling);
        assert!(
            !said.is_a_bug(),
            "{} is not in the vocabulary this machine holds",
            word.named()
        );
        assert_eq!(
            said.came_from(),
            &CameFrom::TheSource,
            "{} does not come from this repository's own list",
            word.named()
        );
    }
}
