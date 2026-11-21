//! **`accessible.picture_window`**: one picture of the window of an application
//! that describes nothing, taken for one approval — the screen road's picture
//! ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md)
//! §1). A change, so at *ask each time* it is taken only after one approval of
//! *take a picture of the {application} window and have it read*, and at full
//! access it is taken at once, recorded the same way.
//!
//! The order, and why each step is where it is:
//!
//! 1. **Never across a pairing**, at any setting: a picture is taken only for
//!    the person at this machine.
//! 2. `alo_adapters::PicturingAWindow::of` — the grants asked again **now**,
//!    the application installed, no adapter of its own; then the person's two
//!    choices, read from their file **now** (`Machine::the_screen_road_now`):
//!    *Pictures and keys* on for this application, and somebody chosen to read
//!    the pictures. Refused before any window is looked at.
//! 3. The tree opened for this one picture: no password field anywhere in the
//!    application's windows, and nothing they describe in words.
//! 4. **The compositor's door**, `picture-of`: the window that has the
//!    keyboard, only once it is the named application's — alo's own copy at
//!    the default — drawn alone.
//!
//! **One approval takes one picture.** What is answered holds it so that the
//! reader the person chose can read it; **nothing here keeps it**, and nothing
//! about it but the act is written down.

use std::sync::Arc;

use alo_acting::{Pictured, ToPicture};
use alo_adapters::picture_words as words;
use alo_adapters::{NotWalked, PicturingAWindow, TreeFault};
use alo_applications::Application;
use alo_capability::{Authorised, Grants, Refused, Value};
use alo_nearby::Origin;
use alo_record::Entry;
use alo_strings::Filling;

use crate::machine::Machine;
use crate::refusing::NotDone;

/// One picture alo took of an application's window, not yet read.
///
/// **It is never written down and never kept**: it lives as long as the answer
/// that carries it to its reader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct APictureTaken {
    /// The application.
    application: String,
    /// The PNG, exactly as the compositor drew it.
    png: Arc<[u8]>,
    /// How wide, in pixels.
    wide: u32,
    /// How tall, in pixels.
    tall: u32,
    /// What the picture is to answer, where the call said.
    question: Option<String>,
}

impl APictureTaken {
    /// A picture of `application`'s window, `wide` by `tall`, as a compositor
    /// drew it. What answers a picture verb is made here from the door's answer
    /// and from nothing else; this is for a reader's own tests, which need a
    /// picture without a compositor.
    #[must_use]
    pub fn drawn(application: &str, png: &[u8], wide: u32, tall: u32) -> Self {
        Self {
            application: application.to_owned(),
            png: Arc::from(png),
            wide,
            tall,
            question: None,
        }
    }

    /// This picture, to answer `question`.
    #[must_use]
    pub fn to_answer(mut self, question: &str) -> Self {
        self.question = Some(question.to_owned());
        self
    }

    /// What the picture is to answer, where the call said.
    #[must_use]
    pub fn question(&self) -> Option<&str> {
        self.question.as_deref()
    }

    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The PNG.
    #[must_use]
    pub fn png(&self) -> &[u8] {
        &self.png
    }

    /// The PNG, shared rather than copied, for the one request that carries it
    /// to its reader.
    #[must_use]
    pub fn shared_png(&self) -> Arc<[u8]> {
        Arc::clone(&self.png)
    }

    /// How wide and tall it is, in pixels.
    #[must_use]
    pub const fn size(&self) -> (u32, u32) {
        (self.wide, self.tall)
    }
}

/// Take one approved picture.
///
/// **A picture the compositor drew is recorded as having run**; every other way
/// this ends is a refusal, written down as one, and no picture was taken.
pub(crate) fn picturing_an_applications_window(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
    origin: Option<&Origin>,
) -> (Entry, Result<APictureTaken, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refusing = |refused: Refused| {
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };

    // **Never across a pairing** — asked first, before anything is reached.
    if origin.is_some() {
        let named = match call.value(alo_adapters::fallback_verbs::APPLICATION) {
            Some(Value::Application(named)) => named.clone(),
            _ => String::new(),
        };
        let said = strings.say(
            &words::ACROSS_A_PAIRING.key(),
            &Filling::of(alo_adapters::fallback_words::APPLICATION, named),
        );
        return refusing(Refused::worded_elsewhere(call, said));
    }

    let (pictures, reading) = machine.the_screen_road_now();
    let picturing = match PicturingAWindow::of(
        authorised,
        machine.the_adapters(),
        grants,
        &machine.the_applications_installed_now(),
        &pictures,
        &reading,
        strings,
    ) {
        Ok(picturing) => picturing,
        Err(refused) => return refusing(refused),
    };
    let Ok(application) = Application::identified(picturing.application()) else {
        return refusing(picturing.refused(words::NO_PICTURE, strings));
    };

    // Opened now, for this one picture, and dropped when it ends.
    let tree = match machine.the_windows_on_the_screen() {
        Some(windows) => windows.opening(),
        None => Err(TreeFault::NotThere),
    };
    let tree = match tree {
        Ok(tree) => tree,
        Err(fault) => {
            let said = NotWalked::Tree(fault).said(picturing.application(), None, strings);
            return refusing(Refused::worded_elsewhere(call, said));
        }
    };
    if let Err(refused) = picturing.may_be_pictured(tree.as_ref(), strings) {
        return refusing(refused);
    }
    drop(tree);

    let Some(door) = machine.at_the_compositors_door() else {
        return refusing(picturing.refused(words::NO_PICTURE, strings));
    };
    let asked = ToPicture { application };
    let (answered, png) = door
        .picturing(&asked)
        .unwrap_or((Pictured::CouldNotDraw, Vec::new()));
    let (wide, tall) = match answered {
        Pictured::Picture { bytes, wide, tall } if bytes == png.len() && !png.is_empty() => {
            (wide, tall)
        }
        Pictured::NotInFront => return refusing(picturing.refused(words::NOT_IN_FRONT, strings)),
        Pictured::NotAlosCopy => return refusing(picturing.refused(words::NOT_ALOS_COPY, strings)),
        Pictured::Picture { .. } | Pictured::CouldNotDraw => {
            return refusing(picturing.refused(words::NO_PICTURE, strings));
        }
    };
    let question = match call.value(alo_adapters::fallback_verbs::QUESTION) {
        Some(Value::Text(question)) => Some(question.clone()),
        _ => None,
    };
    let taken = APictureTaken {
        application: picturing.application().to_owned(),
        png: Arc::from(png),
        wide,
        tall,
        question,
    };
    (Entry::ran(&picturing.into_authorised(), strings), Ok(taken))
}
