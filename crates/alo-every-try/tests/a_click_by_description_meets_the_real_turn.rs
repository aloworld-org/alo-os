//! **The seam between the every-try check's click request and the real turn**
//! (ADR 0156's second rule: both real sides, never one against its own copy of
//! the other).
//!
//! The check holds a model to clicking the game's button by describing it, and
//! its scripted reader finds the button at [`WHERE_THE_RACE_STARTS`]. Here the
//! same request's arguments are proposed to the **real** turn, approved, and
//! the turn is handed the **same** reader's line — and the compositor's door is
//! asked to click where that line says, once, between a picture before and a
//! picture after; and what the daemon would tell the model is what the check's
//! scripted machine tells it.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use alo_acting::{Clicked, Pictured, ToClick, ToPicture};
use alo_adapters::fallback_verbs::{APPLICATION, CLICK_AT, DESCRIPTION};
use alo_adapters::{AccessibilityTree, Facts, NodeAt, Running, TreeFault, WorkingFully};
use alo_applications::{Application, Installed};
use alo_capability::{Given, Grant, Grants, PicturesAndKeys, Reach, WhoReadsPictures};
use alo_every_try::every_other_verb::{THE_GAME_IDENTIFIER, WHERE_THE_RACE_STARTS};
use alo_every_try::{Answered, the_set};
use alo_protocol::Done;
use alo_record::Record;
use alo_turn::{
    Bounding, Doing, Machine, NoBoundary, NoRoad, NotAtTheDoor, SaysHowAnApplicationStarts,
    TheDoorTheCompositorServes, TheRoad, TheScreenRoadAsChosen, TheWindowsOnTheScreen, Turning,
    WhatASoftwareVerbNeeds, WhatAnApplicationVerbNeeds, WhatPublishingNeeds,
};

/// The agent.
const AGENT: &str = "@alo";

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// A game on the accessibility bus with no window: it describes nothing.
struct NoWindows;

impl AccessibilityTree for NoWindows {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        Ok(Vec::new())
    }
    fn facts(&self, _: &NodeAt) -> Result<Facts, TreeFault> {
        Err(TreeFault::Vanished)
    }
    fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
        Err(TreeFault::Vanished)
    }
    fn actions(&self, _: &NodeAt) -> Result<Vec<String>, TreeFault> {
        Err(TreeFault::Vanished)
    }
    fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
        panic!("a click by description never presses a control by its name")
    }
}

struct TheWindows;

impl TheWindowsOnTheScreen for TheWindows {
    fn opening(&mut self) -> Result<Box<dyn AccessibilityTree>, TreeFault> {
        Ok(Box::new(NoWindows))
    }
    fn working_fully(&self, _: &str) -> WorkingFully {
        WorkingFully::NothingMoreToTurnOn
    }
}

/// What the door was asked, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
enum AtTheDoor {
    Pictured(ToPicture),
    Clicked(ToClick),
}

/// A door that pictures the game twice, differently, and clicks once.
struct ADoor {
    asked: Rc<RefCell<Vec<AtTheDoor>>>,
}

impl TheDoorTheCompositorServes for ADoor {
    fn asking(&mut self, _: &alo_acting::Request) -> Result<alo_acting::Answer, NotAtTheDoor> {
        panic!("a click asks nothing but pictures and the click")
    }
    fn picturing(&mut self, asked: &ToPicture) -> Result<(Pictured, Vec<u8>), NotAtTheDoor> {
        let mut seen = self.asked.borrow_mut();
        seen.push(AtTheDoor::Pictured(asked.clone()));
        let png = format!("\u{89}PNG picture {}", seen.len()).into_bytes();
        Ok((
            Pictured::Picture {
                bytes: png.len(),
                wide: 1280,
                tall: 800,
            },
            png,
        ))
    }
    fn clicking(&mut self, asked: &ToClick) -> Result<Clicked, NotAtTheDoor> {
        self.asked
            .borrow_mut()
            .push(AtTheDoor::Clicked(asked.clone()));
        Ok(Clicked::Clicked)
    }
}

/// The person switched pictures and keys on for the game and chose a reader.
struct Chosen;

impl TheScreenRoadAsChosen for Chosen {
    fn pictures_and_keys(&self) -> PicturesAndKeys {
        PicturesAndKeys::of([THE_GAME_IDENTIFIER])
    }
    fn who_reads_pictures(&self) -> WhoReadsPictures {
        WhoReadsPictures::a_provider("Mistral").unwrap()
    }
}

struct NothingStarts;

impl SaysHowAnApplicationStarts for NothingStarts {
    fn how_it_starts(&self, _: &str) -> Result<TheRoad, NoRoad> {
        Err(NoRoad::NoEntry)
    }
}

struct NoBoundaries;

