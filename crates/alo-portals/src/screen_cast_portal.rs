//! Public ScreenCast protocol. Each session is bound to its original bus owner.

mod starting;

use crate::{
    Outcome, Portal, Unanswered,
    asked::Asked,
    capture_sessions::{Phase, Session, SessionObject},
    dialog_request::NativeRequest,
    serving::Backend,
};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use zbus::{
    fdo,
    message::Header,
    zvariant::{OwnedFd, OwnedObjectPath, OwnedValue, Value},
};

/// The standard desktop frontend, with one fresh selection per recording.
pub(super) struct ScreenCastPortal {
    /// Caller identification, one grants policy, native dialogs, and record.
    pub(super) backend: Backend,
}

#[zbus::interface(name = "org.freedesktop.portal.ScreenCast")]
impl ScreenCastPortal {
    /// Version three supports source types and embedded cursor without persistence.
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        3
    }
    /// Only current monitors and independently rendered application windows.
    #[zbus(property, name = "AvailableSourceTypes")]
    fn available_source_types(&self) -> u32 {
        3
    }
    /// Complete output frames and window frames embed the compositor's pointer.
    #[zbus(property, name = "AvailableCursorModes")]
    fn available_cursor_modes(&self) -> u32 {
        2
    }

    /// Create no permission and expose no source catalogue to the application.
    async fn create_session(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] bus: &zbus::Connection,
        options: HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedObjectPath> {
        let asked = Asked::on(bus, &header, &options, &self.backend, Portal::ScreenCapture).await?;
        if crate::caller::named(asked.application()).is_none() {
            return refused(asked, bus, &self.backend, Unanswered::NotIdentified).await;
        }
        let token = options
            .get("session_handle_token")
            .and_then(|v| v.downcast_ref::<&str>().ok());
        let Some(token) = token else {
            return refused(asked, bus, &self.backend, Unanswered::NotAToken).await;
        };
        let path = crate::handle_for(asked.sender().as_str(), Some(token))
            .map_err(|_| fdo::Error::InvalidArgs("invalid session token".into()))?
            .replacen("/request/", "/session/", 1);
        let path = OwnedObjectPath::try_from(path)
            .map_err(|_| fdo::Error::InvalidArgs("invalid session path".into()))?;
        let session = Arc::new(Session {
            path: path.clone(),
            owner: asked.sender().clone(),
            application: asked.application().unwrap_or("").to_owned(),
            closed: AtomicBool::new(false),
            phase: Mutex::new(Phase::Created),
            running: Mutex::new(None),
        });
        if let Err(why) = self.backend.capture_sessions().insert(session.clone()) {
            let _ = asked.recorded(
                &self.backend,
                Outcome::Unanswered(Unanswered::CaptureUnavailable),
            );
            return Err(why);
        }
        if !bus
            .object_server()
            .at(path.clone(), SessionObject(session.clone()))
            .await?
        {
            self.backend
                .capture_sessions()
                .close(bus, &self.backend, &session)
                .await;
            return refused(asked, bus, &self.backend, Unanswered::CaptureUnavailable).await;
        }
        let response = asked
            .recorded(&self.backend, Outcome::CapturePrepared)
            .unwrap_or(2);
        let results = if response == 0 {
            HashMap::from([(
                "session_handle".to_owned(),
                owned(Value::from(path.as_str()))?,
            )])
        } else {
            self.backend
                .capture_sessions()
                .close(bus, &self.backend, &session)
                .await;
            HashMap::new()
        };
        asked.responded_with(bus, response, results).await
    }

    /// Select supported types exactly once. Every invalid selection ends its session.
    async fn select_sources(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] bus: &zbus::Connection,
        session_handle: OwnedObjectPath,
        options: HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedObjectPath> {
        let asked = Asked::on(bus, &header, &options, &self.backend, Portal::ScreenCapture).await?;
        let Some(session) = self
            .backend
            .capture_sessions()
            .owned(&session_handle, &asked)
        else {
            return refused(asked, bus, &self.backend, Unanswered::CaptureUnavailable).await;
        };
        let selection = selected_types(&options);
        let selected = {
            let mut phase = session
                .phase
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if matches!(*phase, Phase::Created)
                && let Some(types) = selection
            {
                *phase = Phase::Selected(types);
                true
            } else {
                false
            }
        };
        if !selected {
            self.backend
                .capture_sessions()
                .close(bus, &self.backend, &session)
                .await;
            return refused(asked, bus, &self.backend, Unanswered::CaptureUnavailable).await;
        }
        let response = asked
            .recorded(&self.backend, Outcome::CapturePrepared)
            .unwrap_or(2);
        if response != 0 {
            self.backend
                .capture_sessions()
                .close(bus, &self.backend, &session)
                .await;
        }
        asked.responded_with(bus, response, HashMap::new()).await
    }

    /// Return the handle immediately while the trusted picker is visible.
    async fn start(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] bus: &zbus::Connection,
        session_handle: OwnedObjectPath,
        parent_window: &str,
        options: HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedObjectPath> {
        let asked = Asked::on(bus, &header, &options, &self.backend, Portal::ScreenCapture).await?;
        let Some(session) = self
            .backend
            .capture_sessions()
            .owned(&session_handle, &asked)
        else {
            return refused(asked, bus, &self.backend, Unanswered::CaptureUnavailable).await;
        };
        let types = {
            let mut phase = session
                .phase
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let selected = if let Phase::Selected(types) = *phase {
                Some(types)
            } else {
                None
            };
            *phase = Phase::Starting;
            selected
        };
        let Some(types) = types else {
            self.backend
                .capture_sessions()
                .close(bus, &self.backend, &session)
                .await;
            return refused(asked, bus, &self.backend, Unanswered::CaptureUnavailable).await;
        };
        let request = self
            .backend
            .native_requests()
            .begin(asked, bus, &self.backend)
            .await?;
        let handle = request.handle();
        let backend = self.backend.clone();
        let connection = bus.clone();
        let parent = parent_window.to_owned();
        bus.executor()
            .spawn(
                async move {
                    starting::start(&request, &connection, &backend, &session, &parent, types)
                        .await;
                },
                "alo-screen-choice",
            )
            .detach();
        Ok(handle)
    }

    /// Return a registered and narrowed connection, never the unrestricted socket.
    #[zbus(name = "OpenPipeWireRemote")]
    async fn open_pipe_wire_remote(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] bus: &zbus::Connection,
        session_handle: OwnedObjectPath,
        options: HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedFd> {
        let asked = Asked::on(bus, &header, &options, &self.backend, Portal::ScreenCapture).await?;
        let result = starting::open_remote(&asked, bus, &self.backend, &session_handle).await;
        if result.is_err() {
            let _ = asked.recorded(
                &self.backend,
                Outcome::Unanswered(Unanswered::CaptureUnavailable),
            );
        }
        result
    }
}

/// Native methods keep refusal records even for foreign or invalid session paths.
async fn refused(
    asked: Asked,
    bus: &zbus::Connection,
    backend: &Backend,
    why: Unanswered,
) -> fdo::Result<OwnedObjectPath> {
    asked.answered(bus, backend, Outcome::Unanswered(why)).await
}

/// Convert a documented variant without leaking an internal encoding failure.
fn owned(value: Value<'_>) -> fdo::Result<OwnedValue> {
    OwnedValue::try_from(value)
        .map_err(|_| fdo::Error::Failed("capture result could not be encoded".into()))
}

/// An unsupported request never silently substitutes another sharing mode.
fn selected_types(options: &HashMap<String, OwnedValue>) -> Option<u32> {
    let number = |key: &str, fallback| {
        options
            .get(key)
            .map_or(Some(fallback), |v| v.downcast_ref::<u32>().ok())
    };
    let types = number("types", 1)?;
    if types == 0
        || types & !3 != 0
        || number("cursor_mode", 1)? != 2
        || number("persist_mode", 0)? != 0
        || options.contains_key("restore_token")
        || options
            .get("multiple")
            .is_some_and(|v| v.downcast_ref::<bool>().ok() != Some(false))
    {
        return None;
    }
    Some(types)
}
