//! The upstream desktop portal's permission barriers and descriptor handoff.

use alo_capture_protocol::MediaNode;
use pipewire::{
    self as pw,
    permissions::{Permission, PermissionFlags},
    types::ObjectType,
};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    io,
    os::fd::OwnedFd,
    rc::Rc,
    time::{Duration, Instant},
};

/// Bound helper lifetime even if a media server stops answering barriers.
const ROUNDTRIP_LIMIT: Duration = Duration::from_secs(4);
/// A pathological graph cannot make the permission-preparation snapshot unbounded.
const MAX_NODES: usize = 8192;

/// The global identities needed to restrict one connection, never frame contents.
#[derive(Default)]
struct Snapshot {
    /// The factory an ordinary consuming client needs to create its stream.
    factory: Option<u32>,
    /// IDs and immutable serials, so an ID reused for another node is refused.
    nodes: BTreeMap<u32, u64>,
    /// An overflowing snapshot is an error, never a partial quiet graph.
    overflowed: bool,
}

/// Prepare the selected-node connection and transfer unique descriptor ownership.
///
/// # Errors
/// An unavailable server, invalid identity, stale node or serial, missing client
/// factory, permission error, timed-out barrier, or descriptor handoff failure.
pub fn restricted_remote(selected: MediaNode, application: &str) -> io::Result<OwnedFd> {
    if application.is_empty()
        || application.len() > 255
        || !application
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid capture application identity",
        ));
    }
    let main_loop = pw::main_loop::MainLoopBox::new(None).map_err(failed)?;
    let context = pw::context::ContextBox::new(main_loop.loop_(), None).map_err(failed)?;
    let mut properties = pw::properties::PropertiesBox::new();
    properties.insert("application.name", "alo capture remote");
    properties.insert("pipewire.access.portal.app_id", application);
    properties.insert("pipewire.access.portal.media_roles", "");
    let core = context.connect(Some(properties)).map_err(failed)?;
    let registry = core.get_registry().map_err(failed)?;
    let snapshot = Rc::new(RefCell::new(Snapshot::default()));
    let seen = snapshot.clone();
    let removed = snapshot.clone();
    let listener = registry
        .add_listener_local()
        .global(move |global| {
            let Some(props) = global.props else {
                return;
            };
            let mut seen = seen.borrow_mut();
            if global.type_ == ObjectType::Factory
                && props.get("factory.type.name") == Some("PipeWire:Interface:ClientNode")
            {
                seen.factory = Some(global.id);
            }
            if global.type_ == ObjectType::Node {
                if seen.nodes.len() >= MAX_NODES {
                    seen.overflowed = true;
                    return;
                }
                if let Some(serial) = props
                    .get("object.serial")
                    .and_then(|serial| serial.parse::<u64>().ok())
                {
                    seen.nodes.insert(global.id, serial);
                }
            }
        })
        .global_remove(move |id| {
            removed.borrow_mut().nodes.remove(&id);
        })
        .register();
    roundtrip(main_loop.loop_(), &core)?;
    let factory = {
        let seen = snapshot.borrow();
        if seen.overflowed || seen.nodes.get(&selected.id) != Some(&selected.serial) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "the selected capture node no longer exists",
            ));
        }
        seen.factory.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "the media server has no client-node factory",
            )
        })?
    };
    let rwx = PermissionFlags::R | PermissionFlags::W | PermissionFlags::X;
    core.update_self_permissions(&[
        Permission::new(pw::core::PW_ID_CORE, rwx),
        Permission::new(factory, PermissionFlags::R),
        Permission::new(selected.id, rwx),
        Permission::new(u32::MAX, PermissionFlags::empty()),
    ])?;
    roundtrip(main_loop.loop_(), &core)?;
    if snapshot.borrow().nodes.get(&selected.id) != Some(&selected.serial) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "the capture source ended while access was prepared",
        ));
    }
    drop(listener);
    drop(registry);
    // Flush destruction of the helper's registry proxy before ownership moves.
    roundtrip(main_loop.loop_(), &core)?;
    core.into_fd()
}

/// Wait only for this barrier, reporting both protocol errors and lost servers.
fn roundtrip(loop_: &pw::loop_::Loop, core: &pw::core::Core) -> io::Result<()> {
    let done = Rc::new(Cell::new(false));
    let errored = Rc::new(Cell::new(false));
    let completed = done.clone();
    let rejected = errored.clone();
    let pending = core.sync(0).map_err(failed)?;
    let _listener = core
        .add_listener_local()
        .done(move |id, sequence| {
            if id == pw::core::PW_ID_CORE && sequence == pending {
                completed.set(true);
            }
        })
        .error(move |_id, _sequence, _code, _message| {
            rejected.set(true);
        })
        .register();
    let deadline = Instant::now() + ROUNDTRIP_LIMIT;
    while !done.get() {
        if errored.get() {
            return Err(io::Error::other(
                "the media server rejected the capture barrier",
            ));
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "the media server did not finish the capture barrier",
            ));
        }
        if loop_.iterate(pw::loop_::Timeout::Finite(Duration::from_millis(25))) < 0 {
            return Err(io::Error::other("the media-server loop stopped"));
        }
    }
    Ok(())
}

/// Preserve rented-engine diagnostics without adding another error protocol.
fn failed(why: impl std::fmt::Display) -> io::Error {
    io::Error::other(why.to_string())
}
