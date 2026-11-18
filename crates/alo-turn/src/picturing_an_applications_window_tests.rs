//! **Taking one picture of an application's window, through a whole turn**
//! (`accessible.picture_window`, ADR 0147 §1 and its two amendments of
//! 2026-11-17), written from the promise by a second author and not from the
//! code (law 7): a game whose window describes nothing, a tree that counts
//! every time it is opened, a door that keeps every picture it is asked for, and
//! the person's two choices held where a test can turn them while a turn waits.
//!
//! - with no invocation nothing is opened and no picture is taken;
//! - at *ask each time* a picture waits for one approval, one approval takes one
//!   picture, and a declined one takes none;
//! - at full access it is taken at once, once;
//! - no picture unless *Pictures and keys* is on for **that** application and
//!   somebody was chosen to read pictures — read at the moment of the act, so a
//!   switch turned off while a picture waits stops it;
//! - a window that describes itself is read, not pictured; a password field in
//!   any of the application's windows is never pictured at any setting; an
//!   application not on the accessibility bus at all goes on to the compositor,
//!   which decides; a tree that cannot be opened is no picture;
//! - in every refusal before the door, the door is never asked;
//! - what the door answers is held to: its three refusals, a picture whose
//!   length is not its bytes, and an empty one are no picture;
//! - a call from a paired machine is refused;
//! - every picture and every refusal leaves exactly one entry, and no entry
//!   holds the picture.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;

use alo_acting::{Pictured, ToPicture};
use alo_adapters::fallback_verbs::APPLICATION;
use alo_adapters::picture_words::{
    A_PASSWORD_FIELD, IT_DESCRIBES_ITSELF, NO_ONE_READS, NO_PICTURE, NOT_ALOS_COPY, NOT_IN_FRONT,
    NOT_SWITCHED_ON,
};
use alo_adapters::{
    AccessibilityTree, Facts, NodeAt, PICTURE_WINDOW, Running, TreeFault, WorkingFully,
};
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

/// The application every test takes a picture of: a game, drawn by SDL, whose
/// window tells the accessibility bus nothing.
const GAME: &str = "net.supertuxkart.SuperTuxKart";

/// Another application on the same machine.
const FILES: &str = "org.gnome.Nautilus";

/// The agent.
const AGENT: &str = "@pictures";

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

/// A text that can be recognised in anything that might have kept it.
const MARKER: &str = "ALO-PICTURE-BYTES-7f3a";

/// The PNG every picturing door answers with: its signature, then the marker.
fn the_png() -> Vec<u8> {
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png.extend_from_slice(MARKER.as_bytes());
    png
}

/// How wide and tall the window is.
const WIDE: u32 = 1280;
const TALL: u32 = 800;

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
        panic!("taking a picture never presses a control")
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

/// A compositor's door that answers every picture as `answers` with `png`, or
/// is not there at all, and keeps every picture it was asked for.
struct ADoor {
    answers: Pictured,
    png: Vec<u8>,
    there: bool,
    asked: Rc<RefCell<Vec<ToPicture>>>,
}

impl TheDoorTheCompositorServes for ADoor {
    fn asking(&mut self, _: &alo_acting::Request) -> Result<alo_acting::Answer, NotAtTheDoor> {
        panic!("taking a picture asks nothing but the picture")
    }

    fn picturing(&mut self, asked: &ToPicture) -> Result<(Pictured, Vec<u8>), NotAtTheDoor> {
        self.asked.borrow_mut().push(asked.clone());
        if !self.there {
            return Err(NotAtTheDoor::NoCompositor);
        }
        Ok((self.answers, self.png.clone()))
    }
}

/// The person's two choices, as their file says at this moment — and a test
/// may change them while a picture waits.
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
    answers: Pictured,
    png: Vec<u8>,
    door_is_there: bool,
    on_in: Vec<&'static str>,
    reads: WhoReadsPictures,
    /// Whether the machine is given the person's choices at all.
    given_the_road: bool,
}

