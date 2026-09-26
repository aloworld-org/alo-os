//! What a touchpad event says, taken out of libinput and handed to the crate
//! that decides what it means.
//!
//! This extracts fields and decides nothing: which swipes count, how far one
//! has to go, which are turned off and what any of them means are all
//! `alo_desktops::Gestures`' answers, reached through its
//! `gesture_events::Event` — the seam that exists so the recogniser can be
//! driven by a deterministic stream in that crate's own tests and by a real
//! touchpad here.
//!
//! # Why this is not `alo_desktops::libinput_gestures`
//!
//! Because that module cannot be used by anything in this workspace, and it
//! took a red gate to find out. It is behind `alo-desktops`' `libinput`
//! feature, whose manifest says *whoever needs gestures asks for them*. Asking
//! turns the feature on for **every** copy of `alo-desktops` in the build, and
//! `alo-saying` depends on `alo-desktops` to collect the machine's vocabulary
//! while `alo-agentd` depends on `alo-saying` — so a shell asking for gestures
//! puts `libudev-sys` into an agent daemon that draws nothing and reads no
//! touchpad, and the image links its daemons statically against musl where that
//! library does not exist. `alo-image`'s
//! `no_daemon_links_a_system_library` says so by name; release 0.0.3 is the one
//! that shipped this fault as `cannot find -linput`.
//!
//! So the extraction happens where libinput is already linked, which is here:
//! this crate reads libinput for keys in `crate::libinput_routing` and for
//! scroll in `crate::libinput_scroll`, through smithay's own reexport. What is
//! **not** copied is any decision — that stays in one place, and this file
//! would be a fault the moment it grew a branch about what a gesture means.
//!
//! This is a workaround, and the finding is written down in `docs/quirks.md`
//! for the owners of `alo-desktops` and `alo-saying`: an instruction in a
//! manifest that cannot be followed is worse than none.

use alo_desktops::gesture_events::{Event, Intent, Kind};
use alo_desktops::gestures::Gestures;
use smithay::reexports::input;
use smithay::reexports::input::event::gesture::{
    GestureEndEvent, GestureEventCoordinates, GestureEventTrait, GesturePinchEvent,
    GesturePinchEventTrait, GestureSwipeEvent,
};
use smithay::reexports::input::event::pointer::{Axis, PointerScrollEvent};
use smithay::reexports::input::event::{EventTrait, GestureEvent, PointerEvent};

/// Extract this event's owned fields and decide it through the recogniser.
///
/// [`None`] for an event that is not one a gesture is made of; a gesture with a
/// finger count nothing is bound to, and anything else libinput reports about a
/// device, cancel whatever was in flight rather than being ignored — half a
/// swipe joined to the next person's is worse than none.
pub(crate) fn decide(gestures: &mut Gestures, event: &input::Event) -> Option<Intent> {
    let extracted = match event {
        input::Event::Pointer(PointerEvent::ScrollFinger(scroll)) => {
            // libinput has already inverted values when natural scrolling is
            // enabled on the device. Undo that before applying our preference.
            Event::finger_scroll(
                scroll
                    .has_axis(Axis::Horizontal)
                    .then(|| scroll.scroll_value(Axis::Horizontal)),
                scroll
                    .has_axis(Axis::Vertical)
                    .then(|| scroll.scroll_value(Axis::Vertical)),
                scroll.device().config_scroll_natural_scroll_enabled(),
            )
        }
        input::Event::Gesture(GestureEvent::Swipe(swipe)) => {
            let Some(kind) = Kind::swipe(swipe.finger_count()) else {
                return gestures.decide(Event::Reset);
            };
            match swipe {
                GestureSwipeEvent::Begin(_) => Event::Begin(kind),
                GestureSwipeEvent::Update(update) => Event::Swipe {
                    dx: update.dx_unaccelerated(),
                    dy: update.dy_unaccelerated(),
                },
                GestureSwipeEvent::End(end) => Event::End {
                    kind,
                    cancelled: end.cancelled(),
                },
                _ => Event::Reset,
            }
        }
        input::Event::Gesture(GestureEvent::Pinch(pinch)) => {
            if Kind::pinch(pinch.finger_count()).is_none() {
                return gestures.decide(Event::Reset);
            }
            match pinch {
                GesturePinchEvent::Begin(_) => Event::Begin(Kind::Pinch),
                GesturePinchEvent::Update(update) => Event::Pinch(update.scale()),
                GesturePinchEvent::End(end) => Event::End {
                    kind: Kind::Pinch,
                    cancelled: end.cancelled(),
                },
                _ => Event::Reset,
            }
        }
        input::Event::Gesture(_) | input::Event::Device(_) => Event::Reset,
        _ => return None,
    };
    gestures.decide(extracted)
}