impl Bounding for NoBoundaries {
    fn carrying_out(
        &mut self,
        _: &alo_files::Reaching,
        _: Doing<'_>,
    ) -> Result<alo_turn::Done, NoBoundary> {
        Err(NoBoundary::because(
            "a click runs before any boundary".to_owned(),
        ))
    }
}

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
        panic!("nothing installs")
    }
    fn updates(&self, _: &alo_software::SourceName) -> Result<Vec<String>, alo_software::Failed> {
        Ok(Vec::new())
    }
    fn update(&self, _: &Application) -> Result<(), alo_software::Failed> {
        panic!("nothing updates")
    }
    fn remove(&self, _: &Application) -> Result<(), alo_software::Failed> {
        panic!("nothing is removed")
    }
}

struct NoPublisher;

impl alo_publishing::Runs for NoPublisher {
    fn run(
        &self,
        _: &alo_capability::ACommand,
        _: &mut dyn alo_capability::AroundACommand,
    ) -> Result<alo_capability::WhatACommandDid, alo_capability::ItDidNotStart> {
        panic!("nothing publishes")
    }
}

#[test]
fn the_click_request_and_its_reader_click_where_the_real_turn_clicks() {
    // **The check's side**: the click step of the request, and a description
    // a model could give that the check takes.
    let set = the_set().expect("the set builds");
    let request = set
        .requests()
        .iter()
        .find(|request| {
            request.named() == "click by description in a window that describes nothing"
        })
        .expect("the set has the click request");
    let click = request
        .road()
        .iter()
        .find(|step| step.verb() == CLICK_AT)
        .expect("the road clicks");
    let application = click
        .expected()
        .iter()
        .find(|(argument, _)| *argument == APPLICATION)
        .map(|(_, is)| is.clone())
        .expect("the click names its application");
    let [(argument, words)] = click.described() else {
        panic!("the click is not held to one description: {click:?}")
    };
    assert_eq!(*argument, DESCRIPTION);
    let described = format!("the {words} button");
    assert!(alo_every_try::taking_a_step::a_description_names(
        &described, words
    ));

    // **The turn's side**: a whole machine, the person's choices, a grant.
    let strings = alo_every_try::words_for_the_answers::the_words().unwrap();
    let asked = Rc::new(RefCell::new(Vec::new()));
    let mut door = ADoor {
        asked: Rc::clone(&asked),
    };
    let mut windows = TheWindows;
    let publisher = NoPublisher;
    let mut bounding = NoBoundaries;
    let mut indicator = alo_egress::Indicator::default();
    let mut record = Record::default();
    let chosen = Chosen;
    let (thing, made, told) = {
        let machine = Machine::carrying_out_file_software_and_application_verbs(
            &strings,
            &alo_files::OnThisMachine,
            WhatASoftwareVerbNeeds {
                tool: &NoTool,
                bound: alo_software::Bound::Nobodys,
                publishing: WhatPublishingNeeds {
                    runs: &publisher,
                    through: alo_publishing::Through::on_github_com(),
                    rule: alo_egress::EgressPolicy::Anywhere,
                },
            },
            WhatAnApplicationVerbNeeds {
                installed: Installed::holding([Application::identified(&application).unwrap()])
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
        let mut machine = machine.with_the_screen_road(&chosen);
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(
                AGENT,
                Reach::Application(application.clone()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        let mut turning = Turning::beginning(
            alo_context::Context::at_invocation(noon()),
            AGENT,
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        let given = [
            (APPLICATION, Given::text(&application)),
            (DESCRIPTION, Given::text(&described)),
        ];
        let id = turning
            .proposing(CLICK_AT, &given, &grants, hour(), noon())
            .expect("the check's click is one the machine takes");
        let did = turning.approving(id, &grants, noon()).unwrap();
        let thing = did.thing_to_find().cloned().expect("a click to be found");
        let made = turning
            .clicking_where_it_was_found(Ok(WHERE_THE_RACE_STARTS.to_owned()))
            .unwrap();
        let made = made.click_made().cloned().expect("a click made");
        let told = Done::of_a_click(made.application(), made.description(), made.changed(), None);
        let _ = turning.ending(&mut grants);
        (thing, made, told)
    };

    assert_eq!(thing.application(), application);
    assert_eq!(thing.description(), described);
    // **Where the check's reader found it is where the door was asked to
    // click**, once, between a picture before and one after.
    let game = Application::identified(&application).unwrap();
    assert_eq!(
        *asked.borrow(),
        [
            AtTheDoor::Pictured(ToPicture {
                application: game.clone()
            }),
            AtTheDoor::Clicked(ToClick::at(game.clone(), 500, 400).unwrap()),
            AtTheDoor::Pictured(ToPicture { application: game }),
        ]
    );
    assert_eq!(made.at(), (50, 40));
    // **What the model is told is the check's shape of it**: a click in the
    // game, of what it described, and never where.
    let Answered::With(Done::Clicked(scripted)) = click.answered() else {
        panic!("the check does not answer the click as a click: {click:?}")
    };
    let Done::Clicked(real) = &told else {
        panic!("{told:?}")
    };
    assert_eq!(real.application, scripted.application);
    assert!(alo_every_try::taking_a_step::a_description_names(
        &real.description,
        &scripted.description
    ));
    let crossed = serde_json::to_string(&told).unwrap();
    assert!(
        !crossed.contains("\"50\"") && !crossed.contains(":50"),
        "{crossed}"
    );
}
