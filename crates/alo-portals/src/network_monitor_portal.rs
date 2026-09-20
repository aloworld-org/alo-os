//! `org.freedesktop.portal.NetworkMonitor`, answered from the network manager.
//!
//! - **`GetAvailable() -> b`** — whether anything is reached through the
//!   connection this machine is sending on.
//! - **`GetMetered() -> b`** — whether that way out is somebody's meter.
//! - **`GetConnectivity() -> u`** — how far it reaches, in GLib's numbering
//!   ([`crate::network_state`], which is where the two services' numberings are
//!   reconciled).
//! - **`GetStatus() -> a{sv}`** — all three in one reply, read at one moment.
//!
//! Nothing here asks a person anything, so every answer is the method's reply:
//! there is no request handle and no `Response`.
//!
//! # `CanReach` is not answered, and that is the point of it
//!
//! The specification's fifth method takes a hostname and a port and says
//! whether they can be reached. Answering it honestly means **sending a packet
//! to a host an application named**, on a machine whose first law is that
//! nothing leaves silently. It would also be a road around every other
//! boundary: an application with no network of its own could read a person's
//! network out one name at a time by asking whether each is reachable.
//!
//! So it is answered `org.freedesktop.portal.Error.NotAllowed`, to every caller,
//! whether or not it holds the grant — a refusal that tells an application
//! nothing about this machine except that this machine does not do that.
//! `docs/contracts/portals.md` says so for people building against it.
//!
//! # What a refused application receives
//!
//! **`org.freedesktop.portal.Error.NotAllowed`**, and the same error, in the
//! same words, as a machine whose network manager is not answering. An
//! application not granted the network state learns nothing from being refused
//! that it would not learn from a machine that cannot see its own network. The
//! record tells the two apart; the application is not told.
//!
//! # In this order
//!
//! The caller is judged ([`network_state::allowed`]) before anything reads the
//! network ([`network_state::read`]). Every answer, and every refusal with its
//! application named, is recorded before the reply is sent.
//!
//! **An answer the record did not keep is not sent.**

use std::collections::HashMap;
use std::time::SystemTime;

use zbus::message::Header;
use zbus::zvariant::OwnedValue;

use alo_networks::Reaching;

use crate::answered::{Answered, Outcome, Unanswered};
use crate::asked::NOT_RECORDED;
use crate::caller::{application_of, named};
use crate::network_state;
use crate::portal::Portal;
use crate::serving::Backend;

/// The version of the interface this answers.
const VERSION: u32 = 3;

/// What every caller is told when it may not be answered — and what a machine
/// that cannot read its own network tells every caller.
const NOT_ALLOWED: &str = "The network state is not available to this application";

/// What a caller asking `CanReach` is told, whoever it is.
const NOT_REACHED: &str = "This machine does not try to reach a host on an application's behalf";

/// The errors this interface answers with, under the portal's own names.
#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.freedesktop.portal.Error")]
pub(crate) enum NetworkError {
    /// Something on the bus itself went wrong.
    #[zbus(error)]
    ZBus(zbus::Error),
    /// The caller may not read the network state, or this machine cannot.
    NotAllowed(String),
    /// The answer could not be written into the record.
    Failed(String),
}

/// The NetworkMonitor portal, served.
pub(crate) struct NetworkMonitorPortal {
    /// What it answers from.
    pub(crate) backend: Backend,
}

#[zbus::interface(name = "org.freedesktop.portal.NetworkMonitor")]
impl NetworkMonitorPortal {
    /// Whether anything is reached through the connection this machine is on.
    async fn get_available(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
    ) -> Result<bool, NetworkError> {
        let reaching = self.answered(connection, &header).await?;
        Ok(network_state::available(reaching.how_far))
    }

    /// Whether the way out is somebody's meter.
    async fn get_metered(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
    ) -> Result<bool, NetworkError> {
        let reaching = self.answered(connection, &header).await?;
        Ok(network_state::metered(reaching.metered))
    }

