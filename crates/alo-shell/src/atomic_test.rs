//! Frozen full-mode KMS requests and their test-only validation.

use crate::{AtomicOutput, drm_inventory::Inventory};
use drm::control::{AtomicCommitFlags, Device, atomic::AtomicModeReq, framebuffer, property};
use std::{collections::BTreeSet, io, num::NonZeroU32, os::fd::BorrowedFd};

#[cfg(test)]
#[path = "atomic_test_tests.rs"]
mod tests;

/// Frozen object/property/value snapshot, built before allocating resources.
pub(crate) struct AtomicPlan {
    /// Thirteen mandatory properties with late-bound owned resource IDs.
    writes: Vec<(NonZeroU32, property::Handle, Value)>,
    /// Detach connector and plane, clear mode and deactivate the CRTC together.
    disable: Vec<(NonZeroU32, property::Handle, u64)>,
}

/// Resource IDs must come from the same allocation that owns the test transport.
enum Value {
    /// Geometry or object routing value.
    Fixed(u64),
    /// Owned framebuffer.
    Framebuffer,
    /// Owned timing blob.
    Mode,
}

impl AtomicPlan {
    /// Refuse missing/aliased property handles and invalid object geometry.
    pub(crate) fn new(output: &AtomicOutput) -> io::Result<Self> {
        let (w, h) = output.output.mode.size();
        let connector: NonZeroU32 = output.output.connector.into();
        let crtc: NonZeroU32 = output.output.crtc.into();
        let plane: NonZeroU32 = output.plane.into();
        if w == 0 || h == 0 || BTreeSet::from([connector, crtc, plane]).len() != 3 {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let c = u64::from(crtc.get());
        let mut writes = Vec::new();
        let mut disable = Vec::new();
        for (object, properties, values) in [
            (
                connector,
                &output.connector_properties,
                vec![("CRTC_ID", Value::Fixed(c))],
            ),
            (
                crtc,
                &output.crtc_properties,
                vec![("ACTIVE", Value::Fixed(1)), ("MODE_ID", Value::Mode)],
            ),
            (
                plane,
                &output.plane_properties,
                vec![
                    ("CRTC_ID", Value::Fixed(c)),
                    ("FB_ID", Value::Framebuffer),
                    ("CRTC_X", Value::Fixed(0)),
                    ("CRTC_Y", Value::Fixed(0)),
                    ("CRTC_W", Value::Fixed(w.into())),
                    ("CRTC_H", Value::Fixed(h.into())),
                    ("SRC_X", Value::Fixed(0)),
                    ("SRC_Y", Value::Fixed(0)),
                    ("SRC_W", Value::Fixed(u64::from(w) << 16)),
                    ("SRC_H", Value::Fixed(u64::from(h) << 16)),
                ],
            ),
        ] {
            let mut seen = BTreeSet::new();
            for (name, value) in values {
                let handle = *properties.get(name).ok_or(io::ErrorKind::InvalidData)?;
                if !seen.insert(u32::from(handle)) {
                    return Err(io::ErrorKind::InvalidData.into());
                }
                if matches!(name, "CRTC_ID" | "ACTIVE" | "MODE_ID" | "FB_ID") {
                    disable.push((object, handle, 0));
                }
                writes.push((object, handle, value));
            }
        }
        Ok(Self { writes, disable })
    }

    /// Resolve allocation-owned IDs into the exact values sent to drm-rs.
    fn values(
        &self,
        fb: framebuffer::Handle,
        blob: u64,
    ) -> Vec<(NonZeroU32, property::Handle, u64)> {
        self.writes
            .iter()
            .map(|(object, property, value)| {
                (
                    *object,
                    *property,
                    match value {
                        Value::Fixed(value) => *value,
                        Value::Framebuffer => u32::from(fb).into(),
                        Value::Mode => blob,
                    },
                )
            })
            .collect()
    }

    /// Fixed flags deliberately exclude nonblocking/page-flip/active commits.
    pub(crate) fn test(
        &self,
        fd: BorrowedFd<'_>,
        fb: framebuffer::Handle,
        blob: u64,
    ) -> io::Result<()> {
        self.submit(fb, blob, |flags, request| {
            Inventory(fd).atomic_commit(flags, request)
        })
    }

    /// Injection boundary exercises the same request and flags as the real ioctl.
    fn submit(
        &self,
        fb: framebuffer::Handle,
        blob: u64,
        submit: impl FnOnce(AtomicCommitFlags, AtomicModeReq) -> io::Result<()>,
    ) -> io::Result<()> {
        submit(
            AtomicCommitFlags::TEST_ONLY | AtomicCommitFlags::ALLOW_MODESET,
            self.request(Some((fb, blob))),
        )
    }

    /// Construct enable or disable from the same frozen routing snapshot.
    pub(crate) fn request(&self, resources: Option<(framebuffer::Handle, u64)>) -> AtomicModeReq {
        let values = match resources {
            Some((fb, blob)) => self.values(fb, blob),
            None => self.disable.clone(),
        };
        let mut request = AtomicModeReq::new();
        for (object, property, value) in values {
            request.add_raw_property(object, property, value);
        }
        request
    }
}
