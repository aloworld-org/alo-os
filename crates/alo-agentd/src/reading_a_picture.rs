//! **A picture alo took, put to the reader the person chose, and only its words
//! sent on** — the screen road's reading
//! ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md)
//! §4, and the owner's *"It will always depend on the user"*).
//!
//! `accessible.picture_window` answers with the picture (`alo_turn::APictureTaken`).
//! **It never crosses to a client.** Before the answer is written, this file
//! has the picture read by whoever the person chose under *Who reads the
//! pictures*, read from their file **now**, and the client is told the size of
//! the picture and the reader's words — or why it was not read.
//!
//! # Who reads it, and why it is not the turn's own model
//!
//! The person chose two things separately: who answers their questions, and who
//! reads pictures of their windows. A picture attached to the turn's next
//! question would go to the first, which is a choice about words, and could send
//! a picture to a service the person never chose for one. So the picture goes
//! to the reader alone, with [`words::WHAT_THE_READER_IS_ASKED`], and the turn's
//! model reads the reader's words like any other answer.
//!
//! | the file says | what happens |
//! |---|---|
//! | no one | not read: alo cannot see inside the application (the verb refused this already; this covers a choice changed in between) |
//! | your model on this computer | not read, and nothing sent: alo cannot yet hand the runtime a picture |
//! | a service, by name | the service the person added under that name, asked through the road every question takes: the rule, the road out, the key, the indicator and the record |
//!
//! **The picture leaves only on that last row**, and only through
//! `alo_turn::Turning::reading_a_picture`, which shows the departure on the
//! indicator as it happens and writes it in the record afterwards (law 1).

use std::time::SystemTime;

use alo_capability::WhoReadsPictures;
use alo_choosing::Settings;
use alo_protocol::Done;
use alo_strings::{Filling, Said, Strings};
use alo_turn::{APictureTaken, Answers, Places, TheScreenRoadAsChosen, Turning, WhatAVerbDid};

use crate::a_provider_ready_to_be_asked::{
    NotPrepared, ThePlaceItIsPut, a_provider_ready_to_be_asked,
};
use crate::doing::{a_rule_refused, nothing_answered, said_about};
use crate::questions::Questions;
use crate::reading_a_picture_words as words;

/// The name the reader's sentences fill with the service's name.
const PROVIDER: &str = "provider";

/// The name every sentence here fills with the application.
const APPLICATION: &str = "application";

/// What a client is told a picture verb did: the reader's words, or why there
/// are none. [`None`] for every other verb, which is told as it always was.
pub fn read_if_a_picture(
    did: &WhatAVerbDid,
    turning: &mut Turning<'_, '_>,
    questions: &mut Questions,
    road: &dyn TheScreenRoadAsChosen,
    strings: &Strings,
    now: SystemTime,
) -> Option<Done> {
    let taken = did.picture_taken()?;
    let read = reading(taken, turning, questions, road, strings, now).map_err(Said::into_text);
    Some(Done::of_a_picture(taken.application(), taken.size(), read))
}

/// The picture, put to its reader: the words, or the sentence saying why not.
fn reading(
    taken: &APictureTaken,
    turning: &mut Turning<'_, '_>,
    questions: &mut Questions,
    road: &dyn TheScreenRoadAsChosen,
    strings: &Strings,
    now: SystemTime,
) -> Result<String, Said> {
    let application = Filling::of(APPLICATION, taken.application().to_owned());
    let name = match road.who_reads_pictures() {
        WhoReadsPictures::NoOne => {
            return Err(strings.say(
                &alo_adapters::picture_words::NO_ONE_READS.key(),
                &application,
            ));
        }
        WhoReadsPictures::ThisComputer => {
            return Err(strings.say(&words::NOT_TO_THIS_COMPUTER_YET.key(), &application));
        }
        WhoReadsPictures::AProvider(name) => name,
    };
    let named = application.clone().and(PROVIDER, name.clone());

    // **The service the person added under that name**, from their own
    // settings, read now — and the model they chose for it where it is the one
    // that answers their questions, or else the first it lists.
    let settings = questions
        .where_the_settings_are()
        .and_then(|at| Settings::at(&at).ok());
    let Some(settings) = settings else {
        return Err(strings.say(&words::THE_READER_IS_NOT_ADDED.key(), &named));
    };
    let Some(provider) = settings.providers().get(&name).cloned() else {
        return Err(strings.say(&words::THE_READER_IS_NOT_ADDED.key(), &named));
    };
    let chosen_for_questions = settings
        .chosen()
        .filter(|chosen| chosen.provider() == Some(name.as_str()))
        .map(|chosen| chosen.model().to_owned());
    let Some(model) = chosen_for_questions.or_else(|| provider.models.first().cloned()) else {
        return Err(strings.say(&words::THE_READER_HAS_NO_MODEL.key(), &named));
    };

    let asked = strings
        .say(&words::WHAT_THE_READER_IS_ASKED.key(), &application)
        .into_text();
    let policy = questions.the_rule();
    let by_an_organisation = questions.by_an_organisation();
    let the_road_out = questions.road_out();
    let keyring = questions.whose_keyring();
    let places = Places::under(&policy);
    let png = taken.shared_png();
    let prepared = a_provider_ready_to_be_asked(
        &provider,
        &policy,
        &the_road_out,
        &keyring,
        |permission, put| {
            let answers = match put {
                ThePlaceItIsPut::AProvider(hosted) => Answers::Provider(hosted),
                ThePlaceItIsPut::AServiceHere(served) => Answers::Service(served),
            };
            turning.reading_a_picture(&asked, png, &model, permission, &answers, &places, now)
        },
    );
    match prepared {
        Ok(Ok(answer)) => Ok(answer.text().to_owned()),
        Ok(Err(why)) => Err(nothing_answered(&why, strings)),
        // **A rule's refusal is written down**, as a turn's question refused
        // by one is (`crate::doing::refused_by_a_rule`).
        Err(NotPrepared::ARuleRefused(why)) => {
            let said = a_rule_refused(&why, by_an_organisation, strings);
            match turning.a_question_that_went_nowhere(&said, now) {
                Ok(()) => Err(said),
                Err(not_recorded) => Err(not_recorded.said(strings)),
            }
        }
        Err(NotPrepared::NoRoadOut(why)) => Err(why.said(strings)),
        Err(NotPrepared::TheKeyIsNotStored(why)) => {
            Err(strings.say(&said_about(why).key(), &Filling::nothing()))
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "reading_a_picture_tests.rs"]
mod tests;
