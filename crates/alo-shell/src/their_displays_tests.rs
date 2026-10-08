//! The arrangement the shell builds from what a person settled.

use super::*;
use alo_displays::{Changes, Moment, NightLight, Reported, Socket};

/// A display of this many pixels on glass this big, in this socket.
fn a_display(socket: &str, pixels: (u32, u32), millimetres: (u32, u32)) -> Reported {
    Reported::of(
        Socket::named(socket).expect("a socket name this test chose"),
        // No panel, for the reason `the_displays_as_reported` gives at length:
        // a make and model without a serial identify a kind of monitor rather
        // than a monitor, and two identical screens would become one.
        None,
        pixels,
        Some(millimetres),
    )
    .expect("a display with pixels and a size is describable")
}

/// A person who has settled the shipped defaults and arranged nothing.
fn settled_nothing() -> TheirDisplays {
    TheirDisplays {
        appearance: alo_appearance::Appearance::shipped(),
        tonight: NightLight::as_shipped().at(a_moment()),
        remembered: Changes::untouched(),
    }
}

/// A moment this test chooses, so nothing here reads a clock.
fn a_moment() -> Moment {
    Moment::at(
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000),
        0,
    )
}

/// **Nothing plugged in is no arrangement, and that is not a fault.**
///
/// A machine with nothing to draw on has no desk to describe. Asserted
/// separately so the tests below cannot pass by accidentally answering `None`.
#[test]
fn nothing_plugged_in_is_no_arrangement() {
    assert!(
        settled_nothing().the_screens_of(Vec::new()).is_none(),
        "an empty desk produced an arrangement"
    );
}

/// **One display is one screen**, which is every machine this lane can test on.
#[test]
fn one_display_becomes_one_screen() {
    let screens = settled_nothing()
        .the_screens_of(vec![a_display("eDP-1", (1920, 1080), (344, 194))])
        .expect("one display is an arrangement");
    assert_eq!(screens.how_many(), 1);
}

/// **Two displays are two screens**, which is the whole point of the plan this
/// task belongs to — and the thing production could not reach at all before it.
#[test]
fn two_displays_become_two_screens() {
    let screens = settled_nothing()
        .the_screens_of(vec![
            a_display("eDP-1", (1920, 1080), (344, 194)),
            a_display("DP-2", (2560, 1440), (597, 336)),
        ])
        .expect("two displays are an arrangement");
    assert_eq!(
        screens.how_many(),
        2,
        "two displays did not make two screens"
    );
}

/// **Each screen is found by the socket it is plugged into**, which is how this
/// compositor tells displays apart — see `the_displays_as_reported` for why
/// there is no panel to prefer.
#[test]
fn each_screen_is_found_by_its_socket() {
    let screens = settled_nothing()
        .the_screens_of(vec![
            a_display("eDP-1", (1920, 1080), (344, 194)),
            a_display("DP-2", (2560, 1440), (597, 336)),
        ])
        .expect("two displays are an arrangement");
    for socket in ["eDP-1", "DP-2"] {
        let identity = alo_displays::Identity::Socket(
            Socket::named(socket).expect("a socket name this test chose"),
        );
        assert!(
            screens.on(&identity).is_some(),
            "{socket} is not in the arrangement built from it"
        );
    }
}

/// **A set nobody has arranged says so**, rather than silently inventing one.
///
/// `Note::NewHere` is the sentence `alo-displays` already has for it, and this
/// is the first test in this crate that it is reached at all — before task 10
/// nothing in or out of production built a `Screens` from a person's settings.
#[test]
fn a_set_nobody_arranged_is_announced_as_new() {
    let screens = settled_nothing()
        .the_screens_of(vec![a_display("eDP-1", (1920, 1080), (344, 194))])
        .expect("one display is an arrangement");
    assert!(
        screens
            .notes()
            .iter()
            .any(|note| matches!(note, alo_displays::Note::NewHere(_))),
        "a screen never seen before was not announced as new: {:?}",
        screens.notes()
    );
}

/// **The size this compositor claims it can draw is the strict answer**, and
/// this test is the guard on that decision rather than on the arithmetic.
///
/// `THE_SIZES_THIS_COMPOSITOR_DRAWS`'s own note gives the reasoning: a client is
/// advertised `Scale::Integer(1)` and no fractional protocol exists, so a
/// fractional size would be an upscale rather than a native render. **If
/// somebody advertises `wp_fractional_scale_v1` this test should fail**, and
/// changing it is then the honest act rather than a nuisance.
#[test]
fn this_compositor_claims_only_the_sizes_it_can_draw_natively() {
    assert_eq!(
        THE_SIZES_THIS_COMPOSITOR_DRAWS,
        alo_displays::Support::WholeMultiplesOnly,
        "the compositor claims fractional sizes; does it advertise \
         wp_fractional_scale_v1 now? If so this test is what should change"
    );
    // And what that answer *does*, so the claim is not only a label: a size
    // between two whole multiples is rounded up rather than down, because text
    // too large can be read and text too small cannot.
    let asked = alo_displays::Scale::per_cent(150).expect("150 % is a scale");
    assert_eq!(
        THE_SIZES_THIS_COMPOSITOR_DRAWS.nearest(asked).as_per_cent(),
        200,
        "150 % did not round up to the nearest whole multiple"
    );
}
