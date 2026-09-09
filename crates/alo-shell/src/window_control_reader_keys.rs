//! Explicit host key transactions; backend mapping and publication are separate.
use std::{collections::BTreeMap, sync::Arc};

use smithay::backend::input::KeyState;

use crate::{Server, WindowControlReader, WindowControlReaderNavigation};

/// Reader-only semantic command selected by the trusted host, never an agent verb.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReaderKeyCommand {
    /// Read the previous page without wrapping.
    Previous,
    /// Read the next page without wrapping.
    Next,
    /// Dismiss this reader, never close its application.
    Dismiss,
}

/// Whether the host must withhold this event from ordinary keyboard routing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReaderKeyRoute {
    /// No ownership acquired; route through the existing keyboard path once.
    Forward,
    /// Owned event, including repeats, boundaries and cancelled releases.
    Consumed,
    /// Owned release selected a new page; request a fresh reader frame.
    Changed,
    /// Owned release dismissed the reader; request removal of its pixels.
    Dismissed,
}

/// Captured execution authority for one consumed press.
struct Press {
    /// Exact reading session, not merely its strip.
    reader: Arc<()>,
    /// Page visit identity, including away-and-back transitions.
    selection: Arc<()>,
    /// Semantic mapping frozen at press.
    command: ReaderKeyCommand,
}

/// One backend's reader key ownership, retained even after its reader disappears.
///
/// This is an explicit transaction component, not a keyboard filter installed by
/// opening a reader. The host may offer new commands only for its successfully
/// published reader and must cancel on frame failure/removal, focus/seat/session
/// loss or binding changes. Route releases through this object even while inactive.
/// Keys already pressed by a client refuse acquisition. Ordinary typing uses None.
#[derive(Default)]
pub struct WindowControlReaderKeys {
    /// The evdev range bounds ownership storage, including cancelled presses.
    held: BTreeMap<u32, Option<Press>>,
}

impl WindowControlReaderKeys {
    /// Disarm every command while retaining all matching release ownership.
    pub fn cancel(&mut self) {
        for press in self.held.values_mut() {
            *press = None;
        }
    }

    /// Route one ordered evdev key event. Only 1..=767 can acquire ownership.
    ///
    /// A new press validates live mapping and captures exact reader/page/command.
    /// A second held command is consumed but disarmed; repeats never execute or
    /// rearm. Release uses the captured command, ignoring the current mapping.
    /// Stale, foreign, replaced, dismissed or page-changed readers cannot execute.
    /// Unknown releases forward, and owned releases drain even with no reader.
    /// No window operation or client focus change is performed here.
    pub fn route(
        &mut self,
        server: &mut Server,
        reader: Option<&mut WindowControlReader>,
        code: u32,
        state: KeyState,
        command: Option<ReaderKeyCommand>,
    ) -> ReaderKeyRoute {
        if state == KeyState::Released {
            let Some(press) = self.held.remove(&code) else {
                return ReaderKeyRoute::Forward;
            };
            let (Some(press), Some(reader)) = (press, reader) else {
                return ReaderKeyRoute::Consumed;
            };
            if !Arc::ptr_eq(&press.reader, &reader.identity) {
                return ReaderKeyRoute::Consumed;
            }
            let selected = reader.selected();
            if server.read_window_control_page(reader, selected).is_none()
                || !Arc::ptr_eq(&press.selection, &reader.selection)
            {
                return ReaderKeyRoute::Consumed;
            }
            return match press.command {
                ReaderKeyCommand::Dismiss => {
                    reader.dismiss();
                    self.cancel();
                    ReaderKeyRoute::Dismissed
                }
                command => {
                    let direction = match command {
                        ReaderKeyCommand::Previous => WindowControlReaderNavigation::Previous,
                        _ => WindowControlReaderNavigation::Next,
                    };
                    if server
                        .navigate_window_control_reader(reader, direction)
                        .is_some()
                    {
                        ReaderKeyRoute::Changed
                    } else {
                        ReaderKeyRoute::Consumed
                    }
                }
            };
        }
        if self.held.contains_key(&code) {
            return ReaderKeyRoute::Consumed;
        }
        let (Some(command), Some(reader)) = (command, reader) else {
            return ReaderKeyRoute::Forward;
        };
        if !(1..=767).contains(&code)
            || server
                .surfaces
                .keyboard
                .as_ref()
                .is_none_or(|keyboard| keyboard.client_holds(code))
        {
            return ReaderKeyRoute::Forward;
        }
        let page = reader.selected();
        if server.read_window_control_page(reader, page).is_none() {
            return ReaderKeyRoute::Forward;
        }
        let press = self.held.is_empty().then(|| Press {
            reader: reader.identity.clone(),
            selection: reader.selection.clone(),
            command,
        });
        self.held.insert(code, press);
        ReaderKeyRoute::Consumed
    }
}
