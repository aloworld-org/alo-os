//! The ordinary desktop on the nested compositor: the frame every other surface
//! sits in.
//!
//! [`Nested::submit_with_desktop`] draws a session's frame with the dock on the
//! edge `alo-dock` names and its status area at the far end, the egress
//! indicator growing from that status area, the two desktop windows when a
//! person has them open, and — above all of those — the record window and a
//! waiting question when there are any. The dock and the indicator are laid out
//! from **one** `alo_dock::Dock`, so the status area and the indicator cannot
//! disagree about where the far end is.
//!
//! [`Nested::pump_running`] and [`Nested::pump_filling`] take the parent
//! window's keys through the seat for whichever desktop window is open.

use alo_strings::Strings;
use smithay::backend::{
    input::{Event, InputEvent, KeyboardKeyEvent},
    winit::WinitEvent,
};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::desktop_raster::{DesktopPicture, filling_shows, running_shows};
use crate::egress_status_raster::EgressStatusPicture;
use crate::nested_egress_status::status_picture;
use crate::scene_native::{NativeLayers, NativeScene};
use crate::{
    ApprovalFrame, DesktopLook, EgressStatus, EgressStatusFrame, FillingPressed, FillingWindow,
    FrameTarget, InputError, Nested, RecordFrame, RenderError, RunningPressed, RunningWindow,
    Server, WindowControlLabels,
};

/// What a frame needs to draw the ordinary desktop.
#[derive(Clone, Copy)]
pub struct DesktopFrame<'a> {
    /// The person's dock, which decides the edge and the status area.
    pub dock: &'a alo_dock::Dock,
    /// The person's appearance at this moment, and the way they read.
    pub look: DesktopLook,
    /// The vocabulary the person reads.
    pub strings: &'a Strings,
    /// What the egress indicator in the status area was last told.
    pub egress: &'a EgressStatus,
    /// The window of what is running, open or closed.
    pub running: &'a RunningWindow,
    /// The window of what is filling the disk, open or closed.
    pub filling: &'a FillingWindow,
    /// The clock, battery, network and volume the status area shows, as the
    /// crates that own them said them. Handed in rather than read here: a
    /// compositor that opened `/sys` would be a compositor measuring.
    pub status: &'a crate::status_items::StatusItems,
}

impl Nested {
    /// Route the parent window's keys to the window of what is running, and
    /// hand back what each did — including every request to read again, which
    /// the host answers with [`RunningWindow::read_again`].
    ///
    /// Nothing is routed once the window is closed, or while the parent window
    /// does not have the keyboard, and nothing is forwarded to any client.
    ///
    /// # Errors
    /// [`RenderError::Closed`] when the parent window closed, and
    /// [`RenderError::Input`] when `server` has no keyboard.
    pub fn pump_running(
        &mut self,
        server: &mut Server,
        window: &mut RunningWindow,
    ) -> Result<Vec<RunningPressed>, RenderError> {
        let mut did = Vec::new();
        let mut failure: Option<InputError> = None;
        let pumped = self.pump_events(|event, focused| {
            if failure.is_some() || !focused || !window.is_open() {
                return;
            }
            let Some(WinitEvent::Input(InputEvent::Keyboard { event })) = event else {
                return;
            };
            let code = u32::from(event.key_code()).saturating_sub(8);
            match server.running_key(code, event.state(), event.time_msec()) {
                Ok(Some(key)) => did.push(window.pressed(key)),
                Ok(None) | Err(InputError::InvalidKey) => {}
                Err(error) => failure = Some(error),
            }
        });
        match failure {
            Some(error) => Err(RenderError::Input(error)),
            None => pumped.map(|()| did),
        }
    }

