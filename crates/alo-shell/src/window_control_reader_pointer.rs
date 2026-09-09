//! Explicit semantic pointer transactions, independent of backend hit testing.
use std::{collections::BTreeMap, sync::Arc};

use smithay::backend::input::ButtonState;

use crate::{
    ReaderKeyCommand, ReaderKeyRoute, Server, WindowControlReader, WindowControlReaderNavigation,
};

/// Trusted hit from the successfully published reader frame, never client input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReaderPointerHit {
    /// Opaque page or position wording, with no executable action.
    Content,
    /// A navigation row, including its currently disabled boundary state.
    Command(ReaderKeyCommand),
}

/// Validated semantic feedback for a later interaction painter; no pixels implied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReaderPointerFeedback {
    /// Enabled command under the pointer, if any.
    pub hovered: Option<ReaderKeyCommand>,
    /// Enabled command whose primary press is still armed, if any.
    pub pressed: Option<ReaderKeyCommand>,
}

/// One exact reader/page visit and enabled semantic target.
#[derive(Clone)]
struct Target {
    /// Reading session identity, distinct even on the same strip.
    reader: Arc<()>,
    /// Page visit, rejecting away-and-back transitions.
    selection: Arc<()>,
    /// None means covered content or disabled navigation.
    command: Option<ReaderKeyCommand>,
}

impl Target {
    /// Compare the whole transaction, never only a command name.
    fn matches(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.reader, &other.reader)
            && Arc::ptr_eq(&self.selection, &other.selection)
            && self.command == other.command
    }
}

/// Pointer ownership for one trusted host, retained through reader retirement.
///
/// Offer hits only from a successfully published reader frame. Route motion before
/// axes; `motion` reports whether to consume both. Route every button release even
/// when inactive. Cancel on frame failure/removal, leave, focus/seat/session loss,
/// geometry changes or competing keyboard interaction. This installs no backend
/// routing, performs no hit testing and changes neither client focus nor pixels.
#[derive(Default)]
pub struct WindowControlReaderPointer {
    /// At most the eight Linux mouse buttons, including cancelled releases.
    held: BTreeMap<u32, Option<Target>>,
    /// Most recently validated target, including opaque non-command content.
    hover: Option<Target>,
}

impl WindowControlReaderPointer {
    /// Disarm and clear feedback while retaining each matching release obligation.
    pub fn cancel(&mut self) {
        self.hover = None;
        for press in self.held.values_mut() {
            *press = None;
        }
    }

    /// Refresh semantic hover and return whether motion/axes must be withheld.
    /// None means outside the published reader. Leaving an armed command disarms
    /// it permanently, even if the pointer returns before release. Disabled rows
    /// and content consume without command feedback. Client-held buttons, missing
    /// pointer seats and stale readers refuse new ownership. Held reader buttons
    /// continue consuming outside the reader until their releases drain.
    pub fn motion(
        &mut self,
        server: &mut Server,
        reader: Option<&mut WindowControlReader>,
        hit: Option<ReaderPointerHit>,
    ) -> bool {
        self.hover = Self::target(server, reader, hit);
        for press in self.held.values_mut() {
            if !press
                .as_ref()
                .zip(self.hover.as_ref())
                .is_some_and(|(a, b)| a.matches(b))
            {
                *press = None;
            }
        }
        self.hover.is_some() || !self.held.is_empty()
    }

    /// Revalidate feedback against the current live reader before painting.
    /// A stale/foreign/replaced/page-changed reader clears all armed commands.
    /// This does not establish frame publication or infer a hit after layout changes.
    pub fn feedback(
        &mut self,
        server: &mut Server,
        reader: Option<&mut WindowControlReader>,
    ) -> ReaderPointerFeedback {
        let hit = self.hover.as_ref().map(|target| {
            target
                .command
                .map_or(ReaderPointerHit::Content, ReaderPointerHit::Command)
        });
        let current = Self::target(server, reader, hit);
        if !self
            .hover
            .as_ref()
            .zip(current.as_ref())
            .is_some_and(|(a, b)| a.matches(b))
        {
            self.cancel();
        }
        ReaderPointerFeedback {
            hovered: self.hover.as_ref().and_then(|target| target.command),
            pressed: self
                .held
                .get(&0x110)
                .and_then(Option::as_ref)
                .and_then(|target| target.command),
        }
    }

    /// Route one BTN_LEFT..=BTN_TASK event using the current published-frame hit.
    /// Only a primary press on an enabled command can arm. A second button disarms
    /// all commands; repeats never rearm. A release executes once on the same live
    /// reader/page visit/target. Owned releases consume even after cancellation;
    /// unmatched releases and invalid button codes forward. Results use the same
    /// Forward/Consumed/Changed/Dismissed semantics as reader key transactions.
    pub fn button(
        &mut self,
        server: &mut Server,
        mut reader: Option<&mut WindowControlReader>,
        hit: Option<ReaderPointerHit>,
        button: u32,
        state: ButtonState,
    ) -> ReaderKeyRoute {
        if !(0x110..=0x117).contains(&button) {
            return ReaderKeyRoute::Forward;
        }
        self.motion(server, reader.as_deref_mut(), hit);
        if state == ButtonState::Released {
            let Some(press) = self.held.remove(&button) else {
                return ReaderKeyRoute::Forward;
            };
            let (Some(press), Some(reader)) = (press, reader) else {
                return ReaderKeyRoute::Consumed;
            };
            let result = match press.command {
                Some(ReaderKeyCommand::Dismiss) => {
                    reader.dismiss();
                    ReaderKeyRoute::Dismissed
                }
                Some(command) => {
                    let direction = if command == ReaderKeyCommand::Previous {
                        WindowControlReaderNavigation::Previous
                    } else {
                        WindowControlReaderNavigation::Next
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
                None => ReaderKeyRoute::Consumed,
            };
            self.cancel();
            return result;
        }
        if self.held.contains_key(&button) {
            return ReaderKeyRoute::Consumed;
        }
        if self.hover.is_none() && self.held.is_empty() {
            return ReaderKeyRoute::Forward;
        }
        let press = if self.held.is_empty() && button == 0x110 {
            self.hover.clone().filter(|target| target.command.is_some())
        } else {
            self.cancel();
            None
        };
        self.held.insert(button, press);
        ReaderKeyRoute::Consumed
    }

    /// Resolve availability only after checking the live reader and pointer seat.
    fn target(
        server: &mut Server,
        reader: Option<&mut WindowControlReader>,
        hit: Option<ReaderPointerHit>,
    ) -> Option<Target> {
        let (reader, hit) = (reader?, hit?);
        if server
            .surfaces
            .pointer
            .as_ref()
            .is_none_or(|pointer| !pointer.buttons.is_empty())
        {
            return None;
        }
        let page = server.read_window_control_page(reader, reader.selected())?;
        let command = match hit {
            ReaderPointerHit::Command(ReaderKeyCommand::Previous) if page.number > 1 => {
                Some(ReaderKeyCommand::Previous)
            }
            ReaderPointerHit::Command(ReaderKeyCommand::Next) if page.number < page.total => {
                Some(ReaderKeyCommand::Next)
            }
            ReaderPointerHit::Command(ReaderKeyCommand::Dismiss) => Some(ReaderKeyCommand::Dismiss),
            _ => None,
        };
        Some(Target {
            reader: reader.identity.clone(),
            selection: reader.selection.clone(),
            command,
        })
    }
}
