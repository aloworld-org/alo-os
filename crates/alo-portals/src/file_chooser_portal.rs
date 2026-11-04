//! Save through the native chooser, then export only the chosen file to Flatpak.

use crate::{
    Outcome, Portal, Request, Unanswered,
    asked::Asked,
    dialog_request::{NATIVE_BACKEND, NATIVE_OBJECT, NativeRequest},
    serving::Backend,
};
use std::{collections::HashMap, path::PathBuf};
use zbus::{
    fdo,
    message::Header,
    zvariant::{OwnedObjectPath, OwnedValue, Value},
};

/// The public save dialog frontend; selection itself is rented, never simulated.
pub(crate) struct FileChooserPortal {
    /// Identity, grants, native request admission, document exports, and record.
    pub(crate) backend: Backend,
}

#[zbus::interface(name = "org.freedesktop.portal.FileChooser")]
impl FileChooserPortal {
    /// Version one includes saving one file and no multi-file save operation.
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        1
    }

    /// A native save destination, returned without granting a folder.
    async fn save_file(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] bus: &zbus::Connection,
        parent_window: &str,
        title: &str,
        mut options: HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedObjectPath> {
        let asked = Asked::on(bus, &header, &options, &self.backend, Portal::FileChooser).await?;
        if crate::caller::named(asked.application()).is_none() {
            return asked
                .answered(
                    bus,
                    &self.backend,
                    Outcome::Unanswered(Unanswered::NotIdentified),
                )
                .await;
        }
        // The native backend receives only the chooser's documented options.
        options.retain(|key, _| {
            matches!(
                key.as_str(),
                "accept_label"
                    | "modal"
                    | "filters"
                    | "current_filter"
                    | "choices"
                    | "current_name"
                    | "current_folder"
                    | "current_file"
            )
        });
        let request = self
            .backend
            .native_requests()
            .begin(asked, bus, &self.backend)
            .await?;
        let handle = request.handle();
        let backend = self.backend.clone();
        let connection = bus.clone();
        let parent = parent_window.to_owned();
        let title = title.to_owned();
        bus.executor()
            .spawn(
                async move {
                    futures_lite::future::race(
                        save(&request, &connection, &backend, &parent, &title, options),
                        async {
                            request.ended(&connection).await;
                            request.rollback_grant();
                            request.close_native(&connection).await;
                            request
                                .complete(&connection, &backend, Outcome::Cancelled, HashMap::new())
                                .await;
                        },
                    )
                    .await;
                },
                "alo-save-file",
            )
            .detach();
        Ok(handle)
    }

    /// File opening is outside this first save integration and is refused explicitly.
    async fn open_file(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] bus: &zbus::Connection,
        _parent_window: &str,
        _title: &str,
        options: HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedObjectPath> {
        Asked::on(bus, &header, &options, &self.backend, Portal::FileChooser)
            .await?
            .answered(
                bus,
                &self.backend,
                Outcome::Unanswered(Unanswered::NotDecidedHere),
            )
            .await
    }
}

