//! Real parent-handle requests, shared with the compositor's protocol fixtures.
#![allow(
    dead_code,
    reason = "the support client also builds in examples without parent dialogs"
)]

use super::{Application, Events};
use wayland_client::{Connection, Dispatch, QueueHandle, delegate_noop};
use wayland_protocols::xdg::foreign::zv1::client::{
    zxdg_exported_v1::{self, ZxdgExportedV1},
    zxdg_exporter_v1::ZxdgExporterV1,
    zxdg_imported_v1::{self, ZxdgImportedV1},
    zxdg_importer_v1::ZxdgImporterV1,
};

impl Dispatch<ZxdgExportedV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &ZxdgExportedV1,
        event: zxdg_exported_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zxdg_exported_v1::Event::Handle { handle } = event {
            state.exported_parent = Some(handle);
        }
    }
}

impl Dispatch<ZxdgImportedV1, ()> for Events {
    fn event(
        state: &mut Self,
        _: &ZxdgImportedV1,
        event: zxdg_imported_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if matches!(event, zxdg_imported_v1::Event::Destroyed) {
            state.invalidated_parents += 1;
        }
    }
}

delegate_noop!(Events: ignore ZxdgExporterV1);
delegate_noop!(Events: ignore ZxdgImporterV1);

impl Application {
    /// Export this exact toplevel, retaining the revocable protocol object.
    pub fn export_parent(&mut self) -> (ZxdgExportedV1, String) {
        let qh = self.queue.handle();
        let name = self
            .events
            .globals
            .iter()
            .find(|(_, interface, _)| interface == "zxdg_exporter_v1")
            .unwrap()
            .0;
        let exporter: ZxdgExporterV1 = self.registry.bind(name, 1, &qh, ());
        self.events.exported_parent = None;
        let exported = exporter.export(&self.surface, &qh, ());
        self.sync();
        (exported, self.events.exported_parent.clone().unwrap())
    }

    /// Import a different client's handle and parent this client's own window.
    pub fn import_parent(&mut self, handle: &str) -> ZxdgImportedV1 {
        let qh = self.queue.handle();
        let name = self
            .events
            .globals
            .iter()
            .find(|(_, interface, _)| interface == "zxdg_importer_v1")
            .unwrap()
            .0;
        let importer: ZxdgImporterV1 = self.registry.bind(name, 1, &qh, ());
        let imported = importer.import(handle.to_owned(), &qh, ());
        imported.set_parent_of(&self.surface);
        self.sync();
        imported
    }
}
