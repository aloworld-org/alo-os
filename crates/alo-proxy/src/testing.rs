//! The strings and the stand-ins this crate's own tests are written against.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! [`NeverAsked`] is the evaluator a test hands over when the answer must not
//! depend on one — it panics if anything asks it, which is how *the script was
//! not consulted* is a fact rather than an assumption. [`Answering`] is the one
//! that answers, and writes down what it was asked, so a test can say what did
//! and did not reach a network's own script.
//!
//! [`NeverKept`] and [`Keeping`] are the same pair for the machine's own
//! passwords: one panics if a road asks it for a credential, which is how *this
//! road never reached the store* is a fact; the other answers, or refuses in
//! whichever way is being tested, and writes down what it was asked for.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::path::PathBuf;

use alo_strings::{Language, Strings, Translation, Vocabulary};

use crate::automatic::{ConfigurationAddress, NotEvaluated, TheEvaluator};
use crate::password::{Password, WhereThePasswordIs};
use crate::signing_in::{NotSignedIn, WhereThePasswordsAre};
use crate::words::{Word, declare_into};

/// Everything this crate says.
fn its_own() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// Nothing translated: what a machine with no translations shows.
pub(crate) fn in_english() -> Strings {
    Strings::of(its_own())
}

/// Some of these words translated into German, and German preferred.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = its_own();
    let german = Language::written("de").unwrap();
    let mut translation = Translation::into_language(german.clone());
    for (word, says) in words {
        translation = translation.says(word.key(), *says);
    }
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german]);
    strings
}

/// An evaluator nothing may ask.
///
/// Handed over wherever a test says *this answer did not come from a script*.
/// A decision that quietly consulted one would pass against a permissive
/// stand-in and fail nowhere.
pub(crate) struct NeverAsked;

impl TheEvaluator for NeverAsked {
    fn asked(
        &self,
        _: &ConfigurationAddress,
        address: &str,
        _: &str,
    ) -> Result<String, NotEvaluated> {
        panic!("nothing should have been asked about {address}")
    }
}

/// An evaluator that answers, and remembers what it was asked.
pub(crate) struct Answering {
    /// What it answers, or why it will not.
    answer: Result<String, NotEvaluated>,
    /// Every question it was asked: where the configuration is, the address,
    /// and the host.
    asked: RefCell<Vec<(String, String, String)>>,
}

impl Answering {
    /// An evaluator that answers this.
    pub(crate) fn saying(answer: &str) -> Self {
        Self {
            answer: Ok(answer.to_owned()),
            asked: RefCell::new(Vec::new()),
        }
    }

    /// An evaluator that will not answer, for this reason.
    pub(crate) fn refusing(why: NotEvaluated) -> Self {
        Self {
            answer: Err(why),
            asked: RefCell::new(Vec::new()),
        }
    }

    /// Everything it was asked about.
    pub(crate) fn asked_about(&self) -> Vec<(String, String, String)> {
        self.asked.borrow().clone()
    }
}

impl TheEvaluator for Answering {
    fn asked(
        &self,
        at: &ConfigurationAddress,
        address: &str,
        host: &str,
    ) -> Result<String, NotEvaluated> {
        self.asked
            .borrow_mut()
            .push((at.as_str().to_owned(), address.to_owned(), host.to_owned()));
        self.answer.clone()
    }
}

/// A directory of this test's own, empty, on the machine the tests run on.
///
/// Named for the test rather than made at random, so that a failure leaves
/// something a person can look at, and emptied first so that one run never
/// reads what the last one wrote.
pub(crate) fn a_directory_of_its_own(named: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("alo-proxy-{named}"));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

/// A store nothing may ask.
///
/// Handed over wherever a test says *this road never reached for a credential*.
/// A road that quietly looked one up would pass against a permissive stand-in
/// and fail nowhere.
pub(crate) struct NeverKept;

impl WhereThePasswordsAre for NeverKept {
    fn password(&self, kept: &WhereThePasswordIs) -> Result<Password, NotSignedIn> {
        panic!(
            "nothing should have been asked for the password at {}",
            kept.as_str()
        )
    }
}

/// A store that answers, and remembers what it was asked for.
///
/// **No `Debug`**, on purpose: it holds a password for the length of a test,
/// and a fixture that can be formatted is the first place a credential reaches
/// a log.
pub(crate) struct Keeping {
    /// The name it has a password under, and the password.
    holding: Option<(String, String)>,
    /// Why it will not answer, where it will not.
    refusing: Option<NotSignedIn>,
    /// Every name it was asked for.
    asked: RefCell<Vec<String>>,
}

impl Keeping {
    /// A store holding this password under this name.
    pub(crate) fn of(named: &str, password: &str) -> Self {
        Self {
            holding: Some((named.to_owned(), password.to_owned())),
            refusing: None,
            asked: RefCell::new(Vec::new()),
        }
    }

    /// A store that refuses every name, this way.
    pub(crate) fn refusing(why: NotSignedIn) -> Self {
        Self {
            holding: None,
            refusing: Some(why),
            asked: RefCell::new(Vec::new()),
        }
    }

    /// Every name it was asked for.
    pub(crate) fn asked_for(&self) -> Vec<String> {
        self.asked.borrow().clone()
    }
}

impl WhereThePasswordsAre for Keeping {
    fn password(&self, kept: &WhereThePasswordIs) -> Result<Password, NotSignedIn> {
        self.asked.borrow_mut().push(kept.as_str().to_owned());
        if let Some(why) = self.refusing {
            return Err(why);
        }
        match &self.holding {
            Some((named, password)) if named == kept.as_str() => {
                Ok(Password::typed(password).unwrap())
            }
            _ => Err(NotSignedIn::NotKept),
        }
    }
}
