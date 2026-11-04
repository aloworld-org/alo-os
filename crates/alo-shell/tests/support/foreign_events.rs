//! Real parent-handle requests, shared with the compositor's protocol fixtures.
#![allow(
    dead_code,
    reason = "the support client also builds in examples without parent dialogs"
)]

use super::{Application, Events};
use wayland_client::{Connection, Dispatch, QueueHandle, delegate_noop};
use wayland_protocols::xdg::foreign::zv2::client::{
    zxdg_exported_v2::{self, ZxdgExportedV2},
    zxdg_exporter_v2::ZxdgExporterV2,
    zxdg_imported_v2::{self, ZxdgImportedV2},
    zxdg_importer_v2::ZxdgImporterV2,
};

impl Dispatch<ZxdgExportedV2, ()> for Events {
    fn event(
        state: &mut Self,
        _: &ZxdgExportedV2,
        event: zxdg_exported_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zxdg_exported_v2::Event::Handle { handle } = event {
            state.exported_parent = Some(handle);
        }
    }
}

impl Dispatch<ZxdgImportedV2, ()> for Events {
    fn event(
        state: &mut Self,
        _: &ZxdgImportedV2,
        event: zxdg_imported_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if matches!(event, zxdg_imported_v2::Event::Destroyed) {
            state.invalidated_parents += 1;
        }
    }
}

delegate_noop!(Events: ignore ZxdgExporterV2);
delegate_noop!(Events: ignore ZxdgImporterV2);

impl Application {
    /// Export this exact toplevel, retaining the revocable protocol object.
    pub fn export_parent(&mut self) -> (ZxdgExportedV2, String) {
        let qh = self.queue.handle();
        let name = self
            .events
            .globals
            .iter()
            .find(|(_, interface, _)| interface == "zxdg_exporter_v2")
            .unwrap()
            .0;
        let exporter: ZxdgExporterV2 = self.registry.bind(name, 1, &qh, ());
        self.events.exported_parent = None;
        let exported = exporter.export_toplevel(&self.surface, &qh, ());
        self.sync();
        (exported, self.events.exported_parent.clone().unwrap())
    }

    /// Import a different client's handle and parent this client's own window.
    pub fn import_parent(&mut self, handle: &str) -> ZxdgImportedV2 {
        let qh = self.queue.handle();
        let name = self
            .events
            .globals
            .iter()
            .find(|(_, interface, _)| interface == "zxdg_importer_v2")
            .unwrap()
            .0;
        let importer: ZxdgImporterV2 = self.registry.bind(name, 1, &qh, ());
        let imported = importer.import_toplevel(handle.to_owned(), &qh, ());
        imported.set_parent_of(&self.surface);
        self.sync();
        imported
    }
}
