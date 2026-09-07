//! Owned, unbound KMS resources; allocation never changes scanout.

use crate::{AtomicOutput, atomic_test::AtomicPlan, drm_inventory::Inventory};
use drm::{
    buffer::{Buffer, DrmFourcc},
    control::{Mode, framebuffer},
};
use std::{io, os::fd::BorrowedFd};

#[cfg(test)]
#[path = "display_resources_tests.rs"]
mod tests;

/// One failed resource operation, retaining the original kernel error.
#[derive(Debug, thiserror::Error)]
#[error("DRM resource {stage} failed: {source}")]
pub struct ResourceFailure {
    /// Diagnostic operation name, not a user-interface string.
    pub stage: &'static str,
    /// Original error, including errno when supplied by the kernel.
    #[source]
    pub source: io::Error,
}

/// Allocation or release failed; cleanup failures never replace the first error.
#[derive(Debug, thiserror::Error)]
#[error("{failure}; cleanup failures: {cleanup:?}")]
pub struct ResourceError {
    /// The original allocation, validation or first release failure.
    #[source]
    pub failure: ResourceFailure,
    /// Additional failures in release order; all remaining releases are attempted.
    pub cleanup: Vec<ResourceFailure>,
}

/// A full-mode XRGB8888 dumb framebuffer and exact mode blob on a borrowed device.
///
/// Allocation leaves these resources unbound; optional test-and-release never changes scanout.
/// Use within `DirectSession::with_device`, with a fresh discovery from that same
/// descriptor. The borrow prevents resources outliving the session descriptor.
/// Handles are valid only until release/drop, and must never be bound to scanout:
/// active framebuffer retirement requires a separate scanout owner.
///
/// Resources cannot escape the descriptor that owns their kernel handles:
/// ```compile_fail
/// use alo_shell::{AtomicOutput, DisplayResources, ResourceError};
/// use std::os::fd::{AsFd, OwnedFd};
/// fn escape(fd: OwnedFd, output: &AtomicOutput)
///     -> Result<DisplayResources<'static>, ResourceError>
/// {
///     DisplayResources::allocate(fd.as_fd(), output)
/// }
/// ```
pub struct DisplayResources<'fd> {
    /// Cleanup owner and borrowed ioctl transport.
    owned: Allocation<Inventory<'fd>>,
    /// Frozen request prevents later caller mutation from changing the tested mode.
    plan: AtomicPlan,
    /// Registered framebuffer, kept private against accidental ownership transfer.
    framebuffer: framebuffer::Handle,
    /// Exact advertised timing blob, not reconstructed from resolution.
    mode_blob: u64,
}

impl<'fd> DisplayResources<'fd> {
    /// Allocate an unbound candidate initialized to black, including all padding.
    ///
    /// Supports a linear dumb allocation only. Format advertisement is necessary,
    /// not proof of compatibility: kernel atomic TEST_ONLY is still required.
    /// Mapping/initialization finishes before framebuffer registration and
    /// requires a read/write session descriptor, as supplied by DirectSession.
    /// Mapping failure releases the buffer and retains any destruction error. The pinned
    /// drm-rs mapping destructor can panic on munmap failure; see docs/quirks.md.
    pub fn allocate(fd: BorrowedFd<'fd>, output: &AtomicOutput) -> Result<Self, ResourceError> {
        let plan = AtomicPlan::new(output).map_err(|source| ResourceError {
            failure: ResourceFailure {
                stage: "atomic request schema",
                source,
            },
            cleanup: Vec::new(),
        })?;
        let (owned, framebuffer, mode_blob) =
            Allocation::allocate(Inventory(fd), &output.output.mode, &output.formats)?;
        Ok(Self {
            owned,
            plan,
            framebuffer,
            mode_blob,
        })
    }

    /// Test the frozen full-mode candidate, then release every owned resource.
    ///
    /// Uses only TEST_ONLY | ALLOW_MODESET: no scanout changes or page-flip event.
    /// Success is an instantaneous kernel validation, never a reservation or proof
    /// that a later modeset succeeds. Use a fresh discovery on this same session
    /// descriptor; pause/hotplug invalidates it. Both success and refusal consume
    /// this candidate. Cleanup failures are retained and require device retirement.
    pub fn test_and_release(mut self) -> Result<(), ResourceError> {
        self.owned
            .test_and_release(|device| self.plan.test(device.0, self.framebuffer, self.mode_blob))
    }
    /// Framebuffer ID for a subsequent TEST_ONLY request on this same descriptor.
    pub fn framebuffer(&self) -> framebuffer::Handle {
        self.framebuffer
    }

    /// Mode blob ID for a subsequent TEST_ONLY request on this same descriptor.
    pub fn mode_blob(&self) -> u64 {
        self.mode_blob
    }

    /// Release all resources, reporting every failed ioctl in reverse allocation order.
    ///
    /// Each release is attempted once, including on partial allocation failure.
    /// Failed destruction may leave kernel resources until the last device fd closes;
    /// retire that session device rather than continue allocating on a failed release.
    /// Drop is a best-effort safety net; use this method to observe cleanup errors.
    pub fn release(mut self) -> Result<(), ResourceError> {
        self.owned.release()
    }
}

