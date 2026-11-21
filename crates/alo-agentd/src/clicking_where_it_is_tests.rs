//! **Where a described thing is, asked of the reader the person chose**,
//! written before [`where_it_is`] was (law 6), against a service of a test's
//! own that hands back what it was sent:
//!
//! - a service chosen by name is sent the picture before the click **with**
//!   the shared question, naming the thing and the application as a person
//!   calls it, and its one line comes back;
//! - the person's own model cannot read pictures, and nobody chosen reads
//!   none: each is refused in words, and nothing is sent anywhere;
//! - every other answer goes on as it was.

use std::cell::RefCell;

use alo_capability::{PicturesAndKeys, WhoReadsPictures};
use alo_files::Answer;
use alo_models::SourcePolicy;
use alo_record::Record;
use alo_turn::{APictureTaken, AThingToFind, TheScreenRoadAsChosen, WhatAVerbDid};
use base64::Engine as _;

use super::{found_and_clicked, where_it_is};
use crate::questions::{Questions, TheBound, WhoseKeyring};
use crate::testing::{a_directory_of_our_own, in_english, noon, on_a_machine_that_answers};

/// The game.
const GAME: &str = "net.supertuxkart.SuperTuxKart";

/// What a person calls it.
const NAMED: &str = "SuperTuxKart";

/// What the model asked to be clicked.
const DESCRIBED: &str = "the Start race button";

/// A PNG's first bytes, then a marker.
const PNG: &[u8] = b"\x89PNG\r\n\x1a\nTHE-WINDOW-BEFORE-THE-CLICK";

/// The person's choice of reader.
struct Chosen(RefCell<WhoReadsPictures>);

impl TheScreenRoadAsChosen for Chosen {
    fn pictures_and_keys(&self) -> PicturesAndKeys {
        PicturesAndKeys::of([GAME])
    }

    fn who_reads_pictures(&self) -> WhoReadsPictures {
        self.0.borrow().clone()
    }
}

fn reading_by(reader: WhoReadsPictures) -> Chosen {
    Chosen(RefCell::new(reader))
}

/// A person whose settings add one service, `Here`, at `endpoint`.
fn a_person_who_added_a_service(what: &str, endpoint: &str) -> Questions {
    let config = a_directory_of_our_own(what);
    let folder = config.join(alo_choosing::THE_FOLDER);
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(
        folder.join(alo_choosing::THE_SETTINGS),
        format!(
            "format = 3\n\n[[provider]]\nname = \"Here\"\nendpoint = \"{endpoint}\"\nneeds-a-key = \
             false\nmodels = [\"a-model\"]\n"
        ),
    )
    .unwrap();
    Questions::of_a_session(
        Some(config.into_os_string()),
        None,
        alo_models::Catalogue::built_in().unwrap(),
        TheBound::ThePersons(SourcePolicy::Anywhere),
        WhoseKeyring::Nobodys,
    )
}

fn the_thing() -> AThingToFind {
    AThingToFind::to_find(
        GAME,
        NAMED,
        DESCRIBED,
        APictureTaken::drawn(GAME, PNG, 1280, 800),
    )
}

fn asked_under(
    questions: &mut Questions,
    road: &dyn TheScreenRoadAsChosen,
) -> Result<String, alo_strings::Said> {
    let mut record = Record::default();
    on_a_machine_that_answers(&mut record, |turning, _grants, strings| {
        where_it_is(&the_thing(), turning, questions, road, strings, noon())
    })
}

/// **The reader is sent the picture before the click with the shared
/// question**, and its one line comes back as it said it.
#[test]
fn a_service_chosen_by_name_is_asked_where_the_thing_is() {
    let (port, heard) = crate::testing::a_service_here_answering("1 50 40");
    let mut questions = a_person_who_added_a_service(
        "click-found-by-a-service",
        &format!("http://127.0.0.1:{port}"),
    );
    let road = reading_by(WhoReadsPictures::a_provider("Here").unwrap());

    let read = asked_under(&mut questions, &road);

    assert_eq!(read.as_ref().map(String::as_str), Ok("1 50 40"), "{read:?}");
    let sent = crate::testing::the_body_of(&heard.join().unwrap());
    let content = &sent["messages"][0]["content"];
    let asked = content[0]["text"].as_str().unwrap();
    let expected = alo_adapters::where_a_thing_is::the_question(NAMED, DESCRIBED, &in_english());
    assert_eq!(asked, expected.text(), "not the shared question: {sent}");
    assert_eq!(
        content[1]["image_url"]["url"].as_str().unwrap(),
        format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(PNG)
        ),
        "not the picture before the click"
    );
}

/// **A reader that cannot read pictures, and nobody chosen, are refused in
/// words**, and nothing is sent anywhere.
#[test]
fn a_reader_that_cannot_read_pictures_is_refused_and_nothing_is_sent() {
    let (at, came) = crate::testing::a_listener_that_reports_connections();
    let mut questions =
        a_person_who_added_a_service("click-found-by-nobody", &format!("http://{at}"));
    for (reader, word) in [
        (
            WhoReadsPictures::ThisComputer,
            crate::reading_a_picture_words::NOT_TO_THIS_COMPUTER_YET,
        ),
        (
            WhoReadsPictures::NoOne,
            alo_adapters::picture_words::NO_ONE_READS,
        ),
    ] {
        let read = asked_under(&mut questions, &reading_by(reader.clone()));
        let expected = in_english().say(
            &word.key(),
            &alo_strings::Filling::of("application", GAME.to_owned()),
        );
        assert_eq!(read, Err(expected), "{reader:?}");
    }
    assert!(!came.join().unwrap(), "something was sent");
}

/// **Every other answer goes on as it was**: nothing is asked of anybody.
#[test]
fn every_other_answer_goes_on_as_it_was() {
    let (at, came) = crate::testing::a_listener_that_reports_connections();
    let mut questions =
        a_person_who_added_a_service("click-found-not-a-click", &format!("http://{at}"));
    let road = reading_by(WhoReadsPictures::a_provider("Here").unwrap());
    let mut record = Record::default();
    let did = WhatAVerbDid::ToFiles(Answer::Read("hello".to_owned()));
    let went_on = on_a_machine_that_answers(&mut record, |turning, _grants, strings| {
        found_and_clicked(did.clone(), turning, &mut questions, &road, strings, noon())
    });
    assert_eq!(went_on.unwrap(), did);
    assert!(!came.join().unwrap(), "something was sent");
}