/// The arrangement under which a picture is taken: a game describing nothing,
/// switched on, read on this computer, and a door that draws it.
fn ready() -> Arranged {
    let png = the_png();
    Arranged {
        shown: Shown::Nothing,
        answers: Pictured::Picture {
            bytes: png.len(),
            wide: WIDE,
            tall: TALL,
        },
        png,
        door_is_there: true,
        on_in: vec![GAME],
        reads: WhoReadsPictures::ThisComputer,
        given_the_road: true,
    }
}

/// What one machine was asked.
struct Asked {
    opened: usize,
    pictures: Vec<ToPicture>,
    entries: usize,
    /// The whole record, as `Debug` writes it.
    written: String,
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
    let pictures = Rc::new(RefCell::new(Vec::new()));
    let mut door = ADoor {
        answers: arranged.answers,
        png: arranged.png,
        there: arranged.door_is_there,
        asked: Rc::clone(&pictures),
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
    let folder = crate::testing::a_folder_of_our_own("pictures");
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
        pictures: pictures.borrow().clone(),
        entries: record.answering(&Asking::anything()).count(),
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

fn the_game() -> Vec<(&'static str, Given)> {
    vec![(APPLICATION, Given::text(GAME))]
}

/// Propose a picture of the game, approve it, and answer what it did.
fn approving_a_picture(
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
) -> Result<crate::WhatAVerbDid, NotDone> {
    let id = turning.proposing(PICTURE_WINDOW, &the_game(), grants, hour(), noon())?;
    turning.approving(id, grants, noon())
}

/// Take a picture of the game at full access.
fn at_full_access(turning: &mut Turning<'_, '_>, grants: &Grants) -> Result<Changed, NotDone> {
    let at = grants.at_full_access_for(&Grantee::named(AGENT));
    turning.changing(
        PICTURE_WINDOW,
        &the_game(),
        &at,
        hour(),
        &mut |_: &Said| None,
        noon(),
    )
}

/// The sentence `word` says about the game.
fn saying(word: Word) -> String {
    let said = in_english().say(&word.key(), &Filling::of("application", GAME));
    assert!(!said.is_a_bug(), "a picture's word is not loaded: {said}");
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

/// The picture the door was asked for, of the game.
fn of_the_game() -> ToPicture {
    ToPicture {
        application: Application::identified(GAME).unwrap(),
    }
}

/// The record holds nothing of the picture: not its marker as text, and not
/// its bytes as `Debug` writes a list of numbers.
fn holds_no_picture(asked: &Asked) {
    let as_numbers = MARKER
        .bytes()
        .map(|byte| byte.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    assert!(
        !asked.written.contains(MARKER),
        "the record kept the picture as text: {}",
        asked.written
    );
    assert!(
        !asked.written.contains(&as_numbers),
        "the record kept the picture's bytes: {}",
        asked.written
    );
}

#[test]
fn with_no_invocation_nothing_is_opened_and_no_picture_is_taken() {
    let ((), asked) = on_a_turn(ready(), the_game_is_granted, |_, _, _| ());
    assert_eq!(
        (asked.opened, asked.pictures.len(), asked.entries),
        (0, 0, 0)
    );
}

#[test]
fn at_ask_each_time_a_picture_waits_for_the_persons_ok() {
    let (waited, asked) = on_a_turn(ready(), the_game_is_granted, |turning, grants, _| {
        let changed = turning
            .changing(
                PICTURE_WINDOW,
                &the_game(),
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
    assert!(waited, "a picture was taken without the OK");
    assert!(
        asked.pictures.is_empty(),
        "the door was asked before the OK"
    );
}

#[test]
fn at_ask_each_time_one_approval_takes_exactly_one_picture_of_that_window() {
    let (did, asked) = on_a_turn(ready(), the_game_is_granted, |turning, grants, _| {
        let id = turning
            .proposing(PICTURE_WINDOW, &the_game(), grants, hour(), noon())
            .unwrap();
        let did = turning.approving(id, grants, noon()).unwrap();
        assert!(
            turning.approving(id, grants, noon()).is_err(),
            "one approval was spent twice"
        );
        did
    });
    let taken = did.picture_taken().unwrap();
    assert_eq!(taken.application(), GAME);
    assert_eq!(taken.png(), the_png().as_slice());
    assert_eq!(taken.size(), (WIDE, TALL));
    assert_eq!(
        asked.pictures,
        [of_the_game()],
        "the door was asked once, for the game"
    );
    assert_eq!(asked.entries, 1);
    holds_no_picture(&asked);
}

#[test]
fn a_declined_picture_takes_none_and_is_recorded() {
    let ((), asked) = on_a_turn(ready(), the_game_is_granted, |turning, grants, _| {
        let id = turning
            .proposing(PICTURE_WINDOW, &the_game(), grants, hour(), noon())
            .unwrap();
        turning.declining(id, noon()).unwrap();
    });
    assert_eq!(
        (asked.opened, asked.pictures.len(), asked.entries),
        (0, 0, 1)
    );
}

#[test]
fn at_full_access_a_picture_is_taken_at_once_and_once() {
    let (changed, asked) = on_a_turn(
        ready(),
        |_| {},
        |turning, grants, _| {
            let changed = at_full_access(turning, grants).unwrap();
            assert_eq!(turning.waiting_at(noon()).count(), 0);
            changed
        },
    );
    let Changed::Done { did, sentence } = changed else {
        panic!("a picture waited at full access")
    };
    let taken = did.picture_taken().unwrap();
    assert_eq!(taken.png(), the_png().as_slice());
    assert!(sentence.text().contains(GAME), "{sentence}");
    assert_eq!(asked.pictures, [of_the_game()]);
    assert_eq!(asked.entries, 1);
    holds_no_picture(&asked);
}

#[test]
fn a_service_the_person_chose_to_read_pictures_lets_a_picture_be_taken() {
    let arranged = Arranged {
        reads: WhoReadsPictures::a_provider("Mistral").unwrap(),
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_a_picture(turning, grants)
    });
    assert!(did.unwrap().picture_taken().is_some());
    assert_eq!((asked.pictures.len(), asked.entries), (1, 1));
    holds_no_picture(&asked);
}

#[test]
fn a_machine_never_told_the_persons_choices_takes_no_picture() {
    let arranged = Arranged {
        given_the_road: false,
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_a_picture(turning, grants)
    });
    assert!(
        matches!(&did, Err(not_done) if not_done.was_refused()),
        "{did:?}"
    );
    assert_eq!((asked.pictures.len(), asked.entries), (0, 1));
}

#[test]
fn pictures_switched_off_for_the_game_take_no_picture_at_either_level() {
    let arranged = Arranged {
        on_in: Vec::new(),
        ..ready()
    };
    let ((asking, full), asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        (
            approving_a_picture(turning, grants),
            at_full_access(turning, grants),
        )
    });
    refused_saying(&asking, NOT_SWITCHED_ON);
    refused_saying(&full, NOT_SWITCHED_ON);
    assert!(asked.pictures.is_empty(), "the door was asked");
    assert_eq!(asked.entries, 2, "each refusal is written down");
}

#[test]
fn pictures_switched_on_for_another_application_are_not_on_for_the_game() {
    let arranged = Arranged {
        on_in: vec![FILES],
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_a_picture(turning, grants)
    });
    refused_saying(&did, NOT_SWITCHED_ON);
    assert_eq!((asked.pictures.len(), asked.entries), (0, 1));
}

#[test]
fn with_no_one_chosen_to_read_pictures_none_is_taken_at_either_level() {
    let arranged = Arranged {
        reads: WhoReadsPictures::NoOne,
        ..ready()
    };
    let ((asking, full), asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        (
            approving_a_picture(turning, grants),
            at_full_access(turning, grants),
        )
    });
    refused_saying(&asking, NO_ONE_READS);
    refused_saying(&full, NO_ONE_READS);
    assert!(asked.pictures.is_empty(), "the door was asked");
    assert_eq!(asked.entries, 2);
}

#[test]
fn no_one_reads_pictures_is_how_a_machine_arrives() {
    assert_eq!(WhoReadsPictures::default(), WhoReadsPictures::NoOne);
    assert_eq!(
        PicturesAndKeys::default(),
        PicturesAndKeys::of(Vec::<String>::new())
    );
}

#[test]
fn pictures_switched_off_while_a_picture_waits_stop_it() {
    let (did, asked) = on_a_turn(ready(), the_game_is_granted, |turning, grants, chosen| {
        let id = turning
            .proposing(PICTURE_WINDOW, &the_game(), grants, hour(), noon())
            .unwrap();
        chosen.switched_off_for_the_game();
        turning.approving(id, grants, noon())
    });
    refused_saying(&did, NOT_SWITCHED_ON);
    assert_eq!((asked.pictures.len(), asked.entries), (0, 1));
}

#[test]
fn a_reader_withdrawn_while_a_picture_waits_stops_it() {
    let (did, asked) = on_a_turn(ready(), the_game_is_granted, |turning, grants, chosen| {
        let id = turning
            .proposing(PICTURE_WINDOW, &the_game(), grants, hour(), noon())
            .unwrap();
        chosen.nobody_reads();
        turning.approving(id, grants, noon())
    });
    refused_saying(&did, NO_ONE_READS);
    assert_eq!((asked.pictures.len(), asked.entries), (0, 1));
}

#[test]
fn a_window_that_describes_itself_is_read_and_never_pictured() {
    for shown in [Shown::ANamedButton, Shown::SomeText] {
        let arranged = Arranged { shown, ..ready() };
        let ((asking, full), asked) =
            on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
                (
                    approving_a_picture(turning, grants),
                    at_full_access(turning, grants),
                )
            });
        refused_saying(&asking, IT_DESCRIBES_ITSELF);
        refused_saying(&full, IT_DESCRIBES_ITSELF);
        assert!(asked.pictures.is_empty(), "{shown:?}: the door was asked");
        assert_eq!(asked.entries, 2, "{shown:?}");
    }
}

