//! Single-output selection from DRM resource snapshots, without modesetting.

use drm::control::{Mode, ModeFlags, ModeTypeFlags, connector, crtc};
use std::{io, os::fd::BorrowedFd};

#[cfg(test)]
#[path = "direct_output_tests.rs"]
mod tests;

/// A connected output and a possible scanout route on the supplied DRM device.
///
/// This is a discovery snapshot, not a reservation or a successful atomic test.
/// The session must own the device and revalidate/test the configuration before
/// modesetting; hotplug and another owner can invalidate any of these handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectOutput {
    /// Connected physical port on the queried device.
    pub connector: connector::Handle,
    /// One CRTC allowed by at least one of the port's advertised encoders.
    pub crtc: crtc::Handle,
    /// Exact advertised progressive mode, retaining all kernel timings.
    pub mode: Mode,
    /// Kernel connector dimensions in millimetres, or unknown; no EDID inference.
    pub physical_size: Option<(u32, u32)>,
}

/// Direct-output discovery failed; no display state was changed.
#[derive(Debug, thiserror::Error)]
pub enum DirectOutputError {
    /// A DRM resource ioctl failed, including permission loss or hot-unplug.
    #[error("DRM {stage} query failed: {source}")]
    Query {
        /// Resource type being queried; diagnostic, not translated UI text.
        stage: &'static str,
        /// Original kernel error retained for session diagnostics.
        #[source]
        source: io::Error,
    },
    /// No connected display has both a supported mode and a possible CRTC.
    #[error("no connected DRM output with a progressive mode and compatible CRTC")]
    NoOutput,
    /// DRM encoder CRTC masks cannot describe this resource list.
    #[error("DRM resource list exceeds the 32-bit encoder CRTC mask")]
    InvalidTopology,
}

/// Discover one output on a caller-owned DRM session descriptor.
///
/// Performs only resource queries: no path opening, master acquisition, client
/// capability changes, connector force-probe, or modesetting. The caller retains
/// descriptor ownership. Errors refuse the entire snapshot instead of silently
/// selecting from incomplete information. Render nodes and non-DRM descriptors
/// fail the resource query. This trusted shell API is never an agent verb.
///
/// Prefer a usable internal panel, then the lowest connector ID. On that port,
/// prefer the first valid preferred mode, otherwise the first valid advertised
/// mode. CRTC choice is the lowest compatible ID; it does not claim availability.
pub fn discover_output(fd: BorrowedFd<'_>) -> Result<DirectOutput, DirectOutputError> {
    select(&crate::drm_inventory::Inventory(fd))
}

/// Relevant connector state separated from ioctl transport for refusal tests.
pub(crate) struct Port {
    /// Connector dimensions supplied by the kernel.
    pub physical_size: Option<(u32, u32)>,
    /// Kernel connector handle.
    pub handle: connector::Handle,
    /// Explicitly connected; unknown is never treated as connected.
    pub connected: bool,
    /// Internal laptop/panel connector preference.
    pub internal: bool,
    /// Writeback connectors are not displays for the person.
    pub display: bool,
    /// Kernel mode order and exact timings.
    pub modes: Vec<Mode>,
    /// Union of compatible CRTCs across advertised encoders.
    pub crtcs: Vec<crtc::Handle>,
}

/// Read a complete snapshot or preserve the first query error.
pub(crate) trait Inventory {
    /// Query connectors and their advertised scanout routes.
    fn ports(&self) -> Result<Vec<Port>, DirectOutputError>;
}

/// Deterministic policy independent of driver enumeration order.
fn select(inventory: &impl Inventory) -> Result<DirectOutput, DirectOutputError> {
    let mut ports = inventory.ports()?;
    ports.sort_by_key(|port| (!port.internal, u32::from(port.handle)));
    for port in ports {
        if !port.connected || !port.display {
            continue;
        }
        let Some(crtc) = port.crtcs.into_iter().min_by_key(|crtc| u32::from(*crtc)) else {
            continue;
        };
        let mode = port
            .modes
            .iter()
            .filter(|mode| supported(mode))
            .find(|mode| mode.mode_type().contains(ModeTypeFlags::PREFERRED))
            .or_else(|| port.modes.iter().find(|mode| supported(mode)));
        if let Some(mode) = mode {
            return Ok(DirectOutput {
                connector: port.handle,
                crtc,
                mode: *mode,
                physical_size: port.physical_size,
            });
        }
    }
    Err(DirectOutputError::NoOutput)
}

/// Initial direct backend uses ordinary progressive two-dimensional timings.
pub(crate) fn supported(mode: &Mode) -> bool {
    let (width, height) = mode.size();
    let (hstart, hend, htotal) = mode.hsync();
    let (vstart, vend, vtotal) = mode.vsync();
    let unsupported = ModeFlags::INTERLACE
        | ModeFlags::DBLSCAN
        | ModeFlags::_3D_FRAME_PACKING
        | ModeFlags::_3D_FIELD_ALTERNATIVE
        | ModeFlags::_3D_LINE_ALTERNATIVE
        | ModeFlags::_3D_SIDE_BY_SIDE_FULL
        | ModeFlags::_3D_L_DEPTH
        | ModeFlags::_3D_L_DEPTH_GFX_GFX_DEPTH
        | ModeFlags::_3D_TOP_AND_BOTTOM
        | ModeFlags::_3D_SIDE_BY_SIDE_HALF;
    width > 0
        && height > 0
        && mode.clock() > 0
        && width <= hstart
        && hstart < hend
        && hend <= htotal
        && height <= vstart
        && vstart < vend
        && vend <= vtotal
        && mode.vscan() <= 1
        && !mode.flags().intersects(unsupported)
}
