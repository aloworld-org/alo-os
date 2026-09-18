//! Public gesture decisions, including cancellation and malformed input.

use alo_desktops::Switch;
use alo_desktops::gesture_events::{Event, Intent, Kind};
use alo_desktops::gesture_settings::{Preferences, Scrolling};
use alo_desktops::gestures::Gestures;

/// A normal end for this family.
fn end(kind: Kind) -> Event {
    Event::End {
        kind,
        cancelled: false,
    }
}

/// A scroll event retaining absent axes.
fn scroll(horizontal: Option<f64>, vertical: Option<f64>) -> Event {
    Event::Scroll {
        horizontal,
        vertical,
    }
}

/// A scroll intent retaining absent axes.
fn scrolled(horizontal: Option<f64>, vertical: Option<f64>) -> Option<Intent> {
    Some(Intent::Scroll {
        horizontal,
        vertical,
    })
}

/// Complete a swipe, asserting that updates cannot commit it early.
fn swipe(gestures: &mut Gestures, kind: Kind, dx: f64, dy: f64) -> Option<Intent> {
    assert_eq!(gestures.decide(Event::Begin(kind)), None);
    assert_eq!(gestures.decide(Event::Swipe { dx, dy }), None);
    gestures.decide(end(kind))
}

/// The adapter's shared extraction refuses unsupported finger counts and
/// applies the person's scrolling choice exactly once.
#[test]
fn library_fields_refuse_other_finger_counts_and_do_not_double_invert_scroll() {
    for fingers in -1..=8 {
        assert_eq!(Kind::pinch(fingers).is_some(), fingers == 2);
        assert_eq!(Kind::swipe(fingers).is_some(), matches!(fingers, 3 | 4));
    }
    for upstream_natural in [false, true] {
        let input = if upstream_natural { -4.0 } else { 4.0 };
        for (scrolling, expected) in [(Scrolling::Natural, -4.0), (Scrolling::Traditional, 4.0)] {
            let mut gestures = Gestures::default();
            gestures.configure(Preferences {
                scrolling,
                ..Preferences::default()
            });
            assert_eq!(
                gestures.decide(Event::finger_scroll(None, Some(input), upstream_natural)),
                scrolled(None, Some(expected))
            );
        }
    }
}

/// Finger scroll preserves both axes, absent axes, and axis stop events.
#[test]
fn scroll_is_an_intent_and_nonfinite_input_is_refused() {
    let mut gestures = Gestures::default();
    assert_eq!(
        gestures.decide(scroll(Some(2.0), Some(-3.0))),
        scrolled(Some(-2.0), Some(3.0))
    );
    assert_eq!(
        gestures.decide(scroll(None, Some(0.0))),
        scrolled(None, Some(0.0))
    );
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(gestures.decide(scroll(Some(bad), Some(1.0))), None);
        assert_eq!(gestures.decide(scroll(Some(1.0), Some(bad))), None);
    }
    assert_eq!(gestures.decide(scroll(None, None)), None);
}

/// Pinch scale is absolute, committed once, and never committed on cancellation.
#[test]
fn pinch_zoom_uses_the_last_scale_and_cancelled_pinches_do_nothing() {
    let mut gestures = Gestures::default();
    for scale in [0.5, 2.0] {
        assert_eq!(gestures.decide(Event::Begin(Kind::Pinch)), None);
        assert_eq!(gestures.decide(Event::Pinch(1.25)), None);
        assert_eq!(gestures.decide(Event::Pinch(scale)), None);
        assert_eq!(gestures.decide(end(Kind::Pinch)), Some(Intent::Zoom(scale)));
        assert_eq!(gestures.decide(end(Kind::Pinch)), None);
    }
    for cancelled in [true, false] {
        let _ = gestures.decide(Event::Begin(Kind::Pinch));
        if cancelled {
            let _ = gestures.decide(Event::Pinch(2.0));
        }
        assert_eq!(
            gestures.decide(Event::End {
                kind: Kind::Pinch,
                cancelled,
            }),
            None
        );
    }
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let _ = gestures.decide(Event::Begin(Kind::Pinch));
        let _ = gestures.decide(Event::Pinch(bad));
        let _ = gestures.decide(Event::Pinch(2.0));
        assert_eq!(gestures.decide(end(Kind::Pinch)), None);
    }
}

/// Both finger counts reach both directions; short or vertical motion and
/// cancellation never turn into a desktop switch.
#[test]
fn three_and_four_finger_swipes_switch_once_and_refuse_ambiguous_motion() {
    let mut gestures = Gestures::default();
    for kind in [Kind::ThreeFingerSwipe, Kind::FourFingerSwipe] {
        assert_eq!(
            swipe(&mut gestures, kind, -100.0, 0.0),
            Some(Intent::Desktop(Switch::Next))
        );
        assert_eq!(
            swipe(&mut gestures, kind, 100.0, 0.0),
            Some(Intent::Desktop(Switch::Previous))
        );
        assert_eq!(gestures.decide(end(kind)), None);
        for (x, y) in [
            (99.0, 0.0),
            (0.0, 500.0),
            (100.0, 50.0),
            (f64::NAN, 0.0),
            (0.0, f64::INFINITY),
        ] {
            assert_eq!(swipe(&mut gestures, kind, x, y), None);
        }
        let _ = gestures.decide(Event::Begin(kind));
        let _ = gestures.decide(Event::Swipe {
            dx: -500.0,
            dy: 0.0,
        });
        assert_eq!(
            gestures.decide(Event::End {
                kind,
                cancelled: true,
            }),
            None
        );
        let _ = gestures.decide(Event::Begin(kind));
        for _ in 0..4 {
            assert_eq!(gestures.decide(Event::Swipe { dx: -25.0, dy: 0.0 }), None);
        }
        assert_eq!(
            gestures.decide(end(kind)),
            Some(Intent::Desktop(Switch::Next))
        );
    }
}

