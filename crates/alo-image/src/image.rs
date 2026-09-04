//! Everything the image declares, read off a directory.
//!
//! Five files, and the directory they sit in is laid out as the machine's own
//! root so that a reader can see where each of them lands. `crate::checking` is
//! what asks whether they agree with each other; this is only what they are.
//!
//! # Why the root is an argument
//!
//! [`Image::at`] takes the directory rather than knowing it, which is
//! `alo-agentd`'s `place.rs` argument and `alo-boundaryd`'s: a rule about files
//! is only a rule with a test if a test can be run against files it may write.
//! `crate::THE_IMAGE` is where alo OS's own are, and a test that breaks one of
//! them copies the directory somewhere it owns first.

use std::path::Path;

use crate::description::Description;
use crate::logins::Declared;
use crate::making::Made;
use crate::refusing::NotAnImage;
use crate::service::Service;
use crate::unit::Unit;

/// The unit that loads the boundary, as systemd names it.
pub const THE_LOADER: &str = "alo-boundaryd.service";

/// The unit that serves the agent, as systemd names it.
pub const THE_AGENT: &str = "alo-agentd.service";

/// Where a unit file goes, beneath the image's root.
const UNITS: &str = "usr/lib/systemd/system";

/// Where the directories made at boot are declared.
const TMPFILES: &str = "usr/lib/tmpfiles.d/alo.conf";

/// Where the logins made at boot are declared.
const SYSUSERS: &str = "usr/lib/sysusers.d/alo.conf";

/// Where the machine description goes.
const DESCRIPTION: &str = "etc/alo/agentd.toml";

/// What one image says about the machine it becomes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// The service that loads the boundary.
    loader: Service,
    /// The service that serves the agent.
    agent: Service,
    /// The directories made at boot.
    made: Vec<Made>,
    /// The logins and groups made at boot.
    declared: Vec<Declared>,
    /// What the machine says about itself.
    description: Description,
}

impl Image {
    /// The image laid out beneath this directory.
    ///
    /// # Errors
    ///
    /// [`NotAnImage`], which is one variant per file and carries the refusal the
    /// reader of that file made. Nothing is checked against anything else here —
    /// that is [`crate::everything_wrong_with`], and an image has to read before
    /// it can disagree with itself.
    pub fn at(root: &Path) -> Result<Self, NotAnImage> {
        let loader = service(root, THE_LOADER)?;
        let agent = service(root, THE_AGENT)?;

        let at = root.join(TMPFILES);
        let made = crate::making::everything_made(&text(&at)?)
            .map_err(|why| NotAnImage::NotMade { at, why })?;

        let at = root.join(SYSUSERS);
        let declared = crate::logins::every_login(&text(&at)?)
            .map_err(|why| NotAnImage::NotDeclared { at, why })?;

        let at = root.join(DESCRIPTION);
        let description =
            Description::read(&text(&at)?).map_err(|why| NotAnImage::NotDescribed { at, why })?;

        Ok(Self {
            loader,
            agent,
            made,
            declared,
            description,
        })
    }

    /// The service that loads the boundary, once at boot.
    #[must_use]
    pub const fn loader(&self) -> &Service {
        &self.loader
    }

    /// The service that serves the agent, as the signed-in person.
    #[must_use]
    pub const fn agent(&self) -> &Service {
        &self.agent
    }

    /// The directory this image makes at this path, if it makes one.
    #[must_use]
    pub fn directory_at(&self, path: &Path) -> Option<&Made> {
        self.made.iter().find(|it| it.is_a_directory_at(path))
    }

    /// The number this image gives a login by this name, if it makes one.
    #[must_use]
    pub fn login_called(&self, name: &str) -> Option<u32> {
        self.declared.iter().find_map(|it| it.login_called(name))
    }

    /// The number this image gives a group by this name, if it makes one.
    #[must_use]
    pub fn group_called(&self, name: &str) -> Option<u32> {
        self.declared.iter().find_map(|it| it.group_called(name))
    }

    /// Whether this image puts that login into that group.
    #[must_use]
    pub fn puts(&self, login: &str, into: &str) -> bool {
        self.declared.iter().any(|it| it.puts(login, into))
    }

    /// Whether this image makes any login with this number.
    #[must_use]
    pub fn makes_a_login_numbered(&self, number: u32) -> bool {
        self.declared.iter().any(|it| it.numbers_a_login(number))
    }

    /// What the machine says about itself.
    #[must_use]
    pub const fn description(&self) -> &Description {
        &self.description
    }
}

/// One unit file, read as a service.
fn service(root: &Path, called: &str) -> Result<Service, NotAnImage> {
    let at = root.join(UNITS).join(called);
    let unit = Unit::read(&text(&at)?).map_err(|why| NotAnImage::NotAUnit {
        at: at.clone(),
        why,
    })?;
    Service::of(called, unit).map_err(|why| NotAnImage::NotAService { why })
}

/// One of the image's files, as text.
fn text(at: &Path) -> Result<String, NotAnImage> {
    std::fs::read_to_string(at).map_err(|why| NotAnImage::Unreadable {
        at: at.to_owned(),
        why,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_copy_of_the_image;

    /// alo OS's own image reads, which is the first thing every other check
    /// depends on.
    #[test]
    fn the_image_this_repository_ships_reads() {
        let image = Image::at(Path::new(crate::THE_IMAGE)).unwrap();

        assert_eq!(image.loader().called(), THE_LOADER);
        assert_eq!(image.agent().called(), THE_AGENT);
        assert!(image.directory_at(Path::new("/run/alo")).is_some());
        assert!(image.directory_at(Path::new("/var/lib/alo")).is_some());
        assert_eq!(image.login_called("alo"), Some(1000));
        assert_eq!(image.group_called("alo-agent"), Some(60989));
        assert!(image.puts("alo", "alo-agent"));
    }

    /// **A missing file is a missing file**, named, rather than an image that
    /// reads with one thing absent from it.
    #[test]
    fn an_image_with_a_file_missing_says_which() {
        let root = a_copy_of_the_image("no-tmpfiles");
        std::fs::remove_file(root.join(TMPFILES)).unwrap();

        let refused = Image::at(&root).unwrap_err();

        assert!(
            matches!(refused, NotAnImage::Unreadable { .. }),
            "{refused}"
        );
        assert!(refused.to_string().contains("alo.conf"), "{refused}");
    }

    /// A file that would not read comes back as that file's own refusal, with
    /// the path in front of it — one reader per file, and nothing swallowed.
    #[test]
    fn a_file_that_will_not_read_comes_back_as_its_own_refusal() {
        let root = a_copy_of_the_image("bad-unit");
        std::fs::write(
            root.join(UNITS).join(THE_AGENT),
            "ExecStart=/usr/bin/alo-agentd\n",
        )
        .unwrap();

        let refused = Image::at(&root).unwrap_err();

        assert!(
            matches!(
                &refused,
                NotAnImage::NotAUnit {
                    why: crate::NotAUnit::NoSection { .. },
                    ..
                }
            ),
            "{refused}"
        );
    }

    /// The two unit names are what systemd calls them, and they are one string
    /// here rather than repeated into every check that names one.
    #[test]
    fn the_units_are_named_once() {
        assert_eq!(THE_LOADER, "alo-boundaryd.service");
        assert_eq!(THE_AGENT, "alo-agentd.service");
    }
}
