//! **alo clicking once at a point of an application's window, through a whole
//! turn** (`accessible.click_at`, ADR 0147 as amended 2026-11-17: *"Allow
//! clicking by position"*), written from the promise by a second author and not
//! from the code (law 7): a game whose window describes nothing, a tree that
//! counts every time it is opened, a door that keeps every picture and every
//! click it is asked for **in the order it was asked**, and the person's two
//! choices held where a test can turn them while a click waits.
//!
//! - with no invocation nothing is opened, pictured or clicked;
//! - at *ask each time* a click waits for one approval, one approval clicks
//!   once, and a declined one clicks nothing;
//! - at full access it clicks at once, once;
//! - the point is given in whole per cent from 0 to 100, reaches the door as
//!   thousandths of the window, and anything outside the window is refused at
//!   the call;
//! - no click unless *Pictures and keys* is on for **that** application and
//!   somebody was chosen to read pictures — read at the moment of the act, so a
//!   switch turned off while a click waits stops it;
//! - a window that describes itself is pressed by name, never clicked by
//!   position; a password field in any of the application's windows is never
//!   clicked at any setting; an application not on the accessibility bus goes
//!   on to the compositor; a tree that cannot be opened is no click;
//! - in every refusal before the door, the door is never asked anything;
//! - a picture is taken before the click and again after it; a picture before
//!   that is refused clicks nothing;
//! - what the door answers to the click is held to: its four refusals are
//!   refusals in the person's words;
//! - where the picture after is the same as the one before, or there is no
//!   picture after, what the click did is recorded as *not known*;
//! - a call from a paired machine is refused;
//! - every click and every refusal leaves exactly one entry, and no entry holds
//!   a picture.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, VecDeque};
use std::path::Path;
use std::rc::Rc;

use alo_acting::{Clicked, Pictured, ToClick, ToPicture};
use alo_adapters::click_words::{
    A_PASSWORD_FIELD, IT_DESCRIBES_ITSELF, NO_CLICK, NO_ONE_READS, NOT_ALOS_COPY, NOT_IN_FRONT,
    NOT_KNOWN, NOT_SWITCHED_ON, THE_PERSON_IS_PRESSING,
};
use alo_adapters::fallback_verbs::{ACROSS, APPLICATION, DOWN};
use alo_adapters::{AccessibilityTree, CLICK_AT, Facts, NodeAt, Running, TreeFault, WorkingFully};
use alo_applications::{Application, Installed};
use alo_capability::{Given, Grant, Grantee, Grants, PicturesAndKeys, Reach, WhoReadsPictures};
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_record::{Asking, Record};
use alo_software::Bound;
use alo_strings::{Filling, Said, Word};

use crate::how_an_application_starts::{NoRoad, SaysHowAnApplicationStarts, TheRoad};
use crate::machine::{Machine, WhatASoftwareVerbNeeds, WhatAnApplicationVerbNeeds};
use crate::refusing::NotDone;
use crate::testing::{hour, in_english, noon, offering};
use crate::the_door_the_compositor_serves::{NotAtTheDoor, TheDoorTheCompositorServes};
use crate::the_screen_road_as_chosen::TheScreenRoadAsChosen;
use crate::the_windows_on_the_screen::TheWindowsOnTheScreen;
use crate::{Arriving, Changed, Turning};

/// The application every test clicks in: a game, drawn by SDL, whose window
/// tells the accessibility bus nothing.
const GAME: &str = "net.supertuxkart.SuperTuxKart";

/// Another application on the same machine.
const FILES: &str = "org.gnome.Nautilus";

/// The agent.
const AGENT: &str = "@clicking";

/// `ATSPI_ROLE_APPLICATION`, `_FRAME`, `_CANVAS`, `_PUSH_BUTTON`, `_LABEL`,
/// `_PASSWORD_TEXT`.
const APPLICATION_ROLE: u32 = 75;
const FRAME: u32 = 23;
const CANVAS: u32 = 6;
const PUSH_BUTTON: u32 = 43;
const LABEL: u32 = 29;
const PASSWORD: u32 = 40;

/// Active, showing and visible.
const SHOWN: u32 = (1 << 1) | (1 << 25) | (1 << 30);

/// Texts that can be recognised in anything that might have kept a picture:
/// one in the picture before the click, one in the picture after it.
const BEFORE: &str = "ALO-CLICK-BEFORE-7f3a";
const AFTER: &str = "ALO-CLICK-AFTER-9c1b";

/// A PNG: its signature, then `marker`.
fn a_png(marker: &str) -> Vec<u8> {
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png.extend_from_slice(marker.as_bytes());
    png
}

/// How wide and tall the window is.
const WIDE: u32 = 1280;
const TALL: u32 = 800;

/// The door's answer to a picture: this PNG, its length told truly.
fn a_picture_of(png: &[u8]) -> (Pictured, Vec<u8>) {
    (
        Pictured::Picture {
            bytes: png.len(),
            wide: WIDE,
            tall: TALL,
        },
        png.to_vec(),
    )
}

/// The door's answer to a picture it refuses: no bytes.
fn no_picture(refused: Pictured) -> (Pictured, Vec<u8>) {
    (refused, Vec::new())
}

/// What the game's windows show on the accessibility bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shown {
    /// One window holding one unnamed canvas with no text: describes nothing.
    Nothing,
    /// The canvas is a button with a name.
    ANamedButton,
    /// The canvas is a label holding text.
    SomeText,
    /// The first window describes nothing, and a second window of the same
    /// application holds a password field.
    APasswordFieldInAnotherWindow,
    /// The game is not on the bus at all; only another application is.
    NotOnTheBus,
    /// There is no tree to open.
    NoTreeAtAll,
}

