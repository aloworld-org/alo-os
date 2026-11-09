//! Keep the exported caller window alive while its system dialog is open.

use ashpd::{WindowIdentifier, WindowIdentifierType};
use futures_util::future::{Either, select};
use gtk::{glib, prelude::*};
use std::{cell::RefCell, time::Duration};

/// Dropping the request revokes its exported relationship with the native dialog.
pub(crate) struct Parent {
    toplevel: gdk_wayland::WaylandToplevel,
    identifier: WindowIdentifier,
}

impl Parent {
    /// Export the actual caller window; unsupported desktops retain normal portals.
    pub(crate) async fn export(window: &gtk::Window) -> Option<Self> {
        let toplevel = window
            .surface()?
            .downcast::<gdk_wayland::WaylandToplevel>()
            .ok()?;
        let (sending, receiving) = futures_channel::oneshot::channel();
        let sending = RefCell::new(Some(sending));
        if !toplevel.export_handle(move |_, handle| {
            if let Some(sending) = sending.borrow_mut().take() {
                let _ = sending.send(handle.ok().map(ToOwned::to_owned));
            }
        }) {
            return None;
        }
        let mut parent = Self {
            toplevel,
            identifier: WindowIdentifierType::Wayland(String::new()).into(),
        };
        let timeout = glib::timeout_future(Duration::from_secs(5));
        futures_util::pin_mut!(receiving, timeout);
        let Either::Left((Ok(Some(handle)), _)) = select(receiving, timeout).await else {
            return None;
        };
        if handle.is_empty() {
            return None;
        }
        parent.identifier = WindowIdentifierType::Wayland(handle).into();
        Some(parent)
    }

    /// The portal serializes this opaque identifier, never a surface address.
    pub(crate) fn identifier(&self) -> &WindowIdentifier {
        &self.identifier
    }
}

impl Drop for Parent {
    fn drop(&mut self) {
        self.toplevel.unexport_handle();
    }
}
