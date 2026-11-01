//! The first account on a machine, as the sheets after the first start ask for
//! it: a name, then a PIN typed twice — or, for a person who would rather not
//! have a PIN, a password typed twice.
//!
//! # Every account has a secret
//!
//! **The owner's answer of 2026-11-01**: *skipping the PIN asks for a password
//! instead, so every account has a secret.* The PIN sheet's way back, *Use
//! password only*, pointed at nothing until then; it now turns the same two
//! fields into a password's, and the password sheet's *Back* turns them back.
//! Neither road makes an account without a secret, and neither drops the name.
//!
//! # Typed twice, and compared here
//!
//! The approved PIN sheet (`562:508`) asks for the PIN and then *Confirm PIN*.
//! Two that differ make no account: the second is cleared and the sheet says
//! so, and the first is kept so the person retypes only the one that was
//! wrong. Nothing about either is ever drawn but whether it is empty.
//!
//! # The keyboard reaches everything a pointer does
//!
//! Tab walks the sheet's fields and then its two buttons, and Enter on a button
//! presses it. Enter in a field goes on to the next field, and from the last
//! field does what the sheet's first button does.
//!
//! **No `Debug`**, because it holds `crate::sign_in_password::TypedPassword`s.

use crate::sign_in_keys::SignInKey;
use crate::sign_in_password::TypedPassword;

/// How many bytes of name this sheet accepts — the sign-in screen's bound.
const NAME_BYTES: usize = crate::sign_in_entry::NAME_BYTES;

/// Which secret the person is choosing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ASecret {
    /// A PIN: *Add a PIN?*, the sheet the first start offers.
    Pin,
    /// A password, for a person who chose *Use password only*.
    Password,
}

/// Which sheet is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheAccountSheet {
    /// *What should we call you?*
    Name,
    /// The secret, typed twice.
    Secret(ASecret),
}

/// Where the keyboard is on the sheet showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnTheSheet {
    /// A field, by its place on the sheet: the name, or the secret then again.
    Field(usize),
    /// The first button: *Continue*, *Save PIN*, *Save password*.
    WayOn,
    /// The second: *Back*, or *Use password only*.
    WayBack,
}

/// What a key did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Went {
    /// The sheets are still up.
    Stayed,
    /// Back past the name, to the first start's overview.
    BackToTheFirstStart,
    /// The name and a secret typed the same twice: make the account.
    MakeIt,
}

/// The first account as a sheet draws it: nothing about a secret but whether
/// it is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheAccountShown<'a> {
    /// Which sheet is showing.
    pub sheet: TheAccountSheet,
    /// Where the keyboard is.
    pub on: OnTheSheet,
    /// The name as typed.
    pub name: &'a str,
    /// Whether the secret is typed, and typed again.
    pub typed: (bool, bool),
    /// Whether the last try typed two different secrets.
    pub they_differed: bool,
}

/// The first account, being asked for.
pub(crate) struct TheFirstAccount {
    /// The name as typed.
    name: String,
    /// The secret as typed.
    secret: TypedPassword,
    /// The secret typed again.
    again: TypedPassword,
    /// Which sheet is showing.
    sheet: TheAccountSheet,
    /// Where the keyboard is on it.
    on: OnTheSheet,
    /// Whether the last try typed two different secrets.
    they_differed: bool,
}

impl TheFirstAccount {
    /// Nothing typed, on the name sheet, the keyboard in its field.
    pub(crate) fn arriving() -> Self {
        Self {
            name: String::new(),
            secret: TypedPassword::empty(),
            again: TypedPassword::empty(),
            sheet: TheAccountSheet::Name,
            on: OnTheSheet::Field(0),
            they_differed: false,
        }
    }

