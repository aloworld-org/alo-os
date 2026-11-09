// Copyright The pipewire-rs Contributors.
// SPDX-License-Identifier: MIT

use std::{marker::PhantomData, ops::Deref};

use crate::context::Context;

use super::Core;

/// Smart pointer providing unique ownership of a PipeWire [core](super).
///
/// For shared ownership, see [`CoreRc`](super::CoreRc).
///
/// For an explanation of these, see [Smart pointers to PipeWire objects](crate#smart-pointers-to-pipewire-objects).
#[derive(Debug)]
pub struct CoreBox<'c> {
    ptr: std::ptr::NonNull<pw_sys::pw_core>,
    context: PhantomData<&'c Context>,
}

impl<'c> CoreBox<'c> {
    /// Create a `CoreBox` by taking ownership of a raw `pw_core`.
    ///
    /// # Safety
    /// The provided pointer must point to a valid, well aligned [`pw_core`](`pw_sys::pw_core`).
    ///
    /// The raw core must not be manually destroyed or moved, as the new [`CoreBox`] takes
    /// ownership of it.
    ///
    /// The lifetime of the returned box is unbounded. The caller is responsible to make sure
    /// that the context used with this core outlives the core.
    pub unsafe fn from_raw(raw: std::ptr::NonNull<pw_sys::pw_core>) -> CoreBox<'c> {
        Self {
            ptr: raw,
            context: PhantomData,
        }
    }

    pub fn into_raw(self) -> std::ptr::NonNull<pw_sys::pw_core> {
        std::mem::ManuallyDrop::new(self).ptr
    }

    /// Transfer this uniquely owned connection to another client.
    ///
    /// The core is disconnected and destroyed after stealing its descriptor,
    /// exactly as in the upstream desktop portal's remote handoff. No shared
    /// core can be consumed through this method. Existing core borrows must
    /// end before the owning box can be moved here.
    ///
    /// # Errors
    /// The error reported by `pw_core_steal_fd`.
    #[cfg(feature = "v0_3_65")]
    pub fn into_fd(self) -> std::io::Result<std::os::fd::OwnedFd> {
        use std::os::fd::FromRawFd;
        // SAFETY: this box uniquely owns a live core. PipeWire transfers
        // ownership and detaches this descriptor from the core on success.
        let descriptor = unsafe { pw_sys::pw_core_steal_fd(self.as_raw_ptr()) };
        if descriptor < 0 {
            return Err(std::io::Error::from_raw_os_error(
                descriptor.saturating_neg(),
            ));
        }
        // SAFETY: the successful steal returns one live, newly owned fd.
        // CoreBox's normal Drop still destroys the now-detached core.
        Ok(unsafe { std::os::fd::OwnedFd::from_raw_fd(descriptor) })
    }
}

impl<'c> std::ops::Deref for CoreBox<'c> {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.cast::<Core>().as_ref() }
    }
}

impl<'c> AsRef<Core> for CoreBox<'c> {
    fn as_ref(&self) -> &Core {
        self.deref()
    }
}

impl<'c> std::ops::Drop for CoreBox<'c> {
    fn drop(&mut self) {
        unsafe {
            pw_sys::pw_core_disconnect(self.as_raw_ptr());
        }
    }
}
