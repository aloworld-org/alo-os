//! The first account's sheets: the name, a secret typed twice, and the road
//! from the PIN to a password and back — refusals as carefully as the way on.

use super::*;

/// Type `words` into the field the keyboard is in.
fn typed(account: &mut TheFirstAccount, words: &str) {
    for letter in words.chars() {
        account.pressed(SignInKey::Letter(letter));
    }
}

/// **A name, then a PIN typed the same twice, makes the account.**
#[test]
fn a_name_and_a_pin_typed_twice_make_the_account() {
    let mut account = TheFirstAccount::arriving();
    typed(&mut account, "anna");
    assert_eq!(account.pressed(SignInKey::Enter), Went::Stayed);
    assert_eq!(account.shown().sheet, TheAccountSheet::Secret(ASecret::Pin));
    typed(&mut account, "4821");
    account.pressed(SignInKey::Enter);
    assert_eq!(account.shown().on, OnTheSheet::Field(1));
    typed(&mut account, "4821");
    assert_eq!(account.pressed(SignInKey::Enter), Went::MakeIt);
    assert_eq!((account.name(), account.secret()), ("anna", "4821"));
}

/// **Two that differ make nothing**: the second is cleared, the first kept,
/// and the sheet says they differed.
#[test]
fn two_different_secrets_make_no_account() {
    let mut account = TheFirstAccount::arriving();
    typed(&mut account, "anna");
    account.pressed(SignInKey::Enter);
    typed(&mut account, "4821");
    account.pressed(SignInKey::Enter);
    typed(&mut account, "4812");
    assert_eq!(account.pressed(SignInKey::Enter), Went::Stayed);
    assert!(account.shown().they_differed);
    assert_eq!(account.typed(), (true, false));
    assert_eq!(account.shown().on, OnTheSheet::Field(1));
    // Typing again stops saying so.
    typed(&mut account, "4");
    assert!(!account.shown().they_differed);
}

/// **Enter on the name goes on, as it always has** — the account store
/// refuses a name nobody typed, in its own words.
#[test]
fn enter_on_the_name_goes_on_to_the_secret() {
    let mut account = TheFirstAccount::arriving();
    account.pressed(SignInKey::Enter);
    assert_eq!(account.shown().sheet, TheAccountSheet::Secret(ASecret::Pin));
    assert_eq!(account.shown().on, OnTheSheet::Field(0));
}

/// **Use password only asks for a password**, with the PIN forgotten and the
/// name kept; and the password's Back returns to the PIN.
#[test]
fn skipping_the_pin_asks_for_a_password_and_back_returns() {
    let mut account = TheFirstAccount::arriving();
    typed(&mut account, "anna");
    account.pressed(SignInKey::Enter);
    typed(&mut account, "48");
    assert_eq!(account.way_back(), Went::Stayed);
    assert_eq!(
        account.shown().sheet,
        TheAccountSheet::Secret(ASecret::Password)
    );
    assert_eq!(account.typed(), (false, false), "the PIN is forgotten");
    assert_eq!(account.name(), "anna", "the name is kept");
    typed(&mut account, "correct horse");
    account.pressed(SignInKey::Enter);
    typed(&mut account, "correct horse");
    assert_eq!(account.way_on(), Went::MakeIt);
    assert_eq!(account.secret(), "correct horse");

    let mut back = TheFirstAccount::arriving();
    typed(&mut back, "anna");
    back.pressed(SignInKey::Enter);
    back.way_back();
    back.way_back();
    assert_eq!(back.shown().sheet, TheAccountSheet::Secret(ASecret::Pin));
}

/// **Back on the name returns to the first start**, and nothing else does.
#[test]
fn back_on_the_name_returns_to_the_first_start() {
    let mut account = TheFirstAccount::arriving();
    typed(&mut account, "anna");
    assert_eq!(account.way_back(), Went::BackToTheFirstStart);
}

/// **The keyboard reaches both buttons**: Tab walks the fields, then the two
/// buttons, then round, and Enter on the second presses it.
#[test]
fn tab_reaches_both_buttons_and_enter_presses_them() {
    let mut account = TheFirstAccount::arriving();
    typed(&mut account, "anna");
    account.pressed(SignInKey::Enter);
    account.pressed(SignInKey::OtherField);
    account.pressed(SignInKey::OtherField);
    assert_eq!(account.shown().on, OnTheSheet::WayOn);
    account.pressed(SignInKey::OtherField);
    assert_eq!(account.shown().on, OnTheSheet::WayBack);
    assert_eq!(account.pressed(SignInKey::Enter), Went::Stayed);
    assert_eq!(
        account.shown().sheet,
        TheAccountSheet::Secret(ASecret::Password)
    );
    account.pressed(SignInKey::Earlier);
    assert_eq!(account.shown().on, OnTheSheet::WayBack);
}

/// **A letter on a button types nothing**, and Escape on the secret's sheet
/// clears the secrets and keeps the name.
#[test]
fn a_letter_on_a_button_types_nothing_and_escape_keeps_the_name() {
    let mut account = TheFirstAccount::arriving();
    typed(&mut account, "anna");
    account.pressed(SignInKey::Enter);
    typed(&mut account, "48");
    account.pressed(SignInKey::OtherField);
    account.pressed(SignInKey::OtherField);
    typed(&mut account, "x");
    assert_eq!(account.typed(), (true, false));
    account.pressed(SignInKey::Clear);
    assert_eq!(account.typed(), (false, false));
    assert_eq!(account.name(), "anna");
}
