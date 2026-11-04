//! Bounded native requests, owned and cancellable only by their bus caller.

use crate::{Outcome, asked::Asked, serving::Backend};
use std::sync::{Mutex, PoisonError};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};
use zbus::{
    fdo,
    message::Header,
    names::BusName,
    zvariant::{OwnedObjectPath, OwnedValue},
};

/// The one native backend rented for selection and consent.
pub(crate) const NATIVE_BACKEND: &str = "org.freedesktop.impl.portal.desktop.gtk";
/// The backend's well-known object.
pub(crate) const NATIVE_OBJECT: &str = "/org/freedesktop/portal/desktop";
/// Bounds both visible dialogs and their pending bus tasks.
const MAX_PENDING: usize = 8;

/// Shared admission and service-lifetime state.
#[derive(Default)]
pub(crate) struct NativeRequests {
    /// Requests still owned by their completion tasks.
    active: AtomicUsize,
    /// Service shutdown cancels every outstanding request.
    stopping: AtomicBool,
}

impl NativeRequests {
    /// End pending dialogs when the frontend stops serving.
    pub(crate) fn stop(&self) {
        self.stopping.store(true, Ordering::Release);
    }

    /// Whether the frontend is ending all of its native requests.
    pub(crate) fn stopped(&self) -> bool {
        self.stopping.load(Ordering::Acquire)
    }

    /// Register a handle before returning it, refusing duplicates and overload.
    pub(crate) async fn begin(
        self: &Arc<Self>,
        asked: Asked,
        bus: &zbus::Connection,
        backend: &Backend,
    ) -> fdo::Result<NativeRequest> {
        if self.stopped() {
            let _kept = asked.recorded(
                backend,
                Outcome::Unanswered(crate::Unanswered::DialogUnavailable),
            );
            return Err(fdo::Error::Failed("the portal service is stopping".into()));
        }
        if self
            .active
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < MAX_PENDING).then_some(active + 1)
            })
            .is_err()
        {
            let _kept = asked.recorded(
                backend,
                Outcome::Unanswered(crate::Unanswered::DialogUnavailable),
            );
            return Err(fdo::Error::LimitsExceeded(
                "too many pending selections".into(),
            ));
        }
        let request = NativeRequest {
            asked,
            admission: self.clone(),
            cancelled: Arc::new(AtomicBool::new(false)),
            finished: AtomicBool::new(false),
            grant: Mutex::new(None),
            backend: backend.clone(),
        };
        let registered = bus
            .object_server()
            .at(
                request.asked.handle(),
                CloseRequest {
                    owner: request.asked.sender().clone(),
                    cancelled: request.cancelled.clone(),
                },
            )
            .await;
        let registered = match registered {
            Ok(registered) => registered,
            Err(why) => {
                let _kept = request.asked.recorded(
                    backend,
                    Outcome::Unanswered(crate::Unanswered::DialogUnavailable),
                );
                return Err(why.into());
            }
        };
        if !registered {
            let _kept = request.asked.recorded(
                backend,
                Outcome::Unanswered(crate::Unanswered::DialogUnavailable),
            );
            return Err(fdo::Error::InvalidArgs(
                "that request handle is already in use".into(),
            ));
        }
        Ok(request)
    }
}

/// The lifetime of one native dialog, including its authenticated caller.
pub(crate) struct NativeRequest {
    /// The request's identity, handle, and answer-record destination.
    pub(crate) asked: Asked,
    /// A bounded admission slot returned by Drop even on failure.
    admission: Arc<NativeRequests>,
    /// Set by the owner's Close, shutdown, or disappearance.
    cancelled: Arc<AtomicBool>,
    /// One completion wins even when a close races the native answer.
    finished: AtomicBool,
    /// A newly selected grant is rolled back unless its result was handed over.
    grant: Mutex<Option<alo_capability::GrantId>>,
    /// The writer used to undo an unfinished selection.
    backend: Backend,
}

impl Drop for NativeRequest {
    fn drop(&mut self) {
        self.rollback_grant();
        self.admission.active.fetch_sub(1, Ordering::AcqRel);
    }
}

impl NativeRequest {
    /// Keep rollback attached to the whole asynchronous operation.
    pub(crate) fn made_grant(&self, id: alo_capability::GrantId) {
        *self.grant.lock().unwrap_or_else(PoisonError::into_inner) = Some(id);
    }

