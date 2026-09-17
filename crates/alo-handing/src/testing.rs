//! The windows, the offers and the counter this crate's own tests are written
//! against.
//!
//! The shape is `alo-clipboard`'s `testing.rs`, copied rather than re-decided,
//! and for the same reason: several of this crate's promises are about
//! something **not** happening — a window that takes none of the forms is
//! refused *without the application being asked*, a drop that does not happen
//! wakes nobody up — and a fixture that only answered bytes could not tell the
//! difference between a refusal and an application that was asked and had its
//! answer thrown away.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use alo_clipboard::{CouldNotGive, Gives, Kind, Offer};
use alo_strings::Strings;

use crate::target::{Application, Target};
use crate::words::handing_words;

/// What the window a drag came from was asked for, readable after it is gone.
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

    /// Every form it was asked for, in the order it was asked.
    pub(crate) fn forms(&self) -> Vec<Kind> {
        self.forms.borrow().clone()
    }

    /// How many times it was asked for anything.
    pub(crate) fn how_many(&self) -> usize {
        self.forms.borrow().len()
    }
}

/// The window a drag came from, which can produce every form it is asked for.
struct AWindow {
    /// What it was asked for, shared with whoever is watching.
    asked: Asked,
}

impl Gives for AWindow {
    fn give(&mut self, form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
        self.asked.forms.borrow_mut().push(form.clone());
        Ok(gives(form))
    }
}

/// What that window produces for a form: the form's own name, so a test
/// asserting *what arrived is what was dragged* is asserting about something
/// that could have come out differently.
pub(crate) fn gives(form: &Kind) -> Vec<u8> {
    format!("what was dragged, as {}", form.media()).into_bytes()
}

/// A window a drag came from, watched by this counter.
pub(crate) fn an_application(asked: &Asked) -> Box<dyn Gives> {
    Box::new(AWindow {
        asked: asked.clone(),
    })
}

/// A window that offered a form and then could not produce it — the one that
/// quit between the pick-up and the drop.
pub(crate) fn nothing_to_give() -> Box<dyn Gives> {
    /// A window that has stopped.
    struct Gone;
    impl Gives for Gone {
        fn give(&mut self, _form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
            Err(CouldNotGive::TheApplicationDidNot)
        }
    }
    Box::new(Gone)
}

/// A copy of these forms.
pub(crate) fn copied(forms: &[Kind]) -> Offer {
    Offer::copied(forms.to_vec()).unwrap()
}

/// A move of these forms.
pub(crate) fn cut(forms: &[Kind]) -> Offer {
    Offer::cut(forms.to_vec()).unwrap()
}

/// A sandboxed window that takes these forms.
pub(crate) fn takes(id: &str, forms: &[Kind]) -> Target {
    Target::AWindow(Application::sandboxed(id, forms.to_vec()))
}

/// A moment, as seconds after the epoch — what a test means by *later*.
pub(crate) fn at(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

/// This crate's own words, with nothing translated.
pub(crate) fn in_english() -> Strings {
    Strings::of(handing_words().unwrap())
}
