//! A portal request: which application, which portal, and what it is for.
//!
//! What a request is *for* is written as the `alo_capability::Ask` an agent's
//! verb would arrive with — a path, an application, or a facility — so one list
//! of grants answers both, and nothing about what a grant covers is restated in
//! this crate.
//!
//! # Three ways to make one, each for the portals it fits
//!
//! - [`Request::of`] — a portal about a facility: the camera, the screen, the
//!   person's notifications. The portal fixes the facility, so the request
//!   names nothing and cannot ask the camera portal for the microphone.
//! - [`Request::over`] — a portal about a file: the file chooser, open-with,
//!   print and trash.
//! - [`Request::opening_with`] — open-with, naming the application the file is
//!   to be opened in as well, so both the file and that application have to
//!   have been granted.
//!
//! Each refuses the portals it does not fit ([`NotARequest`]), and all three
//! check what arrived from outside before anything is judged: an identifier
//! that is one, and a path that is full and has no `..` in it.

use std::path::{Path, PathBuf};

use alo_capability::path::steps_upwards;
use alo_capability::{Applicant, Ask};

use crate::not_a_request::NotARequest;
use crate::portal::{Over, Portal};

/// The most bytes an application's identifier may have.
///
/// A D-Bus name's limit, which is what an application's identifier is on the
/// bus task 5 answers it on. Anything longer is not one of those.
pub const LONGEST_IDENTIFIER: usize = 255;

/// One application's request to one portal, checked and not yet judged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The application asking.
    from: Applicant,
    /// The portal it asked.
    portal: Portal,
    /// What it is for — one ask, or for open-with naming an application, two.
    wanted: Vec<Ask>,
}

impl Request {
    /// A request to a portal about a facility.
    ///
    /// # Errors
    /// [`NotARequest::NeedsAPath`] for a portal about a file, and the
    /// identifier's refusals.
    pub fn of(from: &str, portal: Portal) -> Result<Self, NotARequest> {
        let from = identified(from)?;
        match portal.over() {
            Over::Facility(facility) => Ok(Self {
                from,
                portal,
                wanted: vec![Ask::facility(facility)],
            }),
            Over::APath => Err(NotARequest::NeedsAPath),
        }
    }

    /// A request to a portal about a file.
    ///
    /// The path arrives resolved, as every `Ask::Path` must (`alo_capability::path`
    /// on symbolic links): the backend that receives a request is what resolves
    /// the file a document portal hands over, and asks about that.
    ///
    /// # Errors
    /// [`NotARequest::NotOverAPath`] for a portal about a facility,
    /// [`NotARequest::NotAFullPath`] and [`NotARequest::CouldLeadElsewhere`] for
    /// a path nothing can compare honestly, and the identifier's refusals.
    pub fn over(from: &str, portal: Portal, path: &Path) -> Result<Self, NotARequest> {
        let from = identified(from)?;
        if portal.over() != Over::APath {
            return Err(NotARequest::NotOverAPath);
        }
        Ok(Self {
            from,
            portal,
            wanted: vec![Ask::Path(checked_path(path)?)],
        })
    }

    /// An open-with request: this file, in that application.
    ///
    /// # Errors
    /// Everything [`Request::over`] refuses, and
    /// [`NotARequest::NotAnIdentifier`] for an application that is not named by
    /// an identifier.
    pub fn opening_with(from: &str, path: &Path, application: &str) -> Result<Self, NotARequest> {
        let mut request = Self::over(from, Portal::OpenWith, path)?;
        let opener = identified(application).map_err(|_| NotARequest::NotAnIdentifier)?;
        request.wanted.push(Ask::application(opener.as_str()));
        Ok(request)
    }

    /// The application asking.
    #[must_use]
    pub const fn application(&self) -> &Applicant {
        &self.from
    }

    /// The portal it asked.
    #[must_use]
    pub const fn portal(&self) -> Portal {
        self.portal
    }

    /// What it is for, each as the ask a grant has to cover.
    #[must_use]
    pub fn wanted(&self) -> &[Ask] {
        &self.wanted
    }
}

