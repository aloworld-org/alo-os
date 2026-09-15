//! `org.freedesktop.portal.Secret`, answered from the person's one keyring.
//!
//! One method, `RetrieveSecret(h fd, a{sv} options) -> o handle`: the
//! application hands over the write end of a pipe, and is written its own
//! portal secret — the bytes `alo_secrets::ItsOwn::the_portals_secret` keeps
//! for it (task 3 of the applications plan) — before the `Response` says `0`.
//! Anything else is `2`, with nothing written.
//!
//! In this order: the caller is a sandboxed application; its identifier is
//! one; the grants can be read; the keyring judges the request against them
//! and, only if they allow it, hands the secret over **into this backend's
//! memory**; the answer is recorded; and only once the record has kept it is
//! the secret written to the application. A record that would not keep it is
//! a `2` with nothing written. A write that then fails is recorded after the
//! answer as [`Unanswered::NotWritten`], and is a `2` too.
//!
//! The secret is held in memory from the keyring's answer to the write, and
//! overwritten with zeros once it has been written or refused.

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::SystemTime;

use zbus::fdo;
use zbus::message::Header;
use zbus::zvariant::{OwnedFd, OwnedObjectPath, OwnedValue};

use crate::answered::{Outcome, Unanswered};
use crate::asked::{Asked, REFUSED};
use crate::keeping_secrets::NotKept;
use crate::portal::Portal;
use crate::request::Request;
use crate::serving::Backend;

/// The version of the interface this answers.
const VERSION: u32 = 1;

/// The bytes set aside for a secret before the keyring is asked: many times
/// the portal secret `alo-secrets` keeps.
const SECRET_ROOM: usize = 4096;

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
        // Room for any secret a keyring hands over, so it is never moved and no
        // copy is left behind where zeros cannot reach it.
        let mut secret = Vec::with_capacity(SECRET_ROOM);
        let outcome = self.answer(asked.application(), &mut secret);
        let handing_over = matches!(outcome, Outcome::SecretHandedOver(_));
        let response = match asked.recorded(&self.backend, outcome) {
            Ok(response) if handing_over => {
                let mut into = File::from(std::os::fd::OwnedFd::from(fd));
                if into.write_all(&secret).and_then(|()| into.flush()).is_ok() {
                    response
                } else {
                    // The answer was kept and not delivered; what follows it
                    // says so, and the response is the refusal either way.
                    let _ =
                        asked.recorded(&self.backend, Outcome::Unanswered(Unanswered::NotWritten));
                    REFUSED
                }
            }
            Ok(response) => response,
            Err(_) => REFUSED,
        };
        secret.fill(0);
        asked.responded(connection, response).await
    }

    /// The version of this interface.
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        VERSION
    }
}

impl SecretPortal {
    /// What `application` is answered with, with its secret written into
    /// `secret` when it is handed one.
    fn answer(&self, application: Option<&str>, secret: &mut Vec<u8>) -> Outcome {
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
        match self
            .backend
            .keyring()
            .hand_over(&request, &grants, SystemTime::now(), secret)
        {
            Ok(allowed) => Outcome::SecretHandedOver(allowed),
            Err(NotKept::Refused(refused)) => Outcome::Refused(refused),
            Err(NotKept::Unavailable) => Outcome::Unanswered(Unanswered::KeyringUnavailable),
            Err(NotKept::NotWritten) => Outcome::Unanswered(Unanswered::NotWritten),
        }
    }
}
