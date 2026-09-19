//! Where this machine asks whether there is an update — read, never guessed.
//!
//! `image/pinned.toml` is the one file in this repository that names where a
//! build of alo OS comes from and which release was signed, and `alo-image`
//! is the one crate that reads it (ADR 0033 §3, ADR 0036). So this file holds
//! **no registry**: [`Place::on_this_machine`] is that pin, read through
//! `alo_image::ThePin`, and a second spelling of the address here is exactly
//! the drift that made a pin one file in the first place.
//!
//! # Why the pin is compiled in rather than read off the disk
//!
//! The pin names the image, so it cannot be inside it — `crate::pinned` in
//! `alo-image` says so, and the installer's boot environment is handed a copy
//! because it is a different image from the one it installs. A running alo OS
//! machine has no such copy, and the honest place for a fact that is fixed when
//! the build is made is the build. So the text is taken at compile time and the
//! machine carries the pin of the release its own build was cut from.
//!
//! # What the pin gives, and what it does not
//!
//! | From the pin | What it is used for |
//! |---|---|
//! | `registry` | the repository asked, and the place shown on the indicator |
//! | `version` | the oldest release this machine will be offered |
//!
//! The **oldest** rather than the only one. A machine cannot carry the name of
//! a release that did not exist when it was built, so the place it asks is the
//! only thing that can tell it one exists — and the pin's own release is then
//! the floor rather than the answer: a place offering something older than the
//! release this build was cut from is offering a way backwards, and going
//! backwards is `alo_keeping_up::GoingBack`'s, which a person chooses
//! deliberately and is told the consequences of.
//!
//! The pin's `digest` is deliberately **not** read here. What this machine runs
//! is asked of the base at the moment it is wanted (`alo_updating::running`),
//! never remembered, and a digest written into a binary would be a second
//! answer to that question that is wrong on every machine that has updated
//! once.

use alo_egress::{Destination, DestinationError};
use alo_image::{NotPinned, ThePin};
use alo_keeping_up::{NotASource, Source};

use crate::release::{NotARelease, Release};

/// The pin this repository publishes, as it stood when this was built.
///
/// `image/pinned.toml` itself, rather than a copy: one file, one answer.
const THE_PIN_AS_BUILT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../image/pinned.toml"
));

/// Where this machine asks whether there is an update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// The repository, as the base would be told it.
    source: Source,
    /// The same place, as a person reads it on the indicator.
    destination: Destination,
    /// The oldest release this machine will be offered.
    not_before: Release,
}

/// Why some pin is not a place this machine can ask.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum NotAPlace {
    /// The pin is not a pin.
    #[error(transparent)]
    NotAPin(#[from] NotPinned),
    /// The pin names something that is not a repository a build can come from.
    #[error("the pin does not name a repository updates can come from: {0:?}")]
    NotARepository(NotASource),
    /// It names a host that could not be shown to a person on one line.
    #[error("the pin names a place that cannot be shown on the indicator: {0:?}")]
    NotSomewhereToShow(DestinationError),
    /// It names a release that is not one.
    #[error("the pin does not name a release: {0}")]
    NotARelease(NotARelease),
}

impl Place {
    /// The place this machine's build was pinned to come from.
    ///
    /// # Errors
    /// [`NotAPlace`]. It cannot happen on a machine that shipped — the test at
    /// the bottom of this file runs it against the pin this repository holds,
    /// and `alo-image` refuses a malformed pin long before this does.
    pub fn on_this_machine() -> Result<Self, NotAPlace> {
        Self::the_pin_names(&ThePin::read(THE_PIN_AS_BUILT)?)
    }

    /// The place some pin names — the one above, or a test's.
    ///
    /// # Errors
    /// [`NotAPlace`], naming which half of the pin could not be used.
    pub fn the_pin_names(pin: &ThePin) -> Result<Self, NotAPlace> {
        let source = Source::named(pin.registry()).map_err(NotAPlace::NotARepository)?;
        let host = host_of(&source);
        let destination = Destination::at(host).map_err(NotAPlace::NotSomewhereToShow)?;
        let not_before = Release::named(pin.version()).map_err(NotAPlace::NotARelease)?;
        Ok(Self {
            source,
            destination,
            not_before,
        })
    }