fn at(object: &str) -> NodeAt {
    NodeAt {
        holder: ":1.42".to_owned(),
        object: object.to_owned(),
    }
}

/// The game's tree, as `shown` says.
struct TheGame {
    shown: Shown,
}

impl AccessibilityTree for TheGame {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        let other = Running {
            at: at("/other"),
            is: Some(FILES.to_owned()),
        };
        Ok(if self.shown == Shown::NotOnTheBus {
            vec![other]
        } else {
            vec![
                other,
                Running {
                    at: at("/root"),
                    is: Some(GAME.to_owned()),
                },
            ]
        })
    }

    fn facts(&self, place: &NodeAt) -> Result<Facts, TreeFault> {
        let thing = |role, name: &str, has_text, children: Vec<NodeAt>| Facts {
            role,
            name: name.to_owned(),
            states: vec![SHOWN, 0],
            children,
            has_text,
            unlisted_children: false,
        };
        let windows = if self.shown == Shown::APasswordFieldInAnotherWindow {
            vec![at("/win"), at("/sign-in")]
        } else {
            vec![at("/win")]
        };
        let inside = match self.shown {
            Shown::ANamedButton => thing(PUSH_BUTTON, "Start race", false, Vec::new()),
            Shown::SomeText => thing(LABEL, "", true, Vec::new()),
            Shown::Nothing
            | Shown::APasswordFieldInAnotherWindow
            | Shown::NotOnTheBus
            | Shown::NoTreeAtAll => thing(CANVAS, "", false, Vec::new()),
        };
        BTreeMap::from([
            (at("/other"), thing(APPLICATION_ROLE, "", false, Vec::new())),
            (at("/root"), thing(APPLICATION_ROLE, "", false, windows)),
            (
                at("/win"),
                thing(FRAME, "SuperTuxKart", false, vec![at("/game")]),
            ),
            (at("/game"), inside),
            (
                at("/sign-in"),
                thing(FRAME, "", false, vec![at("/password")]),
            ),
            (at("/password"), thing(PASSWORD, "", false, Vec::new())),
        ])
        .remove(place)
        .ok_or(TreeFault::Vanished)
    }

    fn text(&self, place: &NodeAt) -> Result<String, TreeFault> {
        Ok(if self.shown == Shown::SomeText && *place == at("/game") {
            "Lap 1 of 3".to_owned()
        } else {
            String::new()
        })
    }

    fn actions(&self, place: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Ok(
            if self.shown == Shown::ANamedButton && *place == at("/game") {
                vec!["click".to_owned()]
            } else {
                Vec::new()
            },
        )
    }

    fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
        panic!("clicking by position never presses a control by its name")
    }
}

/// The person's windows, counting every opening.
struct TheWindows {
    shown: Shown,
    opened: Rc<Cell<usize>>,
}

impl TheWindowsOnTheScreen for TheWindows {
    fn opening(&mut self) -> Result<Box<dyn AccessibilityTree>, TreeFault> {
        self.opened.set(self.opened.get() + 1);
        if self.shown == Shown::NoTreeAtAll {
            return Err(TreeFault::NotThere);
        }
        Ok(Box::new(TheGame { shown: self.shown }))
    }

    fn working_fully(&self, _: &str) -> WorkingFully {
        WorkingFully::NothingMoreToTurnOn
    }
}

/// One thing the door was asked, in the order it was asked.
#[derive(Debug, Clone, PartialEq, Eq)]
enum AtTheDoor {
    Pictured(ToPicture),
    Clicked(ToClick),
}

/// A compositor's door that answers its pictures from `pictures`, one per
/// asking and in order, answers every click with `click`, or is not there at
/// all — and keeps everything it was asked, in order.
struct ADoor {
    pictures: VecDeque<(Pictured, Vec<u8>)>,
    click: Result<Clicked, NotAtTheDoor>,
    there: bool,
    asked: Rc<RefCell<Vec<AtTheDoor>>>,
}

impl TheDoorTheCompositorServes for ADoor {
    fn asking(&mut self, _: &alo_acting::Request) -> Result<alo_acting::Answer, NotAtTheDoor> {
        panic!("clicking by position asks nothing but pictures and the click")
    }

    fn picturing(&mut self, asked: &ToPicture) -> Result<(Pictured, Vec<u8>), NotAtTheDoor> {
        self.asked
            .borrow_mut()
            .push(AtTheDoor::Pictured(asked.clone()));
        if !self.there {
            return Err(NotAtTheDoor::NoCompositor);
        }
        Ok(self
            .pictures
            .pop_front()
            .unwrap_or_else(|| panic!("more than one picture before and one after")))
    }

    fn clicking(&mut self, asked: &ToClick) -> Result<Clicked, NotAtTheDoor> {
        self.asked
            .borrow_mut()
            .push(AtTheDoor::Clicked(asked.clone()));
        if !self.there {
            return Err(NotAtTheDoor::NoCompositor);
        }
        self.click.clone()
    }
}

/// The person's two choices, as their file says at this moment — and a test
/// may change them while a click waits.
struct Chosen {
    on_in: RefCell<PicturesAndKeys>,
    reads: RefCell<WhoReadsPictures>,
}

impl Chosen {
    fn switched_off_for_the_game(&self) {
        let now = self.on_in.borrow().clone();
        *self.on_in.borrow_mut() = now.switched_off(GAME);
    }

    fn nobody_reads(&self) {
        *self.reads.borrow_mut() = WhoReadsPictures::NoOne;
    }
}

impl TheScreenRoadAsChosen for Chosen {
    fn pictures_and_keys(&self) -> PicturesAndKeys {
        self.on_in.borrow().clone()
    }

    fn who_reads_pictures(&self) -> WhoReadsPictures {
        self.reads.borrow().clone()
    }
}

