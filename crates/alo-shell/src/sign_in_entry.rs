//! What has been typed at the sign-in screen: a name, a password, and which of
//! the two the next letter goes into.
//!
//! Nothing here decides anything about either. It is a keyboard's worth of
//! editing — a letter, a letter taken back, the other field — and the moment
//! the two are asked about is `crate::SignInScreen`'s, which lends them to
//! `alo_greeting::Greeting` and forgets both straight afterwards.
//!
//! **No `Debug`**, because it holds a `crate::sign_in_password::TypedPassword`
//! and a derived one would be a road for it; `tests/sign_in_source.rs` holds
//! every type that carries a password to that.

use crate::sign_in_password::TypedPassword;

/// How many bytes of name this screen accepts.
///
/// A bound for the same reason as `PASSWORD_BYTES`: a key held down forever is
/// not an allocation that grows forever, and a name longer than this is not
/// one anybody is signed in by.
pub const NAME_BYTES: usize = 256;

/// Which field the next letter goes into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignInField {
    /// The account's name. Where the screen starts.
    Name,
    /// The password.
    Password,
}

/// A name and a password being typed.
pub(crate) struct SignInEntry {
    /// The name as typed so far, drawn as it is.
    name: String,
    /// The password as typed so far, never drawn.
    password: TypedPassword,
    /// Where the next letter goes.
    field: SignInField,
}

impl SignInEntry {
    /// Nothing typed, and the name field waiting.
    pub(crate) fn empty() -> Self {
        Self {
            name: String::new(),
            password: TypedPassword::empty(),
            field: SignInField::Name,
        }
    }

    /// A letter, into whichever field is waiting. A letter past a field's
    /// bound is not taken.
    pub(crate) fn typed(&mut self, letter: char) {
        match self.field {
            SignInField::Name => {
                if self.name.len() + letter.len_utf8() <= NAME_BYTES {
                    self.name.push(letter);
                }
            }
            SignInField::Password => {
                self.password.typed(letter);
            }
        }
    }

    /// The last letter of whichever field is waiting, taken back.
    pub(crate) fn erased(&mut self) {
        match self.field {
            SignInField::Name => {
                self.name.pop();
            }
            SignInField::Password => self.password.erased(),
        }
    }

    /// The other field. Two fields, so forwards and backwards are one move.
    pub(crate) fn other_field(&mut self) {
        self.field = match self.field {
            SignInField::Name => SignInField::Password,
            SignInField::Password => SignInField::Name,
        };
    }

    /// Everything typed, forgotten, and the name field waiting again.
    ///
    /// Called straight after every sign-in, whatever it answered. The name
    /// goes too: a screen that kept a refused name on it would hand the next
    /// person at this machine a hint about the last.
    pub(crate) fn forgotten(&mut self) {
        self.name.clear();
        self.password.forgotten();
        self.field = SignInField::Name;
    }

    /// The name as typed.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// The password, lent for the one call that checks it.
    pub(crate) fn password(&self) -> &str {
        self.password.as_str()
    }

    /// Whether any password is typed — the one fact about it that is drawn.
    pub(crate) fn has_password(&self) -> bool {
        !self.password.is_empty()
    }

    /// Where the next letter goes.
    pub(crate) fn field(&self) -> SignInField {
        self.field
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Letters go into the field that is waiting**, and the other field is
    /// one move away in either direction.
    #[test]
    fn letters_go_where_the_field_is_waiting() {
        let mut entry = SignInEntry::empty();
        assert_eq!(entry.field(), SignInField::Name);
        for letter in "ada".chars() {
            entry.typed(letter);
        }
        entry.other_field();
        assert_eq!(entry.field(), SignInField::Password);
        for letter in "secret".chars() {
            entry.typed(letter);
        }
        entry.erased();
        assert_eq!(entry.name(), "ada");
        assert_eq!(entry.password(), "secre");
        assert!(entry.has_password());
        entry.other_field();
        entry.erased();
        assert_eq!(entry.name(), "ad");
        assert_eq!(entry.password(), "secre");
    }

    /// **Forgetting takes the name as well as the password**, and puts the
    /// screen back at the name.
    #[test]
    fn forgetting_takes_both_and_starts_again_at_the_name() {
        let mut entry = SignInEntry::empty();
        entry.typed('a');
        entry.other_field();
        entry.typed('b');
        entry.forgotten();
        assert_eq!(entry.name(), "");
        assert!(!entry.has_password());
        assert_eq!(entry.field(), SignInField::Name);
    }

    /// **A name past its bound is not taken**, a letter at a time or at once.
    #[test]
    fn a_name_past_its_bound_is_not_taken() {
        let mut entry = SignInEntry::empty();
        for _ in 0..NAME_BYTES + 10 {
            entry.typed('n');
        }
        assert_eq!(entry.name().len(), NAME_BYTES);
        entry.erased();
        entry.typed('é');
        assert_eq!(entry.name().len(), NAME_BYTES - 1);
    }
}
