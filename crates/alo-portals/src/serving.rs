//! The portal backend, served on a session bus.
//!
//! `org.freedesktop.portal.*` is the contract every Linux application already
//! speaks. [`Backend::serve_on`] owns `org.freedesktop.portal.Desktop` on the
//! bus it is given and answers, at `/org/freedesktop/portal/desktop`, **only the
//! portals implemented by this frontend**:
//!
//! | Interface | Answered from |
//! |---|---|
//! | `org.freedesktop.portal.Secret` | the person's one keyring (`crate::secret_portal`) |
//! | `org.freedesktop.portal.OpenURI` | *what opens what*, and the desktop for a link (`crate::open_uri_portal`) |
//! | `org.freedesktop.portal.Settings` | what the person set for how the machine looks (`crate::settings_portal`) |
//! | `org.freedesktop.portal.NetworkMonitor` | how far this machine reaches (`crate::network_monitor_portal`) |
//! | `org.freedesktop.portal.FileChooser` | the person, on alo's own file chooser, and the documents portal to hand the file over (`crate::file_chooser_portal`) |
//! | `org.freedesktop.portal.ScreenCast` | the person's source choice and its revocable capture session (`crate::screen_cast_portal`) |
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

use crate::choosing_files::{ChoosesFiles, ExportsDocuments, NoDocumentsPortal};
use crate::file_chooser_portal::FileChooserPortal;
use crate::keeping_secrets::KeepsSecrets;
use crate::network_monitor_portal::NetworkMonitorPortal;
use crate::open_uri_portal::OpenUriPortal;
use crate::opening_links::{NoDesktopToAsk, OpensLinks};
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
    /// How the desktop is asked to start links.
    links: Arc<dyn OpensLinks>,
    /// The current native file chooser.
    files: Arc<dyn ChoosesFiles>,
    /// Current Documents portal export.
    documents: Arc<dyn ExportsDocuments>,
    /// Pending capture choices and cancellation.
    native_requests: Arc<crate::dialog_request::NativeRequests>,
    /// Session-scoped capture permission and stream owners.
    capture_sessions: Arc<crate::capture_sessions::Sessions>,
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
            links: Arc::new(NoDesktopToAsk),
            files: Arc::new(NoDesktopToAsk),
            documents: Arc::new(NoDocumentsPortal),
            native_requests: Arc::default(),
            capture_sessions: Arc::default(),
        }
    }

    /// The same backend, asking the desktop through `links` to start the
    /// application a link opens in.
    ///
    /// **A backend not given one answers every link with a refusal** naming
    /// the desktop as not there (`crate::opening_links::NoDesktopToAsk`),
    /// rather than claiming a link opened.
    #[must_use]
    pub fn opening_links_with(mut self, links: Arc<dyn OpensLinks>) -> Self {
        self.links = links;
        self
    }

    /// The same backend, asking the person through `files` to choose a file
    /// and handing what they chose over through `documents`.
    ///
    /// **A backend not given them answers every file chooser request with a
    /// refusal** naming the desktop, or the documents portal, as not there —
    /// never with a file nobody chose.
    #[must_use]
    pub fn choosing_files_with(
        mut self,
        files: Arc<dyn ChoosesFiles>,
        documents: Arc<dyn ExportsDocuments>,
    ) -> Self {
        self.files = files;
        self.documents = documents;
        self
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
                crate::screen_cast_portal::ScreenCastPortal {
                    backend: self.clone(),
                },
            )
            .map_err(|_| NotServed::Unreachable)?
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
            .serve_at(
                THE_PORTALS_OBJECT,
                NetworkMonitorPortal {
                    backend: self.clone(),
                },
            )
            .map_err(|_| NotServed::Unreachable)?
            .serve_at(
                THE_PORTALS_OBJECT,
                FileChooserPortal {
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
        self.capture_sessions.ready.store(
            self.machine.begin_capture_sessions(),
            std::sync::atomic::Ordering::Release,
        );
        let bus = connection.inner().clone();
        let seen = crate::appearance_settings::read(self.machine()).ok();
        let native_requests = self.native_requests.clone();
        let capture_sessions = self.capture_sessions.clone();
        let machine = self.machine.clone();
        let native_backend = self.clone();
        let native_bus = bus.clone();
        bus.executor()
            .spawn(
                async move {
                    while !native_backend.native_requests().stopped() {
                        native_backend
                            .capture_sessions()
                            .refresh(&native_bus, &native_backend)
                            .await;
                        async_io::Timer::after(std::time::Duration::from_millis(100)).await;
                    }
                },
                "alo-capture-permissions",
            )
            .detach();
        let watching = std::thread::Builder::new()
            .name("alo-portals-appearance".to_owned())
            .spawn(move || crate::watching_appearance::watch(&bus, &self, seen, &stopped))
            .map_err(|_| NotServed::Unreachable)?;
        Ok(Served {
            connection,
            stop: Some(stop),
            watching: Some(watching),
            native_requests,
            capture_sessions,
            machine,
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

    /// How links open on this desktop.
    pub(crate) fn links(&self) -> &dyn OpensLinks {
        self.links.as_ref()
    }
    /// Current native file chooser.
    pub(crate) fn files(&self) -> &dyn ChoosesFiles {
        self.files.as_ref()
    }
    /// Current Documents portal export.
    pub(crate) fn documents(&self) -> &dyn ExportsDocuments {
        self.documents.as_ref()
    }
    /// Admission and cancellation of capture choices.
    pub(crate) fn native_requests(&self) -> &Arc<crate::dialog_request::NativeRequests> {
        &self.native_requests
    }
    /// Bounded permission-scoped capture sessions.
    pub(crate) fn capture_sessions(&self) -> &crate::capture_sessions::Sessions {
        &self.capture_sessions
    }
}

/// The backend, answering, for as long as this is held.
pub struct Served {
    /// End publishers and remove session grants before the service disappears.
    capture_sessions: Arc<crate::capture_sessions::Sessions>,
    /// The actual writer for those exact session grants.
    machine: Arc<dyn TheMachine>,
    /// The connection the portals are served on; dropping it stops them.
    connection: zbus::blocking::Connection,
    /// Dropped to stop the thread watching how the machine looks.
    stop: Option<Sender<()>>,
    /// That thread, waited for when this is dropped, so nothing is sent on
    /// the portals' behalf after they stop answering.
    watching: Option<JoinHandle<()>>,
    /// Stop dialog tasks before releasing the public frontend name.
    native_requests: Arc<crate::dialog_request::NativeRequests>,
}

impl Drop for Served {
    fn drop(&mut self) {
        self.native_requests.stop();
        self.capture_sessions.stop(self.machine.as_ref());
        let _released = self.connection.release_name(THE_PORTALS_NAME);
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