/// A tool nothing here reaches.
struct NoTool;

impl alo_software::Tool for NoTool {
    fn sources(&self) -> Result<Vec<alo_software::Configured>, alo_software::Failed> {
        Ok(Vec::new())
    }
    fn installed(&self) -> Result<Vec<String>, alo_software::Failed> {
        Ok(Vec::new())
    }
    fn open(&self) -> Result<Vec<String>, alo_software::Failed> {
        Ok(Vec::new())
    }
    fn install(
        &self,
        _: &alo_software::SourceName,
        _: &Application,
    ) -> Result<(), alo_software::Failed> {
        panic!("nothing here installs anything")
    }
    fn updates(&self, _: &alo_software::SourceName) -> Result<Vec<String>, alo_software::Failed> {
        Ok(Vec::new())
    }
    fn update(&self, _: &Application) -> Result<(), alo_software::Failed> {
        panic!("nothing here updates anything")
    }
    fn remove(&self, _: &Application) -> Result<(), alo_software::Failed> {
        panic!("nothing here removes anything")
    }
}

/// Nothing here starts an application.
struct NothingStarts;

impl SaysHowAnApplicationStarts for NothingStarts {
    fn how_it_starts(&self, _: &str) -> Result<TheRoad, NoRoad> {
        Err(NoRoad::NoEntry)
    }
}

/// How one machine is arranged.
struct Arranged {
    shown: Shown,
    /// What the door answers to each picture, in order.
    pictures: Vec<(Pictured, Vec<u8>)>,
    /// What the door answers to the click.
    click: Result<Clicked, NotAtTheDoor>,
    door_is_there: bool,
    on_in: Vec<&'static str>,
    reads: WhoReadsPictures,
    /// Whether the machine is given the person's choices at all.
    given_the_road: bool,
}

/// The arrangement under which a click is made and changes the window: a game
/// describing nothing, switched on, read on this computer, and a door that
/// pictures it, clicks it, and pictures something different afterwards.
fn ready() -> Arranged {
    Arranged {
        shown: Shown::Nothing,
        pictures: vec![a_picture_of(&a_png(BEFORE)), a_picture_of(&a_png(AFTER))],
        click: Ok(Clicked::Clicked),
        door_is_there: true,
        on_in: vec![GAME],
        reads: WhoReadsPictures::ThisComputer,
        given_the_road: true,
    }
}

/// What one machine was asked.
struct Asked {
    opened: usize,
    /// Everything the door was asked, in order.
    at_the_door: Vec<AtTheDoor>,
    entries: usize,
    /// Every line every entry told, in order.
    told: Vec<String>,
    /// The whole record, as `Debug` writes it.
    written: String,
}

impl Asked {
    fn clicks(&self) -> usize {
        self.at_the_door
            .iter()
            .filter(|asked| matches!(asked, AtTheDoor::Clicked(_)))
            .count()
    }

    fn pictures(&self) -> usize {
        self.at_the_door
            .iter()
            .filter(|asked| matches!(asked, AtTheDoor::Pictured(_)))
            .count()
    }
}

