//! Atomic capability and property ioctl transport on a borrowed seat descriptor.

use crate::{
    DirectOutput, DirectOutputError,
    atomic_output::{Inventory, Plane, Property, PropertyKind},
    drm_inventory,
};
use drm::{
    Device as _,
    control::{Device, ResourceHandle, property::ValueType},
};
use std::io;

impl Inventory for drm_inventory::Inventory<'_> {
    fn enable(&self, capability: drm::ClientCapability) -> io::Result<()> {
        self.set_client_capability(capability, true)
    }
    fn output(&self) -> Result<DirectOutput, DirectOutputError> {
        crate::discover_output(self.0)
    }
    fn connector(&self, output: &DirectOutput) -> io::Result<Vec<Property>> {
        properties(self, output.connector)
    }
    fn crtc(&self, output: &DirectOutput) -> io::Result<Vec<Property>> {
        properties(self, output.crtc)
    }
    fn planes(&self, output: &DirectOutput) -> io::Result<Vec<Plane>> {
        let resources = self.resource_handles()?;
        if resources.crtcs().len() > u32::BITS as usize || !resources.crtcs().contains(&output.crtc)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "CRTC disappeared or mask exceeds 32 bits",
            ));
        }
        let mut planes = Vec::new();
        for handle in self.plane_handles()? {
            let info = self.get_plane(handle)?;
            planes.push(Plane {
                handle,
                compatible: resources
                    .filter_crtcs(info.possible_crtcs())
                    .contains(&output.crtc),
                formats: info.formats().to_vec(),
                properties: properties(self, handle)?,
            });
        }
        Ok(planes)
    }
}

/// Preserve all properties and byte names, refusing ambiguous enum metadata.
fn properties(device: &impl Device, object: impl ResourceHandle) -> io::Result<Vec<Property>> {
    let values = device.get_properties(object)?;
    let (handles, raw) = values.as_props_and_values();
    if handles.len() != raw.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "mismatched DRM properties and values",
        ));
    }
    values
        .iter()
        .map(|(handle, value)| {
            let info = device.get_property(*handle)?;
            let kind = info.value_type();
            // Avoid drm-rs enum indexing and UTF-8 unwrap helpers: names are bytes.
            let enum_name = if let ValueType::Enum(enums) = &kind {
                let (numbers, names) = enums.values();
                if numbers.len() != names.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "mismatched DRM enum metadata",
                    ));
                }
                let mut matching = names.iter().filter(|entry| entry.value() == *value);
                let name = matching
                    .next()
                    .map(|entry| entry.name().to_bytes().to_vec());
                if matching.next().is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "ambiguous DRM enum value",
                    ));
                }
                name
            } else {
                None
            };
            let kind = match kind {
                ValueType::Boolean => PropertyKind::Boolean,
                ValueType::UnsignedRange(min, max) => PropertyKind::UnsignedRange(min, max),
                ValueType::SignedRange(min, max) => PropertyKind::SignedRange(min, max),
                ValueType::Enum(_) => PropertyKind::Enum,
                ValueType::Blob => PropertyKind::Blob,
                ValueType::CRTC => PropertyKind::Crtc,
                ValueType::Framebuffer => PropertyKind::Framebuffer,
                _ => PropertyKind::Unknown,
            };
            Ok(Property {
                handle: *handle,
                name: info.name().to_bytes().to_vec(),
                kind,
                mutable: info.mutable(),
                atomic: info.atomic(),
                enum_name,
            })
        })
        .collect()
}