#[test]
fn a_password_field_anywhere_is_never_pictured_at_any_setting() {
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
                    approving_a_picture(turning, grants),
                    at_full_access(turning, grants),
                )
            });
        refused_saying(&asking, A_PASSWORD_FIELD);
        refused_saying(&full, A_PASSWORD_FIELD);
        assert!(asked.pictures.is_empty(), "{reads:?}: the door was asked");
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
        approving_a_picture(turning, grants)
    });
    let did = did.unwrap();
    let taken = did.picture_taken().unwrap();
    assert_eq!(taken.application(), GAME);
    assert_eq!(asked.pictures, [of_the_game()]);
    assert_eq!(asked.entries, 1);
}

#[test]
fn a_game_not_on_the_bus_is_still_refused_where_the_compositor_refuses() {
    let arranged = Arranged {
        shown: Shown::NotOnTheBus,
        answers: Pictured::NotInFront,
        png: Vec::new(),
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_a_picture(turning, grants)
    });
    refused_saying(&did, NOT_IN_FRONT);
    assert_eq!((asked.pictures.len(), asked.entries), (1, 1));
}

#[test]
fn a_tree_that_cannot_be_opened_is_no_picture() {
    let arranged = Arranged {
        shown: Shown::NoTreeAtAll,
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_a_picture(turning, grants)
    });
    assert!(did.is_err(), "{did:?}");
    assert!(asked.pictures.is_empty(), "the door was asked");
    assert_eq!(asked.entries, 1);
}

