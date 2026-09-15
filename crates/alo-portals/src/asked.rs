//! One request as it arrived on the bus: who sent it, which application that
//! is, where its answer goes — and, once answered, the record and the response.
//!
//! Both interfaces the backend serves begin and end a request here, so there is
//! one place where a request is identified, one where its answer is written
//! down, and the order of the two is the same for both: **recorded, then
//! sent.** An application is never answered with something the record does not
//! hold.
//!
//! # The response is sent before the handle is returned
//!
//! Every answer here is decided while the method call is being handled, so the
//! `Response` signal goes out before the reply carrying the handle. The
//! specification tells an application to listen on the handle it can predict
//! from its `handle_token` before it calls, and GTK, libportal and libsecret
//! all do; an application that passed no token and listens only after the
//! reply would miss the answer, which is the specification's own warning about
//! that shape.

use std::collections::HashMap;
use std::time::SystemTime;

use alo_capability::Applicant;
use zbus::fdo;
use zbus::message::Header;
use zbus::names::{BusName, OwnedUniqueName};
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

use crate::answered::{Answered, Outcome, Unanswered};
use crate::handle::handle_for;
use crate::portal::Portal;
use crate::request::identified;
use crate::serving::Backend;

/// The interface every `Response` is sent on.
const A_REQUEST: &str = "org.freedesktop.portal.Request";

/// A request that arrived, identified and given its handle.
pub(crate) struct Asked {
    /// Who sent it, which is who the response is sent to.
    sender: OwnedUniqueName,
    /// Where its response goes.
    handle: OwnedObjectPath,
    /// The application its sandbox names, if it has one.
    application: Option<String>,
    /// The portal it asked.
    portal: Portal,
}

impl Asked {
    /// The request this method call is: its sender, the application the
    /// sender's sandbox names, and its handle.
    ///
    /// # Errors
    /// `InvalidArgs` for a `handle_token` that is not one — recorded first,
    /// because there is no handle to send a response to.
    pub(crate) async fn on(
        connection: &zbus::Connection,
        header: &Header<'_>,
        options: &HashMap<String, OwnedValue>,
        backend: &Backend,
        portal: Portal,
    ) -> fdo::Result<Self> {
        let sender = header
            .sender()
            .map(|sender| OwnedUniqueName::from(sender.to_owned()))
            .ok_or_else(|| fdo::Error::Failed("a request with no sender".to_owned()))?;
        let application = application_of(connection, &sender, backend).await;
        let token = options
            .get("handle_token")
            .map(|token| token.downcast_ref::<&str>().unwrap_or(""));
        let Ok(handle) = handle_for(sender.as_str(), token) else {
            backend.record().keep(Answered::new(
                SystemTime::now(),
                named(application.as_deref()),
                portal,
                Outcome::Unanswered(Unanswered::NotAToken),
            ));
            return Err(fdo::Error::InvalidArgs(
                "handle_token is letters, digits and underscores".to_owned(),
            ));
        };
        let handle = OwnedObjectPath::try_from(handle)
            .map_err(|_| fdo::Error::InvalidArgs("a sender with no handle".to_owned()))?;
        Ok(Self {
            sender,
            handle,
            application,
            portal,
        })
    }

    /// The application the sender's sandbox names, if it has one.
    pub(crate) fn application(&self) -> Option<&str> {
        self.application.as_deref()
    }

    /// Record `outcome`, send it as the response, and give back the handle.
    ///
    /// # Errors
    /// Whatever the bus says when the response cannot be sent — after the
    /// answer is recorded.
    pub(crate) async fn answered(
        self,
        connection: &zbus::Connection,
        backend: &Backend,
        outcome: Outcome,
    ) -> fdo::Result<OwnedObjectPath> {
        let response = outcome.response();
        backend.record().keep(Answered::new(
            SystemTime::now(),
            named(self.application()),
            self.portal,
            outcome,
        ));
        connection
            .emit_signal(
                Some(BusName::Unique(self.sender.as_ref())),
                self.handle.as_ref(),
                A_REQUEST,
                "Response",
                &(response, HashMap::<String, OwnedValue>::new()),
            )
            .await?;
        Ok(self.handle)
    }
}

/// The application the sandbox of the process behind `sender` names.
///
/// The process is the bus daemon's answer, read from the socket — never
/// anything the sender said. A bus that will not answer is nobody.
async fn application_of(
    connection: &zbus::Connection,
    sender: &OwnedUniqueName,
    backend: &Backend,
) -> Option<String> {
    let bus = fdo::DBusProxy::new(connection).await.ok()?;
    let credentials = bus
        .get_connection_credentials(BusName::Unique(sender.as_ref()))
        .await
        .ok()?;
    backend
        .sandboxes()
        .application_of(credentials.process_id()?)
}

/// The applicant the record names, when the sandbox named one that is an
/// identifier.
fn named(application: Option<&str>) -> Option<Applicant> {
    application.and_then(|id| identified(id).ok())
}
