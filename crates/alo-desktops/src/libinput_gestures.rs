//! Borrow unmodified libinput events. The session owns devices and dispatch;
//! call this for each event on that device's recognizer. Reset the recognizer
//! on seat pause, device removal and focus changes. No raw device access here.

use input::event::gesture::{
    GestureEndEvent, GestureEventCoordinates, GestureEventTrait, GesturePinchEvent,
    GesturePinchEventTrait, GestureSwipeEvent,
};
use input::event::pointer::{Axis, PointerScrollEvent};
use input::event::{EventTrait, GestureEvent, PointerEvent};

use crate::gesture_events::{Event, Intent, Kind};
use crate::gestures::Gestures;

/// Extract the touchpad event's owned fields, then decide it through the same
/// state machine used by deterministic tests. Other pointer events are ignored;
/// unsupported gestures cancel pending recognition.
pub fn decide(gestures: &mut Gestures, event: &input::Event) -> Option<Intent> {
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
