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
//! 4. asks that application to open the file (`crate::opening`), and says `0`
//!    only when it did.
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
use crate::asked::Asked;
use crate::open_with::{NotOpened, answered};
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
        let outcome = self.answer(connection, asked.application(), fd).await;
        asked.answered(connection, &self.backend, outcome).await
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

    /// What `application` is answered with for the file behind `fd`.
    async fn answer(
        &self,
        connection: &zbus::Connection,
        application: Option<&str>,
        fd: OwnedFd,
    ) -> Outcome {
        let Some(application) = application else {
            return Outcome::Unanswered(Unanswered::NotIdentified);
        };
        let handed = File::from(std::os::fd::OwnedFd::from(fd));
        if !handed.metadata().is_ok_and(|about| about.is_file()) {
            return Outcome::Unanswered(Unanswered::NotAFile);
        }
        let named = PathBuf::from(format!("/proc/self/fd/{}", handed.as_raw_fd()));
        let (Ok(path), Ok(mut file)) = (std::fs::read_link(&named), File::open(&named)) else {
            return Outcome::Unanswered(Unanswered::NotAFile);
        };
        let Some(grants) = self.backend.machine().grants() else {
            return Outcome::Unanswered(Unanswered::GrantsUnread);
        };
        let Some(applications) = self.backend.machine().applications() else {
            return Outcome::Unanswered(Unanswered::ApplicationsUnread);
        };
        let opens = match answered(
            application,
            &path,
            &mut file,
            &applications.what_opens(),
            &grants,
            SystemTime::now(),
        ) {
            Ok(opens) => opens,
            Err(NotOpened::NotARequest(not)) => return Outcome::NotARequest(not),
            Err(NotOpened::Refused(refused)) => return Outcome::Refused(refused),
            Err(NotOpened::NothingOpens(nothing)) => return Outcome::NothingOpens(nothing),
        };
        let opener = opens.opener().application().identifier().to_owned();
        if opened_in(connection, &opener, &path).await {
            Outcome::Opened(opens)
        } else {
            Outcome::Unanswered(Unanswered::NotOpened { opener })
        }
    }
}