/// Run `doing` on a whole machine arranged as `arranged`.
fn on_a_machine<T>(
    arranged: Arranged,
    granting: impl FnOnce(&mut Grants),
    doing: impl FnOnce(&mut Machine<'_>, &mut Grants, &Chosen, &Path) -> T,
) -> (T, Asked) {
    let strings = in_english();
    let tool = NoTool;
    let publisher = crate::testing::APublisher::that_publishes();
    let at_the_door = Rc::new(RefCell::new(Vec::new()));
    let mut door = ADoor {
        pictures: arranged.pictures.into(),
        click: arranged.click,
        there: arranged.door_is_there,
        asked: Rc::clone(&at_the_door),
    };
    let opened = Rc::new(Cell::new(0));
    let mut windows = TheWindows {
        shown: arranged.shown,
        opened: Rc::clone(&opened),
    };
    let chosen = Chosen {
        on_in: RefCell::new(PicturesAndKeys::of(arranged.on_in)),
        reads: RefCell::new(arranged.reads),
    };
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let mut bounding = crate::testing::NothingIsBounded;
    let folder = crate::testing::a_folder_of_our_own("clicking");
    let there = folder.join("a-note.txt");
    std::fs::write(&there, "anything").unwrap();
    let answered = {
        let machine = Machine::carrying_out_file_software_and_application_verbs(
            &strings,
            &OnThisMachine,
            WhatASoftwareVerbNeeds {
                tool: &tool,
                bound: Bound::Nobodys,
                publishing: crate::testing::publishing_through(&publisher),
            },
            WhatAnApplicationVerbNeeds {
                installed: Installed::holding([
                    Application::identified(GAME).unwrap(),
                    Application::identified(FILES).unwrap(),
                ])
                .into(),
                door: &mut door,
                starting: &NothingStarts,
                windows: &mut windows,
            },
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let mut machine = if arranged.given_the_road {
            machine.with_the_screen_road(&chosen)
        } else {
            machine
        };
        let mut grants = Grants::default();
        granting(&mut grants);
        doing(&mut machine, &mut grants, &chosen, &there)
    };
    let asked = Asked {
        opened: opened.get(),
        at_the_door: at_the_door.borrow().clone(),
        entries: record.answering(&Asking::anything()).count(),
        told: record
            .answering(&Asking::anything())
            .flat_map(|entry| entry.told().iter().map(ToString::to_string))
            .collect(),
        written: format!("{record:?}"),
    };
    (answered, asked)
}

/// Run `doing` in a turn on a machine arranged as `arranged`.
fn on_a_turn<T>(
    arranged: Arranged,
    granting: impl FnOnce(&mut Grants),
    doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants, &Chosen) -> T,
) -> (T, Asked) {
    on_a_machine(arranged, granting, |machine, grants, chosen, there| {
        let mut turning =
            Turning::beginning(offering(there), AGENT, hour(), grants, machine).unwrap();
        doing(&mut turning, grants, chosen)
    })
}

fn the_game_is_granted(grants: &mut Grants) {
    grants
        .grant(Grant::checked(AGENT, Reach::Application(GAME.to_owned()), noon(), hour()).unwrap());
}

/// A click in the game, `across` and `down` per cent of its window.
fn in_the_game_at(across: i64, down: i64) -> Vec<(&'static str, Given)> {
    vec![
        (APPLICATION, Given::text(GAME)),
        (ACROSS, Given::number(across)),
        (DOWN, Given::number(down)),
    ]
}

/// The click most tests make: half way across, a quarter of the way down.
fn the_click() -> Vec<(&'static str, Given)> {
    in_the_game_at(50, 25)
}

/// Propose `given`, approve it, and answer what it did.
fn approving(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    given: &[(&'static str, Given)],
) -> Result<crate::WhatAVerbDid, NotDone> {
    let id = turning.proposing(CLICK_AT, given, grants, hour(), noon())?;
    turning.approving(id, grants, noon())
}

/// Propose [`the_click`] and approve it.
fn approving_the_click(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
) -> Result<crate::WhatAVerbDid, NotDone> {
    approving(turning, grants, &the_click())
}

/// Make `given` at full access.
fn at_full_access(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    given: &[(&'static str, Given)],
) -> Result<Changed, NotDone> {
    let at = grants.at_full_access_for(&Grantee::named(AGENT));
    turning.changing(CLICK_AT, given, &at, hour(), &mut |_: &Said| None, noon())
}

/// The sentence `word` says about the game.
fn saying(word: Word) -> String {
    let said = in_english().say(&word.key(), &Filling::of("application", GAME));
    assert!(!said.is_a_bug(), "a click's word is not loaded: {said}");
    said.into_text()
}

/// Assert `answered` is a refusal saying `word` about the game.
fn refused_saying<D: std::fmt::Debug>(answered: &Result<D, NotDone>, word: Word) {
    let Err(not_done) = answered else {
        panic!("it was not refused: {answered:?}")
    };
    assert!(not_done.was_refused(), "not a refusal: {not_done:?}");
    assert_eq!(
        not_done.said(&in_english()).into_text(),
        saying(word),
        "{not_done:?}"
    );
}

/// The picture the door is asked for, of the game.
fn picture_of_the_game() -> AtTheDoor {
    AtTheDoor::Pictured(ToPicture {
        application: Application::identified(GAME).unwrap(),
    })
}

/// The click the door is asked for, in the game, in thousandths.
fn click_in_the_game(across: u16, down: u16) -> AtTheDoor {
    AtTheDoor::Clicked(ToClick {
        application: Application::identified(GAME).unwrap(),
        across,
        down,
    })
}

/// What the door is asked for [`the_click`]: a picture, the click, a picture.
fn around_the_click() -> Vec<AtTheDoor> {
    vec![
        picture_of_the_game(),
        click_in_the_game(500, 250),
        picture_of_the_game(),
    ]
}

/// The record holds nothing of either picture: not its marker as text, and not
/// its bytes as `Debug` writes a list of numbers.
fn holds_no_picture(asked: &Asked) {
    for marker in [BEFORE, AFTER] {
        let as_numbers = marker
            .bytes()
            .map(|byte| byte.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        assert!(
            !asked.written.contains(marker),
            "the record kept a picture as text: {}",
            asked.written
        );
        assert!(
            !asked.written.contains(&as_numbers),
            "the record kept a picture's bytes: {}",
            asked.written
        );
    }
}

/// Whether the record told the person what the click did is not known.
fn told_not_known(asked: &Asked) -> bool {
    let not_known = saying(NOT_KNOWN);
    asked.told.iter().any(|line| line.contains(&not_known))
}

#[test]
fn with_no_invocation_nothing_is_opened_pictured_or_clicked() {
    let ((), asked) = on_a_turn(ready(), the_game_is_granted, |_, _, _| ());
    assert_eq!(
        (asked.opened, asked.at_the_door.len(), asked.entries),
        (0, 0, 0)
    );
}

#[test]
fn at_ask_each_time_a_click_waits_for_the_persons_ok() {
    let (waited, asked) = on_a_turn(ready(), the_game_is_granted, |turning, grants, _| {
        let changed = turning
            .changing(
                CLICK_AT,
                &the_click(),
                grants,
                hour(),
                &mut |_: &Said| None,
                noon(),
            )
            .unwrap();
        let waited = matches!(changed, Changed::Waiting(_));
        assert_eq!(turning.waiting_at(noon()).count(), 1);
        waited
    });
    assert!(waited, "a click was made without the OK");
    assert!(
        asked.at_the_door.is_empty(),
        "the door was asked before the OK: {:?}",
        asked.at_the_door
    );
}

#[test]
fn one_approval_pictures_clicks_once_and_pictures_again_in_that_order() {
    let (did, asked) = on_a_turn(ready(), the_game_is_granted, |turning, grants, _| {
        let id = turning
            .proposing(CLICK_AT, &the_click(), grants, hour(), noon())
            .unwrap();
        let did = turning.approving(id, grants, noon()).unwrap();
        assert!(
            turning.approving(id, grants, noon()).is_err(),
            "one approval was spent twice"
        );
        did
    });
    assert_eq!(
        asked.at_the_door,
        around_the_click(),
        "not one picture, one click and one picture, in that order"
    );
    let made = did.click_made().unwrap();
    assert_eq!(made.application(), GAME);
    assert_eq!(made.at(), (50, 25), "the point is answered in per cent");
    let after = made.after().unwrap();
    assert_eq!(
        after.png(),
        a_png(AFTER).as_slice(),
        "the picture answered is not the one after the click"
    );
    assert!(
        made.changed(),
        "a window that changed was read as unchanged"
    );
    assert_eq!(asked.entries, 1);
    assert!(
        !told_not_known(&asked),
        "a click that changed the window was recorded as not known: {:?}",
        asked.told
    );
    holds_no_picture(&asked);
}

#[test]
fn a_declined_click_clicks_nothing_and_is_recorded() {
    let ((), asked) = on_a_turn(ready(), the_game_is_granted, |turning, grants, _| {
        let id = turning
            .proposing(CLICK_AT, &the_click(), grants, hour(), noon())
            .unwrap();
        turning.declining(id, noon()).unwrap();
    });
    assert_eq!(
        (asked.opened, asked.at_the_door.len(), asked.entries),
        (0, 0, 1)
    );
}

#[test]
fn at_full_access_a_click_is_made_at_once_and_once() {
    let (changed, asked) = on_a_turn(
        ready(),
        |_| {},
        |turning, grants, _| {
            let changed = at_full_access(turning, grants, &the_click()).unwrap();
            assert_eq!(turning.waiting_at(noon()).count(), 0);
            changed
        },
    );
    let Changed::Done { did, sentence } = changed else {
        panic!("a click waited at full access")
    };
    let made = did.click_made().unwrap();
    assert_eq!(made.at(), (50, 25));
    assert!(made.changed());
    assert!(sentence.text().contains(GAME), "{sentence}");
    assert_eq!(asked.at_the_door, around_the_click());
    assert_eq!(asked.entries, 1);
    holds_no_picture(&asked);
}

#[test]
fn the_point_reaches_the_door_in_thousandths_of_the_window() {
    for (across, down) in [(0, 0), (100, 100), (0, 100), (33, 67), (50, 25), (1, 99)] {
        let (changed, asked) = on_a_turn(
            ready(),
            |_| {},
            |turning, grants, _| at_full_access(turning, grants, &in_the_game_at(across, down)),
        );
        let Ok(Changed::Done { did, .. }) = changed else {
            panic!("({across}, {down}) was not clicked: {changed:?}")
        };
        let made = did.click_made().unwrap();
        assert_eq!(
            made.at(),
            (u8::try_from(across).unwrap(), u8::try_from(down).unwrap())
        );
        assert_eq!(
            asked.at_the_door,
            [
                picture_of_the_game(),
                click_in_the_game(
                    u16::try_from(across * 10).unwrap(),
                    u16::try_from(down * 10).unwrap()
                ),
                picture_of_the_game(),
            ],
            "({across}, {down})"
        );
    }
}

#[test]
fn a_point_outside_the_window_is_refused_at_the_call_at_both_levels() {
    for (across, down) in [(101, 50), (50, 101), (-1, 50), (50, -1), (1000, 1000)] {
        let ((asking, full), asked) =
            on_a_turn(ready(), the_game_is_granted, |turning, grants, _| {
                let given = in_the_game_at(across, down);
                (
                    approving(turning, grants, &given),
                    at_full_access(turning, grants, &given),
                )
            });
        assert!(
            matches!(&asking, Err(not_done) if not_done.was_refused()),
            "({across}, {down}) was not refused at ask each time: {asking:?}"
        );
        assert!(
            matches!(&full, Err(not_done) if not_done.was_refused()),
            "({across}, {down}) was not refused at full access: {full:?}"
        );
        assert_eq!(
            (asked.opened, asked.at_the_door.len()),
            (0, 0),
            "({across}, {down}) reached the window or the door: {:?}",
            asked.at_the_door
        );
        assert_eq!(
            asked.entries, 2,
            "({across}, {down}): each refusal is written down"
        );
    }
}

#[test]
fn a_service_the_person_chose_to_read_pictures_lets_a_click_be_made() {
    let arranged = Arranged {
        reads: WhoReadsPictures::a_provider("Mistral").unwrap(),
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_the_click(turning, grants)
    });
    assert!(did.unwrap().click_made().is_some());
    assert_eq!(asked.at_the_door, around_the_click());
    assert_eq!(asked.entries, 1);
    holds_no_picture(&asked);
}

#[test]
fn a_machine_never_told_the_persons_choices_clicks_nothing() {
    let arranged = Arranged {
        given_the_road: false,
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_the_click(turning, grants)
    });
    assert!(
        matches!(&did, Err(not_done) if not_done.was_refused()),
        "{did:?}"
    );
    assert_eq!((asked.at_the_door.len(), asked.entries), (0, 1));
}

#[test]
fn pictures_and_keys_switched_off_for_the_game_click_nothing_at_either_level() {
    let arranged = Arranged {
        on_in: Vec::new(),
        ..ready()
    };
    let ((asking, full), asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        (
            approving_the_click(turning, grants),
            at_full_access(turning, grants, &the_click()),
        )
    });
    refused_saying(&asking, NOT_SWITCHED_ON);
    refused_saying(&full, NOT_SWITCHED_ON);
    assert!(
        asked.at_the_door.is_empty(),
        "the door was asked: {:?}",
        asked.at_the_door
    );
    assert_eq!(asked.entries, 2, "each refusal is written down");
}

#[test]
fn pictures_and_keys_on_for_another_application_are_not_on_for_the_game() {
    let arranged = Arranged {
        on_in: vec![FILES],
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_the_click(turning, grants)
    });
    refused_saying(&did, NOT_SWITCHED_ON);
    assert_eq!((asked.at_the_door.len(), asked.entries), (0, 1));
}

#[test]
fn with_no_one_chosen_to_read_pictures_nothing_is_clicked_at_either_level() {
    let arranged = Arranged {
        reads: WhoReadsPictures::NoOne,
        ..ready()
    };
    let ((asking, full), asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        (
            approving_the_click(turning, grants),
            at_full_access(turning, grants, &the_click()),
        )
    });
    refused_saying(&asking, NO_ONE_READS);
    refused_saying(&full, NO_ONE_READS);
    assert!(
        asked.at_the_door.is_empty(),
        "the door was asked: {:?}",
        asked.at_the_door
    );
    assert_eq!(asked.entries, 2);
}

#[test]
fn pictures_and_keys_switched_off_while_a_click_waits_stop_it() {
    let (did, asked) = on_a_turn(ready(), the_game_is_granted, |turning, grants, chosen| {
        let id = turning
            .proposing(CLICK_AT, &the_click(), grants, hour(), noon())
            .unwrap();
        chosen.switched_off_for_the_game();
        turning.approving(id, grants, noon())
    });
    refused_saying(&did, NOT_SWITCHED_ON);
    assert_eq!((asked.at_the_door.len(), asked.entries), (0, 1));
}

#[test]
fn a_reader_withdrawn_while_a_click_waits_stops_it() {
    let (did, asked) = on_a_turn(ready(), the_game_is_granted, |turning, grants, chosen| {
        let id = turning
            .proposing(CLICK_AT, &the_click(), grants, hour(), noon())
            .unwrap();
        chosen.nobody_reads();
        turning.approving(id, grants, noon())
    });
    refused_saying(&did, NO_ONE_READS);
    assert_eq!((asked.at_the_door.len(), asked.entries), (0, 1));
}

#[test]
fn a_window_that_describes_itself_is_never_clicked_by_position() {
    for shown in [Shown::ANamedButton, Shown::SomeText] {
        let arranged = Arranged { shown, ..ready() };
        let ((asking, full), asked) =
            on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
                (
                    approving_the_click(turning, grants),
                    at_full_access(turning, grants, &the_click()),
                )
            });
        refused_saying(&asking, IT_DESCRIBES_ITSELF);
        refused_saying(&full, IT_DESCRIBES_ITSELF);
        assert!(
            asked.at_the_door.is_empty(),
            "{shown:?}: the door was asked: {:?}",
            asked.at_the_door
        );
        assert_eq!(asked.entries, 2, "{shown:?}");
    }
}

