//! Buffers a client handed over, held until a renderer has really imported one.
//!
//! A client offering a buffer through `zwp_linux_dmabuf_v1` is told whether the
//! import worked, and by the owner's ruling of 2026-10-02 that telling follows a
//! real import rather than preceding it: *report successful imports only after
//! actual validation; do not wait until drawing to discover failure.* An
//! unconditional "accepted" is the one answer forbidden outright, because the
//! client then draws into a buffer this shell will refuse at scanout, and the
//! person sees the failure instead of the client handling it.
//!
//! **The renderer and the protocol state are not in one place, and that is why
//! this file exists.** `crate::surfaces::Surfaces` holds every wayland global
//! and is what `dispatch_clients` is handed; the renderer belongs to the
//! graphics backend, inside `crate::direct_target::Target`. So the handler
//! cannot import at the moment it is called. It holds the offer here, and the
//! loop — the one place holding both halves — answers it.
//!
//! **This costs no frame.** `crate::direct_loop::run_with_input` dispatches
//! clients and then draws, in that order, inside one turn, and the drain sits
//! between the two. A buffer offered during a turn is answered in that same
//! turn, before anything has been painted with it. That is the difference
//! between this and the move in `crate::canvas_fixed_controls`, which is one
//! frame behind for a reason it states.

use smithay::{backend::allocator::dmabuf::Dmabuf, wayland::dmabuf::ImportNotifier};

/// One buffer a client offered, and the client waiting to hear about it.
pub(crate) struct HandedOver {
    /// What the client offered.
    pub(crate) buffer: Dmabuf,
    /// How that client is told, once a renderer has tried the import.
    pub(crate) telling: ImportNotifier,
}

impl HandedOver {
    /// Tell the client the import worked.
    ///
    /// **The dropped result is the client having gone away** between offering
    /// the buffer and being told about it. There is no remedy and nothing is
    /// wrong: the only party the answer was for is no longer there to receive
    /// it, and `successful` says so by handing back an invalid id rather than
    /// by failing.
    pub(crate) fn worked(self) {
        let _ = self.telling.successful::<crate::surfaces::Surfaces>();
    }

    /// Tell the client the import failed.
    ///
    /// Said out loud rather than by dropping the notifier. A client told
    /// nothing waits for an event that never comes; a client told `failed`
    /// falls back to shared memory, which this shell can draw.
    pub(crate) fn failed(self) {
        self.telling.failed();
    }
}
