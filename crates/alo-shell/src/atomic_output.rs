//! Atomic KMS schema discovery, separate from allocation and modesetting.

use crate::{DirectOutput, DirectOutputError};
use drm::control::{plane, property};
use std::{collections::BTreeMap, io, os::fd::BorrowedFd};

#[cfg(test)]
#[path = "atomic_output_tests.rs"]
mod tests;

/// One atomic-capable route and the standard properties needed to configure it.
///
/// A snapshot only: neither a reservation nor a successful kernel atomic test.
/// Discard on seat pause/hotplug and rediscover after acquiring a new descriptor.
#[derive(Debug)]
pub struct AtomicOutput {
    /// Freshly discovered connector, CRTC and advertised mode.
    pub output: DirectOutput,
    /// Lowest-ID compatible primary plane advertising at least one format.
    pub plane: plane::Handle,
    /// Advertised FourCC codes; modifiers/allocation compatibility are not tested.
    pub formats: Vec<u32>,
    /// Mutable atomic connector property: `CRTC_ID`.
    pub connector_properties: BTreeMap<&'static str, property::Handle>,
    /// Mutable atomic CRTC properties: `ACTIVE` and `MODE_ID`.
    pub crtc_properties: BTreeMap<&'static str, property::Handle>,
    /// Mutable atomic plane properties: `CRTC_ID`, `FB_ID`, `CRTC_X/Y/W/H`,
    /// and `SRC_X/Y/W/H`. Ranges are checked for a full-mode, unscaled rectangle.
    pub plane_properties: BTreeMap<&'static str, property::Handle>,
}

