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
//! **`OpenURI(s parent_window, s uri, a{sv} options)`** opens a link — `http`,
//! `https` or `mailto` — in the person's default application for it, **with no
//! grant**, by the owner's decision of 2026-11-12 (*"No — open in my default,
//! but record it (Recommended)"*). The backend:
//!
//! 1. names the asking application by its sandbox, as for every request;
//! 2. reads what opens what, and asks `crate::link_answered` what opens the
//!    link — refused, and recorded, when it is not a link or nothing opens it;
//! 3. records the answer, [`Outcome::LinkOpened`], naming who asked, the
//!    address and what opens it — and **only once the record has kept it** asks
//!    the desktop to start that application with the link, exactly as a
//!    person's press starts it (`crate::opening_links`);
//! 4. says `0` only when the desktop took it. When it did not, that is
//!    recorded after the answer as [`Unanswered::NoDesktop`] or
//!    [`Unanswered::NotOpenedALink`], and is a `2`.
//!
//! **`ask` shows the person the *Open with* sheet** (the owner's approval of
//! 2026-11-12, frame `25:52`). The handle is returned at once — the person may
//! take a minute, and the specification's shape for a dialogue is a handle now
//! and a `Response` later — and, off the bus's own thread
//! (`crate::the_person_chooses_a_link`), the desktop is asked through its door's
//! `choose-link` act. What the person chose is held to what was offered,
//! recorded, and started as every link is; nothing chosen is recorded as
//! [`Outcome::LinkUnchosen`], and **the default is never opened instead**.
//!
//! **`OpenDirectory` is answered `2`**, recorded as
//! [`Unanswered::FolderNotDecidedHere`]: nothing on this machine decides what
//! opens a folder yet, and a backend that answered *opened* without opening
//! would keep an application from asking again for something that never
//! happened.
//!
//! `writable` and `activation_token` are read by nothing: a file is handed to
//! its opener as the person's file, and the desktop gives a new window the
//! keyboard as it gives every window a person opened.

use std::collections::HashMap;
use std::fs::File;
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::time::SystemTime;

use zbus::fdo;
use zbus::message::Header;
use zbus::zvariant::{OwnedFd, OwnedObjectPath, OwnedValue};

use alo_capability::Applicant;

use crate::answered::{Outcome, Unanswered};
use crate::asked::{Asked, REFUSED};
use crate::link_answered::{LinkNotOpened, link_answered};
use crate::open_with::{NotOpened, OpensWith, answered};
use crate::opening::opened_in;
use crate::opening_links::LinkStarted;
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
    /// A link, opened in the person's default application for its kind.
    #[zbus(name = "OpenURI")]
    async fn open_uri(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        _parent_window: &str,
        uri: &str,
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
        let asked_to_choose = options
            .get("ask")
            .is_some_and(|ask| ask.downcast_ref::<bool>().unwrap_or(false));
        let opened = match self.link(asked.application(), uri, asked_to_choose) {
            Ok(opened) => opened,
            Err(outcome) => return asked.answered(connection, &self.backend, *outcome).await,
        };
        if opened.asked_to_choose() {
            return Ok(crate::the_person_chooses_a_link::the_person_chooses(
                connection,
                &self.backend,
                asked,
                opened,
            ));
        }
        let opener = opened.opener().application().identifier().to_owned();
        let link = opened.link().clone();
        let Ok(response) = asked.recorded(&self.backend, Outcome::LinkOpened(opened)) else {
            return asked.responded(connection, REFUSED).await;
        };
        let after = match self.backend.links().opened(&opener, &link) {
            LinkStarted::Asked => return asked.responded(connection, response).await,
            LinkStarted::NoDesktop => Unanswered::NoDesktop { opener },
            LinkStarted::NotInstalled | LinkStarted::CouldNotAsk => {
                Unanswered::NotOpenedALink { opener }
            }
        };
        // The answer was kept and the link not opened; what follows it says
        // why, and the response is the refusal either way.
        let _ = asked.recorded(&self.backend, Outcome::Unanswered(after));
        asked.responded(connection, REFUSED).await
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
        let asked = Asked::on(
            connection,
            &header,
            &options,
            &self.backend,
            Portal::OpenWith,
        )
        .await?;
        asked
            .answered(
                connection,
                &self.backend,
                Outcome::Unanswered(Unanswered::FolderNotDecidedHere),
            )
            .await
    }

    /// The version of this interface.
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        VERSION
    }
}

impl OpenUriPortal {
    /// What opens the link `uri` for `application` — or the outcome it is
    /// answered with instead. **No grant is read**: the owner's decision of
    /// 2026-11-12, `crate::link_answered`'s header.
    fn link(
        &self,
        application: Option<&str>,
        uri: &str,
        asked_to_choose: bool,
    ) -> Result<crate::LinkOpened, Box<Outcome>> {
        let Some(application) = application else {
            return Err(Box::new(Outcome::Unanswered(Unanswered::NotIdentified)));
        };
        let Some(applications) = self.backend.machine().applications() else {
            return Err(Box::new(Outcome::Unanswered(
                Unanswered::ApplicationsUnread,
            )));
        };
        link_answered(
            Applicant::named(application),
            uri,
            asked_to_choose,
            &applications.what_opens(),
        )
        .map_err(|not| {
            Box::new(match not {
                LinkNotOpened::NotALink(not) => Outcome::NotALink(not),
                LinkNotOpened::NothingOpensLinks(nothing) => Outcome::NothingOpensLinks(nothing),
            })
        })
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