#[test]
fn a_window_the_compositor_will_not_picture_is_refused_and_written_down() {
    for (answers, word) in [
        (Pictured::NotInFront, NOT_IN_FRONT),
        (Pictured::NotAlosCopy, NOT_ALOS_COPY),
        (Pictured::CouldNotDraw, NO_PICTURE),
    ] {
        let arranged = Arranged {
            answers,
            png: Vec::new(),
            ..ready()
        };
        let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
            approving_a_picture(turning, grants)
        });
        refused_saying(&did, word);
        assert_eq!(asked.pictures, [of_the_game()], "{answers:?}");
        assert_eq!(asked.entries, 1, "{answers:?}");
    }
}

#[test]
fn a_door_that_is_not_there_takes_no_picture_and_says_so() {
    let arranged = Arranged {
        door_is_there: false,
        ..ready()
    };
    let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
        approving_a_picture(turning, grants)
    });
    let Err(not_done) = &did else {
        panic!("a picture came from no door: {did:?}")
    };
    assert_eq!(not_done.said(&in_english()).into_text(), saying(NO_PICTURE));
    assert_eq!(asked.entries, 1);
}

#[test]
fn a_picture_whose_length_is_not_its_bytes_or_is_empty_is_no_picture() {
    let png = the_png();
    for (answers, bytes) in [
        (
            Pictured::Picture {
                bytes: png.len() + 10,
                wide: WIDE,
                tall: TALL,
            },
            png.clone(),
        ),
        (
            Pictured::Picture {
                bytes: png.len() - 1,
                wide: WIDE,
                tall: TALL,
            },
            png.clone(),
        ),
        (
            Pictured::Picture {
                bytes: 0,
                wide: WIDE,
                tall: TALL,
            },
            Vec::new(),
        ),
    ] {
        let arranged = Arranged {
            answers,
            png: bytes,
            ..ready()
        };
        let (did, asked) = on_a_turn(arranged, the_game_is_granted, |turning, grants, _| {
            approving_a_picture(turning, grants)
        });
        assert!(did.is_err(), "{answers:?}: answered as a picture: {did:?}");
        assert_eq!(asked.entries, 1, "{answers:?}");
        holds_no_picture(&asked);
    }
}

