//! `org.freedesktop.portal.Secret`, answered from the person's one keyring.
//!
//! One method, `RetrieveSecret(h fd, a{sv} options) -> o handle`: the
//! application hands over the write end of a pipe, and is written its own
//! portal secret — the bytes `alo_secrets::ItsOwn::the_portals_secret` keeps
//! for it (task 3 of the applications plan) — before the `Response` says `0`.
//! Anything else is `2`, with nothing written.
//!
//! In this order: the caller is a sandboxed application; its identifier is
//! one; the grants can be read; and the keyring judges the request against
//! them and, only if they allow it, writes.

use std::collections::HashMap;
use std::fs::File;
use std::time::SystemTime;

use zbus::fdo;
use zbus::message::Header;
use zbus::zvariant::{OwnedFd, OwnedObjectPath, OwnedValue};

use crate::answered::{Outcome, Unanswered};
use crate::asked::Asked;
use crate::keeping_secrets::NotKept;
use crate::portal::Portal;
use crate::request::Request;
use crate::serving::Backend;

/// The version of the interface this answers.
const VERSION: u32 = 1;

/// The Secret portal, served.
pub(crate) struct SecretPortal {
    /// What it answers from.
    pub(crate) backend: Backend,
}

#[zbus::interface(name = "org.freedesktop.portal.Secret")]
impl SecretPortal {
    /// Write the application's own secret into `fd`, if it may have it.
    async fn retrieve_secret(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        fd: OwnedFd,
        options: HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedObjectPath> {
        let asked = Asked::on(connection, &header, &options, &self.backend, Portal::Secret).await?;
        let outcome = self.answer(asked.application(), fd);
        asked.answered(connection, &self.backend, outcome).await
    }

    /// The version of this interface.
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        VERSION
    }
}

impl SecretPortal {
    /// What `application` is answered with.
    fn answer(&self, application: Option<&str>, fd: OwnedFd) -> Outcome {
        let Some(application) = application else {
            return Outcome::Unanswered(Unanswered::NotIdentified);
        };
        let request = match Request::of(application, Portal::Secret) {
            Ok(request) => request,
            Err(not) => return Outcome::NotARequest(not),
        };
        let Some(grants) = self.backend.machine().grants() else {
            return Outcome::Unanswered(Unanswered::GrantsUnread);
        };
        let mut into = File::from(std::os::fd::OwnedFd::from(fd));
        match self
            .backend
            .keyring()
            .hand_over(&request, &grants, SystemTime::now(), &mut into)
        {
            Ok(allowed) => Outcome::SecretHandedOver(allowed),
            Err(NotKept::Refused(refused)) => Outcome::Refused(refused),
            Err(NotKept::Unavailable) => Outcome::Unanswered(Unanswered::KeyringUnavailable),
            Err(NotKept::NotWritten) => Outcome::Unanswered(Unanswered::NotWritten),
        }
    }
}
