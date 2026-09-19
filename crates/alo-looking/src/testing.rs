//! What this crate's own tests are written against.
//!
//! A place that answers whatever a test needs it to, the two builds every test
//! compares, and the vocabulary a sentence is read in. Compiled only for tests:
//! nothing that ships holds a stand-in.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::Cell;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_egress::Indicator;
use alo_keeping_up::{Digest, Offered, Running, Standing, a_check_at};
use alo_strings::{Strings, Vocabulary};

use crate::asking::ThePlace;
use crate::because::Because;
use crate::found::Found;
use crate::place::Place;
use crate::refusing::NoAnswer;
use crate::release::Release;

/// A whole build, from one repeated pair.
pub(crate) fn build(pair: &str) -> Digest {
    Digest::read(&format!("sha256:{}", pair.repeat(32))).unwrap()
}

/// A moment, handed to whatever needs one: nothing in this crate reads a clock.
pub(crate) fn a_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

/// The place this machine's build was pinned to come from.
pub(crate) fn the_place() -> Place {
    Place::on_this_machine().expect("the pin this repository ships")
}

/// This crate's own words and `alo-keeping-up`'s, which is every sentence
/// anything here can produce.
pub(crate) fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    crate::words::declare_into(&mut vocabulary).unwrap();
    alo_keeping_up::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A folder of this machine's own, emptied first.
pub(crate) fn a_folder(named: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-looking-{named}-{}", std::process::id()));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// An answer saying an update is ready, heard the only way one can be.
pub(crate) fn an_update(running: Digest, offered: Digest) -> Found {
    let standing = standing_between(&running, offered);
    assert!(standing.is_ready(), "the two builds given were the same");
    Found::of(running, standing, Because::ThePersonAsked, a_moment())
}

/// An answer saying this machine is up to date.
pub(crate) fn up_to_date(running: Digest) -> Found {
    let standing = standing_between(&running, running.clone());
    Found::of(running, standing, Because::ThePersonAsked, a_moment())
}

/// Where a machine running `running` stands against `offered`, with the offer
/// heard during a check that was shown — which is the only way to hear one.
fn standing_between(running: &Digest, offered: Digest) -> Standing {
    let mut indicator = Indicator::default();
    let underway =
        indicator.beginning_on_its_own(a_check_at(the_place().destination().clone()), a_moment());
    let offered = Offered::heard(&underway, offered).expect("an offer heard during a check");
    let standing = Standing::between(&Running::reported(running.clone()), &offered);
    indicator.ended_on_its_own(underway);
    standing
}

/// A place that answers whatever a test needs it to.
#[derive(Debug)]
pub(crate) struct APlaceThatAnswers {
    /// Every name it holds.
    names: Vec<String>,
    /// What it says each release is.
    builds: Vec<(String, String)>,
    /// What it refuses with instead of answering at all.
    refusing: Option<NoAnswer>,
    /// How often it has been asked anything.
    asked: Cell<usize>,
}

impl APlaceThatAnswers {
    /// A place holding these names and saying nothing about any of them yet.
    pub(crate) fn holding(names: &[&str]) -> Self {
        Self {
            names: names.iter().map(|&name| name.to_owned()).collect(),
            builds: Vec::new(),
            refusing: None,
            asked: Cell::new(0),
        }
    }

    /// A place nothing can reach.
    pub(crate) fn that_cannot_be_reached() -> Self {
        Self {
            names: Vec::new(),
            builds: Vec::new(),
            refusing: Some(NoAnswer::NoWayOut),
            asked: Cell::new(0),
        }
    }

    /// A place that answers in some other way.
    pub(crate) fn that_answers_with(refusal: NoAnswer) -> Self {
        Self {
            names: Vec::new(),
            builds: Vec::new(),
            refusing: Some(refusal),
            asked: Cell::new(0),
        }
    }

    /// The same place, saying these releases are these builds.
    pub(crate) fn whose_builds(mut self, builds: &[(&str, Digest)]) -> Self {
        self.builds = builds
            .iter()
            .map(|(release, build)| ((*release).to_owned(), build.as_str().to_owned()))
            .collect();
        self
    }

    /// The same place, saying these releases are named these things — which
    /// need not be builds at all.
    pub(crate) fn whose_names(mut self, named: &[(&str, &str)]) -> Self {
        self.builds = named
            .iter()
            .map(|(release, named)| ((*release).to_owned(), (*named).to_owned()))
            .collect();
        self
    }

    /// How often it has been asked anything.
    pub(crate) fn how_often_it_was_asked(&self) -> usize {
        self.asked.get()
    }
}

impl ThePlace for APlaceThatAnswers {
    fn every_name(&self) -> Result<Vec<String>, NoAnswer> {
        self.asked.set(self.asked.get() + 1);
        self.refusing.map_or_else(|| Ok(self.names.clone()), Err)
    }

    fn the_build_of(&self, release: &Release) -> Result<String, NoAnswer> {
        self.asked.set(self.asked.get() + 1);
        if let Some(refusal) = self.refusing {
            return Err(refusal);
        }
        self.builds
            .iter()
            .find(|(named, _)| named == release.named_as())
            .map(|(_, build)| build.clone())
            .ok_or(NoAnswer::ItRefused)
    }
}
