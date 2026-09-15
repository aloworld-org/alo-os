//! The portal backend, served on a session bus.
//!
//! `org.freedesktop.portal.*` is the contract every Linux application already
//! speaks. [`Backend::serve_on`] owns `org.freedesktop.portal.Desktop` on the
//! bus it is given and answers, at `/org/freedesktop/portal/desktop`, **only the
//! portals whose answers were decided without a dialog**:
//!
//! | Interface | Answered from |
//! |---|---|
//! | `org.freedesktop.portal.Secret` | the person's one keyring (`crate::secret_portal`) |
//! | `org.freedesktop.portal.OpenURI` | *what opens what* (`crate::open_uri_portal`) |
//! | `org.freedesktop.portal.Settings` | what the person set for how the machine looks (`crate::settings_portal`) |
//!
//! Every other portal is not registered, so the bus itself tells an application
//! nothing answers it — [`crate::Portal::answered_on_the_bus`] is the list, and
//! `docs/contracts/portals.md` is the list for people building against it.
//!
//! # What a request is answered from
//!
//! - **Who is asking** — the sandbox of the process the bus says sent it
//!   ([`Sandboxes`]).
//! - **What was granted, what opens what, and how the machine looks** — read at
//!   every request ([`TheMachine`]).
//! - **The keyring** — which judges a Secret request itself ([`KeepsSecrets`]).
//! - **The record** — every answer, the refusals as carefully as the rest,
//!   written before the response is sent ([`Recording`]).
//!
//! While [`Served`] is held, a thread looks at how the machine looks every
//! [`crate::watching_appearance::WATCHED_EVERY`] and sends `SettingChanged` to the
//! applications that may read what moved (`crate::watching_appearance`).

use std::sync::Arc;
use std::sync::mpsc::{Sender, channel};
use std::thread::JoinHandle;

use crate::keeping_secrets::KeepsSecrets;
use crate::open_uri_portal::OpenUriPortal;
use crate::recording::Recording;
use crate::sandboxed::Sandboxes;
use crate::secret_portal::SecretPortal;
use crate::settings_portal::SettingsPortal;
use crate::the_machine::TheMachine;

pub use crate::handle::THE_PORTALS_OBJECT;

/// The name the backend owns on the bus.
pub const THE_PORTALS_NAME: &str = "org.freedesktop.portal.Desktop";

/// Everything a request is answered from, shared by the interfaces served.
#[derive(Clone)]
pub struct Backend {
    /// The grants and what opens what.
    machine: Arc<dyn TheMachine>,
    /// The keyring.
    keyring: Arc<dyn KeepsSecrets>,
    /// Where a caller's sandbox is read.
    sandboxes: Sandboxes,
    /// Where every answer is written.
    record: Arc<dyn Recording>,
}

impl Backend {
    /// A backend answering from this machine, this keyring, these sandboxes,
    /// and writing every answer into `record`.
    #[must_use]
    pub fn answering_from(
        machine: Arc<dyn TheMachine>,
        keyring: Arc<dyn KeepsSecrets>,
        sandboxes: Sandboxes,
        record: Arc<dyn Recording>,
    ) -> Self {
        Self {
            machine,
            keyring,
            sandboxes,
            record,
        }
    }

    /// Serve the decided portals on the bus at `address`, until the returned
    /// [`Served`] is dropped.
    ///
    /// # Errors
    /// [`NotServed::NotAnAddress`], [`NotServed::Unreachable`] when nothing
    /// answers there, and [`NotServed::NameTaken`] when another backend already
    /// answers the portals on that bus — which is refused rather than queued
    /// behind, so two backends never answer one machine's applications.
    pub fn serve_on(self, address: &str) -> Result<Served, NotServed> {
        let connection = zbus::blocking::connection::Builder::address(address)
            .map_err(|_| NotServed::NotAnAddress)?
            .serve_at(
                THE_PORTALS_OBJECT,
                SecretPortal {
                    backend: self.clone(),
                },
            )
            .map_err(|_| NotServed::Unreachable)?
            .serve_at(
                THE_PORTALS_OBJECT,
                OpenUriPortal {
                    backend: self.clone(),
                },
            )
            .map_err(|_| NotServed::Unreachable)?
            .serve_at(
                THE_PORTALS_OBJECT,
                SettingsPortal {
                    backend: self.clone(),
                },
            )
            .map_err(|_| NotServed::Unreachable)?
            .name(THE_PORTALS_NAME)
            .map_err(|_| NotServed::Unreachable)?
            .build()
            .map_err(|why| match why {
                zbus::Error::NameTaken => NotServed::NameTaken,
                _ => NotServed::Unreachable,
            })?;
        let (stop, stopped) = channel();
        let bus = connection.inner().clone();
        let seen = crate::appearance_settings::read(self.machine()).ok();
        let watching = std::thread::Builder::new()
            .name("alo-portals-appearance".to_owned())
            .spawn(move || crate::watching_appearance::watch(&bus, &self, seen, &stopped))
            .map_err(|_| NotServed::Unreachable)?;
        Ok(Served {
            connection,
            stop: Some(stop),
            watching: Some(watching),
        })
    }

    /// The grants and what opens what.
    pub(crate) fn machine(&self) -> &dyn TheMachine {
        self.machine.as_ref()
    }

    /// The keyring.
    pub(crate) fn keyring(&self) -> &dyn KeepsSecrets {
        self.keyring.as_ref()
    }

    /// Where a caller's sandbox is read.
    pub(crate) const fn sandboxes(&self) -> &Sandboxes {
        &self.sandboxes
    }

    /// Where every answer is written.
    pub(crate) fn record(&self) -> &dyn Recording {
        self.record.as_ref()
    }
}

/// The backend, answering, for as long as this is held.
pub struct Served {
    /// The connection the portals are served on; dropping it stops them.
    connection: zbus::blocking::Connection,
    /// Dropped to stop the thread watching how the machine looks.
    stop: Option<Sender<()>>,
    /// That thread, waited for when this is dropped, so nothing is sent on
    /// the portals' behalf after they stop answering.
    watching: Option<JoinHandle<()>>,
}

impl Drop for Served {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(watching) = self.watching.take() {
            // A watcher that panicked has already stopped, which is all that
            // is being waited for.
            let _ = watching.join();
        }
    }
}

impl std::fmt::Debug for Served {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Served")
            .field("on", &self.connection.unique_name())
            .finish()
    }
}

/// Why the portals could not be served.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotServed {
    /// What was given is not a D-Bus address.
    NotAnAddress,
    /// Nothing answered at that address, or it would not take the portals.
    Unreachable,
    /// Something else already owns `org.freedesktop.portal.Desktop` there.
    NameTaken,
}