#[test]
fn a_password_field_anywhere_is_never_clicked_at_any_setting() {
    for reads in [
        WhoReadsPictures::ThisComputer,
        WhoReadsPictures::a_provider("Mistral").unwrap(),
    ] {
        let arranged = Arranged {
            shown: Shown::APasswordFieldInAnotherWindow,
            reads: reads.clone(),
            ..ready()
        };
        let ((asking, full), asked) =
            on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
                (
                    approving_the_click(turning, grants),
                    at_full_access(turning, grants, &the_click()),
                )
            });
        refused_saying(&asking, A_PASSWORD_FIELD);
        refused_saying(&full, A_PASSWORD_FIELD);
        assert!(
            asked.at_the_door.is_empty(),
            "{reads:?}: the door was asked: {:?}",
            asked.at_the_door
        );
        assert_eq!(asked.entries, 2, "{reads:?}");
    }
}

#[test]
fn a_game_not_on_the_accessibility_bus_goes_on_to_the_compositor() {
    let arranged = Arranged {
        shown: Shown::NotOnTheBus,
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_the_click(turning, grants)
    });
    let did = did.unwrap();
    let made = did.click_made().unwrap();
    assert_eq!(made.application(), GAME);
    assert_eq!(asked.at_the_door, around_the_click());
    assert_eq!(asked.entries, 1);
}

