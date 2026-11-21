//! **`accessible.click_at`**: one click on one described thing in the window of
//! an application that describes nothing, for one approval — the screen road's
//! click ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md),
//! amended 2026-11-17: *"Allow clicking by position"*, and 2026-11-21: *"Yes,
//! click by description"*). A change, so at *ask each time* it runs only after
//! one approval of *click "{description}" in {application}*, and at full access
//! it runs at once, recorded the same way. **Dragging stays the person's**:
//! nothing here holds a button.
//!
//! **Two halves under one approval**, because the reader the person chose is
//! asked between them, by the daemon, through the road every picture takes —
//! the indicator and the record (law 1) — and that road is not this crate's:
//!
//! 1. [`readying_a_click`], after the OK (or at once at full access):
//!    - **never across a pairing**, at any setting;
//!    - `alo_adapters::ClickingAt::of` — the grants asked again **now**, the
//!      application installed, no adapter of its own, a description given,
//!      *Pictures and keys* on for this application and somebody chosen to
//!      read pictures, from the person's file **now**;
//!    - the tree opened for this one click: no password field anywhere, and
//!      nothing the windows describe in words (then a control is pressed by
//!      its name, never by position);
//!    - **a picture before**, through the compositor's `picture-of` — the
//!      picture the reader finds the thing in — and no picture, no click.
//!
//!    What it answers is [`AThingToFind`]: what to find, and in which picture.
//!    The approval itself stays inside the turn (`AClickToFind`), so nothing
//!    outside can spend it twice or make one up.
//! 2. [`clicking_where_it_was_found`], with the reader's one line
//!    (`alo_adapters::where_a_thing_is::found`): **exactly one found** gives a
//!    point; none, more than one, an answer that is not the one line, or a
//!    reader that could not be asked each refuse in plain words, and nothing
//!    is clicked. Then the compositor's door, `click-at`, the window that has
//!    the keyboard, only once it is the named application's — alo's own copy
//!    at the default — and that client alone; then **a picture after**. Where
//!    it is the same as the one before, or none could be taken, the click is
//!    recorded as having run with its outcome **not known**.
//!
//! **One approval clicks at most once**, and every road out of either half
//! leaves exactly one entry.

use alo_acting::{Clicked, Pictured, ToClick, ToPicture};
use alo_adapters::click_words as words;
use alo_adapters::where_a_thing_is;
use alo_adapters::{ClickingAt, NotWalked, TreeFault};
use alo_applications::Application;
use alo_capability::{Authorised, Grants, Refused, Value};
use alo_nearby::Origin;
use alo_record::Entry;
use alo_strings::{Filling, Said};

use crate::machine::Machine;
use crate::picturing_an_applications_window::APictureTaken;
use crate::refusing::NotDone;
use crate::the_door_the_compositor_serves::TheDoorTheCompositorServes;

/// **What the reader is to find before a click, and the picture to find it
/// in** — what the first half answers. It carries no authority: the approval
/// waits inside the turn, and only [`crate::Turning::clicking_where_it_was_found`]
/// spends it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AThingToFind {
    /// The application, by identifier.
    application: String,
    /// The application, as a person calls it.
    named: String,
    /// The thing to click, as described.
    description: String,
    /// The window as pictured before the click.
    before: APictureTaken,
}

impl AThingToFind {
    /// `description`, to be found in `before`, a picture of `application`'s
    /// window, which a person calls `named`. What a click answers is made by
    /// the turn and nothing else; this is for a reader's own tests, and makes
    /// nothing anybody could click with.
    #[must_use]
    pub fn to_find(
        application: &str,
        named: &str,
        description: &str,
        before: APictureTaken,
    ) -> Self {
        Self {
            application: application.to_owned(),
            named: named.to_owned(),
            description: description.to_owned(),
            before,
        }
    }

    /// The application, by identifier.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The application, as a person calls it.
    #[must_use]
    pub fn named(&self) -> &str {
        &self.named
    }

    /// The thing to click, as described.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// The window as pictured before the click, for the reader.
    #[must_use]
    pub const fn before(&self) -> &APictureTaken {
        &self.before
    }
}

/// An approved click whose point is not yet found, held inside the turn.
///
/// Deliberately not `Clone`, like the [`Authorised`] inside it.
#[derive(Debug)]
pub(crate) struct AClickToFind {
    /// What may run.
    clicking: ClickingAt,
    /// The application, as the door is asked for it.
    application: Application,
    /// The window as pictured before the click.
    before: APictureTaken,
    /// Whether the moment it was decided at gave full access (ADR 0149).
    at_full_access: bool,
}