    /// Route the parent window's keys to the window of what is filling the
    /// disk, for somebody reading `reading`, and hand back what each did.
    ///
    /// # Errors
    /// As [`Nested::pump_running`].
    pub fn pump_filling(
        &mut self,
        server: &mut Server,
        window: &mut FillingWindow,
        reading: alo_strings::Direction,
    ) -> Result<Vec<FillingPressed>, RenderError> {
        let mut did = Vec::new();
        let mut failure: Option<InputError> = None;
        let pumped = self.pump_events(|event, focused| {
            if failure.is_some() || !focused || !window.is_open() {
                return;
            }
            let Some(WinitEvent::Input(InputEvent::Keyboard { event })) = event else {
                return;
            };
            let code = u32::from(event.key_code()).saturating_sub(8);
            match server.filling_key(code, event.state(), event.time_msec(), reading) {
                Ok(Some(key)) => did.push(window.pressed(key)),
                Ok(None) | Err(InputError::InvalidKey) => {}
                Err(error) => failure = Some(error),
            }
        });
        match failure {
            Some(error) => Err(RenderError::Input(error)),
            None => pumped.map(|()| did),
        }
    }

    /// Submit clients, popups and native controls with the desktop above them,
    /// the record window and a waiting question above the desktop, the egress
    /// indicator above all of it, and the cursor on top.
    ///
    /// # Errors
    /// [`RenderError::EgressStatusUnknown`] and every other refusal the
    /// indicator makes, first; the dock's and the desktop windows' refusals;
    /// every refusal the record window and the approval surface make; and the
    /// backend's own submission failures. A refused frame draws nothing.
    #[expect(
        clippy::too_many_arguments,
        reason = "one session frame: clients, popups, cursor, controls, fonts, the desktop, the record and the question"
    )]
    pub fn submit_with_desktop(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
        controls: Option<crate::WindowControlScene<'_>>,
        labels: &mut WindowControlLabels,
        desktop: DesktopFrame<'_>,
        record: Option<RecordFrame<'_>>,
        approval: Option<ApprovalFrame<'_>>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        let size = self.size();
        let pictures = frame_pictures(desktop, record, approval, labels, (size.w, size.h))?;
        self.submit_native_layers(
            roots,
            popups,
            cursor,
            NativeLayers {
                scene: controls.map(NativeScene::Controls),
                desktop: Some(&pictures.desktop),
                record: pictures.record.as_ref(),
                settings: None,
                approval: pictures.approval.as_ref(),
                status: Some(&pictures.status),
            },
        )
    }
}

/// Every native picture in one desktop frame.
pub(crate) struct DesktopPictures {
    /// The egress indicator.
    pub(crate) status: EgressStatusPicture,
    /// The dock and the desktop windows.
    pub(crate) desktop: DesktopPicture,
    /// The record window, when the frame carries one.
    pub(crate) record: Option<crate::record_raster::RecordPicture>,
    /// A question, when the frame carries one.
    pub(crate) approval: Option<crate::approval_raster::ApprovalPicture>,
}

/// Every picture for one frame, or the refusal that stops the whole frame —
/// the indicator's first, because a frame that cannot say what is leaving is
/// refused whatever else it holds.
pub(crate) fn frame_pictures(
    desktop: DesktopFrame<'_>,
    record: Option<RecordFrame<'_>>,
    approval: Option<ApprovalFrame<'_>>,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
) -> Result<DesktopPictures, RenderError> {
    let status = status_picture(
        EgressStatusFrame {
            status: desktop.egress,
            strings: desktop.strings,
            dock: desktop.dock,
            look: desktop.look.egress(),
        },
        labels,
        size,
    )?;
    let running = running_shows(desktop.running, desktop.strings);
    let filling = filling_shows(desktop.filling, desktop.strings);
    let drawn = crate::desktop_raster::picture(
        desktop.dock,
        desktop.look,
        &running,
        &filling,
        desktop.status,
        &mut labels.fonts,
        size,
    )?;
    let record = match record {
        Some(record) => Some(crate::record_raster::picture(
            record.window.shows(),
            record.strings,
            labels,
            size,
            record.look,
        )?),
        None => None,
    };
    let approval = match approval {
        Some(approval) => Some(crate::approval_raster::picture(
            approval.screen.shows(),
            approval.strings,
            labels,
            size,
            approval.look,
        )?),
        None => None,
    };
    Ok(DesktopPictures {
        status,
        desktop: drawn,
        record,
        approval,
    })
}

#[cfg(test)]
#[path = "nested_desktop_tests.rs"]
mod tests;
