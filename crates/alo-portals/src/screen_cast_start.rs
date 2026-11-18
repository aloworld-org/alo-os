//! Fresh choice → exact grant → record → native publisher → selected-node remote.

use super::*;
use crate::{capture_choice, capture_sessions::Running};
use std::{
    sync::PoisonError,
    time::{Duration, SystemTime},
};

/// The whole native operation is cancellable, including media-worker setup.
pub(super) async fn start(
    request: &NativeRequest,
    bus: &zbus::Connection,
    backend: &Backend,
    session: &Session,
    parent: &str,
    types: u32,
) {
    let result = futures_lite::future::race(
        prepare(request, bus, backend, session, parent, types),
        async {
            futures_lite::future::race(request.ended(bus), async {
                while !session.closed.load(Ordering::Acquire) {
                    async_io::Timer::after(Duration::from_millis(25)).await;
                }
            })
            .await;
            Ok(None)
        },
    )
    .await;
    match result {
        Ok(Some(results)) => {
            request.respond(bus, 0, results).await;
            request.keep_grant();
        }
        other => {
            request.rollback_grant();
            request.close_native(bus).await;
            backend
                .capture_sessions()
                .close(bus, backend, session)
                .await;
            let outcome = match other {
                Err(why) => Outcome::Unanswered(why),
                _ => Outcome::Cancelled,
            };
            request
                .complete(bus, backend, outcome, HashMap::new())
                .await;
        }
    }
}

/// No frame source is started until the person's choice is judged and recorded.
async fn prepare(
    request: &NativeRequest,
    bus: &zbus::Connection,
    backend: &Backend,
    session: &Session,
    parent: &str,
    types: u32,
) -> Result<Option<HashMap<String, OwnedValue>>, Unanswered> {
    let Some((wayland, source)) =
        capture_choice::choose(request, bus, backend, parent, types).await?
    else {
        return Ok(None);
    };
    if request.cancelled() || session.closed.load(Ordering::Acquire) {
        return Ok(None);
    }
    let (grants, grant) = backend
        .machine()
        .grant_choice(
            &session.application,
            alo_capability::Reach::Facility(alo_capability::Facility::ScreenContinuously),
        )
        .ok_or(Unanswered::GrantsUnread)?;
    request.made_grant(grant);
    let domain = crate::Request::of(&session.application, Portal::ScreenCapture)
        .map_err(|_| Unanswered::NotIdentified)?;
    let allowed = domain
        .judged_choice(&grants, grant, SystemTime::now())
        .ok_or(Unanswered::GrantsUnread)?;
    request
        .asked
        .recorded(backend, Outcome::ScreenShared(allowed))
        .map_err(|_| Unanswered::NotWritten)?;
    let stream = capture_choice::resolved(
        alo_media_server::capture::start(wayland, source.clone(), session.application.clone())
            .map_err(|_| Unanswered::CaptureUnavailable)?,
    )
    .await?;
    let still_allowed = backend
        .machine()
        .grants()
        .and_then(|grants| domain.judged_choice(&grants, grant, SystemTime::now()))
        .is_some();
    if request.cancelled() || !still_allowed || stream.ended() {
        return Err(Unanswered::CaptureUnavailable);
    }
    let kind: u32 = match source.kind {
        alo_capture_protocol::SourceKind::Monitor => 1,
        alo_capture_protocol::SourceKind::Window => 2,
    };
    let props = HashMap::from([
        (
            "size".to_owned(),
            owned(Value::from((source.width as i32, source.height as i32)))
                .map_err(|_| Unanswered::CaptureUnavailable)?,
        ),
        ("source_type".to_owned(), OwnedValue::from(kind)),
    ]);
    let streams = owned(Value::from(vec![(stream.node.id, props)]))
        .map_err(|_| Unanswered::CaptureUnavailable)?;
    {
        let mut running = session
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if session.closed.load(Ordering::Acquire) {
            return Ok(None);
        }
        *running = Some(Running { grant, stream });
    }
    *session.phase.lock().unwrap_or_else(PoisonError::into_inner) = Phase::Streaming;
    Ok(Some(HashMap::from([("streams".to_owned(), streams)])))
}

/// Judge before preparation and immediately before returning the narrowed socket.
pub(super) async fn open_remote(
    asked: &Asked,
    _bus: &zbus::Connection,
    backend: &Backend,
    path: &OwnedObjectPath,
) -> fdo::Result<OwnedFd> {
    let session = backend
        .capture_sessions()
        .owned(path, asked)
        .ok_or_else(denied)?;
    let (node, grant) = {
        let running = session
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let running = running
            .as_ref()
            .filter(|running| !running.stream.ended())
            .ok_or_else(denied)?;
        (running.stream.node, running.grant)
    };
    let domain =
        crate::Request::of(&session.application, Portal::ScreenCapture).map_err(|_| denied())?;
    let allowed = backend
        .machine()
        .grants()
        .and_then(|grants| domain.judged_choice(&grants, grant, SystemTime::now()))
        .ok_or_else(denied)?;
    asked
        .recorded(backend, Outcome::ScreenShared(allowed))
        .map_err(|_| denied())?;
    let remote = capture_choice::resolved(
        alo_media_server::capture::remote(node, session.application.clone())
            .map_err(|_| denied())?,
    )
    .await
    .map_err(|_| denied())?;
    let active = session
        .running
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .is_some_and(|running| running.grant == grant && !running.stream.ended());
    if session.closed.load(Ordering::Acquire)
        || !active
        || backend
            .machine()
            .grants()
            .and_then(|grants| domain.judged_choice(&grants, grant, SystemTime::now()))
            .is_none()
    {
        return Err(denied());
    }
    Ok(remote.into())
}

/// A caller never receives enough detail to probe someone else's session.
fn denied() -> fdo::Error {
    fdo::Error::AccessDenied("this recording session is unavailable".into())
}
