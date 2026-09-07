//! Blocking atomic scanout ownership and ordered resource retirement.

use crate::{
    DisplayResources, ResourceError, ResourceFailure,
    atomic_test::AtomicPlan,
    display_resources::{Allocation, ResourceDevice},
    drm_inventory::Inventory,
};
use drm::control::{AtomicCommitFlags, Device, atomic::AtomicModeReq, framebuffer};
use std::io;

/// An enabled, initialized single output on a borrowed session descriptor.
///
/// Created only by `DisplayResources::activate`. Commits are synchronous: no
/// pending flip event or writable buffer is exposed. Explicit `disable` reports
/// failures; drop attempts the same disable as a safety net. Neither restores
/// another compositor's configuration. The caller must exclusively own this KMS
/// output and keep the session active until retirement finishes.
///
/// If disable fails, resources are quarantined, not destroyed while potentially
/// scanning out. Retire the session device (all duplicate fds as well) before
/// using it again. Drop cannot report errors, so production must use `disable`.
/// Session pause ordering and nonblocking frame presentation are separate work.
///
/// Active resources cannot escape their session descriptor:
/// ```compile_fail
/// use alo_shell::{ActiveScanout, AtomicOutput, DisplayResources, ResourceError};
/// use std::os::fd::{AsFd, OwnedFd};
/// fn escape(fd: OwnedFd, output: &AtomicOutput)
///     -> Result<ActiveScanout<'static>, ResourceError>
/// {
///     DisplayResources::allocate(fd.as_fd(), output)?.activate()
/// }
/// ```
pub struct ActiveScanout<'fd> {
    /// Keeps the allocation and atomic transport on the same borrowed descriptor.
    active: Scanout<Inventory<'fd>>,
}

impl<'fd> DisplayResources<'fd> {
    /// Validate then synchronously enable this initialized full-mode candidate.
    /// Displays black by default, or the pixels supplied through `with_frame`.
    ///
    /// Consumes unbound ownership. A failed TEST_ONLY or active commit releases
    /// all resources and retains cleanup failures; there is no retry or fallback.
    /// Successful validation does not reserve hardware: active commit may refuse.
    /// This changes the output and must run under exclusive session ownership,
    /// using fresh discovery on this descriptor, with no concurrent KMS commits.
    pub fn activate(self) -> Result<ActiveScanout<'fd>, ResourceError> {
        Scanout::activate(self.owned, self.plan, self.framebuffer, self.mode_blob)
            .map(|active| ActiveScanout { active })
    }
}

impl ActiveScanout<'_> {
    /// Synchronously detach plane/connector and deactivate CRTC before cleanup.
    ///
    /// On disable refusal, returns the original errno and issues no destruction
    /// ioctls. The session descriptor must then be retired. On successful disable,
    /// every resource release is attempted and all cleanup failures are reported.
    pub fn disable(mut self) -> Result<(), ResourceError> {
        self.active.retire()
    }
}

/// Atomic transport paired with the allocation transport for fault injection.
pub(crate) trait ScanoutDevice: ResourceDevice {
    /// Submit exactly one atomic request with the caller's fixed flags.
    fn commit(&self, flags: AtomicCommitFlags, request: AtomicModeReq) -> io::Result<()>;
}

impl ScanoutDevice for Inventory<'_> {
    fn commit(&self, flags: AtomicCommitFlags, request: AtomicModeReq) -> io::Result<()> {
        self.atomic_commit(flags, request)
    }
}

/// No asynchronous commits: the enabled bit is cleared only by retirement.
pub(crate) struct Scanout<D: ScanoutDevice> {
    /// Resource owner never independently destroys an enabled framebuffer.
    owned: Allocation<D>,
    /// Immutable routing used for both enable and disable.
    plan: AtomicPlan,
    /// Whether retirement must disable the output first.
    enabled: bool,
}

impl<D: ScanoutDevice> Scanout<D> {
    /// Test before enabling; either refusal retires the still-unbound candidate.
    pub(crate) fn activate(
        mut owned: Allocation<D>,
        plan: AtomicPlan,
        framebuffer: framebuffer::Handle,
        blob: u64,
    ) -> Result<Self, ResourceError> {
        for (stage, flags) in [
            (
                "atomic TEST_ONLY",
                AtomicCommitFlags::TEST_ONLY | AtomicCommitFlags::ALLOW_MODESET,
            ),
            ("atomic enable", AtomicCommitFlags::ALLOW_MODESET),
        ] {
            if let Err(source) = owned
                .device
                .commit(flags, plan.request(Some((framebuffer, blob))))
            {
                let mut cleanup = Vec::new();
                if let Err(error) = owned.release() {
                    cleanup.push(error.failure);
                    cleanup.extend(error.cleanup);
                }
                return Err(ResourceError {
                    failure: ResourceFailure { stage, source },
                    cleanup,
                });
            }
        }
        Ok(Self {
            owned,
            plan,
            enabled: true,
        })
    }

    /// A blocking disable is the only path from enabled to destructible.
    pub(crate) fn retire(&mut self) -> Result<(), ResourceError> {
        if self.enabled {
            // Clear before calling out: explicit refusal must not retry on drop.
            self.enabled = false;
            if let Err(source) = self
                .owned
                .device
                .commit(AtomicCommitFlags::ALLOW_MODESET, self.plan.request(None))
            {
                self.owned.quarantine();
                return Err(ResourceError {
                    failure: ResourceFailure {
                        stage: "atomic disable; retire session device",
                        source,
                    },
                    cleanup: Vec::new(),
                });
            }
        }
        self.owned.release()
    }
}

impl<D: ScanoutDevice> Drop for Scanout<D> {
    fn drop(&mut self) {
        let _ = self.retire();
    }
}
