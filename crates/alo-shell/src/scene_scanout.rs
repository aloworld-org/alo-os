//! Initial scene activation: pixels and identities share one scanout lifetime.

use crate::scanout::{Scanout, ScanoutDevice};
use crate::{AtomicOutput, PreparedScanout, ResourceError, ResourceFailure};
use crate::{atomic_test::AtomicPlan, display_resources::Allocation, drm_inventory::Inventory};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use std::{io, os::fd::BorrowedFd};

/// A successfully enabled initial scene on an exclusively owned session output.
///
/// Owns immutable scanout storage and the exact drawn surface identities. This
/// is a static initial modeset, not a continuous `FrameTarget` or a presentation
/// timestamp. No method dispatches clients, publishes membership or sends frame
/// callbacks. Keep the session active and call `disable` before releasing it.
/// A failed disable quarantines resources until all device descriptors close.
pub struct ActiveScene<'fd> {
    /// The same private ownership path is exercised by injected DRM tests.
    submitted: Submitted<Inventory<'fd>>,
}

impl PreparedScanout {
    /// Consume this scene, upload it and synchronously TEST_ONLY then enable it.
    ///
    /// Use fresh discovery from this descriptor within `DirectSession::with_device`,
    /// exclusively owning an inactive output. Never call while another scanout
    /// owner is active. Refuses a mode-size mismatch before allocating anything.
    /// All failures return no active scene and preserve resource cleanup errors.
    /// Render and activate without interleaving client dispatch; only success
    /// permits the backend to use the returned identities for frame callbacks.
    pub fn activate<'fd>(
        self,
        fd: BorrowedFd<'fd>,
        output: &AtomicOutput,
    ) -> Result<ActiveScene<'fd>, ResourceError> {
        activate(self.into_parts(), Inventory(fd), output)
            .map(|submitted| ActiveScene { submitted })
    }
}

impl ActiveScene<'_> {
    /// Identities drawn in the successfully enabled scene, not a timestamp guarantee.
    pub fn surfaces(&self) -> &[WlSurface] {
        &self.submitted.surfaces
    }

    /// Disable before destruction, retaining every failure and never retrying refusal.
    pub fn disable(mut self) -> Result<(), ResourceError> {
        self.submitted.active.retire()
    }
}

/// Generic ownership boundary; the production transport is the borrowed DRM fd.
pub(crate) struct Submitted<D: ScanoutDevice> {
    /// Retired before releasing the session descriptor, never writable while active.
    pub(crate) active: Scanout<D>,
    /// Published to the caller only after successful enable.
    pub(crate) surfaces: Vec<WlSurface>,
}

/// Shared production/test transaction. No allocation occurs before CPU validation.
pub(crate) fn activate<D: ScanoutDevice>(
    (pixels, surfaces): (crate::ScanoutPixels, Vec<WlSurface>),
    device: D,
    output: &AtomicOutput,
) -> Result<Submitted<D>, ResourceError> {
    let invalid = |source| ResourceError {
        failure: ResourceFailure {
            stage: "validate prepared scene",
            source,
        },
        cleanup: Vec::new(),
    };
    let (width, height) = output.output.mode.size();
    if pixels.size() != (u32::from(width), u32::from(height)) {
        return Err(invalid(io::ErrorKind::InvalidInput.into()));
    }
    let frame = pixels.frame().map_err(invalid)?;
    let plan = AtomicPlan::new(output).map_err(invalid)?;
    let (mut owned, fb, blob) = Allocation::allocate(device, &output.output.mode, &output.formats)?;
    owned.write_frame(&frame)?;
    let active = Scanout::activate(owned, plan, fb, blob)?;
    Ok(Submitted { active, surfaces })
}
