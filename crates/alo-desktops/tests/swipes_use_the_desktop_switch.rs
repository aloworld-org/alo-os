//! Gesture intents use the same desktop operation and edge refusal as chords.

use alo_desktops::gesture_events::{Event, Intent, Kind};
use alo_desktops::gestures::Gestures;
use alo_desktops::{Desktops, DisplayId, Promises, Refused, Switch};
use alo_dividing::{Area, Point, Size, WindowId};

/// Walk the real desktop operation with intents, including both end-of-row refusals.
#[test]
fn swipes_reach_neighbours_and_refuse_to_wrap() -> Result<(), Box<dyn std::error::Error>> {
    let mut desktops = Desktops::default();
    let area = Area::of(Point::at(0, 0), Size::of(1920, 1080))?;
    let promises = Promises::of(
        WindowId::from_compositor(1),
        WindowId::from_compositor(2),
        WindowId::from_compositor(3),
    )?;
    let display = desktops.plug_in(DisplayId::from_compositor(1), area, promises)?;
    let first = display.current();
    let second = display
        .add()
        .map_err(|why| std::io::Error::other(format!("{why:?}")))?;
    let mut gestures = Gestures::default();
    for (dx, expected) in [
        (100.0, Err(Refused::NoDesktopThatWay(Switch::Previous))),
        (-100.0, Ok(second)),
        (-100.0, Err(Refused::NoDesktopThatWay(Switch::Next))),
        (100.0, Ok(first)),
    ] {
        let _ = gestures.decide(Event::Begin(Kind::ThreeFingerSwipe));
        let _ = gestures.decide(Event::Swipe { dx, dy: 0.0 });
        let intent = gestures.decide(Event::End {
            kind: Kind::ThreeFingerSwipe,
            cancelled: false,
        });
        let switch = if dx < 0.0 {
            Switch::Next
        } else {
            Switch::Previous
        };
        assert_eq!(intent, Some(Intent::Desktop(switch)));
        assert_eq!(display.switch(switch), expected);
    }
    assert_eq!(display.current(), first);
    Ok(())
}
