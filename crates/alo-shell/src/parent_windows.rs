//! GTK's version 1 parent handles over Smithay's existing surface model.
//!
//! A handle associates windows; it grants no content, input, or recording access.
//! ADR 0149 records why this bounded adapter accompanies the native portals.

use super::Surfaces;
use smithay::{
    reexports::{
        wayland_protocols::xdg::foreign::zv1::server::{
            zxdg_exported_v1::{self, ZxdgExportedV1},
            zxdg_exporter_v1::{self, ZxdgExporterV1},
            zxdg_imported_v1::{self, ZxdgImportedV1},
            zxdg_importer_v1::{self, ZxdgImporterV1},
        },
        wayland_server::{
            Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New, Resource,
            backend::{ClientId, GlobalId},
            protocol::wl_surface::WlSurface,
        },
    },
    wayland::{
        compositor::with_states,
        shell::{
            is_valid_parent,
            xdg::{XdgShellHandler, XdgToplevelSurfaceData},
        },
    },
};
use std::collections::BTreeMap;

/// Bound retained protocol metadata independently of a client's resource count.
const MAX_EXPORTS: usize = 128;
/// More than the bounded portal request list can need for one caller window.
const MAX_IMPORTS: usize = 32;
/// A handle can parent several independently owned dialog windows.
const MAX_CHILDREN: usize = 32;

/// Registrations and revocable handles belong to one compositor display.
pub(super) struct ParentWindows {
    /// Export factory registration on this display.
    _exporter: GlobalId,
    /// Import factory registration on this display.
    _importer: GlobalId,
    /// Random handles identify exact resources, never reusable object numbers.
    exports: BTreeMap<String, Export>,
}

/// One live exported parent and the exact import objects using it.
struct Export {
    /// The exporting client's own toplevel surface.
    parent: WlSurface,
    /// Importers notified when the export or parent is destroyed.
    imports: Vec<ZxdgImportedV1>,
    /// Relationships are removed only if this import still owns them.
    children: Vec<(WlSurface, ZxdgImportedV1)>,
}

/// Only a successful export owns a map entry.
pub(super) struct Exported(Option<String>);
/// Unknown and over-budget imports retain no caller-controlled handle text.
pub(super) struct Imported(Option<String>);

impl ParentWindows {
    /// Advertise the version supported by the GTK portal backend.
    pub(super) fn new(display: &DisplayHandle) -> Self {
        Self {
            _exporter: display.create_global::<Surfaces, ZxdgExporterV1, _>(1, ()),
            _importer: display.create_global::<Surfaces, ZxdgImporterV1, _>(1, ()),
            exports: BTreeMap::new(),
        }
    }
}

