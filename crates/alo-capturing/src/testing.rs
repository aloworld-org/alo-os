//! The strings, the folders, the mechanisms and the grants this crate's own
//! tests are written against.
//!
//! Every file here that says something has the same two questions to answer —
//! *what does this say on a machine with no translations* and *what does it say
//! when somebody has translated it* — and answering them from one fixture is
//! what stops the files inventing vocabularies that resemble the real one. The
//! shape is `alo-in-use`'s `testing.rs`, copied rather than re-decided.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! # Three mechanisms, because three things can happen
//!
//! [`one_frame_of_the_screen`] hands a picture over, [`nothing_reads_the_screen`]
//! refuses, and [`counting_what_it_was_asked`] does the first while keeping
//! count — which is how *before anything is taken* stops being a claim about
//! the order of some lines and becomes a number a test can read.
//!
//! # And the folders are real folders
//!
//! [`an_empty_folder`] makes one on this host's disk and empties it first, so a
//! test about writing a file is a test about writing a file. The shape is
//! `alo-accounts`' and `alo-appearance`'s, which put their own fixtures under
//! the temporary directory with the running process's number in the name.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use alo_capability::{Applicant, Facility, Grant, Grants, Reach};
use alo_portals::{Portal, Refused, Request};
use alo_strings::{Language, Strings, Translation, Vocabulary};

use crate::folder::Folder;
use crate::grabs::{Grabs, NotGrabbed};
use crate::on_this_day::OnThisDay;
use crate::picture::Picture;
use crate::region::Region;
use crate::screen::Screen;
use crate::session::WhoseSession;
use crate::words::{Word, declare_into};

/// How long the grants in these tests last.
pub(crate) const THE_HOUR: Duration = Duration::from_secs(60 * 60);

/// Everything this crate says, and everything a refusal of its can quote.
///
/// A refused application reads `alo-capability`'s own sentence through
/// `alo-portals`, so a fixture holding only this crate's list would answer that
/// with a key in guillemets and the tests about refusals would still pass —
/// which is a fixture proving the tests rather than the code.
fn its_own_and_what_it_quotes() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).unwrap();
    alo_portals::declare_into(&mut vocabulary).unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// This crate's own words, with nothing translated: what a machine that has no
/// translations of them shows, which is what most of these tests are about.
pub(crate) fn in_english() -> Strings {
    Strings::of(its_own_and_what_it_quotes())
}

/// The same, with some of these words translated into German and German
/// preferred — German because it is the language the rest of this repository
/// tests translation with.
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

/// The person whose machine these tests are about.
pub(crate) fn anna() -> WhoseSession {
    WhoseSession::of("anna")
}

/// The screen these tests are taken on.
pub(crate) fn a_screen() -> Screen {
    Screen::measuring(1920, 1080).unwrap()
}

/// Noon on the sixteenth of September 2026, as a moment on a clock.
pub(crate) fn noon() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1_789_560_000)
}

/// The same moment, on the calendar a picture is named after.
pub(crate) fn a_moment() -> OnThisDay {
    OnThisDay::at(noon(), 0)
}

/// A picture, as the rented mechanism would hand one over.
pub(crate) fn a_picture() -> Picture {
    Picture::of(b"\x89PNG\r\n\x1a\nwhatever was on the screen".to_vec()).unwrap()
}

/// A folder somebody chose, without insisting it is on a disk.
pub(crate) fn a_folder_somebody_chose(what: &str) -> Folder {
    Folder::chosen(&where_a_fixture_goes(what)).unwrap()
}

/// The same, made on this host's disk and emptied first.
pub(crate) fn an_empty_folder(what: &str) -> Folder {
    let at = where_a_fixture_goes(what);
    if at.exists() {
        std::fs::remove_dir_all(&at).unwrap();
    }
    std::fs::create_dir_all(&at).unwrap();
    Folder::chosen(&at).unwrap()
}

/// Where a fixture of this name goes on this host.
fn where_a_fixture_goes(what: &str) -> PathBuf {
    std::env::temp_dir().join(format!("alo-capturing-{what}-{}", std::process::id()))
}

/// A screen-capture mechanism that hands one frame over.
pub(crate) fn one_frame_of_the_screen() -> impl Grabs {
    Counting {
        answering: Ok(()),
        how_often: 0,
    }
}

/// A screen-capture mechanism that is not on this machine.
pub(crate) fn nothing_reads_the_screen() -> impl Grabs {
    Counting {
        answering: Err(NotGrabbed::NothingReadsTheScreen {
            said: "nothing on this machine reads the screen".to_owned(),
        }),
        how_often: 0,
    }
}

/// A mechanism that hands a frame over and keeps count of how often it was
/// asked.
pub(crate) fn counting_what_it_was_asked() -> Counting {
    Counting {
        answering: Ok(()),
        how_often: 0,
    }
}

/// A screen-capture mechanism a test can watch.
#[derive(Debug)]
pub(crate) struct Counting {
    /// What it answers with, the same way every time.
    answering: Result<(), NotGrabbed>,
    /// How many times it has been asked for a picture.
    how_often: usize,
}

impl Counting {
    /// How many times it has been asked for a picture.
    pub(crate) const fn how_often(&self) -> usize {
        self.how_often
    }
}

impl Grabs for Counting {
    fn grab(&mut self, across: Region, on: Screen) -> Result<Picture, NotGrabbed> {
        assert!(
            across.is_on(on),
            "the mechanism was asked for {across:?}, which is not on {on:?}"
        );
        self.how_often = self.how_often.saturating_add(1);
        match &self.answering {
            Ok(()) => Ok(a_picture()),
            Err(why) => Err(why.clone()),
        }
    }
}

/// A machine on which nothing has been granted to anybody.
pub(crate) fn nothing_granted() -> Grants {
    Grants::default()
}

/// A machine on which this application was granted one picture of the screen,
/// at [`noon`], for [`THE_HOUR`].
pub(crate) fn granted_a_screenshot(application: &str) -> Grants {
    granted(application, Facility::ScreenOnce)
}

/// The same, for the camera — which is a grant this machine has and is not a
/// grant to photograph the screen.
pub(crate) fn granted_the_camera(application: &str) -> Grants {
    granted(application, Facility::Camera)
}

/// A machine on which this application was granted this facility.
fn granted(application: &str, facility: Facility) -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked_for(
            &Applicant::named(application).grantee(),
            Reach::Facility(facility),
            noon(),
            THE_HOUR,
        )
        .unwrap(),
    );
    grants
}

/// An application refused a picture of the screen, as `alo-portals` refuses
/// one.
///
/// Built by asking the real question of a real machine with nothing granted on
/// it, rather than by constructing the refusal here: a fixture that assembled
/// somebody else's refusal would stop being their refusal the day they changed
/// it.
pub(crate) fn refused_a_screenshot() -> Refused {
    Request::of("com.example.Stranger", Portal::Screenshot)
        .unwrap()
        .judged(&nothing_granted(), noon())
        .unwrap_err()
}