    /// How far this machine reaches, in the specification's numbering.
    async fn get_connectivity(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
    ) -> Result<u32, NetworkError> {
        let reaching = self.answered(connection, &header).await?;
        Ok(network_state::connectivity(reaching.how_far))
    }

    /// All three, read at one moment, so a caller cannot be told about three.
    async fn get_status(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
    ) -> Result<HashMap<String, OwnedValue>, NetworkError> {
        let reaching = self.answered(connection, &header).await?;
        let available = OwnedValue::from(network_state::available(reaching.how_far));
        let metered = OwnedValue::from(network_state::metered(reaching.metered));
        let connectivity = OwnedValue::from(network_state::connectivity(reaching.how_far));
        Ok(HashMap::from([
            ("available".to_owned(), available),
            ("metered".to_owned(), metered),
            ("connectivity".to_owned(), connectivity),
        ]))
    }

    /// Whether a host and port can be reached — **never answered**.
    ///
    /// Answering means sending a packet to a host an application named, which
    /// this machine does not do on an application's behalf. Refused to every
    /// caller, granted or not, so the refusal says nothing about the grants
    /// either.
    #[expect(
        clippy::unused_async,
        reason = "the interface's methods are async; this one refuses without awaiting anything"
    )]
    async fn can_reach(&self, _hostname: &str, _port: u32) -> Result<bool, NetworkError> {
        Err(NetworkError::NotAllowed(NOT_REACHED.to_owned()))
    }

    /// The network's state changed.
    ///
    /// Declared because the specification declares it, and **not sent by
    /// anything yet**: nothing in this backend watches the network manager for
    /// changes. An application that reads on its own schedule is answered
    /// correctly; one that waits for this waits. `docs/contracts/portals.md`
    /// says so.
    #[zbus(signal)]
    pub(crate) async fn changed(
        emitter: &zbus::object_server::SignalEmitter<'_>,
    ) -> zbus::Result<()>;

    /// The version of this interface.
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        VERSION
    }
}

impl NetworkMonitorPortal {
    /// What the network manager reports, when the caller may be told — recorded
    /// either way before it is returned.
    async fn answered(
        &self,
        connection: &zbus::Connection,
        header: &Header<'_>,
    ) -> Result<Reaching, NetworkError> {
        let application = self.application(connection, header).await;
        let machine = self.backend.machine();
        let answer = network_state::allowed(application.as_deref(), machine, SystemTime::now())
            .and_then(|allowed| network_state::read(machine).map(|reaching| (allowed, reaching)));
        match answer {
            Ok((allowed, reaching)) => {
                self.keep(application.as_deref(), Outcome::NetworkRead(allowed))?;
                Ok(reaching)
            }
            Err(outcome) => {
                let error = error_for(&outcome);
                self.keep(application.as_deref(), *outcome)?;
                Err(error)
            }
        }
    }

    /// The application the sender's sandbox names, if it has one.
    async fn application(
        &self,
        connection: &zbus::Connection,
        header: &Header<'_>,
    ) -> Option<String> {
        let sender = header.sender()?;
        application_of(connection, sender, &self.backend).await
    }

    /// Write this answer into the record.
    ///
    /// # Errors
    /// [`NetworkError::Failed`] when the record did not keep it, which the
    /// caller receives in place of the answer.
    fn keep(&self, application: Option<&str>, outcome: Outcome) -> Result<(), NetworkError> {
        self.backend
            .record()
            .keep(Answered::new(
                SystemTime::now(),
                named(application),
                Portal::NetworkMonitor,
                outcome,
            ))
            .map_err(|_| NetworkError::Failed(NOT_RECORDED.to_owned()))
    }
}

/// The error a caller receives for `outcome`, which was not an answer.
///
/// One error and one sentence for every refusal, so that an application refused
/// by the grants and one asking a machine that cannot see its own network are
/// told the same thing.
fn error_for(outcome: &Outcome) -> NetworkError {
    match outcome {
        Outcome::Unanswered(Unanswered::GrantsUnread) => {
            NetworkError::Failed("The grants could not be read".to_owned())
        }
        _ => NetworkError::NotAllowed(NOT_ALLOWED.to_owned()),
    }
}