/// A cryptographic handle contains no surface ID, process ID, or address.
fn handle() -> Option<String> {
    let mut bytes = [0; 24];
    getrandom::fill(&mut bytes).ok()?;
    Some(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

impl GlobalDispatch<ZxdgExporterV1, ()> for Surfaces {
    fn bind(
        _: &mut Self,
        _: &DisplayHandle,
        _: &Client,
        resource: New<ZxdgExporterV1>,
        _: &(),
        init: &mut DataInit<'_, Self>,
    ) {
        init.init(resource, ());
    }
}
impl GlobalDispatch<ZxdgImporterV1, ()> for Surfaces {
    fn bind(
        _: &mut Self,
        _: &DisplayHandle,
        _: &Client,
        resource: New<ZxdgImporterV1>,
        _: &(),
        init: &mut DataInit<'_, Self>,
    ) {
        init.init(resource, ());
    }
}

impl Dispatch<ZxdgExporterV1, ()> for Surfaces {
    fn request(
        state: &mut Self,
        _: &Client,
        resource: &ZxdgExporterV1,
        request: zxdg_exporter_v1::Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        if let zxdg_exporter_v1::Request::Export { id, surface } = request {
            let valid = surface.is_alive()
                && with_states(&surface, |s| {
                    s.data_map.get::<XdgToplevelSurfaceData>().is_some()
                });
            let key = handle().filter(|key| {
                valid
                    && state.parent_windows.exports.len() < MAX_EXPORTS
                    && !state.parent_windows.exports.contains_key(key)
            });
            let exported = init.init(id, Exported(key.clone()));
            let Some(key) = key else {
                resource.post_error(0u32, "a parent window cannot be exported");
                return;
            };
            state.parent_windows.exports.insert(
                key.clone(),
                Export {
                    parent: surface,
                    imports: Vec::new(),
                    children: Vec::new(),
                },
            );
            exported.handle(key);
        }
    }
}

impl Dispatch<ZxdgExportedV1, Exported> for Surfaces {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &ZxdgExportedV1,
        _: zxdg_exported_v1::Request,
        _: &Exported,
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
    }
    fn destroyed(state: &mut Self, _: ClientId, _: &ZxdgExportedV1, data: &Exported) {
        if let Some(key) = &data.0 {
            state.parent_export_removed(key);
        }
    }
}

impl Dispatch<ZxdgImporterV1, ()> for Surfaces {
    fn request(
        state: &mut Self,
        _: &Client,
        _: &ZxdgImporterV1,
        request: zxdg_importer_v1::Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        if let zxdg_importer_v1::Request::Import { id, handle } = request {
            let key = state
                .parent_windows
                .exports
                .get(&handle)
                .filter(|export| export.parent.is_alive() && export.imports.len() < MAX_IMPORTS)
                .map(|_| handle);
            let imported = init.init(id, Imported(key.clone()));
            if let Some(export) = key
                .as_ref()
                .and_then(|key| state.parent_windows.exports.get_mut(key))
            {
                export.imports.push(imported);
            } else {
                imported.destroyed();
            }
        }
    }
}

impl Dispatch<ZxdgImportedV1, Imported> for Surfaces {
    fn request(
        state: &mut Self,
        _: &Client,
        resource: &ZxdgImportedV1,
        request: zxdg_imported_v1::Request,
        data: &Imported,
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        if let zxdg_imported_v1::Request::SetParentOf { surface } = request {
            let Some(key) = data.0.as_ref() else {
                return;
            };
            let Some(export) = state.parent_windows.exports.get(key) else {
                return;
            };
            if !export.imports.contains(resource) {
                return;
            }
            let known = export.children.iter().any(|(child, _)| *child == surface);
            if (!known && export.children.len() >= MAX_CHILDREN)
                || !is_valid_parent(&surface, &export.parent)
            {
                resource.post_error(0u32, "invalid parent relationship");
                return;
            }
            let parent = export.parent.clone();
            let changed = with_states(&surface, |s| {
                let role = s.data_map.get::<XdgToplevelSurfaceData>()?;
                let mut role = role.lock().ok()?;
                let changed = role.parent.as_ref() != Some(&parent);
                role.parent = Some(parent.clone());
                Some(changed)
            });
            let Some(changed) = changed else {
                resource.post_error(0u32, "only a toplevel can have a parent");
                return;
            };
            for export in state.parent_windows.exports.values_mut() {
                export.children.retain(|(child, _)| *child != surface);
            }
            if let Some(export) = state.parent_windows.exports.get_mut(key) {
                export.children.push((surface.clone(), resource.clone()));
            }
            if changed {
                state.parent_relationship_changed(&surface);
            }
        }
    }
    fn destroyed(state: &mut Self, _: ClientId, resource: &ZxdgImportedV1, data: &Imported) {
        let Some(export) = data
            .0
            .as_ref()
            .and_then(|key| state.parent_windows.exports.get_mut(key))
        else {
            return;
        };
        export.imports.retain(|import| import != resource);
        let parent = export.parent.clone();
        let removed: Vec<_> = export
            .children
            .iter()
            .filter(|(_, import)| import == resource)
            .map(|(child, _)| child.clone())
            .collect();
        export.children.retain(|(_, import)| import != resource);
        for child in removed {
            state.parent_relationship_removed(&child, &parent);
        }
    }
}

impl Surfaces {
    /// Match the normal XDG parent callback after updating its existing role.
    fn parent_relationship_changed(&mut self, child: &WlSurface) {
        if let Some(top) = self
            .xdg
            .toplevel_surfaces()
            .iter()
            .find(|top| top.wl_surface() == child)
            .cloned()
        {
            self.parent_changed(top);
        }
    }

    /// A stale import never clears a relationship replaced by another parent.
    fn parent_relationship_removed(&mut self, child: &WlSurface, parent: &WlSurface) {
        if !child.is_alive() {
            return;
        }
        let changed = with_states(child, |s| {
            let Some(role) = s.data_map.get::<XdgToplevelSurfaceData>() else {
                return false;
            };
            let Ok(mut role) = role.lock() else {
                return false;
            };
            if role.parent.as_ref() != Some(parent) {
                return false;
            }
            role.parent = None;
            true
        });
        if changed {
            self.parent_relationship_changed(child);
        }
    }

    /// Explicit unexport and owner disconnection revoke every associated import.
    fn parent_export_removed(&mut self, key: &str) {
        let Some(export) = self.parent_windows.exports.remove(key) else {
            return;
        };
        for import in export.imports {
            if import.is_alive() {
                import.destroyed();
            }
        }
        for (child, _) in export.children {
            self.parent_relationship_removed(&child, &export.parent);
        }
    }

    /// A destroyed parent or child retains no exported relationship.
    pub(super) fn parent_surface_destroyed(&mut self, surface: &WlSurface) {
        let removed: Vec<_> = self
            .parent_windows
            .exports
            .iter()
            .filter(|(_, export)| export.parent == *surface)
            .map(|(key, _)| key.clone())
            .collect();
        for key in removed {
            self.parent_export_removed(&key);
        }
        for export in self.parent_windows.exports.values_mut() {
            export.children.retain(|(child, _)| child != surface);
        }
    }
}