#[test]
fn a_tree_that_cannot_be_opened_clicks_nothing() {
    let arranged = Arranged {
        shown: Shown::NoTreeAtAll,
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_the_click(turning, grants)
    });
    assert!(did.is_err(), "{did:?}");
    assert!(
        asked.at_the_door.is_empty(),
        "the door was asked: {:?}",
        asked.at_the_door
    );
    assert_eq!(asked.entries, 1);
}

#[test]
fn a_picture_before_the_click_that_is_refused_clicks_nothing() {
    for (refused, word) in [
        (Pictured::NotInFront, NOT_IN_FRONT),
        (Pictured::NotAlosCopy, NOT_ALOS_COPY),
        (Pictured::CouldNotDraw, NO_CLICK),
    ] {
        let arranged = Arranged {
            pictures: vec![no_picture(refused)],
            ..ready()
        };
        let (asking, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
            approving_the_click(turning, grants)
        });
        refused_saying(&asking, word);
        assert_eq!(
            asked.at_the_door,
            [picture_of_the_game()],
            "{refused:?}: something was clicked after the picture before it was refused"
        );
        assert_eq!(asked.entries, 1, "{refused:?}");
    }
}

#[test]
fn a_picture_before_the_click_that_is_refused_clicks_nothing_at_full_access() {
    for (refused, word) in [
        (Pictured::NotInFront, NOT_IN_FRONT),
        (Pictured::NotAlosCopy, NOT_ALOS_COPY),
        (Pictured::CouldNotDraw, NO_CLICK),
    ] {
        let arranged = Arranged {
            pictures: vec![no_picture(refused)],
            ..ready()
        };
        let (full, asked) = on_a_turn(
            arranged,
            |_| {},
            |turning, grants, _| at_full_access(turning, grants, &the_click()),
        );
        refused_saying(&full, word);
        assert_eq!(asked.clicks(), 0, "{refused:?}: {:?}", asked.at_the_door);
        assert_eq!(asked.entries, 1, "{refused:?}");
    }
}