#[test]
fn a_picture_with_no_grant_over_the_game_is_refused_before_any_window_is_opened() {
    let (did, asked) = on_a_turn(
        ready(),
        |_| {},
        |turning, grants, _| approving_a_picture(turning, grants),
    );
    assert!(
        matches!(&did, Err(not_done) if not_done.was_refused()),
        "{did:?}"
    );
    assert_eq!(
        (asked.opened, asked.pictures.len(), asked.entries),
        (0, 0, 1)
    );
}

#[test]
fn a_revoked_grant_stops_the_picture_before_any_window_is_opened() {
    let (did, asked) = on_a_turn(
        ready(),
        |_| {},
        |turning, grants, _| {
            let granted = grants.grant(
                Grant::checked(AGENT, Reach::Application(GAME.to_owned()), noon(), hour()).unwrap(),
            );
            let id = turning.proposing(PICTURE_WINDOW, &the_game(), grants, hour(), noon())?;
            assert!(grants.revoke(granted));
            turning.approving(id, grants, noon())
        },
    );
    assert!(matches!(did, Err(NotDone::Refused(_))), "{did:?}");
    assert_eq!(
        (asked.opened, asked.pictures.len(), asked.entries),
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
fn a_picture_asked_for_from_a_paired_machine_is_always_refused() {
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
                .proposing(
                    PICTURE_WINDOW,
                    &the_game(),
                    &pairings,
                    grants,
                    hour(),
                    noon(),
                )
                .and_then(|id| arriving.approving(id, &pairings, grants, noon()));
            arriving.ending(grants);
            did
        },
    );
    assert!(
        did.is_err(),
        "a paired machine was given a picture: {did:?}"
    );
    assert!(
        asked.pictures.is_empty(),
        "the door was asked for a paired machine"
    );
    assert_eq!(asked.entries, 1, "the refusal was not written down");
}