    /// The repository, as `alo_keeping_up::Staging` would be handed it.
    ///
    /// The same value the update is later staged from, so *where it was
    /// checked* and *where it was fetched* cannot come apart.
    #[must_use]
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// The place a person reads on the indicator while the check happens.
    #[must_use]
    pub fn destination(&self) -> &Destination {
        &self.destination
    }

    /// The oldest release this machine will be offered.
    #[must_use]
    pub fn not_before(&self) -> &Release {
        &self.not_before
    }

    /// The host, for the road out and the request.
    #[must_use]
    pub fn host(&self) -> &str {
        host_of(&self.source)
    }

    /// The repository beneath the host, with no leading slash.
    #[must_use]
    pub fn repository(&self) -> &str {
        self.source
            .as_str()
            .split_once('/')
            .map_or("", |(_, path)| path)
    }
}

/// The host half of a repository.
///
/// A `Source` is `host[:port]/path` and cannot be made without one, so the
/// whole string is the host only for a value that could not exist.
fn host_of(source: &Source) -> &str {
    source
        .as_str()
        .split_once('/')
        .map_or(source.as_str(), |(host, _)| host)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **The place is the pin's, and this file names no address of its own.**
    #[test]
    fn the_place_this_machine_asks_is_the_one_the_pin_names() {
        let pin = ThePin::read(THE_PIN_AS_BUILT).unwrap();
        let place = Place::on_this_machine().unwrap();

        assert_eq!(place.source().as_str(), pin.registry());
        assert_eq!(place.not_before().named_as(), pin.version());
        assert!(pin.registry().starts_with(place.host()));
        assert!(pin.registry().ends_with(place.repository()));
        assert_eq!(
            format!("{}/{}", place.host(), place.repository()),
            pin.registry()
        );
    }

    /// **The whole build is never named here.** What this machine runs is asked
    /// of the base at the moment it is wanted, and a digest compiled into a
    /// binary would be a second answer that is wrong on every machine that has
    /// updated once.
    #[test]
    fn the_place_carries_no_build() {
        let place = Place::on_this_machine().unwrap();
        let written = format!("{place:?}");
        assert!(!written.contains("sha256"), "{written}");
        assert!(!place.source().as_str().contains('@'), "{place:?}");
    }

    /// **A pin naming something that is not a repository is refused**, rather
    /// than becoming a place with a malformed address in it. The pin itself
    /// cannot say this — `alo-image` refuses most of it first — so the refusal
    /// is reached through the pin a test writes.
    #[test]
    fn a_pin_that_does_not_name_a_repository_is_not_a_place() {
        let refused = Place::the_pin_names(&a_pin_whose_registry_is("place.example")).unwrap_err();
        assert!(
            matches!(refused, NotAPlace::NotARepository(NotASource::NoRepository)),
            "{refused}"
        );

        let refused =
            Place::the_pin_names(&a_pin_whose_registry_is("PLACE.EXAMPLE/a/b")).unwrap_err();
        assert!(
            matches!(refused, NotAPlace::NotARepository(NotASource::NotAName)),
            "{refused}"
        );

        let refused =
            Place::the_pin_names(&a_pin_whose_registry_is("place.example/a/b:0.0.2")).unwrap_err();
        assert!(
            matches!(refused, NotAPlace::NotARepository(NotASource::NamesABuild)),
            "{refused}"
        );
    }

    /// **Text that is not a pin is not a place**, and says which.
    #[test]
    fn text_that_is_not_a_pin_is_not_a_place() {
        let refused = ThePin::read("not toml at all").unwrap_err();
        let refused = NotAPlace::from(refused);
        assert!(matches!(refused, NotAPlace::NotAPin(_)), "{refused}");
    }

    /// The pin this repository ships, with its registry replaced.
    fn a_pin_whose_registry_is(registry: &str) -> ThePin {
        let written = THE_PIN_AS_BUILT
            .lines()
            .map(|line| {
                if line.starts_with("registry = ") {
                    format!("registry = \"{registry}\"")
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<String>>()
            .join("\n");
        ThePin::read(&written).unwrap()
    }
}