/// Disabling one family leaves others available and retires an in-flight gesture.
#[test]
fn each_gesture_can_be_disabled_without_disabling_the_others() {
    let mut gestures = Gestures::default();
    gestures.configure(Preferences {
        scroll: false,
        ..Preferences::default()
    });
    assert_eq!(gestures.decide(scroll(None, Some(1.0))), None);
    assert!(swipe(&mut gestures, Kind::ThreeFingerSwipe, -100.0, 0.0).is_some());
    for disabled in [Kind::Pinch, Kind::ThreeFingerSwipe, Kind::FourFingerSwipe] {
        let preferences = Preferences {
            pinch: disabled != Kind::Pinch,
            three_finger_swipe: disabled != Kind::ThreeFingerSwipe,
            four_finger_swipe: disabled != Kind::FourFingerSwipe,
            ..Preferences::default()
        };
        gestures.configure(preferences);
        assert!(gestures.decide(scroll(None, Some(1.0))).is_some());
        for kind in [Kind::Pinch, Kind::ThreeFingerSwipe, Kind::FourFingerSwipe] {
            let _ = gestures.decide(Event::Begin(kind));
            let _ = gestures.decide(if kind == Kind::Pinch {
                Event::Pinch(2.0)
            } else {
                Event::Swipe {
                    dx: -100.0,
                    dy: 0.0,
                }
            });
            assert_eq!(gestures.decide(end(kind)).is_some(), kind != disabled);
        }
        gestures.configure(Preferences::default());
        let _ = gestures.decide(Event::Begin(disabled));
        let _ = gestures.decide(if disabled == Kind::Pinch {
            Event::Pinch(2.0)
        } else {
            Event::Swipe {
                dx: -100.0,
                dy: 0.0,
            }
        });
        gestures.configure(preferences);
        gestures.configure(Preferences::default());
        assert_eq!(gestures.decide(end(disabled)), None);
    }
}

/// Natural/traditional affects scroll, never the direction of desktop swipes.
#[test]
fn natural_and_traditional_scrolling_are_the_persons_setting() {
    let mut gestures = Gestures::default();
    for (scrolling, sign) in [(Scrolling::Natural, -1.0), (Scrolling::Traditional, 1.0)] {
        gestures.configure(Preferences {
            scrolling,
            ..Preferences::default()
        });
        assert_eq!(
            gestures.decide(scroll(Some(4.0), Some(7.0))),
            scrolled(Some(4.0 * sign), Some(7.0 * sign))
        );
        assert_eq!(
            swipe(&mut gestures, Kind::FourFingerSwipe, -100.0, 0.0),
            Some(Intent::Desktop(Switch::Next))
        );
    }
}

/// Exhaustive matching makes an added agent intent a compile failure here.
#[test]
fn no_gesture_can_invoke_the_agent_and_broken_sequences_are_retired() {
    let mut gestures = Gestures::default();
    for event in [
        scroll(None, Some(1.0)),
        Event::Begin(Kind::Pinch),
        Event::Pinch(2.0),
        end(Kind::Pinch),
        Event::Begin(Kind::ThreeFingerSwipe),
        Event::Swipe {
            dx: -100.0,
            dy: 0.0,
        },
        end(Kind::ThreeFingerSwipe),
    ] {
        match gestures.decide(event) {
            None | Some(Intent::Scroll { .. } | Intent::Zoom(_) | Intent::Desktop(_)) => {}
        }
    }
    for interrupt in [
        Event::Reset,
        Event::Begin(Kind::Pinch),
        Event::Pinch(2.0),
        end(Kind::FourFingerSwipe),
    ] {
        let _ = gestures.decide(Event::Begin(Kind::ThreeFingerSwipe));
        let _ = gestures.decide(Event::Swipe {
            dx: -100.0,
            dy: 0.0,
        });
        assert_eq!(gestures.decide(interrupt), None);
        assert_eq!(gestures.decide(end(Kind::ThreeFingerSwipe)), None);
    }
    assert_eq!(gestures.decide(Event::Pinch(2.0)), None);
    assert_eq!(
        gestures.decide(Event::Swipe {
            dx: -100.0,
            dy: 0.0
        }),
        None
    );
    assert!(swipe(&mut gestures, Kind::ThreeFingerSwipe, -100.0, 0.0).is_some());
}

/// Overflow retires a sequence; a second device cannot end the first's swipe.
#[test]
fn overflowing_motion_and_another_device_cannot_finish_a_swipe() {
    let mut first = Gestures::default();
    let mut second = Gestures::default();
    let kind = Kind::ThreeFingerSwipe;
    let _ = first.decide(Event::Begin(kind));
    let _ = first.decide(Event::Swipe {
        dx: -100.0,
        dy: 0.0,
    });
    assert_eq!(second.decide(end(kind)), None);
    assert_eq!(first.decide(end(kind)), Some(Intent::Desktop(Switch::Next)));
    let _ = first.decide(Event::Begin(kind));
    for _ in 0..2 {
        let _ = first.decide(Event::Swipe {
            dx: f64::MAX,
            dy: 0.0,
        });
    }
    assert_eq!(first.decide(end(kind)), None);
    assert_eq!(
        swipe(&mut first, kind, -100.0, 0.0),
        Some(Intent::Desktop(Switch::Next))
    );
}
