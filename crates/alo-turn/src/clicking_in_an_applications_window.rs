//! **`accessible.click_at`**: one click at one point of the window of an
//! application that describes nothing, for one approval — the screen road's
//! click ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md),
//! amended 2026-11-17: *"Allow clicking by position"*). A change, so at *ask
//! each time* it runs only after one approval of *click {across}% across and
//! {down}% down the {application} window*, and at full access it runs at once,
//! recorded the same way. **Dragging stays the person's**: nothing here holds a
//! button.
//!
//! The order, and why each step is where it is:
//!
//! 1. **Never across a pairing**, at any setting.
//! 2. `alo_adapters::ClickingAt::of` — the grants asked again **now**, the
//!    application installed, no adapter of its own, the point inside the
//!    window, *Pictures and keys* on for this application and somebody chosen
//!    to read pictures, from the person's file **now**.
//! 3. The tree opened for this one click: no password field anywhere, and
//!    nothing the windows describe in words (then a control is pressed by its
//!    name, never by position).
//! 4. **A picture before**, through the compositor's `picture-of`, so that
//!    what the click changed can be told — and no picture, no click.
//! 5. **The compositor's door**, `click-at`: the window that has the keyboard,
//!    only once it is the named application's — alo's own copy at the default
//!    — and that client alone.
//! 6. **Read again after**: a picture after, for the reader the person chose.
//!    Where it is the same as the one before — or none could be taken — the
//!    click is recorded as having run with its outcome **not known**.
//!
//! **One approval clicks once.**

use alo_acting::{Clicked, Pictured, ToClick, ToPicture};
use alo_adapters::click_words as words;
use alo_adapters::{ClickingAt, NotWalked, TreeFault};
use alo_applications::Application;
use alo_capability::{Authorised, Grants, Refused, Value};
use alo_nearby::Origin;
use alo_record::Entry;
use alo_strings::Filling;

use crate::machine::Machine;
use crate::picturing_an_applications_window::APictureTaken;
use crate::refusing::NotDone;
use crate::the_door_the_compositor_serves::TheDoorTheCompositorServes;

/// One click alo made at a point of an application's window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AClickMade {
    /// The application.
    application: String,
    /// How far across and down, in per cent of the window.
    at: (u8, u8),
    /// The window as it was pictured after the click, not yet read — where a
    /// picture could be taken.
    after: Option<APictureTaken>,
    /// Whether the window visibly changed: `false` where the picture after is
    /// the same as the one before, or there is no picture after.
    changed: bool,
}

impl AClickMade {
    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// How far across and down, in per cent of the window.
    #[must_use]
    pub const fn at(&self) -> (u8, u8) {
        self.at
    }

    /// The picture after the click, for its reader.
    #[must_use]
    pub const fn after(&self) -> Option<&APictureTaken> {
        self.after.as_ref()
    }

    /// Whether the window visibly changed. **`false` is *not known***, never
    /// *nothing happened*: a click can do what no picture shows.
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.changed
    }
}

/// One picture of the window in front, if the door answers with one.
fn a_picture(
    door: &mut dyn TheDoorTheCompositorServes,
    application: &Application,
) -> Result<APictureTaken, Pictured> {
    let asked = ToPicture {
        application: application.clone(),
    };
    match door.picturing(&asked) {
        Ok((Pictured::Picture { bytes, wide, tall }, png)) if bytes == png.len() && bytes > 0 => {
            Ok(APictureTaken::drawn(
                application.identifier(),
                &png,
                wide,
                tall,
            ))
        }
        Ok((Pictured::Picture { .. }, _)) | Err(_) => Err(Pictured::CouldNotDraw),
        Ok((refused, _)) => Err(refused),
    }
}

/// Click once at the approved point.
///
/// **A click the compositor says it made is recorded as having run**, with its
/// outcome *not known* where the window did not visibly change; every other way
/// this ends is a refusal, written down as one, and nothing was clicked.
pub(crate) fn clicking_in_an_applications_window(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
    origin: Option<&Origin>,
) -> (Entry, Result<AClickMade, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refusing = |refused: Refused| {
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };

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
    let clicking = match ClickingAt::of(
        authorised,
        machine.the_adapters(),
        grants,
        &machine.the_applications_installed_now(),
        &pictures,
        &reading,
        strings,
    ) {
        Ok(clicking) => clicking,
        Err(refused) => return refusing(refused),
    };
    let Ok(application) = Application::identified(clicking.application()) else {
        return refusing(clicking.refused(words::NO_CLICK, strings));
    };
    let (across, down) = clicking.at();
    let per_mille = |per_cent: u8| u16::from(per_cent) * 10;
    let Some(asked) = ToClick::at(application.clone(), per_mille(across), per_mille(down)) else {
        return refusing(clicking.refused(words::NO_CLICK, strings));
    };

    let tree = match machine.the_windows_on_the_screen() {
        Some(windows) => windows.opening(),
        None => Err(TreeFault::NotThere),
    };
    let tree = match tree {
        Ok(tree) => tree,
        Err(fault) => {
            let said = NotWalked::Tree(fault).said(clicking.application(), None, strings);
            return refusing(Refused::worded_elsewhere(call, said));
        }
    };
    if let Err(refused) = clicking.may_be_clicked(tree.as_ref(), strings) {
        return refusing(refused);
    }
    drop(tree);

    let Some(door) = machine.at_the_compositors_door() else {
        return refusing(clicking.refused(words::NO_CLICK, strings));
    };
    // **A picture before, or no click**: without it, what the click did could
    // never be told.
    let before = match a_picture(door, &application) {
        Ok(before) => before,
        Err(Pictured::NotInFront) => {
            return refusing(clicking.refused(words::NOT_IN_FRONT, strings));
        }
        Err(Pictured::NotAlosCopy) => {
            return refusing(clicking.refused(words::NOT_ALOS_COPY, strings));
        }
        Err(Pictured::Picture { .. } | Pictured::CouldNotDraw) => {
            return refusing(clicking.refused(words::NO_CLICK, strings));
        }
    };
    let word = match door.clicking(&asked).unwrap_or(Clicked::CouldNotClick) {
        Clicked::Clicked => None,
        Clicked::NotInFront => Some(words::NOT_IN_FRONT),
        Clicked::NotAlosCopy => Some(words::NOT_ALOS_COPY),
        Clicked::ThePersonIsPressing => Some(words::THE_PERSON_IS_PRESSING),
        Clicked::CouldNotClick => Some(words::NO_CLICK),
    };
    if let Some(word) = word {
        return refusing(clicking.refused(word, strings));
    }
    // **Read again after**, whatever it shows.
    let after = a_picture(door, &application).ok();
    let changed = after
        .as_ref()
        .is_some_and(|after| after.png() != before.png());
    let made = AClickMade {
        application: clicking.application().to_owned(),
        at: (across, down),
        after,
        changed,
    };
    let not_known = (!changed).then(|| {
        strings.say(
            &words::NOT_KNOWN.key(),
            &Filling::of(
                alo_adapters::fallback_words::APPLICATION,
                made.application.clone(),
            ),
        )
    });
    let entry = Entry::ran(&clicking.into_authorised(), strings);
    let entry = match not_known {
        Some(said) => entry.telling(&[said]),
        None => entry,
    };
    (entry, Ok(made))
}
