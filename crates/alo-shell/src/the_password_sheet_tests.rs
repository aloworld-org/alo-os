//! The password sheet's keys and what it is told, with polkit's side played by
//! a channel: one Enter sends what was typed once, Escape is a cancel, a wrong
//! password leaves the sheet up, and a question taken back takes the sheet down.

use std::sync::mpsc::{Receiver, channel};

use super::*;

/// A question for alo's own action, and the channel its answer arrives on.
fn asked(cookie: &str) -> (ToTheSheet, Receiver<FromTheSheet>) {
    let (answer, answered) = channel();
    (
        ToTheSheet::Ask(PasswordAsked {
            cookie: cookie.to_owned(),
            what_for: WhatItIsFor::ACommandAloWasAsked("dnf install -y htop".to_owned()),
            answer,
        }),
        answered,
    )
}

/// Type `these` into the sheet.
fn typed(sheet: &mut ThePasswordSheet, these: &str) {
    for letter in these.chars() {
        sheet.pressed(SignInKey::Letter(letter));
    }
}

/// What came back, as words a test can compare.
fn said(answered: &Receiver<FromTheSheet>) -> Vec<String> {
    answered
        .try_iter()
        .map(|answer| match answer {
            FromTheSheet::Typed(typed) => format!("typed {}", typed.as_str()),
            FromTheSheet::Cancelled => "cancelled".to_owned(),
        })
        .collect()
}

/// **Nothing is up until polkit asks**, and keys then do nothing at all.
#[test]
fn nothing_is_up_until_polkit_asks() {
    let mut sheet = ThePasswordSheet::none();
    assert!(!sheet.is_up());
    assert!(sheet.shows().is_none());
    typed(&mut sheet, "1234");
    sheet.pressed(SignInKey::Enter);
    assert!(!sheet.is_up());
}

/// **One Enter sends what was typed, once**, and the field is empty again at
/// once: an Enter held down sends nothing more.
#[test]
fn one_enter_sends_what_was_typed_once() {
    let mut sheet = ThePasswordSheet::none();
    let (ask, answered) = asked("c1");
    sheet.told(ask);
    assert!(sheet.is_up());
    typed(&mut sheet, "1234");
    assert!(sheet.shows().unwrap().typed_something);
    sheet.pressed(SignInKey::Enter);
    sheet.pressed(SignInKey::Enter);
    assert_eq!(said(&answered), ["typed 1234"]);
    let shows = sheet.shows().unwrap();
    assert!(!shows.typed_something, "the field forgets at once");
    assert_eq!(shows.under, TheLineUnder::Checking);
    assert_eq!(
        shows.what_for,
        &WhatItIsFor::ACommandAloWasAsked("dnf install -y htop".to_owned())
    );
}

/// **An Enter with nothing typed sends nothing**, rather than an empty password
/// polkit would count as a wrong one.
#[test]
fn an_enter_with_nothing_typed_sends_nothing() {
    let mut sheet = ThePasswordSheet::none();
    let (ask, answered) = asked("c1");
    sheet.told(ask);
    sheet.pressed(SignInKey::Enter);
    assert!(said(&answered).is_empty());
    assert!(sheet.is_up());
}

/// **A wrong password leaves the sheet up, saying so**, and the next try is
/// sent like the first.
#[test]
fn a_wrong_password_leaves_the_sheet_up() {
    let mut sheet = ThePasswordSheet::none();
    let (ask, answered) = asked("c1");
    sheet.told(ask);
    typed(&mut sheet, "4321");
    sheet.pressed(SignInKey::Enter);
    sheet.told(ToTheSheet::NotRight {
        cookie: "c1".to_owned(),
    });
    assert!(sheet.is_up());
    assert_eq!(sheet.shows().unwrap().under, TheLineUnder::NotRight);
    typed(&mut sheet, "1234");
    sheet.pressed(SignInKey::Enter);
    assert_eq!(said(&answered), ["typed 4321", "typed 1234"]);
    sheet.told(ToTheSheet::Over {
        cookie: "c1".to_owned(),
    });
    assert!(!sheet.is_up());
}

/// **A password that cannot be checked says why**, rather than *not right*.
#[test]
fn a_password_that_cannot_be_checked_says_why() {
    let mut sheet = ThePasswordSheet::none();
    let (ask, _answered) = asked("c1");
    sheet.told(ask);
    typed(&mut sheet, "1234");
    sheet.pressed(SignInKey::Enter);
    sheet.told(ToTheSheet::CannotBeChecked {
        cookie: "c1".to_owned(),
        why: WhyNotChecked::NotInStep,
    });
    assert_eq!(
        sheet.shows().unwrap().under,
        TheLineUnder::CannotBeChecked(WhyNotChecked::NotInStep)
    );
}

/// **Escape is a cancel**, said to polkit, and the sheet comes down.
#[test]
fn escape_is_a_cancel() {
    let mut sheet = ThePasswordSheet::none();
    let (ask, answered) = asked("c1");
    sheet.told(ask);
    typed(&mut sheet, "12");
    sheet.pressed(SignInKey::Clear);
    assert_eq!(said(&answered), ["cancelled"]);
    assert!(!sheet.is_up());
}

/// **A question polkit takes back takes the sheet down**, and the next one in
/// line comes up with an empty field.
#[test]
fn a_question_taken_back_takes_the_sheet_down() {
    let mut sheet = ThePasswordSheet::none();
    let (first, first_answered) = asked("c1");
    let (second, second_answered) = asked("c2");
    sheet.told(first);
    sheet.told(second);
    typed(&mut sheet, "12");
    sheet.told(ToTheSheet::Over {
        cookie: "c1".to_owned(),
    });
    assert!(sheet.is_up(), "the second question is in front now");
    assert!(!sheet.shows().unwrap().typed_something);
    assert!(said(&first_answered).is_empty());
    sheet.pressed(SignInKey::Clear);
    assert_eq!(said(&second_answered), ["cancelled"]);
    assert!(!sheet.is_up());
}

/// **A word about another question changes nothing on this one.**
#[test]
fn a_word_about_another_question_changes_nothing() {
    let mut sheet = ThePasswordSheet::none();
    let (ask, _answered) = asked("c1");
    sheet.told(ask);
    sheet.told(ToTheSheet::NotRight {
        cookie: "c9".to_owned(),
    });
    assert_eq!(sheet.shows().unwrap().under, TheLineUnder::HowToAnswer);
}

/// **A sheet that could not be drawn cancels rather than holding every key
/// invisibly.**
#[test]
fn a_sheet_that_could_not_be_drawn_cancels() {
    let mut sheet = ThePasswordSheet::none();
    let (ask, answered) = asked("c1");
    sheet.told(ask);
    sheet.could_not_be_drawn();
    assert_eq!(said(&answered), ["cancelled"]);
    assert!(!sheet.is_up());
}

/// **A question whose asker has gone is taken down**, not left holding keys.
#[test]
fn a_question_whose_asker_has_gone_is_taken_down() {
    let mut sheet = ThePasswordSheet::none();
    let (ask, answered) = asked("c1");
    drop(answered);
    sheet.told(ask);
    typed(&mut sheet, "1234");
    sheet.pressed(SignInKey::Enter);
    assert!(!sheet.is_up());
}