impl AClickToFind {
    /// Whether the moment it was decided at gave full access.
    pub(crate) const fn at_full_access(&self) -> bool {
        self.at_full_access
    }

    /// And it was decided at full access.
    pub(crate) const fn decided_at_full_access(mut self, at_full_access: bool) -> Self {
        self.at_full_access = at_full_access;
        self
    }
}

/// One click alo made on a described thing in an application's window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AClickMade {
    /// The application.
    application: String,
    /// The thing clicked, as described.
    description: String,
    /// Where the reader found it, in per cent of the window: across, down.
    at: (u8, u8),
    /// The window as pictured after the click, not yet read — where a
    /// picture could be taken.
    after: Option<Box<APictureTaken>>,
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

    /// The thing clicked, as described.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Where the reader found it, in per cent of the window: across, down.
    #[must_use]
    pub const fn at(&self) -> (u8, u8) {
        self.at
    }

    /// The picture after the click, for its reader.
    #[must_use]
    pub fn after(&self) -> Option<&APictureTaken> {
        self.after.as_deref()
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

/// A refusal of `refused`, written down at the authorisation's moment.
fn refusing<T>(
    authorised_at: std::time::SystemTime,
    agent: &alo_capability::Grantee,
    machine: &Machine<'_>,
    refused: Refused,
) -> Result<T, Box<(Entry, NotDone)>> {
    let entry = Entry::refused(&refused, agent, machine.strings(), authorised_at);
    Err(Box::new((entry, NotDone::Refused(refused))))
}

/// **The first half**: every check, and the picture before — or the refusal,
/// with its entry. Nothing is clicked here.
pub(crate) fn readying_a_click(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
    origin: Option<&Origin>,
) -> Result<(AClickToFind, AThingToFind), Box<(Entry, NotDone)>> {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();

    if origin.is_some() {
        let named = match call.value(alo_adapters::fallback_verbs::APPLICATION) {
            Some(Value::Application(named)) => machine.what_a_person_calls(named),
            _ => String::new(),
        };
        let said = strings.say(
            &words::ACROSS_A_PAIRING.key(),
            &Filling::of(alo_adapters::fallback_words::APPLICATION, named),
        );
        return refusing(at, &agent, machine, Refused::worded_elsewhere(call, said));
    }

    let (pictures, reading) = machine.the_screen_road_now();
    let called = |identifier: &str| machine.what_a_person_calls(identifier);
    let clicking = ClickingAt::of(
        authorised,
        machine.the_adapters(),
        grants,
        &machine.the_applications_installed_now(),
        &pictures,
        &reading,
        &called,
        strings,
    );
    let clicking = match clicking {
        Ok(clicking) => clicking,
        Err(refused) => return refusing(at, &agent, machine, refused),
    };
    let Ok(application) = Application::identified(clicking.application()) else {
        let refused = clicking.refused(words::NO_CLICK, strings);
        return refusing(at, &agent, machine, refused);
    };

    let tree = match machine.the_windows_on_the_screen() {
        Some(windows) => windows.opening(),
        None => Err(TreeFault::NotThere),
    };
    let tree = match tree {
        Ok(tree) => tree,
        Err(fault) => {
            let said = NotWalked::Tree(fault).said(clicking.named(), None, strings);
            let refused = clicking.refused_saying(said);
            return refusing(at, &agent, machine, refused);
        }
    };
    if let Err(refused) = clicking.may_be_clicked(tree.as_ref(), strings) {
        return refusing(at, &agent, machine, refused);
    }
    drop(tree);

    let Some(door) = machine.at_the_compositors_door() else {
        let refused = clicking.refused(words::NO_CLICK, strings);
        return refusing(at, &agent, machine, refused);
    };
    // **A picture before, or no click**: it is what the reader finds the thing
    // in, and what the window after is told apart from.
    let before = match a_picture(door, &application) {
        Ok(before) => before,
        Err(refused) => {
            let word = match refused {
                Pictured::NotInFront => words::NOT_IN_FRONT,
                Pictured::NotAlosCopy => words::NOT_ALOS_COPY,
                Pictured::Picture { .. } | Pictured::CouldNotDraw => words::NO_CLICK,
            };
            let refused = clicking.refused(word, strings);
            return refusing(at, &agent, machine, refused);
        }
    };
    let thing = AThingToFind {
        application: clicking.application().to_owned(),
        named: clicking.named().to_owned(),
        description: clicking.description().to_owned(),
        before: before.clone(),
    };
    let ready = AClickToFind {
        clicking,
        application,
        before,
        at_full_access: false,
    };
    Ok((ready, thing))
}

/// **The second half**: the reader's answer read, and the click made where it
/// found the thing — or refused, and nothing clicked.
///
/// `read` is the reader's one line, or the sentence saying why no reader could
/// be asked (the person's own model cannot read pictures yet).
pub(crate) fn clicking_where_it_was_found(
    machine: &mut Machine<'_>,
    ready: AClickToFind,
    read: Result<String, Said>,
) -> (Entry, Result<AClickMade, NotDone>) {
    let strings = machine.strings();
    let AClickToFind {
        clicking,
        application,
        before,
        ..
    } = ready;
    let refusing = |clicking: ClickingAt, refused: Refused| {
        let authorised = clicking.into_authorised();
        let entry = Entry::refused(&refused, authorised.under(), strings, authorised.at());
        (entry, Err(NotDone::Refused(refused)))
    };
    // **No reader could be asked** — the person's own model cannot read
    // pictures yet — so nothing can say where, and nothing is clicked.
    let read = match read {
        Ok(read) => read,
        Err(said) => {
            let refused = clicking.refused_saying(said);
            return refusing(clicking, refused);
        }
    };
    // **Exactly one found gives a point**; anything else clicks nothing.
    let (across, down) = match where_a_thing_is::found(&read) {
        Ok(point) => point,
        Err(why) => {
            let said = why.said(clicking.named(), clicking.description(), strings);
            let refused = clicking.refused_saying(said);
            return refusing(clicking, refused);
        }
    };
    let per_mille = |per_cent: u8| u16::from(per_cent) * 10;
    let Some(asked) = ToClick::at(application.clone(), per_mille(across), per_mille(down)) else {
        let refused = clicking.refused(words::NO_CLICK, strings);
        return refusing(clicking, refused);
    };
    let Some(door) = machine.at_the_compositors_door() else {
        let refused = clicking.refused(words::NO_CLICK, strings);
        return refusing(clicking, refused);
    };
    let word = match door.clicking(&asked).unwrap_or(Clicked::CouldNotClick) {
        Clicked::Clicked => None,
        Clicked::NotInFront => Some(words::NOT_IN_FRONT),
        Clicked::NotAlosCopy => Some(words::NOT_ALOS_COPY),
        Clicked::ThePersonIsPressing => Some(words::THE_PERSON_IS_PRESSING),
        Clicked::CouldNotClick => Some(words::NO_CLICK),
    };
    if let Some(word) = word {
        let refused = clicking.refused(word, strings);
        return refusing(clicking, refused);
    }
    // **Read again after**, whatever it shows.
    let after = a_picture(door, &application).ok().map(Box::new);
    let changed = after
        .as_ref()
        .is_some_and(|after| after.png() != before.png());
    let filled = Filling::of(
        alo_adapters::fallback_words::APPLICATION,
        clicking.named().to_owned(),
    )
    .and(
        where_a_thing_is::DESCRIPTION,
        clicking.description().to_owned(),
    );
    // **Where alo clicked is in the record**, as the reader found it, because
    // afterwards nobody can say what was at that place.
    let mut told = vec![
        strings.say(
            &words::CLICKED_THERE.key(),
            &filled
                .clone()
                .and("across", across.to_string())
                .and("down", down.to_string()),
        ),
    ];
    if !changed {
        told.push(strings.say(&words::NOT_KNOWN.key(), &filled));
    }
    let made = AClickMade {
        application: clicking.application().to_owned(),
        description: clicking.description().to_owned(),
        at: (across, down),
        after,
        changed,
    };
    let entry = Entry::ran(&clicking.into_authorised(), strings).telling(&told);
    (entry, Ok(made))
}

/// **A click whose point never came**: the approval was spent and nothing was
/// clicked, which is a refusal, written down as one — so a turn that ends with
/// one still waiting leaves its entry.
pub(crate) fn never_found(machine: &Machine<'_>, ready: AClickToFind) -> Entry {
    let strings = machine.strings();
    let refused = ready.clicking.refused(words::NO_CLICK, strings);
    let authorised = ready.clicking.into_authorised();
    Entry::refused(&refused, authorised.under(), strings, authorised.at())
}