    /// A completed file selection retains its revocable grant.
    pub(crate) fn keep_grant(&self) {
        self.grant
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
    }

    /// Cancellation or an aborted future undoes only its own selection.
    pub(crate) fn rollback_grant(&self) {
        if let Some(id) = self
            .grant
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
        {
            self.backend.machine().revoke_choice(id);
        }
    }

    /// Whether an explicit Close or service shutdown has ended the request.
    pub(crate) fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire) || self.admission.stopping.load(Ordering::Acquire)
    }

    /// Wait for cancellation or caller disappearance while a backend is open.
    pub(crate) async fn ended(&self, bus: &zbus::Connection) {
        loop {
            if self.finished.load(Ordering::Acquire) {
                std::future::pending::<()>().await;
            }
            let alive = match fdo::DBusProxy::new(bus).await {
                Ok(daemon) => daemon
                    .name_has_owner(BusName::Unique(self.asked.sender().as_ref()))
                    .await
                    .unwrap_or(false),
                Err(_) => false,
            };
            if self.cancelled() || !alive {
                self.cancelled.store(true, Ordering::Release);
                return;
            }
            async_io::Timer::after(Duration::from_millis(100)).await;
        }
    }

    /// Close the native backend's matching request after cancellation.
    pub(crate) async fn close_native(&self, bus: &zbus::Connection) {
        let close = async {
            if let Ok(proxy) = zbus::Proxy::new(
                bus,
                NATIVE_BACKEND,
                self.asked.handle(),
                "org.freedesktop.impl.portal.Request",
            )
            .await
            {
                let _: zbus::Result<()> = proxy.call("Close", &()).await;
            }
        };
        futures_lite::future::race(close, async {
            async_io::Timer::after(Duration::from_secs(2)).await;
        })
        .await;
    }

    /// Keep the answer, emit only to the owner, and retire the request object.
    pub(crate) async fn complete(
        &self,
        bus: &zbus::Connection,
        backend: &Backend,
        outcome: Outcome,
        mut results: HashMap<String, OwnedValue>,
    ) {
        if self.finished.swap(true, Ordering::AcqRel) {
            return;
        }
        let response = self.asked.recorded(backend, outcome).unwrap_or(2);
        if response != 0 {
            results.clear();
        }
        self.respond_inner(bus, response, results).await;
    }

    /// Finish an answer already kept before its side effect was attempted.
    pub(crate) async fn respond(
        &self,
        bus: &zbus::Connection,
        response: u32,
        results: HashMap<String, OwnedValue>,
    ) {
        if self.finished.swap(true, Ordering::AcqRel) {
            return;
        }
        self.respond_inner(bus, response, results).await;
    }

    /// Emit the claimed completion and retire its public request object.
    async fn respond_inner(
        &self,
        bus: &zbus::Connection,
        response: u32,
        results: HashMap<String, OwnedValue>,
    ) {
        if !self.cancelled() {
            let _sent = self.asked.responded_with(bus, response, results).await;
        }
        let _removed = bus
            .object_server()
            .remove::<CloseRequest, _>(self.asked.handle())
            .await;
    }

    /// The method reply is returned while this task owns the ongoing dialog.
    pub(crate) fn handle(&self) -> OwnedObjectPath {
        self.asked.handle().clone()
    }
}

/// The sole method exported at a pending request's handle.
struct CloseRequest {
    /// Another application cannot close this owner's chooser.
    owner: zbus::names::OwnedUniqueName,
    /// Shared with the completion task, without waiting for that task's locks.
    cancelled: Arc<AtomicBool>,
}

#[zbus::interface(name = "org.freedesktop.portal.Request")]
impl CloseRequest {
    /// Closing somebody else's request is refused without changing it.
    async fn close(&self, #[zbus(header)] header: Header<'_>) -> fdo::Result<()> {
        if header.sender().map(|sender| sender.as_str()) != Some(self.owner.as_str()) {
            return Err(fdo::Error::AccessDenied(
                "the request belongs to another caller".into(),
            ));
        }
        self.cancelled.store(true, Ordering::Release);
        Ok(())
    }
}
