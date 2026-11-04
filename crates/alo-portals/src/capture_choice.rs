//! The trusted native source question, always asked again for a new session.

use crate::{
    Unanswered,
    dialog_request::{NATIVE_BACKEND, NATIVE_OBJECT, NativeRequest},
    serving::Backend,
    words,
};
use alo_capture_protocol::{Source, SourceKind};
use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, Instant},
};
use zbus::zvariant::{OwnedValue, Value};

/// A fresh confirmed source from this session's actual compositor.
pub(super) async fn choose(
    request: &NativeRequest,
    bus: &zbus::Connection,
    backend: &Backend,
    parent: &str,
    types: u32,
) -> Result<Option<(PathBuf, Source)>, Unanswered> {
    let wayland = wayland(bus).await?;
    let mut sources = resolved(
        alo_media_server::capture::sources(wayland.clone())
            .map_err(|_| Unanswered::CaptureUnavailable)?,
    )
    .await?;
    sources.retain(|source| match source.kind {
        SourceKind::Monitor => types & 1 != 0,
        SourceKind::Window => types & 2 != 0,
    });
    if sources.is_empty() {
        return Err(Unanswered::CaptureUnavailable);
    }
    let strings = backend
        .machine()
        .strings()
        .ok_or(Unanswered::DialogUnavailable)?;
    let say = |word: alo_strings::Word| {
        strings
            .say(&word.key(), &alo_strings::Filling::nothing())
            .text()
            .to_owned()
    };
    let entries: Vec<_> = sources
        .iter()
        .map(|source| {
            let kind = say(match source.kind {
                SourceKind::Monitor => words::SHARE_SCREEN,
                SourceKind::Window => words::SHARE_WINDOW,
            });
            let label = if source.name.is_empty() {
                kind
            } else {
                format!("{kind} · {}", source.name)
            };
            (source.id.clone(), label)
        })
        .collect();
    let first = sources.first().ok_or(Unanswered::CaptureUnavailable)?;
    let choices = vec![(
        "source".to_owned(),
        say(words::SHARE_SOURCE),
        entries,
        first.id.clone(),
    )];
    let choices =
        OwnedValue::try_from(Value::from(choices)).map_err(|_| Unanswered::DialogUnavailable)?;
    let options = HashMap::from([
        ("choices", choices),
        ("modal", OwnedValue::from(true)),
        (
            "deny_label",
            OwnedValue::try_from(Value::from(say(words::SHARE_CANCEL)))
                .map_err(|_| Unanswered::DialogUnavailable)?,
        ),
        (
            "grant_label",
            OwnedValue::try_from(Value::from(say(words::SHARE_CONFIRM)))
                .map_err(|_| Unanswered::DialogUnavailable)?,
        ),
    ]);
    let title = say(words::SHARE_TITLE);
    let subtitle = strings
        .say(
            &words::SHARE_SUBTITLE.key(),
            &alo_strings::Filling::of("application", request.asked.application().unwrap_or("")),
        )
        .text()
        .to_owned();
    let body = say(words::SHARE_BODY);
    let native = zbus::Proxy::new(
        bus,
        NATIVE_BACKEND,
        NATIVE_OBJECT,
        "org.freedesktop.impl.portal.Access",
    )
    .await
    .map_err(|_| Unanswered::DialogUnavailable)?;
    let (code, mut results): (u32, HashMap<String, OwnedValue>) = native
        .call(
            "AccessDialog",
            &(
                request.handle(),
                request.asked.application().unwrap_or(""),
                parent,
                title,
                subtitle,
                body,
                options,
            ),
        )
        .await
        .map_err(|_| Unanswered::DialogUnavailable)?;
    if code == 1 {
        return Ok(None);
    }
    if code != 0 {
        return Err(Unanswered::DialogUnavailable);
    }
    let returned = results
        .remove("choices")
        .and_then(|choices| Vec::<(String, String)>::try_from(choices).ok())
        .ok_or(Unanswered::DialogUnavailable)?;
    let selected = returned
        .iter()
        .find(|(key, _)| key == "source")
        .map(|(_, id)| id)
        .ok_or(Unanswered::DialogUnavailable)?;
    let source = sources
        .into_iter()
        .find(|source| source.id == *selected)
        .ok_or(Unanswered::CaptureUnavailable)?;
    Ok(Some((wayland, source)))
}

/// The desktop tells its user manager where it draws before launching apps.
async fn wayland(bus: &zbus::Connection) -> Result<PathBuf, Unanswered> {
    let manager = zbus::Proxy::new(
        bus,
        "org.freedesktop.systemd1",
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
    )
    .await
    .map_err(|_| Unanswered::CaptureUnavailable)?;
    let environment: Vec<String> = manager
        .get_property("Environment")
        .await
        .map_err(|_| Unanswered::CaptureUnavailable)?;
    environment
        .into_iter()
        .find_map(|entry| entry.strip_prefix("WAYLAND_DISPLAY=").map(PathBuf::from))
        .filter(|path| path.is_absolute())
        .ok_or(Unanswered::CaptureUnavailable)
}

/// Poll a bounded native operation while leaving request cancellation runnable.
pub(super) async fn resolved<T>(
    mut pending: alo_media_server::capture::Pending<T>,
) -> Result<T, Unanswered> {
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(result) = pending.try_result() {
            return result.map_err(|why| {
                eprintln!("Native capture could not continue: {why}");
                Unanswered::CaptureUnavailable
            });
        }
        if Instant::now() >= until {
            return Err(Unanswered::CaptureUnavailable);
        }
        async_io::Timer::after(Duration::from_millis(20)).await;
    }
}