/// Keep the method responsive while the person's native dialog remains open.
async fn save(
    request: &NativeRequest,
    bus: &zbus::Connection,
    backend: &Backend,
    parent: &str,
    title: &str,
    options: HashMap<String, OwnedValue>,
) {
    let result = choose(request, bus, parent, title, options).await;
    if request.cancelled() {
        request.close_native(bus).await;
        request
            .complete(bus, backend, Outcome::Cancelled, HashMap::new())
            .await;
        return;
    }
    let mut chosen = match result {
        Ok((0, chosen)) => chosen,
        Ok((1, _)) => {
            request
                .complete(bus, backend, Outcome::Cancelled, HashMap::new())
                .await;
            return;
        }
        _ => {
            request.close_native(bus).await;
            request
                .complete(
                    bus,
                    backend,
                    Outcome::Unanswered(Unanswered::DialogUnavailable),
                    HashMap::new(),
                )
                .await;
            return;
        }
    };
    let Some(path) = chosen
        .remove("uris")
        .and_then(|value| Vec::<String>::try_from(value).ok())
        .filter(|uris| uris.len() == 1)
        .and_then(|uris| uris.into_iter().next())
        .and_then(|uri| selected_path(&uri).ok())
    else {
        request
            .complete(
                bus,
                backend,
                Outcome::Unanswered(Unanswered::DocumentUnavailable),
                HashMap::new(),
            )
            .await;
        return;
    };
    let Some(application) = request.asked.application() else {
        request
            .complete(
                bus,
                backend,
                Outcome::Unanswered(Unanswered::NotIdentified),
                HashMap::new(),
            )
            .await;
        return;
    };
    let domain = match Request::over(application, Portal::FileChooser, &path) {
        Ok(domain) => domain,
        Err(why) => {
            request
                .complete(bus, backend, Outcome::NotARequest(why), HashMap::new())
                .await;
            return;
        }
    };
    let Some((grants, grant)) = backend
        .machine()
        .grant_choice(application, alo_capability::Reach::File(path.clone()))
    else {
        request
            .complete(
                bus,
                backend,
                Outcome::Unanswered(Unanswered::GrantsUnread),
                HashMap::new(),
            )
            .await;
        return;
    };
    request.made_grant(grant);
    let allowed = match domain.judged_choice(&grants, grant, std::time::SystemTime::now()) {
        Some(allowed) => allowed,
        None => {
            backend.machine().revoke_choice(grant);
            request
                .complete(
                    bus,
                    backend,
                    Outcome::Unanswered(Unanswered::GrantsUnread),
                    HashMap::new(),
                )
                .await;
            return;
        }
    };
    if request.cancelled() {
        backend.machine().revoke_choice(grant);
        request
            .complete(bus, backend, Outcome::Cancelled, HashMap::new())
            .await;
        return;
    }
    if request
        .asked
        .recorded(backend, Outcome::FilesChosen(allowed))
        .is_err()
    {
        backend.machine().revoke_choice(grant);
        request.respond(bus, 2, HashMap::new()).await;
        return;
    }
    let (uri, export) = match backend
        .documents()
        .export(bus, backend, &path, grant, request)
        .await
    {
        Ok(exported) => exported,
        Err(why) => {
            backend.machine().revoke_choice(grant);
            request
                .complete(bus, backend, Outcome::Unanswered(why), HashMap::new())
                .await;
            return;
        }
    };
    let still_granted = backend.machine().grants().is_some_and(|grants| {
        grants
            .active_at(std::time::SystemTime::now())
            .any(|held| held.id == grant)
    });
    if request.cancelled() || !still_granted {
        backend.machine().revoke_choice(grant);
        backend.documents().withdraw(bus, backend, &export).await;
        request
            .complete(bus, backend, Outcome::Cancelled, HashMap::new())
            .await;
        return;
    }
    let Ok(uris) = OwnedValue::try_from(Value::from(vec![uri])) else {
        backend.machine().revoke_choice(grant);
        backend.documents().withdraw(bus, backend, &export).await;
        request
            .complete(
                bus,
                backend,
                Outcome::Unanswered(Unanswered::DocumentUnavailable),
                HashMap::new(),
            )
            .await;
        return;
    };
    chosen.retain(|key, _| matches!(key.as_str(), "choices" | "current_filter"));
    chosen.insert("uris".into(), uris);
    request.respond(bus, 0, chosen).await;
    request.keep_grant();
}

/// The native backend is asked on the request handle already owned by this caller.
async fn choose(
    request: &NativeRequest,
    bus: &zbus::Connection,
    parent: &str,
    title: &str,
    options: HashMap<String, OwnedValue>,
) -> zbus::Result<(u32, HashMap<String, OwnedValue>)> {
    let proxy = zbus::Proxy::new(
        bus,
        NATIVE_BACKEND,
        NATIVE_OBJECT,
        "org.freedesktop.impl.portal.FileChooser",
    )
    .await?;
    proxy
        .call(
            "SaveFile",
            &(
                request.handle(),
                request.asked.application().unwrap_or(""),
                parent,
                title,
                options,
            ),
        )
        .await
}

/// Canonicalise the chosen file or its existing parent without creating anything.
fn selected_path(uri: &str) -> Result<PathBuf, ()> {
    let path = url::Url::parse(uri)
        .map_err(|_| ())?
        .to_file_path()
        .map_err(|()| ())?;
    match path.symlink_metadata() {
        Ok(_) => {
            let path = path.canonicalize().map_err(|_| ())?;
            if path.is_file() { Ok(path) } else { Err(()) }
        }
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
            let name = path.file_name().ok_or(())?;
            let parent = path.parent().ok_or(())?.canonicalize().map_err(|_| ())?;
            if !parent.is_dir() {
                return Err(());
            }
            Ok(parent.join(name))
        }
        Err(_) => Err(()),
    }
}
