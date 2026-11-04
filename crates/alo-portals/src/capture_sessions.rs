//! Session ownership and teardown, shared by every ScreenCast method.

use crate::{asked::Asked, serving::Backend};
use alo_capability::GrantId;
use alo_media_server::capture::Stream;
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, Ordering},
    },
    time::SystemTime,
};
use zbus::{
    fdo,
    message::Header,
    names::{BusName, OwnedUniqueName},
    zvariant::OwnedObjectPath,
};

/// The ordered public protocol phases. Starting cannot be replayed on one session.
#[derive(Default)]
pub(super) enum Phase {
    /// Created, but no types have been selected.
    #[default]
    Created,
    /// SelectSources was called once with these supported source bits.
    Selected(u32),
    /// A trusted picker or its selected source is being prepared.
    Starting,
    /// One successfully started native source.
    Streaming,
}

/// Exact session identity, never transferable between bus connections.
pub(super) struct Session {
    /// Predictable path built only from the original caller and validated token.
    pub(super) path: OwnedObjectPath,
    /// The authenticated original unique bus name.
    pub(super) owner: OwnedUniqueName,
    /// The sandbox identity authenticated for that bus owner.
    pub(super) application: String,
    /// Session.Close, disconnect, revoked permission, or ended source stops all work.
    pub(super) closed: AtomicBool,
    /// Ordered state transitions are committed before starting asynchronous tasks.
    pub(super) phase: Mutex<Phase>,
    /// Source, publisher, and exact grant survive until teardown.
    pub(super) running: Mutex<Option<Running>>,
}

/// The one materialisation of a fresh, session-scoped capture choice.
pub(super) struct Running {
    /// Only this new grant can authorise this session, even if broader grants exist.
    pub(super) grant: GrantId,
    /// The supervised native publisher owns the media node's entire lifetime.
    pub(super) stream: Stream,
}

/// A bounded service-owned list; bus objects contain session data, not a backend.
#[derive(Default)]
pub(super) struct Sessions(Mutex<BTreeMap<String, Arc<Session>>>);

impl Sessions {
    /// Service shutdown stops publishers before releasing the public bus name.
    pub(super) fn stop(&self, machine: &dyn crate::TheMachine) {
        let all = std::mem::take(&mut *self.0.lock().unwrap_or_else(PoisonError::into_inner));
        for session in all.values() {
            session.closed.store(true, Ordering::Release);
            if let Some(running) = session
                .running
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take()
            {
                drop(running.stream);
                machine.revoke_choice(running.grant);
            }
        }
    }
    /// Insert before responding to CreateSession; reject duplicates and overload.
    pub(super) fn insert(&self, session: Arc<Session>) -> fdo::Result<()> {
        let mut all = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if all.len() >= 8 {
            return Err(fdo::Error::LimitsExceeded(
                "too many recording sessions".into(),
            ));
        }
        if all.contains_key(session.path.as_str()) {
            return Err(fdo::Error::InvalidArgs(
                "session token already in use".into(),
            ));
        }
        all.insert(session.path.as_str().to_owned(), session);
        Ok(())
    }
    /// Every operation rechecks both the connection and its authenticated sandbox.
    pub(super) fn owned(&self, path: &OwnedObjectPath, asked: &Asked) -> Option<Arc<Session>> {
        self.0
            .lock()
            .ok()?
            .get(path.as_str())
            .filter(|session| {
                session.owner == *asked.sender()
                    && Some(session.application.as_str()) == asked.application()
                    && !session.closed.load(Ordering::Acquire)
            })
            .cloned()
    }
    /// Reconcile source lifetime and grants before allowing another frame interval.
    pub(super) async fn refresh(&self, bus: &zbus::Connection, backend: &Backend) {
        let all: Vec<_> = self
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .values()
            .cloned()
            .collect();
        if all.is_empty() {
            return;
        }
        let grants = backend.machine().grants();
        for session in all {
            let alive = match fdo::DBusProxy::new(bus).await {
                Ok(daemon) => daemon
                    .name_has_owner(BusName::Unique(session.owner.as_ref()))
                    .await
                    .unwrap_or(false),
                Err(_) => false,
            };
            let failed = session
                .running
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .as_ref()
                .is_some_and(|running| {
                    running.stream.ended()
                        || grants.as_ref().is_none_or(|grants| {
                            crate::Request::of(&session.application, crate::Portal::ScreenCapture)
                                .ok()
                                .and_then(|request| {
                                    request.judged_choice(grants, running.grant, SystemTime::now())
                                })
                                .is_none()
                        })
                });
            if backend.native_requests().stopped()
                || !alive
                || failed
                || session.closed.load(Ordering::Acquire)
            {
                self.close(bus, backend, &session).await;
            }
        }
    }
    /// Stop media first, revoke its exact permission, notify, then remove objects.
    pub(super) async fn close(&self, bus: &zbus::Connection, backend: &Backend, session: &Session) {
        session.closed.store(true, Ordering::Release);
        let running = session
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(running) = running {
            drop(running.stream);
            backend.machine().revoke_choice(running.grant);
        }
        let _sent = bus
            .emit_signal(
                Some(BusName::Unique(session.owner.as_ref())),
                session.path.as_ref(),
                "org.freedesktop.portal.Session",
                "Closed",
                &(std::collections::HashMap::<
                    String,
                    zbus::zvariant::OwnedValue,
                >::new(),),
            )
            .await;
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(session.path.as_str());
        let _removed = bus
            .object_server()
            .remove::<SessionObject, _>(session.path.as_ref())
            .await;
    }
}

/// The caller can end only its own session, including during an open chooser.
pub(super) struct SessionObject(pub(super) Arc<Session>);
#[zbus::interface(name = "org.freedesktop.portal.Session")]
impl SessionObject {
    /// Session protocol version.
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        1
    }
    /// Mark for teardown on the frontend's bounded lifetime watcher.
    async fn close(&self, #[zbus(header)] header: Header<'_>) -> fdo::Result<()> {
        if header.sender().map(|name| name.as_str()) != Some(self.0.owner.as_str()) {
            return Err(fdo::Error::AccessDenied(
                "the session belongs to another caller".into(),
            ));
        }
        self.0.closed.store(true, Ordering::Release);
        // Stop the publisher before replying; permission and bus cleanup follow.
        if let Some(running) = self
            .0
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
        {
            running.stream.stop();
        }
        Ok(())
    }
}
