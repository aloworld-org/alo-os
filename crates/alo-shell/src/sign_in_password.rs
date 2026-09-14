//! The password as it is being typed, and the only place in this crate that
//! holds one.
//!
//! # What this type refuses to be
//!
//! **It has no `Debug`, no `Display`, no `Clone` and no `PartialEq`.** Each of
//! those is a road out: a `Debug` reaches a panic message or a trace, a
//! `Clone` is a second copy nobody zeroes, and an equality is a comparison
//! that belongs to `alo-accounts` and nowhere else. `tests/sign_in_source.rs`
//! reads this file for all four rather than trusting this paragraph.
//!
//! **Its bytes are overwritten before they are let go.** Every road that
//! shortens the buffer — erasing a letter, forgetting after a sign-in, the
//! screen being dropped — writes zeros over what was there first. The buffer
//! is reserved once at its bound and typing past the bound is refused, so the
//! allocation never moves and never leaves an unzeroed copy behind in memory
//! the allocator has taken back.
//!
//! **It is lent, never handed over.** `TypedPassword::as_str` borrows it for
//! the one call that needs it — `alo_greeting::Greeting::signs_in` — and there
//! is no method that returns an owned copy.

/// How many bytes of password this screen accepts.
///
/// A bound rather than a policy about what a password is worth, which is
/// `alo-accounts`' and not this crate's: it exists so that the buffer can be
/// reserved once and never reallocated, and so that a key held down forever is
/// not an allocation that grows forever. Far above anything a person types.
pub const PASSWORD_BYTES: usize = 1024;

/// A password being typed. See this module's header for what it will not do.
pub(crate) struct TypedPassword {
    /// Reserved at [`PASSWORD_BYTES`] and never grown past it.
    typed: String,
}

impl TypedPassword {
    /// Nothing typed yet, with the whole bound reserved.
    pub(crate) fn empty() -> Self {
        Self {
            typed: String::with_capacity(PASSWORD_BYTES),
        }
    }

    /// One more letter, or `false` when it would not fit in the bound.
    pub(crate) fn typed(&mut self, letter: char) -> bool {
        if self.typed.len() + letter.len_utf8() > PASSWORD_BYTES {
            return false;
        }
        self.typed.push(letter);
        true
    }

    /// Take the last letter back, zeroing the bytes it occupied.
    pub(crate) fn erased(&mut self) {
        let keep = self
            .typed
            .char_indices()
            .next_back()
            .map_or(0, |(start, _)| start);
        self.shortened_to(keep);
    }

    /// Forget everything typed, zeroing it first.
    pub(crate) fn forgotten(&mut self) {
        self.shortened_to(0);
    }

    /// Whether anything is typed. The only fact about the password a screen
    /// may draw, and it is not its length.
    pub(crate) fn is_empty(&self) -> bool {
        self.typed.is_empty()
    }

    /// Lent for the one call that checks it.
    pub(crate) fn as_str(&self) -> &str {
        &self.typed
    }

    /// Zero everything from `keep` on and shorten to it, in the same
    /// allocation.
    fn shortened_to(&mut self, keep: usize) {
        let mut bytes = std::mem::take(&mut self.typed).into_bytes();
        zeroed_from(&mut bytes, keep);
        bytes.truncate(keep);
        self.typed = match String::from_utf8(bytes) {
            Ok(typed) => typed,
            // `keep` is always a letter boundary, so this cannot happen; if it
            // ever did, what is left is zeroed and the password is forgotten
            // rather than kept in a shape nobody checked.
            Err(refused) => {
                let mut bytes = refused.into_bytes();
                bytes.fill(0);
                String::with_capacity(PASSWORD_BYTES)
            }
        };
    }
}

/// Overwrite every byte from `keep` on with zero.
///
/// Its own function so that the zeroing is tested on the bytes themselves,
/// before a truncation puts them where safe code cannot read them.
fn zeroed_from(bytes: &mut [u8], keep: usize) {
    if let Some(tail) = bytes.get_mut(keep..) {
        tail.fill(0);
    }
}

impl Drop for TypedPassword {
    /// A screen that goes away takes nothing typed at it into freed memory.
    fn drop(&mut self) {
        self.forgotten();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Erasing takes back a whole letter**, however many bytes it was.
    #[test]
    fn erasing_takes_back_a_whole_letter() {
        let mut password = TypedPassword::empty();
        for letter in "aé€".chars() {
            assert!(password.typed(letter));
        }
        password.erased();
        assert_eq!(password.as_str(), "aé");
        password.erased();
        password.erased();
        assert!(password.is_empty());
        password.erased();
        assert!(password.is_empty());
    }

    /// **Nothing past the bound is taken**, so the reserved buffer is never
    /// reallocated into a second copy.
    #[test]
    fn typing_past_the_bound_is_refused_and_the_buffer_never_moves() {
        let mut password = TypedPassword::empty();
        let reserved = password.typed.capacity();
        for _ in 0..PASSWORD_BYTES {
            assert!(password.typed('x'));
        }
        assert!(!password.typed('x'), "a letter past the bound was taken");
        assert!(!password.typed('é'));
        assert_eq!(password.typed.capacity(), reserved);
        assert_eq!(password.as_str().len(), PASSWORD_BYTES);
    }

    /// **What is taken back is zeroed first**: the letters erased, and the
    /// whole of it when it is forgotten. Read on the bytes, because after the
    /// truncation that follows they are memory safe code cannot look at.
    #[test]
    fn what_is_taken_back_is_zeroed_before_it_is_let_go() {
        let mut erased = "a€".as_bytes().to_vec();
        zeroed_from(&mut erased, 1);
        assert_eq!(erased, [b'a', 0, 0, 0]);

        let mut forgotten = b"hunter2".to_vec();
        zeroed_from(&mut forgotten, 0);
        assert!(forgotten.iter().all(|byte| *byte == 0));

        let mut nothing = b"kept".to_vec();
        zeroed_from(&mut nothing, 99);
        assert_eq!(nothing, b"kept");
    }

    /// **Forgetting leaves nothing, in the same allocation**, so the next
    /// attempt is typed into the zeroed buffer rather than a fresh one.
    #[test]
    fn forgetting_keeps_the_reserved_buffer() {
        let mut password = TypedPassword::empty();
        for letter in "hunter2".chars() {
            assert!(password.typed(letter));
        }
        let reserved = password.typed.capacity();
        password.forgotten();
        assert!(password.is_empty());
        assert_eq!(password.typed.capacity(), reserved);
    }
}
