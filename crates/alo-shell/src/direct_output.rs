//! Display selection from DRM resource snapshots, without modesetting.
//!
//! [`select_every`] answers with every usable display and [`discover_output`]
//! keeps the single-display road its callers already take. The ordering, the
//! refusals and the rule that nothing here modesets are the same for both.

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
    // **No panic path, including an unreachable one.** `select_every` refuses
    // an empty answer rather than returning one, so the `ok_or` below cannot
    // fire — and it is written as a refusal anyway, because an `unreachable!`
    // is a panic that depends on another function keeping a promise, and law 3
    // does not make an exception for panics somebody has reasoned about.
    select_every(inventory)?
        .into_iter()
        .next()
        .ok_or(DirectOutputError::NoOutput)
}

/// Every usable display on this device, in the order a compositor should
/// prefer them.
///
/// The order is the one a single-output shell already used — a usable internal
/// panel first, then by ascending connector ID — so the display that was the
/// only one is still the first.
///
/// # Two displays cannot be given the same CRTC
///
/// **The fault this function exists to not have.** A `Port` carries the *union*
/// of CRTCs its encoders allow, and those unions overlap: asking each port
/// independently for its lowest compatible CRTC hands the same one to two
/// displays, which is not a configuration any kernel will accept. So a CRTC is
/// taken as it is assigned and the next display chooses from what is left.
///
/// It is invisible with one display, which is why it is written down here
/// rather than discovered by a modeset refusing on a desk with a monitor
/// plugged in.
///
/// # A display with no mode does not reserve a CRTC
///
/// The mode is chosen **before** the CRTC, which is the other way round from
/// the single-output version. With one display the order could not matter — a
/// port missing either is skipped either way. With several it does: reserving
/// a CRTC for a port that is then skipped for having no supported mode would
/// take that CRTC away from a display that could have used it.
///
/// # Errors
/// [`DirectOutputError::Query`] if the snapshot could not be read, and
/// [`DirectOutputError::NoOutput`] when no connected port has both a supported
/// mode and a CRTC still free. **An empty list is never an answer**: a caller
/// handed `Ok(vec![])` would have to invent the refusal this already names.
pub fn select_every(inventory: &impl Inventory) -> Result<Vec<DirectOutput>, DirectOutputError> {
    let mut ports = inventory.ports()?;
    ports.sort_by_key(|port| (!port.internal, u32::from(port.handle)));
    let mut taken: Vec<crtc::Handle> = Vec::new();
    let mut found: Vec<DirectOutput> = Vec::new();
    for port in ports {
        if !port.connected || !port.display {
            continue;
        }
        let Some(mode) = port
            .modes
            .iter()
            .filter(|mode| supported(mode))
            .find(|mode| mode.mode_type().contains(ModeTypeFlags::PREFERRED))
            .or_else(|| port.modes.iter().find(|mode| supported(mode)))
            .copied()
        else {
            continue;
        };
        let Some(crtc) = port
            .crtcs
            .iter()
            .copied()
            .filter(|crtc| !taken.contains(crtc))
            .min_by_key(|crtc| u32::from(*crtc))
        else {
            continue;
        };
        taken.push(crtc);
        found.push(DirectOutput {
            connector: port.handle,
            crtc,
            mode,
            physical_size: port.physical_size,
        });
    }
    if found.is_empty() {
        return Err(DirectOutputError::NoOutput);
    }
    Ok(found)
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