/// An application's identifier, checked at the boundary.
pub(crate) fn identified(id: &str) -> Result<Applicant, NotARequest> {
    let id = id.trim();
    if id.is_empty() {
        return Err(NotARequest::NoApplication);
    }
    if id.len() > LONGEST_IDENTIFIER
        || id
            .chars()
            .any(|letter| letter.is_whitespace() || letter.is_control())
    {
        return Err(NotARequest::NotAnIdentifier);
    }
    Ok(Applicant::named(id))
}

/// A path a grant can be compared against honestly, or why it is not one.
fn checked_path(path: &Path) -> Result<PathBuf, NotARequest> {
    if !path.has_root() {
        return Err(NotARequest::NotAFullPath);
    }
    if steps_upwards(path) {
        return Err(NotARequest::CouldLeadElsewhere);
    }
    Ok(path.to_path_buf())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_capability::Facility;

    const CHEESE: &str = "org.gnome.Cheese";

    /// **A facility portal asks for its own facility and nothing else.**
    #[test]
    fn a_facility_portal_asks_for_its_own_facility() {
        let request = Request::of(CHEESE, Portal::Camera).unwrap();
        assert_eq!(request.wanted(), [Ask::facility(Facility::Camera)]);
        assert_eq!(request.application(), &Applicant::named(CHEESE));
        assert_eq!(request.portal(), Portal::Camera);
        assert_eq!(
            Request::of(CHEESE, Portal::Screenshot).unwrap().wanted(),
            [Ask::facility(Facility::ScreenOnce)]
        );
    }

    /// **A portal is made the way it fits, and refused the ways it does not.**
    #[test]
    fn a_request_is_refused_for_a_portal_it_does_not_fit() {
        assert_eq!(
            Request::of(CHEESE, Portal::Print).unwrap_err(),
            NotARequest::NeedsAPath
        );
        assert_eq!(
            Request::over(CHEESE, Portal::Camera, Path::new("/dev/video0")).unwrap_err(),
            NotARequest::NotOverAPath
        );
        let print =
            Request::over(CHEESE, Portal::Print, Path::new("/home/anna/march.pdf")).unwrap();
        assert_eq!(print.wanted(), [Ask::path("/home/anna/march.pdf")]);
    }

    /// **What arrives from outside is checked before anything is judged.**
    #[test]
    fn what_arrives_from_outside_is_checked_first() {
        for (from, refused) in [
            ("", NotARequest::NoApplication),
            ("   ", NotARequest::NoApplication),
            ("org.gnome.Cheese extra", NotARequest::NotAnIdentifier),
            (
                "org.gnome.Cheese\nforged line",
                NotARequest::NotAnIdentifier,
            ),
        ] {
            assert_eq!(
                Request::of(from, Portal::Camera).unwrap_err(),
                refused,
                "{from:?}"
            );
        }
        let long = "a".repeat(LONGEST_IDENTIFIER + 1);
        assert_eq!(
            Request::of(&long, Portal::Camera).unwrap_err(),
            NotARequest::NotAnIdentifier
        );
        assert!(Request::of(&"a".repeat(LONGEST_IDENTIFIER), Portal::Camera).is_ok());

        for (path, refused) in [
            ("march.pdf", NotARequest::NotAFullPath),
            ("", NotARequest::NotAFullPath),
            (
                "/home/anna/../root/.ssh/id_ed25519",
                NotARequest::CouldLeadElsewhere,
            ),
        ] {
            assert_eq!(
                Request::over(CHEESE, Portal::Trash, Path::new(path)).unwrap_err(),
                refused,
                "{path}"
            );
        }
    }

    /// **Open-with names two things, and both must be well-formed.**
    #[test]
    fn open_with_asks_for_the_file_and_the_application() {
        let request = Request::opening_with(
            CHEESE,
            Path::new("/home/anna/march.pdf"),
            "org.gnome.Papers",
        )
        .unwrap();
        assert_eq!(request.portal(), Portal::OpenWith);
        assert_eq!(
            request.wanted(),
            [
                Ask::path("/home/anna/march.pdf"),
                Ask::application("org.gnome.Papers")
            ]
        );
        assert_eq!(
            Request::opening_with(CHEESE, Path::new("/home/anna/march.pdf"), "").unwrap_err(),
            NotARequest::NotAnIdentifier
        );
        assert_eq!(
            Request::opening_with(CHEESE, Path::new("march.pdf"), "org.gnome.Papers").unwrap_err(),
            NotARequest::NotAFullPath
        );
    }
}
