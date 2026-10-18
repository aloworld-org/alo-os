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
    /// The field this entry began on, and the one forgetting returns to.
    ///
    /// A screen that draws one field must come back to **that** field after
    /// a refused attempt, or the second try goes somewhere nobody can see.
    starts_on: SignInField,
}

impl SignInEntry {
    /// Nothing typed, and the name field waiting.
    pub(crate) fn empty() -> Self {
        Self {
            name: String::new(),
            password: TypedPassword::empty(),
            field: SignInField::Name,
            starts_on: SignInField::Name,
        }
    }

    /// Nothing typed, and the **password** field waiting.
    ///
    /// **For the screen a person comes back to.** It already knows whose
    /// machine it is — it says *alo, disan* above a single field labelled
    /// `Enter your PIN` — so there is no name to type and no name field drawn.
    ///
    /// [`Self::empty`] starts on the name, which is right for the screen that
    /// asks for one and wrong here: a person typing their PIN would be filling
    /// a field that is not on the screen, and **nothing would appear**. They
    /// would press the keys and watch nothing happen.
    ///
    /// Found on 2026-10-16, on a machine booted from a kept snapshot. Two
    /// overlays took no visible keystrokes at that screen, and the screen is
    /// not at fault for the keys — they were going somewhere a person cannot
    /// see. Nobody had reached this screen before, because until that evening
    /// no machine of ours had ever been signed into twice.
    pub(crate) fn waiting_for_a_password() -> Self {
        Self {
            name: String::new(),
            password: TypedPassword::empty(),
            field: SignInField::Password,
            starts_on: SignInField::Password,
        }
    }
    /// Edit one key, answering true only when credentials should be submitted.
    pub(crate) fn pressed(&mut self, key: crate::SignInKey) -> bool {
        match key {
            crate::SignInKey::Letter(letter) => self.typed(letter),
            crate::SignInKey::Erase => self.erased(),
            crate::SignInKey::OtherField => self.other_field(),
            crate::SignInKey::Enter if self.field == SignInField::Name => self.other_field(),
            crate::SignInKey::Enter => return true,
            // Escape clears what was typed (`alo_access::leaving`), which is
            // the same forgetting that follows a sign-in: the password first,
            // the name with it, and the name field waiting again.
            crate::SignInKey::Clear => self.forgotten(),
            // **An arrow does nothing to a field, and says so by name.** The
            // two fields have no list to move through; the language sheet
            // does, and routes these before an entry ever sees them. An arm
            // each rather than a catch-all, because this match is where the
            // compiler tells whoever adds the next key that there is a second
            // screen to think about.
            crate::SignInKey::Earlier | crate::SignInKey::Later => {}
            crate::SignInKey::Nothing => {}
        }
        false
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

    /// Everything typed, forgotten, and **the field this entry started on**
    /// waiting again.
    ///
    /// Called straight after every sign-in, whatever it answered. The name
    /// goes too: a screen that kept a refused name on it would hand the next
    /// person at this machine a hint about the last.
    ///
    /// **It returned to the name unconditionally until 2026-10-16**, which
    /// put the same fault one wrong password later than the one this type
    /// was changed to fix: a person at the screen they come back to would
    /// type their PIN once into the field they can see, mistype it, and have
    /// every attempt after that go into a name field their screen does not
    /// draw. They could never sign in again.
    pub(crate) fn forgotten(&mut self) {
        self.name.clear();
        self.password.forgotten();
        self.field = self.starts_on;
    }

    /// Forget the secret and **keep the name**.
    ///
    /// For the first account on a machine, and for nothing else. The sibling
    /// above clears the name too, and its reason is a real one — *a screen
    /// that kept a refused name on it would hand the next person at this
    /// machine a hint about the last*. **On this road there is no last
    /// person**: `Greeting::makes_an_account` refuses outright when the
    /// accounts store is not empty, so a name on this sheet can only be the
    /// name of whoever is standing there typing it.
    ///
    /// What clearing it costs was measured on a booted machine on 2026-10-18.
    /// Enter on the name field moves to the secret; a second Enter submits an
    /// empty one; the refusal — *an account needs a password, and an empty one
    /// is not one* — is correct, and it arrived **with the name field wiped**.
    /// A person's first act on their own machine was to type their name twice
    /// because they pressed Enter once too often.
    ///
    /// The secret still goes, here as everywhere, for the reason the caller
    /// gives: a refusal that left a password in memory would be a refusal that
    /// kept it.
    pub(crate) fn the_secret_is_forgotten(&mut self) {
        self.password.forgotten();
        self.field = self.starts_on;
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

    /// **A wrong password does not lock a returning person out of their own
    /// screen.**
    ///
    /// Forgetting runs after every attempt, refused or not. It returned to the
    /// **name** unconditionally until 2026-10-16, which put the fault this
    /// type was changed to fix one wrong password later: the first PIN would
    /// go into the field a person can see, and every attempt after it into a
    /// name field their screen does not draw. They could never sign in again.
    ///
    /// This is the second face of the same fault and the reason the entry
    /// remembers where it began rather than assuming.
    #[test]
    fn a_refused_attempt_leaves_a_returning_person_on_the_field_they_can_see() {
        let mut returning = SignInEntry::waiting_for_a_password();
        returning.typed('1');
        returning.forgotten();

        assert_eq!(
            returning.field(),
            SignInField::Password,
            "a second try would go into a field this screen does not draw"
        );

        returning.typed('2');
        assert!(returning.has_password(), "the second try went nowhere");
        assert_eq!(returning.name(), "", "the second try reached the name");
    }

    /// And the screen that asks for a name comes back to the name, which is
    /// what it draws first.
    #[test]
    fn a_refused_attempt_leaves_the_account_screen_on_its_name() {
        let mut making_one = SignInEntry::empty();
        making_one.typed('a');
        making_one.other_field();
        making_one.typed('b');
        making_one.forgotten();

        assert_eq!(making_one.field(), SignInField::Name);
        assert_eq!(making_one.name(), "");
        assert!(!making_one.has_password());
    }

    /// **A returning person's keys go into the field they can see.**
    ///
    /// The screen a person comes back to draws **one** field, `Enter your
    /// PIN`, above a name it already knows — it says *alo, disan*. There is no
    /// name field on it.
    ///
    /// Starting on the name would send every keystroke into a field that is
    /// not drawn: a person types their PIN and **watches nothing happen**.
    /// That is what two booted machines did on 2026-10-16, and nobody had
    /// found it because no machine of ours had ever been signed into twice.
    ///
    /// The account screen is the other way round and this asserts both, since
    /// a fix that sent every screen to the password field would break the one
    /// that asks for a name.
    #[test]
    fn a_returning_person_types_into_the_only_field_their_screen_draws() {
        let returning = SignInEntry::waiting_for_a_password();
        assert_eq!(
            returning.field(),
            SignInField::Password,
            "a returning person's keys would go somewhere their screen does not draw"
        );

        let making_one = SignInEntry::empty();
        assert_eq!(
            making_one.field(),
            SignInField::Name,
            "the screen that asks for a name must still start on it"
        );
    }

    /// And a letter really lands there, rather than the field merely being
    /// named — which is the difference between a state and a behaviour.
    #[test]
    fn what_a_returning_person_types_reaches_the_password() {
        let mut returning = SignInEntry::waiting_for_a_password();
        returning.typed('1');
        returning.typed('2');

        assert!(
            returning.has_password(),
            "two keys went nowhere a returning person could see"
        );
        assert_eq!(
            returning.name(),
            "",
            "a returning person's PIN was typed into the name they never see"
        );
    }
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
