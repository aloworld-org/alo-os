//! The strings, the uses and the records this crate's own tests are written
//! against.
//!
//! Every file here that says something has the same two questions to answer —
//! *what does this say on a machine with no translations* and *what does it say
//! when somebody has translated it* — and answering them from one fixture is
//! what stops the files inventing vocabularies that resemble the real one. The
//! real one is [`crate::in_use_words`], and both of these are built from it.
//! The shape is `alo-indicator`'s `testing.rs`, copied rather than re-decided.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! # It is not only this crate's words
//!
//! A line about an application puts that application's name inside its
//! sentence, and the name and the identifier are put together by
//! `alo-applications`' own string. A fixture holding only this crate's list
//! would answer that with a key in guillemets and the tests about lines would
//! still pass, which is a fixture proving the tests rather than the code.
//!
//! # And the records are records
//!
//! [`a_record_of`] writes the shape the rented media server writes, so a test
//! about reading one is a test about reading what a machine would hand over
//! rather than about a shape invented here to be easy to parse.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_applications::Application;
use alo_capability::Grantee;
use alo_strings::{Language, Strings, Translation, Vocabulary};

use crate::by::By;
use crate::refusing::NotHeard;
use crate::streams::Streams;
use crate::used::Used;
use crate::uses::{Use, UseId};
use crate::words::{Word, declare_into};

/// Everything this crate says, and everything it puts inside a line beside it.
fn its_own_and_what_it_quotes() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).unwrap();
    alo_applications::words::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// This crate's own words, with nothing translated: what a machine that has no
/// translations of them shows, which is what most of these tests are about.
pub(crate) fn in_english() -> Strings {
    Strings::of(its_own_and_what_it_quotes())
}

/// The same, with some of these words translated into German and German
/// preferred — German because it is the language the rest of this repository
/// tests translation with, so a translator's file exercised here looks like the
/// one exercised everywhere else.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = its_own_and_what_it_quotes();
    let mut german = Translation::into_language(german_language());
    for (word, says) in words {
        german = german.says(word.key(), *says);
    }
    let speaking = vocabulary.check(german).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german_language()]);
    strings
}

/// German, as `alo-strings` names a language.
fn german_language() -> Language {
    Language::written("de").unwrap()
}

/// The agent every test here is written about.
pub(crate) fn the_agent() -> Grantee {
    Grantee::named("@alo")
}

/// A video-call application, which is the plan's own example of something that
/// uses the camera.
pub(crate) fn a_video_call() -> Application {
    Application::called("com.example.VideoCall", "Video Call").unwrap()
}

/// The camera, used by somebody, under this number.
pub(crate) fn the_camera_by(by: By, number: u32) -> Use {
    Use::of(UseId::recorded(number), Used::Camera, by)
}

/// One use of each shape there is: an application, the agent, alo OS itself and
/// something the machine cannot name.
///
/// Four, so that a test walking them cannot quietly skip the one that matters —
/// which is whichever one somebody forgot.
pub(crate) fn every_shape_of_use() -> Vec<Use> {
    vec![
        the_camera_by(By::an_application(a_video_call()), 1),
        Use::of(
            UseId::recorded(2),
            Used::Screen,
            By::the_agent(&the_agent()).unwrap(),
        ),
        Use::of(UseId::recorded(3), Used::Microphone, By::alo_os_itself()),
        Use::of(
            UseId::recorded(4),
            Used::Microphone,
            By::something_on_this_machine(),
        ),
    ]
}

/// A media server answering with these uses.
pub(crate) fn a_server_answering(uses: Vec<Use>) -> AServer {
    AServer(Ok(uses))
}

/// A media server that cannot be read, for this reason.
pub(crate) fn a_server_refusing(why: NotHeard) -> AServer {
    AServer(Err(why))
}

/// A media server with one answer in it.
pub(crate) struct AServer(Result<Vec<Use>, NotHeard>);

impl Streams for AServer {
    fn in_use_now(&mut self) -> Result<Vec<Use>, NotHeard> {
        self.0.clone()
    }
}

/// A record of the graph, the way the rented media server writes one.
pub(crate) fn a_record_of(objects: &[String]) -> String {
    format!("[{}]", objects.join(","))
}

/// One node in such a record.
pub(crate) fn a_recorded_node(
    number: u32,
    class: &str,
    state: &str,
    props: &[(&str, &str)],
) -> String {
    let mut written = vec![format!("\"media.class\": \"{class}\"")];
    for (named, says) in props {
        written.push(format!("\"{named}\": \"{says}\""));
    }
    format!(
        "{{\"id\": {number}, \"type\": \"PipeWire:Interface:Node\", \"info\": {{\"state\": \
         \"{state}\", \"props\": {{{}}}}}}}",
        written.join(",")
    )
}

/// One link in such a record, joining one node's output to another's input.
pub(crate) fn a_recorded_link(number: u32, from: u32, to: u32) -> String {
    format!(
        "{{\"id\": {number}, \"type\": \"PipeWire:Interface:Link\", \"info\": {{\"props\": \
         {{\"link.output.node\": {from}, \"link.input.node\": {to}}}}}}}"
    )
}