/// Discovery refuses incomplete atomic schemas without changing scanout.
#[derive(Debug, thiserror::Error)]
pub enum AtomicOutputError {
    /// Capability negotiation or resource/property ioctl failed.
    #[error("DRM atomic {stage} failed: {source}")]
    Query {
        /// Diagnostic operation name, not translated UI text.
        stage: &'static str,
        /// Original kernel error.
        #[source]
        source: io::Error,
    },
    /// Connector/mode discovery refused the device.
    #[error(transparent)]
    Output(#[from] DirectOutputError),
    /// Required property is absent, duplicated, or has an unusable schema.
    #[error("DRM object {object} has missing, ambiguous or invalid property {name}")]
    Property {
        /// Kernel object ID.
        object: u32,
        /// Standard property name.
        name: &'static str,
    },
    /// No compatible primary plane with advertised formats exists.
    #[error("no compatible DRM primary plane with advertised formats")]
    NoPlane,
}

/// Negotiate atomic/universal-plane visibility and discover a standard KMS route.
///
/// Call inside [`crate::DirectSession::with_device`]. Enables per-open-file client
/// capabilities (also visible through duplicated descriptors), even if a later
/// query fails; does not reset them. No master acquisition, property writes,
/// framebuffer/blob allocation, atomic commit or legacy fallback. The caller owns
/// the descriptor. Use kernel TEST_ONLY before any subsequent modeset: this schema
/// check cannot establish that hardware supports a proposed configuration.
pub fn discover_atomic_output(fd: BorrowedFd<'_>) -> Result<AtomicOutput, AtomicOutputError> {
    discover(&crate::drm_inventory::Inventory(fd))
}

/// Normalized property metadata retaining the driver-advertised type/range.
#[derive(Clone)]
pub(crate) struct Property {
    /// Kernel property ID.
    pub handle: property::Handle,
    /// Exact property name bytes.
    pub name: Vec<u8>,
    /// Advertised type and bounds.
    pub kind: PropertyKind,
    /// Kernel permits writes.
    pub mutable: bool,
    /// Property carries the atomic flag.
    pub atomic: bool,
    /// Resolved enum value name, without assuming numeric enum ordering.
    pub enum_name: Option<Vec<u8>>,
}

/// Only standard KMS types used here; unknown types never match a schema.
#[derive(Clone)]
pub(crate) enum PropertyKind {
    /// Unsupported metadata.
    Unknown,
    /// Range exactly zero through one.
    Boolean,
    /// Inclusive unsigned bounds.
    UnsignedRange(u64, u64),
    /// Inclusive signed bounds.
    SignedRange(i64, i64),
    /// Enumeration resolved separately by name.
    Enum,
    /// Kernel blob reference.
    Blob,
    /// Typed CRTC reference.
    Crtc,
    /// Typed framebuffer reference.
    Framebuffer,
}

/// One plane's complete metadata from the current resource list.
pub(crate) struct Plane {
    /// Kernel plane ID.
    pub handle: plane::Handle,
    /// Current possible-CRTC mask includes the selected CRTC.
    pub compatible: bool,
    /// Advertised FourCC codes.
    pub formats: Vec<u32>,
    /// Complete property snapshot.
    pub properties: Vec<Property>,
}

/// Transport boundary for deterministic fault injection.
pub(crate) trait Inventory {
    /// Enable one per-file client capability.
    fn enable(&self, capability: drm::ClientCapability) -> io::Result<()>;
    /// Discover the connector and mode afresh.
    fn output(&self) -> Result<DirectOutput, DirectOutputError>;
    /// Read connector properties.
    fn connector(&self, output: &DirectOutput) -> io::Result<Vec<Property>>;
    /// Read CRTC properties.
    fn crtc(&self, output: &DirectOutput) -> io::Result<Vec<Property>>;
    /// Read planes with current CRTC compatibility and metadata.
    fn planes(&self, output: &DirectOutput) -> io::Result<Vec<Plane>>;
}

/// Preserve the operation and original errno.
fn query<T>(stage: &'static str, value: io::Result<T>) -> Result<T, AtomicOutputError> {
    value.map_err(|source| AtomicOutputError::Query { stage, source })
}

/// Fail closed in transport order, then select the lowest usable primary ID.
fn discover(device: &impl Inventory) -> Result<AtomicOutput, AtomicOutputError> {
    query(
        "universal planes",
        device.enable(drm::ClientCapability::UniversalPlanes),
    )?;
    query(
        "atomic capability",
        device.enable(drm::ClientCapability::Atomic),
    )?;
    let output = device.output()?;
    let connector = query("connector properties", device.connector(&output))?;
    let crtc = query("CRTC properties", device.crtc(&output))?;
    let connector_properties = schema(
        u32::from(output.connector),
        &connector,
        &[("CRTC_ID", Kind::Crtc)],
    )?;
    let crtc_properties = schema(
        u32::from(output.crtc),
        &crtc,
        &[("ACTIVE", Kind::Boolean), ("MODE_ID", Kind::Blob)],
    )?;
    let mut planes = query("planes", device.planes(&output))?;
    planes.sort_by_key(|plane| u32::from(plane.handle));
    for plane in planes {
        if !plane.compatible {
            continue;
        }
        let id = u32::from(plane.handle);
        let kind = unique(id, &plane.properties, "type")?;
        if kind.mutable || !matches!(kind.kind, PropertyKind::Enum) || kind.enum_name.is_none() {
            return Err(invalid(id, "type"));
        }
        if kind.enum_name.as_deref() != Some(b"Primary") || plane.formats.is_empty() {
            continue;
        }
        let (width, height) = output.mode.size();
        let plane_properties = schema(
            id,
            &plane.properties,
            &[
                ("CRTC_ID", Kind::Crtc),
                ("FB_ID", Kind::Framebuffer),
                ("CRTC_X", Kind::SignedZero),
                ("CRTC_Y", Kind::SignedZero),
                ("CRTC_W", Kind::Unsigned(width.into())),
                ("CRTC_H", Kind::Unsigned(height.into())),
                ("SRC_X", Kind::Unsigned(0)),
                ("SRC_Y", Kind::Unsigned(0)),
                ("SRC_W", Kind::Unsigned(u64::from(width) << 16)),
                ("SRC_H", Kind::Unsigned(u64::from(height) << 16)),
            ],
        )?;
        return Ok(AtomicOutput {
            output,
            plane: plane.handle,
            formats: plane.formats,
            connector_properties,
            crtc_properties,
            plane_properties,
        });
    }
    Err(AtomicOutputError::NoPlane)
}

/// Name the unusable object's property in diagnostics.
fn invalid(object: u32, name: &'static str) -> AtomicOutputError {
    AtomicOutputError::Property { object, name }
}

/// Reject duplicate names and aliased property IDs before looking at types.
fn unique<'a>(
    object: u32,
    properties: &'a [Property],
    name: &'static str,
) -> Result<&'a Property, AtomicOutputError> {
    let mut matching = properties.iter().filter(|p| p.name == name.as_bytes());
    let found = matching.next().ok_or_else(|| invalid(object, name))?;
    if matching.next().is_some()
        || properties
            .iter()
            .filter(|p| p.handle == found.handle)
            .count()
            != 1
    {
        return Err(invalid(object, name));
    }
    Ok(found)
}

/// Required type, and where relevant the full-mode value it must accommodate.
enum Kind {
    /// CRTC object reference.
    Crtc,
    /// Framebuffer object reference.
    Framebuffer,
    /// ACTIVE boolean.
    Boolean,
    /// MODE_ID blob.
    Blob,
    /// Signed screen coordinate including zero.
    SignedZero,
    /// Unsigned dimension or source coordinate including this value.
    Unsigned(u64),
}

/// Validate names, mutability, atomic flags, types and requested-value bounds.
fn schema(
    object: u32,
    properties: &[Property],
    required: &[(&'static str, Kind)],
) -> Result<BTreeMap<&'static str, property::Handle>, AtomicOutputError> {
    let mut result = BTreeMap::new();
    for (name, kind) in required {
        let p = unique(object, properties, name)?;
        let matches = match (kind, &p.kind) {
            (Kind::Crtc, PropertyKind::Crtc)
            | (Kind::Framebuffer, PropertyKind::Framebuffer)
            | (Kind::Boolean, PropertyKind::Boolean)
            | (Kind::Blob, PropertyKind::Blob) => true,
            (Kind::SignedZero, PropertyKind::SignedRange(min, max)) => *min <= 0 && *max >= 0,
            (Kind::Unsigned(value), PropertyKind::UnsignedRange(min, max)) => {
                min <= value && value <= max
            }
            _ => false,
        };
        if !p.mutable || !p.atomic || !matches {
            return Err(invalid(object, name));
        }
        result.insert(*name, p.handle);
    }
    Ok(result)
}
