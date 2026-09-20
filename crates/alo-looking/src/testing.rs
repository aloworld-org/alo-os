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
use alo_image::ThePin;
use alo_keeping_up::{Digest, Offered, Running, Standing, Vouching, a_check_at};
use alo_strings::{Strings, Vocabulary};

use crate::asking::ThePlace;
use crate::because::Because;
use crate::found::Found;
use crate::place::{Place, THE_PIN_AS_BUILT};
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

/// A place whose floor is the release named here, rather than whichever one
/// this repository happens to be pinned at today.
///
/// **A test that means *a release this machine would take* says which one.**
/// The tests in [`crate::looking`] are about which of the releases a place
/// holds is offered and what is made of the answer; not one of them is about
/// the number in `image/pinned.toml`. Taking their floor from the shipped pin
/// tied every one of them to the release process, and on 2026-09-19 that came
/// due: `0.0.3` was pinned, and a test whose place held `0.0.2` — as every
/// machine built before that release does — began answering *nothing is
/// offered*. [`the_place`] stays for the tests that are about the shipped pin
/// itself, which are the only ones that should move when it does.
pub(crate) fn a_place_not_before(release: &str) -> Place {
    Place::the_pin_names(&the_pin_where("version", release)).expect("a pin naming a place")
}

/// The pin this repository ships, with one `field = "value"` line replaced.
///
/// The pin's own reader over the pin's own text: a pin written out here would
/// be the second spelling of the address `crate::place` exists so that nothing
/// carries.
pub(crate) fn the_pin_where(field: &str, value: &str) -> ThePin {
    let opening = format!("{field} = ");
    let written = THE_PIN_AS_BUILT
        .lines()
        .map(|line| {
            if line.starts_with(&opening) {
                format!("{opening}\"{value}\"")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");
    ThePin::read(&written).expect("the shipped pin with one line replaced")
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

/// An answer saying an update is ready, for a build the place vouched for.
pub(crate) fn an_update(running: Digest, offered: Digest) -> Found {
    an_update_vouched(running, offered, Vouching::ThePlaceVouchesForIt)
}

/// An answer saying an update is ready, for a build nothing vouched for.
pub(crate) fn an_update_nobody_vouched_for(running: Digest, offered: Digest) -> Found {
    an_update_vouched(running, offered, Vouching::NobodyHasVouchedForIt)
}

/// An answer saying an update is ready, heard the only way one can be.
fn an_update_vouched(running: Digest, offered: Digest, vouching: Vouching) -> Found {
    let standing = standing_between(&running, offered, vouching);
    assert!(standing.is_ready(), "the two builds given were the same");
    Found::of(running, standing, Because::ThePersonAsked, a_moment())
}

/// An answer saying this machine is up to date.
pub(crate) fn up_to_date(running: Digest) -> Found {
    let standing = standing_between(&running, running.clone(), Vouching::ThePlaceVouchesForIt);
    Found::of(running, standing, Because::ThePersonAsked, a_moment())
}

/// Where a machine running `running` stands against `offered`, with the offer
/// heard during a check that was shown — which is the only way to hear one.
fn standing_between(running: &Digest, offered: Digest, vouching: Vouching) -> Standing {
    let mut indicator = Indicator::default();
    let underway =
        indicator.beginning_on_its_own(a_check_at(the_place().destination().clone()), a_moment());
    let offered =
        Offered::heard(&underway, offered, vouching).expect("an offer heard during a check");
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

    /// The same place, also holding a signature for each of these builds —
    /// under the name `crate::vouching` says one is held under.
    pub(crate) fn also_vouching_for(mut self, builds: &[Digest]) -> Self {
        self.names
            .extend(builds.iter().map(crate::vouching::the_name_vouching_for));
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

/// What a check was written into, for a test that wants to read it back.
///
/// The real one on a machine writes `alo_record::Entry::left_on_its_own` into
/// the machine's record; this keeps what that entry would be made from, so
/// that this crate's tests can say *one check, one departure written down*
/// without taking a dependency on the record to do it.
#[derive(Debug, Default)]
pub(crate) struct WhatWasWrittenDown {
    /// Each check that left, as the errand and the place it was made to.
    departures: Vec<(alo_egress::Errand, alo_egress::Destination)>,
}

impl WhatWasWrittenDown {
    /// Every departure written down, oldest first.
    pub(crate) fn departures(&self) -> &[(alo_egress::Errand, alo_egress::Destination)] {
        &self.departures
    }

    /// How many were written down.
    pub(crate) fn how_many(&self) -> usize {
        self.departures.len()
    }
}

impl crate::noting::Noting for WhatWasWrittenDown {
    fn the_check_left(&mut self, underway: &alo_egress::Underway) {
        self.departures
            .push((underway.errand(), underway.destination().clone()));
    }
}