#[test]
fn the_doors_refusals_of_the_click_are_refusals_in_the_persons_words() {
    for (answer, word) in [
        (Clicked::NotInFront, NOT_IN_FRONT),
        (Clicked::NotAlosCopy, NOT_ALOS_COPY),
        (Clicked::ThePersonIsPressing, THE_PERSON_IS_PRESSING),
        (Clicked::CouldNotClick, NO_CLICK),
    ] {
        let arranged = Arranged {
            click: Ok(answer),
            ..ready()
        };
        let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
            approving_the_click(turning, grants)
        });
        refused_saying(&did, word);
        assert_eq!(
            asked.at_the_door.get(..2),
            Some([picture_of_the_game(), click_in_the_game(500, 250)].as_slice()),
            "{answer:?}: not a picture, then the click: {:?}",
            asked.at_the_door
        );
        assert_eq!(asked.clicks(), 1, "{answer:?}");
        assert!(asked.pictures() <= 2, "{answer:?}");
        assert_eq!(asked.entries, 1, "{answer:?}");
        holds_no_picture(&asked);
    }
}

#[test]
fn a_door_that_is_not_there_clicks_nothing_and_says_so() {
    let arranged = Arranged {
        door_is_there: false,
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_the_click(turning, grants)
    });
    let Err(not_done) = &did else {
        panic!("a click came from no door: {did:?}")
    };
    assert_eq!(not_done.said(&in_english()).into_text(), saying(NO_CLICK));
    assert_eq!(
        asked.clicks(),
        0,
        "a click was asked for with no picture before it: {:?}",
        asked.at_the_door
    );
    assert_eq!(asked.entries, 1);
}

#[test]
fn a_click_the_door_did_not_answer_is_no_click_and_is_written_down() {
    let arranged = Arranged {
        click: Err(NotAtTheDoor::NotAnswered {
            why: "the socket closed".to_owned(),
        }),
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_the_click(turning, grants)
    });
    let Err(not_done) = &did else {
        panic!("an unanswered click was answered as made: {did:?}")
    };
    assert_eq!(not_done.said(&in_english()).into_text(), saying(NO_CLICK));
    assert_eq!(asked.clicks(), 1);
    assert_eq!(asked.entries, 1);
}

#[test]
fn a_window_that_did_not_visibly_change_is_recorded_as_not_known() {
    let same = a_png(BEFORE);
    let arranged = Arranged {
        pictures: vec![a_picture_of(&same), a_picture_of(&same)],
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_the_click(turning, grants)
    });
    let did = did.unwrap();
    let made = did.click_made().unwrap();
    assert!(
        !made.changed(),
        "two identical pictures were read as a change"
    );
    assert_eq!(made.after().unwrap().png(), same.as_slice());
    assert_eq!(asked.at_the_door, around_the_click());
    assert_eq!(asked.entries, 1);
    assert!(
        told_not_known(&asked),
        "a click that changed nothing visible was not recorded as not known: {:?}",
        asked.told
    );
    holds_no_picture(&asked);
}

#[test]
fn a_picture_after_the_click_that_fails_leaves_what_it_did_not_known() {
    let short = a_png(AFTER);
    for after in [
        no_picture(Pictured::NotInFront),
        no_picture(Pictured::NotAlosCopy),
        no_picture(Pictured::CouldNotDraw),
        (
            Pictured::Picture {
                bytes: short.len() + 10,
                wide: WIDE,
                tall: TALL,
            },
            short,
        ),
    ] {
        let answered = after.0;
        let arranged = Arranged {
            pictures: vec![a_picture_of(&a_png(BEFORE)), after],
            ..ready()
        };
        let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
            approving_the_click(turning, grants)
        });
        let Ok(did) = did else {
            panic!("{answered:?}: a click that was made was answered as not made: {did:?}")
        };
        let made = did.click_made().unwrap();
        assert!(
            !made.changed(),
            "{answered:?}: no picture after was read as a change"
        );
        assert!(
            made.after().is_none(),
            "{answered:?}: a picture after was answered that the door never drew"
        );
        assert_eq!(asked.at_the_door, around_the_click(), "{answered:?}");
        assert_eq!(asked.entries, 1, "{answered:?}");
        assert!(
            told_not_known(&asked),
            "{answered:?}: not recorded as not known: {:?}",
            asked.told
        );
        holds_no_picture(&asked);
    }
}

#[test]
fn a_click_with_no_grant_over_the_game_is_refused_before_any_window_is_opened() {
    let (did, asked) = on_a_turn(
        ready(),
        |_| {},
        |turning, grants, _| approving_the_click(turning, grants),
    );
    assert!(
        matches!(&did, Err(not_done) if not_done.was_refused()),
        "{did:?}"
    );
    assert_eq!(
        (asked.opened, asked.at_the_door.len(), asked.entries),
        (0, 0, 1)
    );
}

#[test]
fn a_revoked_grant_stops_the_click_before_any_window_is_opened() {
    let (did, asked) = on_a_turn(
        ready(),
        |_| {},
        |turning, grants, _| {
            let granted = grants.grant(
                Grant::checked(AGENT, Reach::Application(GAME.to_owned()), noon(), hour()).unwrap(),
            );
            let id = turning.proposing(CLICK_AT, &the_click(), grants, hour(), noon())?;
            assert!(grants.revoke(granted));
            turning.approving(id, grants, noon())
        },
    );
    assert!(matches!(did, Err(NotDone::Refused(_))), "{did:?}");
    assert_eq!(
        (asked.opened, asked.at_the_door.len(), asked.entries),
        (0, 0, 1)
    );
}

