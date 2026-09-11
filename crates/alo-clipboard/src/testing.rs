//! The owners, the strings and the counter this crate's own tests are written
//! against.
//!
//! Every file here has the same three things to build before it can say
//! anything — an application that owns a selection, a way to see what that
//! application was asked for, and a vocabulary — and building them from one
//! fixture is what stops four files inventing four machines that resemble each
//! other. The shape is `alo-indicator`'s and `alo-telling`'s `testing.rs`,
//! copied rather than re-decided.
//!
//! # The counter is the fixture that matters
//!
//! Several of this crate's promises are about something **not** happening: a
//! form nobody offered is refused *without the owner being asked*, and a
//! transfer against a retired offer *moves nothing*. A fixture that only
//! answered bytes could not show either — the refusal would pass whether or not
//! an application had been woken up and its answer thrown away.
//!
//! So [`Asked`] is a count that outlives the owner it belongs to. The owner is
//! dropped when the clipboard retires it, which is the point; what it was asked
//! for before that is still readable afterwards, which is what lets a test say
//! *nobody was asked* rather than *nothing came back*.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::rc::Rc;

use alo_strings::{Language, Strings, Translation, Vocabulary};

use crate::giving::{CouldNotGive, Gives};
use crate::kind::Kind;
use crate::words::{Word, clipboard_words, declare_into};

/// What an application was asked for, readable after that application is gone.
#[derive(Debug, Clone, Default)]
pub(crate) struct Asked {
    /// Every form it was asked for, in order.
    forms: Rc<RefCell<Vec<Kind>>>,
}

impl Asked {
    /// A counter that has seen nothing.
    pub(crate) fn nothing_yet() -> Self {
        Self::default()
    }

    /// Every form the owner was asked for, in the order it was asked.
    pub(crate) fn forms(&self) -> Vec<Kind> {
        self.forms.borrow().clone()
    }

    /// How many times it was asked for anything.
    pub(crate) fn how_many(&self) -> usize {
        self.forms.borrow().len()
    }
}

/// An application that owns a selection and can produce every form it is asked
/// for.
pub(crate) struct AnOwner {
    /// What it was asked for, shared with whoever is watching.
    asked: Asked,
}

impl AnOwner {
    /// What this owner produces for a form: the form's own name, so that a test
    /// asserting *what is pasted is what was copied* is asserting about
    /// something that could have come out differently.
    pub(crate) fn gives(form: &Kind) -> Vec<u8> {
        format!("what was copied, as {}", form.media()).into_bytes()
    }
}

impl Gives for AnOwner {
    fn give(&mut self, form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
        self.asked.forms.borrow_mut().push(form.clone());
        Ok(Self::gives(form))
    }
}

/// An application that owns a selection, watched by this counter.
pub(crate) fn an_owner(asked: &Asked) -> Box<dyn Gives> {
    Box::new(AnOwner {
        asked: asked.clone(),
    })
}

/// An application that offered a form and then could not produce it — the one
/// that quit between the copy and the paste.
pub(crate) fn nothing_to_give() -> Box<dyn Gives> {
    struct Gone;
    impl Gives for Gone {
        fn give(&mut self, _form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
            Err(CouldNotGive::TheApplicationDidNot)
        }
    }
    Box::new(Gone)
}

/// This crate's own words, with nothing translated: what a machine that has no
/// translations of them shows, which is what most of these tests are about.
pub(crate) fn in_english() -> Strings {
    Strings::of(clipboard_words().unwrap())
}

/// The same, with some of them translated into German and German preferred —
/// German because it is the language the rest of this repository tests
/// translation with, so a translator's file exercised here looks like the one
/// exercised everywhere else.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).unwrap();
    let mut german = Translation::into_language(german_language());
    for (word, says) in words {
        german = german.says(word.key(), *says);
    }
    let speaking = vocabulary.check(german).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german_language()]);
    strings
}

/// German, as `alo-strings` names a language.
fn german_language() -> Language {
    Language::written("de").unwrap()
}
