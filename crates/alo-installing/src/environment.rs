//! What the recipe put inside the environment: the pin and the key.
//!
//! `image/installing/Containerfile` copies `image/pinned.toml` and the public
//! key it names into [`WHERE_IT_IS`], at the same relative paths they have in
//! `image/`, and `crates/alo-image` holds the recipe to that. So the environment
//! carries exactly one release it will install and exactly one key it will
//! accept, and neither can arrive over the network.
//!
//! If either is missing or unreadable, the environment is damaged and says so
//! before it reads anything else. It never falls back to a release or a key it
//! was not built with, because there is no such thing to fall back to.

use std::path::{Path, PathBuf};

use alo_image::ThePin;

use crate::ended::Refusal;

/// Where the recipe puts the pin and the key, inside the environment.
pub const WHERE_IT_IS: &str = "/usr/lib/alo/installing";

/// The pin and the key, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    /// The one release this environment installs.
    pin: ThePin,
    /// Where the one key it accepts is.
    key: PathBuf,
}

impl Environment {
    /// The environment beneath `inside`, given the text of its pin and a way to
    /// ask whether a file is there to read.
    ///
    /// # Errors
    /// [`Refusal::Damaged`] when the pin could not be read, is not a pin, or
    /// names a key that is not there.
    pub fn read(
        inside: &Path,
        pinned: std::io::Result<String>,
        is_there: impl FnOnce(&Path) -> bool,
    ) -> Result<Self, Refusal> {
        let pin = pinned
            .ok()
            .and_then(|pinned| ThePin::read(&pinned).ok())
            .ok_or(Refusal::Damaged)?;
        let key = inside.join(pin.key());
        if !is_there(&key) {
            return Err(Refusal::Damaged);
        }
        Ok(Self { pin, key })
    }

    /// The release.
    #[must_use]
    pub fn pin(&self) -> &ThePin {
        &self.pin
    }

    /// The key.
    #[must_use]
    pub fn key(&self) -> &Path {
        &self.key
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The pin this repository ships, as text.
    fn the_pin() -> String {
        std::fs::read_to_string(Path::new(alo_image::THE_IMAGE).join(alo_image::THE_PIN)).unwrap()
    }

    /// The shipped pin, with its key there, is an environment whose key is
    /// beside it.
    #[test]
    fn the_shipped_pin_with_its_key_is_an_environment() {
        let environment =
            Environment::read(Path::new(WHERE_IT_IS), Ok(the_pin()), |_| true).unwrap();
        assert_eq!(
            environment.key(),
            Path::new("/usr/lib/alo/installing/signing/alo-os.pub")
        );
        assert_eq!(environment.pin().registry(), "ghcr.io/aloworld-org/alo-os");
    }

    /// **A missing pin, a pin that is not one, or a missing key is damage.**
    #[test]
    fn a_missing_or_malformed_pin_or_key_is_damage() {
        let inside = Path::new(WHERE_IT_IS);
        assert_eq!(
            Environment::read(
                inside,
                Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
                |_| true
            ),
            Err(Refusal::Damaged)
        );
        assert_eq!(
            Environment::read(inside, Ok("registry = \"elsewhere\"".to_owned()), |_| true),
            Err(Refusal::Damaged)
        );
        assert_eq!(
            Environment::read(inside, Ok(the_pin()), |_| false),
            Err(Refusal::Damaged)
        );
    }
}