    /// The account as a sheet draws it.
    pub(crate) fn shown(&self) -> TheAccountShown<'_> {
        TheAccountShown {
            sheet: self.sheet,
            on: self.on,
            name: &self.name,
            typed: self.typed(),
            they_differed: self.they_differed,
        }
    }

    /// Which sheet is showing.
    pub(crate) const fn sheet(&self) -> TheAccountSheet {
        self.sheet
    }

    /// Where the keyboard is.
    pub(crate) const fn on(&self) -> OnTheSheet {
        self.on
    }

    /// The name as typed.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// The secret, lent for the one call that makes the account.
    pub(crate) fn secret(&self) -> &str {
        self.secret.as_str()
    }

    /// Whether anything is typed in the secret, and in it again — the one
    /// fact about either that is drawn.
    pub(crate) fn typed(&self) -> (bool, bool) {
        (!self.secret.is_empty(), !self.again.is_empty())
    }

    /// Whether the last try typed two different secrets.
    pub(crate) const fn they_differed(&self) -> bool {
        self.they_differed
    }

    /// How many fields the sheet showing has.
    const fn fields(&self) -> usize {
        match self.sheet {
            TheAccountSheet::Name => 1,
            TheAccountSheet::Secret(_) => 2,
        }
    }

    /// One key.
    pub(crate) fn pressed(&mut self, key: SignInKey) -> Went {
        match key {
            SignInKey::Letter(letter) => {
                self.typed_a_letter(letter);
                Went::Stayed
            }
            SignInKey::Erase => {
                self.erased();
                Went::Stayed
            }
            SignInKey::OtherField | SignInKey::Later => {
                self.on = self.next();
                Went::Stayed
            }
            SignInKey::Earlier => {
                self.on = self.before();
                Went::Stayed
            }
            SignInKey::Enter => self.entered(),
            // **Escape clears what is typed on this sheet**, as it does on the
            // sign-in screen, and keeps the name on the secret's sheet.
            SignInKey::Clear => {
                match self.sheet {
                    TheAccountSheet::Name => self.name.clear(),
                    TheAccountSheet::Secret(_) => self.forget_the_secrets(),
                }
                self.on = OnTheSheet::Field(0);
                Went::Stayed
            }
            SignInKey::Nothing => Went::Stayed,
        }
    }

    /// A press of the sheet's first button.
    pub(crate) fn way_on(&mut self) -> Went {
        self.on = OnTheSheet::WayOn;
        self.entered()
    }

    /// A press of the sheet's second button.
    pub(crate) fn way_back(&mut self) -> Went {
        self.on = OnTheSheet::WayBack;
        self.entered()
    }

    /// The secret is forgotten after an attempt, whatever it answered; the
    /// name stays, for `crate::sign_in_entry`'s first-account reason.
    pub(crate) fn the_attempt_is_over(&mut self) {
        self.forget_the_secrets();
        self.on = OnTheSheet::Field(0);
    }

    /// A letter into the field the keyboard is in.
    fn typed_a_letter(&mut self, letter: char) {
        self.they_differed = false;
        match (self.sheet, self.on) {
            (TheAccountSheet::Name, OnTheSheet::Field(_)) => {
                if self.name.len() + letter.len_utf8() <= NAME_BYTES {
                    self.name.push(letter);
                }
            }
            (TheAccountSheet::Secret(_), OnTheSheet::Field(0)) => {
                self.secret.typed(letter);
            }
            (TheAccountSheet::Secret(_), OnTheSheet::Field(_)) => {
                self.again.typed(letter);
            }
            (_, OnTheSheet::WayOn | OnTheSheet::WayBack) => {}
        }
    }

    /// The last letter of the field the keyboard is in, taken back.
    fn erased(&mut self) {
        match (self.sheet, self.on) {
            (TheAccountSheet::Name, OnTheSheet::Field(_)) => {
                self.name.pop();
            }
            (TheAccountSheet::Secret(_), OnTheSheet::Field(0)) => self.secret.erased(),
            (TheAccountSheet::Secret(_), OnTheSheet::Field(_)) => self.again.erased(),
            (_, OnTheSheet::WayOn | OnTheSheet::WayBack) => {}
        }
    }

    /// Enter, where the keyboard is.
    fn entered(&mut self) -> Went {
        match (self.sheet, self.on) {
            // **The name goes on to the secret**, as Enter on the name always
            // has: a name nobody typed is refused by the account store in its
            // own words, on the next sheet, rather than here in none.
            (TheAccountSheet::Name, OnTheSheet::Field(_) | OnTheSheet::WayOn) => {
                self.sheet = TheAccountSheet::Secret(ASecret::Pin);
                self.on = OnTheSheet::Field(0);
                Went::Stayed
            }
            (TheAccountSheet::Name, OnTheSheet::WayBack) => Went::BackToTheFirstStart,
            (TheAccountSheet::Secret(_), OnTheSheet::Field(0)) => {
                self.on = OnTheSheet::Field(1);
                Went::Stayed
            }
            (TheAccountSheet::Secret(_), OnTheSheet::Field(_) | OnTheSheet::WayOn) => {
                if self.secret.as_str() == self.again.as_str() {
                    // Empty twice is the same twice, and still no secret: the
                    // account store refuses it in its own words.
                    Went::MakeIt
                } else {
                    self.they_differed = true;
                    self.again.forgotten();
                    self.on = OnTheSheet::Field(1);
                    Went::Stayed
                }
            }
            // **Use password only**, and the password sheet's **Back**: the
            // other kind of secret, with nothing typed in it yet.
            (TheAccountSheet::Secret(kind), OnTheSheet::WayBack) => {
                self.sheet = TheAccountSheet::Secret(match kind {
                    ASecret::Pin => ASecret::Password,
                    ASecret::Password => ASecret::Pin,
                });
                self.forget_the_secrets();
                self.on = OnTheSheet::Field(0);
                Went::Stayed
            }
        }
    }

    /// Both secrets forgotten, and the difference with them.
    fn forget_the_secrets(&mut self) {
        self.secret.forgotten();
        self.again.forgotten();
        self.they_differed = false;
    }

    /// The next place the keyboard goes: the fields in order, then the two
    /// buttons, then round.
    fn next(&self) -> OnTheSheet {
        match self.on {
            OnTheSheet::Field(it) if it + 1 < self.fields() => OnTheSheet::Field(it + 1),
            OnTheSheet::Field(_) => OnTheSheet::WayOn,
            OnTheSheet::WayOn => OnTheSheet::WayBack,
            OnTheSheet::WayBack => OnTheSheet::Field(0),
        }
    }

    /// The place before.
    fn before(&self) -> OnTheSheet {
        match self.on {
            OnTheSheet::Field(0) => OnTheSheet::WayBack,
            OnTheSheet::Field(it) => OnTheSheet::Field(it - 1),
            OnTheSheet::WayOn => OnTheSheet::Field(self.fields() - 1),
            OnTheSheet::WayBack => OnTheSheet::WayOn,
        }
    }
}

#[cfg(test)]
#[path = "the_first_account_tests.rs"]
mod tests;