/// This machine, paired with the reception machine, and the reception as a
/// place a verb arrives from — proven with a proof made on its own row, as
/// `crate::arriving`'s own tests make one.
fn paired_with_the_reception() -> (alo_nearby::Origin, alo_nearby::Pairings) {
    use alo_nearby::{
        Deliberating, Keying, MachineId, MayAskIts, Origin, Pairings, Proof, Proposal, Seen, Side,
    };
    let reception = || MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap();
    let here = || MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap();
    let at_reception = Keying::fresh().unwrap();
    let proposal = Proposal::checked(
        reception(),
        here(),
        &[MayAskIts::Models],
        hour(),
        at_reception.offer().clone(),
    )
    .unwrap();
    let on_here = Deliberating::asked(proposal.clone(), Keying::fresh().unwrap());
    let on_reception = Deliberating::asking(proposal, at_reception)
        .unwrap()
        .answered_with(on_here.answered().unwrap().clone())
        .unwrap()
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(noon())
        .unwrap();
    let on_here = on_here
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(noon())
        .unwrap();
    let mut pairings = Pairings::none();
    pairings.keep(on_here);
    let proof = Proof::made(&on_reception, &reception(), b"a turn", noon());
    let origin = Origin::proven(
        &pairings,
        &here(),
        &proof,
        b"a turn",
        "the reception machine",
        noon(),
        &mut Seen::nothing(),
    )
    .unwrap();
    (origin, pairings)
}

#[test]
fn a_click_asked_for_from_a_paired_machine_is_always_refused() {
    let (origin, pairings) = paired_with_the_reception();
    let principal = origin.principal();
    let (did, asked) = on_a_machine(
        ready(),
        |grants| {
            grants.grant(
                Grant::checked(
                    &principal,
                    Reach::Application(GAME.to_owned()),
                    noon(),
                    hour(),
                )
                .unwrap(),
            );
        },
        |machine, grants, _, _| {
            let mut arriving =
                Arriving::beginning(&origin, hour(), noon(), grants, machine).unwrap();
            let did = arriving
                .proposing(CLICK_AT, &the_click(), &pairings, grants, hour(), noon())
                .and_then(|id| arriving.approving(id, &pairings, grants, noon()));
            arriving.ending(grants);
            did
        },
    );
    assert!(did.is_err(), "a paired machine was given a click: {did:?}");
    assert!(
        asked.at_the_door.is_empty(),
        "the door was asked for a paired machine: {:?}",
        asked.at_the_door
    );
    assert_eq!(asked.entries, 1, "the refusal was not written down");
}

/// **A click by position is offered only once a read in this turn found a
/// window that describes nothing**, as the picture is — and not before.
#[test]
fn the_click_is_offered_only_after_a_read_finds_a_window_that_describes_nothing() {
    let ((before, after), _) = on_a_turn(ready(), the_game_is_granted, |turning, grants, _| {
        let offers = |turning: &Turning<'_, '_>| {
            turning
                .the_verbs_offered_now()
                .of(alo_adapters::CLICK_AT)
                .is_some()
        };
        let before = offers(turning);
        let read = turning.reading(
            alo_adapters::READ_WINDOW,
            &[(alo_adapters::fallback_verbs::APPLICATION, Given::text(GAME))],
            grants,
            noon(),
        );
        assert!(read.is_ok(), "{read:?}");
        (before, offers(turning))
    });
    assert!(!before, "offered before any read");
    assert!(
        after,
        "not offered after a read of a window that describes nothing"
    );
}

/// **A control named in the game, after a read found it names nothing, is
/// refused before any card**, in words that send alo to a picture; the door is
/// never asked.
#[test]
fn a_control_named_where_nothing_is_named_is_refused_before_any_card() {
    let (answered, asked) = on_a_turn(ready(), the_game_is_granted, |turning, grants, _| {
        let read = turning.reading(
            alo_adapters::READ_WINDOW,
            &[(APPLICATION, Given::text(GAME))],
            grants,
            noon(),
        );
        assert!(read.is_ok(), "{read:?}");
        turning.proposing(
            alo_adapters::ACTIVATE_CONTROL,
            &[
                (APPLICATION, Given::text(GAME)),
                (alo_adapters::fallback_verbs::KIND, Given::text("button")),
                (
                    alo_adapters::fallback_verbs::NAME,
                    Given::text("Start Race"),
                ),
            ],
            grants,
            hour(),
            noon(),
        )
    });
    assert!(answered.is_err(), "proposed: {answered:?}");
    let Err(not_done) = &answered else { return };
    let said = not_done.said(&in_english()).into_text();
    assert!(said.contains("names none of its controls"), "{said}");
    assert!(said.contains("Start Race"), "{said}");
    assert!(
        asked.at_the_door.is_empty(),
        "the door was asked: {:?}",
        asked.at_the_door
    );
}

/// **A window that names its controls is unaffected**: a press there is put
/// to the person as before.
#[test]
fn a_control_named_in_a_window_that_names_its_controls_is_proposed() {
    let arranged = Arranged {
        shown: Shown::ANamedButton,
        ..ready()
    };
    let (answered, _) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        let read = turning.reading(
            alo_adapters::READ_WINDOW,
            &[(APPLICATION, Given::text(GAME))],
            grants,
            noon(),
        );
        assert!(read.is_ok(), "{read:?}");
        turning.proposing(
            alo_adapters::ACTIVATE_CONTROL,
            &[
                (APPLICATION, Given::text(GAME)),
                (alo_adapters::fallback_verbs::KIND, Given::text("button")),
                (
                    alo_adapters::fallback_verbs::NAME,
                    Given::text("Start race"),
                ),
            ],
            grants,
            hour(),
            noon(),
        )
    });
    assert!(answered.is_ok(), "{answered:?}");
}
