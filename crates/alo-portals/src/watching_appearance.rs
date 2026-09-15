//! `SettingChanged`, sent when what an application read has moved.
//!
//! An application that follows light and dark reads the setting once and then
//! listens. What it listens for changes two ways: a person changes their
//! settings, or the clock crosses the time their schedule turns dark or light,
//! and no file changes at all. So the backend **looks** — the person's
//! appearance and the time of day, read through [`crate::TheMachine`] every
//! [`WATCHED_EVERY`] — and compares what it resolves with what it last
//! resolved.
//!
//! # Sent only to an application that could read it
//!
//! When a value moved, every connection on the bus is found (`ListNames`), and
//! each is judged **at that moment** the way a `ReadOne` from it would be
//! ([`appearance_settings::allowed`]): the sandbox of the process behind it
//! names an application, and that application holds a grant of the appearance
//! settings now. `SettingChanged` is sent to that connection alone — a signal
//! with a destination, never a broadcast — and a connection that could not
//! read the value is sent nothing. A revoked grant stops the next change
//! reaching the application, as it stops the next read.
//!
//! Every signal is recorded as [`Outcome::AppearanceSent`] with its application
//! named **before** it is sent, as every answer the backend gives is: an
//! application is never sent something the record does not hold, and a signal
//! the record did not keep is not sent. A connection
//! that was sent nothing is not recorded: it asked for nothing, and a record of
//! every program on the bus at every change would bury the requests that were
//! refused.
//!
//! **Except a connection whose process was gone** before it could be named
//! ([`Sandboxed::Gone`]). What was read for it may have been another process's
//! sandbox, under a number that had been given away, and that is exactly the
//! refusal a person reading the record needs to find. It is recorded as
//! [`Unanswered::NotIdentified`], naming nobody, and sent nothing.

use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, SystemTime};

use zbus::fdo;
use zbus::names::{BusName, OwnedBusName, UniqueName};
use zbus::object_server::SignalEmitter;

use crate::answered::{Answered, Outcome, Unanswered};
use crate::appearance_settings::{self, THE_NAMESPACE, Values};
use crate::caller::{caller_of, named};
use crate::handle::THE_PORTALS_OBJECT;
use crate::portal::Portal;
use crate::sandboxed::Sandboxed;
use crate::serving::Backend;
use crate::settings_portal::{SettingsPortal, written};

/// How often the person's appearance and the time of day are looked at.
///
/// A second: a person who flips to dark sees their applications follow before
/// they have moved the pointer away, and a schedule is exact to the minute.
pub const WATCHED_EVERY: Duration = Duration::from_secs(1);

/// Look at the appearance every [`WATCHED_EVERY`], and send what moved since
/// `last` to whoever could read it, until `stop` is sent to or dropped.
///
/// `last` is what the backend resolved as it began serving, read before this
/// thread starts, so a change made the moment the portals answer is a change.
pub(crate) fn watch(
    bus: &zbus::Connection,
    backend: &Backend,
    mut last: Option<Values>,
    stop: &Receiver<()>,
) {
    loop {
        match stop.recv_timeout(WATCHED_EVERY) {
            Err(RecvTimeoutError::Timeout) => {}
            Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
        }
        // Settings that did not read this time are not a change: what an
        // application was last told stands until they read again.
        let Ok(now) = appearance_settings::read(backend.machine()) else {
            continue;
        };
        if let Some(before) = &last {
            let changed = appearance_settings::changed(before, &now);
            if !changed.is_empty() {
                zbus::block_on(sent_to_whoever_may_read(bus, backend, &changed));
            }
        }
        last = Some(now);
    }
}

/// Send `changed` to each connection whose application may read it now.
async fn sent_to_whoever_may_read(bus: &zbus::Connection, backend: &Backend, changed: &Values) {
    let Ok(daemon) = fdo::DBusProxy::new(bus).await else {
        return;
    };
    let Ok(names) = daemon.list_names().await else {
        return;
    };
    let ours = bus.unique_name();
    for name in names {
        let Some(connection) = unique(&name) else {
            continue;
        };
        if ours.is_some_and(|ours| ours.as_str() == connection.as_str()) {
            continue;
        }
        let caller = caller_of(bus, &connection, backend).await;
        if caller == Sandboxed::Gone {
            // Nothing is sent to it whether or not this is kept.
            let _ = backend.record().keep(Answered::new(
                SystemTime::now(),
                None,
                Portal::Settings,
                Outcome::Unanswered(Unanswered::NotIdentified),
            ));
            continue;
        }
        let application = caller.into_application();
        let Ok(allowed) = appearance_settings::allowed(
            application.as_deref(),
            backend.machine(),
            SystemTime::now(),
        ) else {
            continue;
        };
        let kept = backend.record().keep(Answered::new(
            SystemTime::now(),
            named(application.as_deref()),
            Portal::Settings,
            Outcome::AppearanceSent(allowed),
        ));
        if kept.is_ok() {
            send(bus, &connection, changed).await;
        }
    }
}

/// A bus name that is a connection's own, rather than a well-known name.
fn unique(name: &OwnedBusName) -> Option<UniqueName<'_>> {
    match name.as_ref() {
        BusName::Unique(unique) => Some(unique),
        BusName::WellKnown(_) => None,
    }
}

/// Send every changed setting to `connection`.
///
/// A connection that has gone by the time it is sent to is not an error: it is
/// an application that stopped listening, and the next change finds the bus
/// without it.
async fn send(bus: &zbus::Connection, connection: &UniqueName<'_>, changed: &Values) {
    let Ok(emitter) = SignalEmitter::new(bus, THE_PORTALS_OBJECT) else {
        return;
    };
    let emitter = emitter.set_destination(BusName::Unique(connection.as_ref()));
    for (setting, value) in changed {
        if SettingsPortal::setting_changed(&emitter, THE_NAMESPACE, setting.key(), written(*value))
            .await
            .is_err()
        {
            return;
        }
    }
}
