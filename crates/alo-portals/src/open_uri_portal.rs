//! `org.freedesktop.portal.OpenURI`, answered from *what opens what*.
//!
//! **`OpenFile(s parent_window, h fd, a{sv} options) -> o handle`** is the
//! open-with portal task 4 of the applications plan decided. The application
//! hands over the file — usually opened `O_PATH`, which can be looked at and
//! not read — and the backend:
//!
//! 1. checks it is a file, and resolves the handle to the path it names on this
//!    machine, `/proc/self/fd/N`;
//! 2. reads the grants and what opens what;
//! 3. asks `crate::open_with::answered`, which judges the file's grant **before
//!    the file is read**, reads its kind from its bytes, and judges the grant
//!    over the application that opens it;
//! 4. records the answer, [`Outcome::Opened`] — and **only once the record has
//!    kept it** asks that application to open the file (`crate::opening`). A
//!    record that would not keep it is a `2`, and nothing is opened;
//! 5. says `0` only when the application opened it. When it did not, that is
//!    recorded after the answer as [`Unanswered::NotOpened`], and is a `2`.
//!
//! **`OpenURI` and `OpenDirectory` are answered `2`**, recorded as
//! [`Unanswered::NotDecidedHere`]. Nothing on this machine decides what opens a
//! web link or a folder yet, and a backend that answered *opened* without
//! opening would keep an application from asking again for something that never
//! happened. They are methods of the same interface as `OpenFile`, so declining
//! them by not registering is not possible without declining open-with too.
//!
//! `writable`, `ask` and `activation_token` are read by nothing: a file is
//! handed to its opener as the person's file, nothing here asks anybody
//! anything, and no window is raised by a backend with no windows.

use std::collections::HashMap;
use std::fs::File;
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::time::SystemTime;

use zbus::fdo;
use zbus::message::Header;
use zbus::zvariant::{OwnedFd, OwnedObjectPath, OwnedValue};

use crate::answered::{Outcome, Unanswered};
use crate::asked::{Asked, REFUSED};
use crate::open_with::{NotOpened, OpensWith, answered};
use crate::opening::opened_in;
use crate::portal::Portal;
use crate::serving::Backend;

/// The version of the interface this answers: the one with `OpenDirectory`.
const VERSION: u32 = 3;

/// The open-with portal, served.
pub(crate) struct OpenUriPortal {
    /// What it answers from.
    pub(crate) backend: Backend,
}

#[zbus::interface(name = "org.freedesktop.portal.OpenURI")]
impl OpenUriPortal {
    /// A web link, which nothing here decides what opens.
    #[zbus(name = "OpenURI")]
    async fn open_uri(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        _parent_window: &str,
        _uri: &str,
        options: HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedObjectPath> {
        self.not_decided_here(connection, &header, &options).await
    }

    /// A file, opened in the application that opens its kind.
    async fn open_file(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        _parent_window: &str,
        fd: OwnedFd,
        options: HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedObjectPath> {
        let asked = Asked::on(
            connection,
            &header,
            &options,
            &self.backend,
            Portal::OpenWith,
        )
        .await?;
        let (opens, path) = match self.answer(asked.application(), fd) {
            Ok(allowed) => allowed,
            Err(outcome) => return asked.answered(connection, &self.backend, *outcome).await,
        };
        let opener = opens.opener().application().identifier().to_owned();
        let Ok(response) = asked.recorded(&self.backend, Outcome::Opened(opens)) else {
            return asked.responded(connection, REFUSED).await;
        };
        if opened_in(connection, &opener, &path).await {
            return asked.responded(connection, response).await;
        }
        // The answer was kept and the file not opened; what follows it says
        // so, and the response is the refusal either way.
        let _ = asked.recorded(
            &self.backend,
            Outcome::Unanswered(Unanswered::NotOpened { opener }),
        );
        asked.responded(connection, REFUSED).await
    }

    /// The folder a file is in, which nothing here decides what opens.
    async fn open_directory(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        _parent_window: &str,
        _fd: OwnedFd,
        options: HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedObjectPath> {
        self.not_decided_here(connection, &header, &options).await
    }

    /// The version of this interface.
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        VERSION
    }
}

impl OpenUriPortal {
    /// Answer a request nothing here decides, and record it with its asker.
    async fn not_decided_here(
        &self,
        connection: &zbus::Connection,
        header: &Header<'_>,
        options: &HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedObjectPath> {
        let asked = Asked::on(connection, header, options, &self.backend, Portal::OpenWith).await?;
        asked
            .answered(
                connection,
                &self.backend,
                Outcome::Unanswered(Unanswered::NotDecidedHere),
            )
            .await
    }

    /// What opens the file behind `fd` for `application`, and the path it is
    /// opened at — or the outcome `application` is answered with instead.
    fn answer(
        &self,
        application: Option<&str>,
        fd: OwnedFd,
    ) -> Result<(OpensWith, PathBuf), Box<Outcome>> {
        let Some(application) = application else {
            return Err(Box::new(Outcome::Unanswered(Unanswered::NotIdentified)));
        };
        let handed = File::from(std::os::fd::OwnedFd::from(fd));
        if !handed.metadata().is_ok_and(|about| about.is_file()) {
            return Err(Box::new(Outcome::Unanswered(Unanswered::NotAFile)));
        }
        let named = PathBuf::from(format!("/proc/self/fd/{}", handed.as_raw_fd()));
        let (Ok(path), Ok(mut file)) = (std::fs::read_link(&named), File::open(&named)) else {
            return Err(Box::new(Outcome::Unanswered(Unanswered::NotAFile)));
        };
        let Some(grants) = self.backend.machine().grants() else {
            return Err(Box::new(Outcome::Unanswered(Unanswered::GrantsUnread)));
        };
        let Some(applications) = self.backend.machine().applications() else {
            return Err(Box::new(Outcome::Unanswered(
                Unanswered::ApplicationsUnread,
            )));
        };
        answered(
            application,
            &path,
            &mut file,
            &applications.what_opens(),
            &grants,
            SystemTime::now(),
        )
        .map(|opens| (opens, path))
        .map_err(|not| {
            Box::new(match not {
                NotOpened::NotARequest(not) => Outcome::NotARequest(not),
                NotOpened::Refused(refused) => Outcome::Refused(refused),
                NotOpened::NothingOpens(nothing) => Outcome::NothingOpens(nothing),
            })
        })
    }
}
