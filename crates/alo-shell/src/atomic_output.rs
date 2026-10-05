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
    /// Discover every usable display afresh, in preference order.
    ///
    /// **There is no single-display method beside this one.** There was until
    /// 2026-10-05, and it went when `discover_every` stopped calling it:
    /// a trait method nothing calls is a second road into the same question
    /// that cannot disagree with the first only because nobody takes it.
    fn outputs(&self) -> Result<Vec<DirectOutput>, DirectOutputError>;
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
    // The single-display road, kept as the first of the many so that every
    // caller it already has is unmoved. `discover_every` refuses an empty
    // answer, and this is still written as a refusal rather than a panic —
    // see `direct_output::select` for why an `unreachable!` is not allowed
    // either.
    discover_every(device)?
        .into_iter()
        .next()
        .ok_or(AtomicOutputError::NoPlane)
}

/// An atomic route for every usable display on this device.
///
/// `docs/autonomy/more-than-one-display-plan.md` task 2.
///
/// # A plane is given to one display, as a CRTC is
///
/// **The same fault as task 1's, one level down.** A plane's
/// `possible_crtcs` is a bitmask, so a plane can be compatible with several
/// CRTCs — and asking each display independently for its lowest compatible
/// primary plane hands the same plane to two of them. A plane scans out for
/// one CRTC at a time; two displays pointed at it is not a configuration that
/// can be committed.
///
/// So a plane is taken as it is assigned, exactly as a CRTC is, and the next
/// display chooses from what is left. Invisible with one display, which is
/// why it is written here rather than found by an atomic commit refusing on a
/// desk with a monitor plugged in.
///
/// # A display that cannot be routed does not refuse the ones that can
///
/// A machine with one working output and one whose plane advertises no usable
/// format is a machine a person can use. So a failure that names **this
/// display's** objects — its connector, its CRTC, its planes, its property
/// schemas — skips that display and the run carries on; what is kept is the
/// first such failure, and it is returned only if no display survives.
///
/// A failure that is the **device's** — a client capability the kernel would
/// not enable, a resource list that could not be read — refuses everything,
/// because none of it is about one display and retrying per display would ask
/// the same question again.
///
/// # Errors
/// [`AtomicOutputError`], as [`discover_atomic_output`]. An empty list is
/// never an answer.
fn discover_every(device: &impl Inventory) -> Result<Vec<AtomicOutput>, AtomicOutputError> {
    query(
        "universal planes",
        device.enable(drm::ClientCapability::UniversalPlanes),
    )?;
    query(
        "atomic capability",
        device.enable(drm::ClientCapability::Atomic),
    )?;
    let outputs = device.outputs()?;
    let mut taken: Vec<plane::Handle> = Vec::new();
    let mut routed: Vec<AtomicOutput> = Vec::new();
    let mut first_refusal: Option<AtomicOutputError> = None;
    for output in outputs {
        match route(device, output, &mut taken) {
            Ok(Some(one)) => routed.push(one),
            Ok(None) => {}
            Err(why) => {
                if first_refusal.is_none() {
                    first_refusal = Some(why);
                }
            }
        }
    }
    if routed.is_empty() {
        // The first display's reason, not a generic one invented here: a
        // caller told *no compatible primary plane* about a device whose
        // connector would not read has been sent to the wrong place.
        return Err(first_refusal.unwrap_or(AtomicOutputError::NoPlane));
    }
    Ok(routed)
}

/// One display's atomic route, or `None` where it has no primary plane left.
fn route(
    device: &impl Inventory,
    output: DirectOutput,
    taken: &mut Vec<plane::Handle>,
) -> Result<Option<AtomicOutput>, AtomicOutputError> {
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
        if !plane.compatible || taken.contains(&plane.handle) {
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
        taken.push(plane.handle);
        return Ok(Some(AtomicOutput {
            output,
            plane: plane.handle,
            formats: plane.formats,
            connector_properties,
            crtc_properties,
            plane_properties,
        }));
    }
    Ok(None)
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