/// Small transport boundary permitting allocation and cleanup fault injection.
pub(crate) trait ResourceDevice {
    /// Buffer whose public metadata can be validated before registration.
    type Buffer: Buffer;
    /// Allocate a 32-bit XRGB8888 dumb buffer.
    fn create_buffer(&self, size: (u32, u32)) -> io::Result<Self::Buffer>;
    /// Borrow mapped bytes only inside the callback; unmap before returning.
    fn with_mapping(
        &self,
        buffer: &mut Self::Buffer,
        initialize: impl FnOnce(&mut [u8]) -> io::Result<()>,
    ) -> io::Result<()>;
    /// Register a framebuffer for the buffer.
    fn create_framebuffer(&self, buffer: &Self::Buffer) -> io::Result<framebuffer::Handle>;
    /// Create an exact mode blob.
    fn create_blob(&self, mode: &Mode) -> io::Result<u64>;
    /// Remove a mode blob.
    fn destroy_blob(&self, blob: u64) -> io::Result<()>;
    /// Remove an unbound framebuffer.
    fn destroy_framebuffer(&self, framebuffer: framebuffer::Handle) -> io::Result<()>;
    /// Release the GEM dumb buffer handle.
    fn destroy_buffer(&self, buffer: Self::Buffer) -> io::Result<()>;
}

/// Tracks partial allocation so every successfully acquired resource is retired.
struct Allocation<D: ResourceDevice> {
    /// Same open file description for all allocation and destruction operations.
    device: D,
    /// First resource acquired, last released.
    buffer: Option<D::Buffer>,
    /// Registered buffer view.
    framebuffer: Option<framebuffer::Handle>,
    /// Last resource acquired, first released.
    blob: Option<u64>,
}

impl<D: ResourceDevice> Allocation<D> {
    /// Validate before and after allocation, unwinding all completed stages.
    fn allocate(
        device: D,
        mode: &Mode,
        formats: &[u32],
    ) -> Result<(Self, framebuffer::Handle, u64), ResourceError> {
        let mut owned = Self {
            device,
            buffer: None,
            framebuffer: None,
            blob: None,
        };
        let attempt = (|| {
            let (w, h) = mode.size();
            let size = (u32::from(w), u32::from(h));
            if w == 0 || h == 0 || !formats.contains(&(DrmFourcc::Xrgb8888 as u32)) {
                return Err(invalid("dimensions or XRGB8888 format"));
            }
            let buffer = operation("create dumb buffer", owned.device.create_buffer(size))?;
            let valid = buffer.size() == size
                && buffer.format() == DrmFourcc::Xrgb8888
                && buffer.pitch() >= size.0 * 4
                && buffer.pitch() % 4 == 0;
            owned.buffer = Some(buffer);
            if !valid {
                return Err(invalid("dumb buffer layout"));
            }
            let buffer = owned
                .buffer
                .as_mut()
                .ok_or_else(|| invalid("missing buffer"))?;
            operation(
                "initialize dumb buffer",
                crate::scanout_buffer::initialize(&owned.device, buffer),
            )?;
            let framebuffer = operation(
                "create framebuffer",
                owned.device.create_framebuffer(buffer),
            )?;
            owned.framebuffer = Some(framebuffer);
            let blob = operation("create mode blob", owned.device.create_blob(mode))?;
            if blob == 0 || blob > u64::from(u32::MAX) {
                return Err(invalid("mode blob ID"));
            }
            owned.blob = Some(blob);
            Ok((framebuffer, blob))
        })();
        match attempt {
            Ok((fb, blob)) => Ok((owned, fb, blob)),
            Err(failure) => Err(ResourceError {
                failure,
                cleanup: owned.cleanup(),
            }),
        }
    }

    /// Drain in reverse order, continuing after errors and never double-destroying.
    fn cleanup(&mut self) -> Vec<ResourceFailure> {
        let mut errors = Vec::new();
        if let Some(blob) = self.blob.take()
            && let Err(e) = operation("destroy mode blob", self.device.destroy_blob(blob))
        {
            errors.push(e);
        }
        if let Some(fb) = self.framebuffer.take()
            && let Err(e) = operation("destroy framebuffer", self.device.destroy_framebuffer(fb))
        {
            errors.push(e);
        }
        if let Some(buffer) = self.buffer.take()
            && let Err(e) = operation("destroy dumb buffer", self.device.destroy_buffer(buffer))
        {
            errors.push(e);
        }
        errors
    }

    /// Retain ioctl refusal before all release errors, with no retry or fallback.
    fn test_and_release(
        &mut self,
        test: impl FnOnce(&D) -> io::Result<()>,
    ) -> Result<(), ResourceError> {
        let result = operation("atomic TEST_ONLY", test(&self.device));
        match result {
            Ok(()) => self.release(),
            Err(failure) => Err(ResourceError {
                failure,
                cleanup: self.cleanup(),
            }),
        }
    }
    /// Surface all errors from an explicit release.
    fn release(&mut self) -> Result<(), ResourceError> {
        let mut errors = self.cleanup().into_iter();
        match errors.next() {
            Some(failure) => Err(ResourceError {
                failure,
                cleanup: errors.collect(),
            }),
            None => Ok(()),
        }
    }
}

impl<D: ResourceDevice> Drop for Allocation<D> {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

/// Attach the failing operation without discarding errno.
fn operation<T>(stage: &'static str, value: io::Result<T>) -> Result<T, ResourceFailure> {
    value.map_err(|source| ResourceFailure { stage, source })
}

/// Refuse unusable metadata before it can be registered or tested.
fn invalid(stage: &'static str) -> ResourceFailure {
    ResourceFailure {
        stage,
        source: io::Error::from(io::ErrorKind::InvalidData),
    }
}
