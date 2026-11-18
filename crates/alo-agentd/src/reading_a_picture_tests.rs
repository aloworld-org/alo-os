//! **A picture put to the reader the person chose** (ADR 0147 §4), against a
//! service of a test's own that hands back what it was sent.
//!
//! - a service chosen by name is sent the picture **with** the question, and
//!   only its words go on — the picture is never in what the client is told;
//! - no one chosen, the person's own model, a service that is not added and one
//!   with no model are each refused in words, and nothing is sent anywhere;
//! - every other verb is told as it always was.

use std::cell::RefCell;

use alo_capability::{PicturesAndKeys, WhoReadsPictures};
use alo_files::Answer;
use alo_models::SourcePolicy;
use alo_protocol::{APictureRead, Done};
use alo_record::Record;
use alo_turn::{APictureTaken, TheScreenRoadAsChosen, WhatAVerbDid};
use base64::Engine as _;

use super::read_if_a_picture;
use crate::questions::{Questions, TheBound, WhoseKeyring};
use crate::testing::{a_directory_of_our_own, in_english, noon, on_a_machine_that_answers};

/// The game pictured.
const GAME: &str = "net.supertuxkart.SuperTuxKart";

/// A PNG's first bytes, then a marker no answer may carry.
const PNG: &[u8] = b"\x89PNG\r\n\x1a\nA-MARKER-ONLY-THE-READER-MAY-SEE";

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

/// A person whose settings add one service, `Here`, at `endpoint`, listing
/// `models`.
fn a_person_who_added_a_service(what: &str, endpoint: &str, models: &str) -> Questions {
    let config = a_directory_of_our_own(what);
    let folder = config.join(alo_choosing::THE_FOLDER);
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(
        folder.join(alo_choosing::THE_SETTINGS),
        format!(
            "format = 3\n\n[[provider]]\nname = \"Here\"\nendpoint = \"{endpoint}\"\nmodels = \
             [{models}]\n"
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

fn the_picture() -> WhatAVerbDid {
    WhatAVerbDid::from(APictureTaken::drawn(GAME, PNG, 1280, 800))
}

/// What the client is told, read under these choices.
fn told(questions: &mut Questions, road: &dyn TheScreenRoadAsChosen) -> Option<Done> {
    let mut record = Record::default();
    on_a_machine_that_answers(&mut record, |turning, _grants, strings| {
        read_if_a_picture(&the_picture(), turning, questions, road, strings, noon())
    })
}

fn the_picture_read(done: Option<Done>) -> APictureRead {
    match done {
        Some(Done::PictureRead(read)) => read,
        other => panic!("not a picture read: {other:?}"),
    }
}

/// **The service the person chose is sent the picture with the question**, as
/// the convention's data URL, and only its words go on.
#[test]
fn a_service_chosen_by_name_is_sent_the_picture_and_its_words_go_on() {
    let (port, heard) = crate::testing::a_service_here_answering("a menu: Start race at 0.5, 0.4");
    let mut questions = a_person_who_added_a_service(
        "picture-read-by-a-service",
        &format!("http://127.0.0.1:{port}"),
        "\"a-model\"",
    );
    let road = reading_by(WhoReadsPictures::a_provider("Here").unwrap());

    let read = the_picture_read(told(&mut questions, &road));

    assert_eq!(read.application, GAME);
    assert_eq!((read.wide, read.tall), (1280, 800));
    assert_eq!(read.read.as_deref(), Some("a menu: Start race at 0.5, 0.4"));
    assert_eq!(read.not_read, None);
    let sent = crate::testing::the_body_of(&heard.join().unwrap());
    assert_eq!(sent["model"], "a-model");
    let content = &sent["messages"][0]["content"];
    assert!(
        content[0]["text"].as_str().unwrap().contains(GAME),
        "the reader was not told which application: {sent}"
    );
    assert_eq!(
        content[1]["image_url"]["url"].as_str().unwrap(),
        format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(PNG)
        )
    );
    // **The picture is never in what the client is told.**
    let crossed = serde_json::to_string(&Done::PictureRead(read)).unwrap();
    assert!(!crossed.contains("A-MARKER"), "{crossed}");
    assert!(!crossed.contains("base64"), "{crossed}");
}

/// **Nobody reads, and the person's own model cannot be sent one yet**: each
/// refused in words, the picture not read, and nothing sent.
#[test]
fn no_one_and_this_computer_are_refused_in_words_and_send_nothing() {
    let (at, came) = crate::testing::a_listener_that_reports_connections();
    let mut questions =
        a_person_who_added_a_service("picture-read-by-nobody", &format!("http://{at}"), "\"m\"");
    for (reader, word) in [
        (
            WhoReadsPictures::NoOne,
            alo_adapters::picture_words::NO_ONE_READS,
        ),
        (
            WhoReadsPictures::ThisComputer,
            crate::reading_a_picture_words::NOT_TO_THIS_COMPUTER_YET,
        ),
    ] {
        let read = the_picture_read(told(&mut questions, &reading_by(reader)));
        assert_eq!(read.read, None);
        let expected = in_english()
            .say(
                &word.key(),
                &alo_strings::Filling::of("application", GAME.to_owned()),
            )
            .into_text();
        assert_eq!(read.not_read.unwrap(), expected);
    }
    assert!(!came.join().unwrap(), "something was sent");
}

/// **A service that is not added, or names no model, is refused by name**, and
/// nothing is sent.
#[test]
fn a_service_not_added_or_with_no_model_is_refused_by_name() {
    let (at, came) = crate::testing::a_listener_that_reports_connections();
    let mut no_model =
        a_person_who_added_a_service("picture-read-no-model", &format!("http://{at}"), "");
    let here = reading_by(WhoReadsPictures::a_provider("Here").unwrap());
    let why = the_picture_read(told(&mut no_model, &here))
        .not_read
        .unwrap();
    assert!(
        why.contains("“Here”") && why.contains("names no model"),
        "{why}"
    );

    let elsewhere = reading_by(WhoReadsPictures::a_provider("Lantern").unwrap());
    let why = the_picture_read(told(&mut no_model, &elsewhere))
        .not_read
        .unwrap();
    assert!(
        why.contains("“Lantern”") && why.contains("not among"),
        "{why}"
    );
    assert!(!came.join().unwrap(), "something was sent");
}

/// **Every other verb is told as it always was**: this reads only pictures.
#[test]
fn every_other_verb_is_not_a_picture() {
    let mut questions =
        a_person_who_added_a_service("picture-read-not-one", "http://127.0.0.1:9", "\"m\"");
    let road = reading_by(WhoReadsPictures::a_provider("Here").unwrap());
    let mut record = Record::default();
    let told = on_a_machine_that_answers(&mut record, |turning, _grants, strings| {
        read_if_a_picture(
            &WhatAVerbDid::ToFiles(Answer::Read("hello".to_owned())),
            turning,
            &mut questions,
            &road,
            strings,
            noon(),
        )
    });
    assert_eq!(told, None);
}
